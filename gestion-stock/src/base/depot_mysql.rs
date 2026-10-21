//! Le dépôt MySQL : les mêmes opérations que `DepotFichier`, mais dans la base partagée.
//!
//! Chaque opération est UNE transaction : on verrouille les lignes concernées (le poste, puis les
//! produits dans l'ordre de leur clé, pour que deux postes ne s'attendent jamais en croix), on les
//! charge dans un `Stock` partiel, on appelle la règle du cœur (inchangée), puis on écrit la
//! différence et on valide. Une règle qui refuse fait annuler la transaction : rien ne change.

use super::migrations::{VERSION_COURANTE, connecter, version_de};
use super::{Configuration, erreur_de_base};
use crate::depot::Depot;
use crate::erreur::ErreurStock;
use crate::horodatage::Horodatage;
use crate::journal::{HASH_INITIAL, empreinte};
use crate::monnaie::{Devise, Montant};
use crate::reglages::{Profil, Reglages};
use crate::stock::{Genre, Mouvement, Produit, Stock};
use crate::vente::{GenrePiece, LigneVente, Moyen, Piece};
use mysql::prelude::Queryable;
use mysql::{AccessMode, Conn, IsolationLevel, Transaction, TxOpts};
use std::cell::RefCell;
use std::collections::BTreeMap;

/// Combien de fois une transaction annulée par le serveur (blocage mutuel) est rejouée : trois
/// essais en tout. Les verrous sont pris dans un ordre fixe, ce qui doit éviter les blocages ; ceci
/// n'est qu'un filet.
const ESSAIS: u32 = 3;

/// La longueur maximale d'un nom de produit, en caractères (la colonne est un `VARCHAR(190)`).
const LONGUEUR_NOM_MAX: usize = 190;

/// Ce qui peut faire échouer une opération dans une transaction : une règle du métier (le
/// message est celui du cœur) ou le serveur (que l'on traduit à la sortie).
pub(super) enum Echec {
    Regle(ErreurStock),
    Serveur(mysql::Error),
}

impl From<ErreurStock> for Echec {
    fn from(e: ErreurStock) -> Echec {
        Echec::Regle(e)
    }
}

impl From<mysql::Error> for Echec {
    fn from(e: mysql::Error) -> Echec {
        Echec::Serveur(e)
    }
}

/// Un blocage mutuel ou une attente de verrou trop longue : la transaction a été annulée par le
/// serveur, on peut la rejouer sans risque.
fn est_blocage(e: &mysql::Error) -> bool {
    matches!(e, mysql::Error::MySqlError(m) if m.code == 1213 || m.code == 1205)
}

fn est_doublon(e: &mysql::Error) -> bool {
    matches!(e, mysql::Error::MySqlError(m) if m.code == 1062)
}

/// La clé d'un nom : nettoyé et en minuscules, comme la compare le cœur.
pub(super) fn cle(nom: &str) -> String {
    nom.trim().to_lowercase()
}

fn trop_long(nom: &str) -> ErreurStock {
    ErreurStock::Utilisation(format!(
        "Le nom d'un produit est limité à {LONGUEUR_NOM_MAX} caractères (« {}… » est trop long).",
        nom.chars().take(30).collect::<String>()
    ))
}

/// Refuse un nom (déjà accepté par le cœur) que la colonne ne pourrait pas garder entier.
pub(super) fn verifier_longueur(nom: &str) -> Result<(), ErreurStock> {
    if nom.chars().count() > LONGUEUR_NOM_MAX || cle(nom).chars().count() > LONGUEUR_NOM_MAX {
        return Err(trop_long(nom));
    }
    Ok(())
}

fn illisible(quoi: &str) -> ErreurStock {
    ErreurStock::Base(format!("La base contient une valeur illisible : {quoi}."))
}

// ---- Lecture et écriture des lignes ---------------------------------------------------------

fn lire_reglages(tx: &mut Transaction<'_>, verrou: bool) -> Result<Reglages, Echec> {
    let requete = if verrou {
        "SELECT cle, valeur FROM reglage FOR UPDATE"
    } else {
        "SELECT cle, valeur FROM reglage"
    };
    let lignes: Vec<(String, String)> = tx.query(requete)?;
    let mut reglages = Reglages::default();
    for (cle, valeur) in lignes {
        match cle.as_str() {
            "profil" => {
                reglages.profil = Profil::depuis_code(&valeur)
                    .ok_or_else(|| illisible(&format!("réglage « profil » = « {valeur} »")))?;
            }
            "devise" => {
                reglages.devise = Devise::depuis_code(&valeur)
                    .ok_or_else(|| illisible(&format!("réglage « devise » = « {valeur} »")))?;
            }
            "taux_defaut_bp" => {
                reglages.taux_defaut_bp = valeur.parse().map_err(|_| {
                    illisible(&format!("réglage « taux_defaut_bp » = « {valeur} »"))
                })?;
            }
            _ => {}
        }
    }
    Ok(reglages)
}

pub(super) fn ecrire_reglages(tx: &mut Transaction<'_>, reglages: &Reglages) -> Result<(), Echec> {
    let lignes = [
        ("profil", reglages.profil.code().to_string()),
        ("devise", reglages.devise.code().to_string()),
        ("taux_defaut_bp", reglages.taux_defaut_bp.to_string()),
    ];
    for (cle, valeur) in lignes {
        tx.exec_drop(
            "REPLACE INTO reglage (cle, valeur) VALUES (?, ?)",
            (cle, valeur),
        )?;
    }
    Ok(())
}

type LigneProduit = (String, u32, u32, Option<i64>, Option<u32>);

fn produit_depuis((nom, quantite, seuil, prix, taux_bp): LigneProduit) -> Produit {
    Produit {
        nom,
        quantite,
        seuil,
        prix: prix.map(Montant),
        taux_bp,
    }
}

/// Les produits de ces clés (ceux qui existent), verrouillés, dans l'ordre alphabétique des clés.
fn charger_produits(tx: &mut Transaction<'_>, cles: &[String]) -> Result<Vec<Produit>, Echec> {
    let mut cles = cles.to_vec();
    cles.sort();
    cles.dedup();
    if cles.is_empty() {
        return Ok(Vec::new());
    }
    let marques = vec!["?"; cles.len()].join(", ");
    let requete = format!(
        "SELECT nom, quantite, seuil, prix, taux_bp FROM produit \
         WHERE nom_cle IN ({marques}) ORDER BY nom_cle FOR UPDATE"
    );
    let lignes: Vec<LigneProduit> = tx.exec(requete, cles)?;
    Ok(lignes.into_iter().map(produit_depuis).collect())
}

/// `numero, genre, annule_numero, utc, decalage_minutes, total, moyen, recu, rendu,
/// hash_precedent, hash`
type LignePiece = (
    u64,
    String,
    Option<u64>,
    i64,
    i32,
    i64,
    String,
    i64,
    i64,
    String,
    String,
);

const COLONNES_PIECE: &str = "numero, genre, annule_numero, utc, decalage_minutes, total, moyen, \
                              recu, rendu, hash_precedent, hash";

fn piece_depuis(ligne: LignePiece, lignes: Vec<LigneVente>) -> Result<Piece, ErreurStock> {
    let (numero, genre, annule, utc, decalage, total, moyen, recu, rendu, precedent, hash) = ligne;
    let genre = match (genre.as_str(), annule) {
        ("V", None) => GenrePiece::Vente,
        ("A", Some(de)) => GenrePiece::Annulation { de },
        _ => return Err(illisible(&format!("genre de la pièce n° {numero}"))),
    };
    let moyen = Moyen::depuis_code(&moyen)
        .ok_or_else(|| illisible(&format!("moyen de paiement de la pièce n° {numero}")))?;
    Ok(Piece {
        numero,
        genre,
        horodatage: Horodatage {
            utc,
            decalage_minutes: decalage,
        },
        lignes,
        moyen,
        total: Montant(total),
        recu: Montant(recu),
        rendu: Montant(rendu),
        hash_precedent: precedent,
        hash,
    })
}

fn ligne_vente((nom, quantite, prix, taux_bp): (String, u32, i64, u32)) -> LigneVente {
    LigneVente {
        nom,
        quantite,
        prix: Montant(prix),
        taux_bp,
    }
}

/// Une pièce avec ses lignes, ou `None` ; verrouillée si `verrou`.
fn charger_piece(
    tx: &mut Transaction<'_>,
    caisse: &str,
    numero: u64,
    verrou: bool,
) -> Result<Option<Piece>, Echec> {
    let fin = if verrou { " FOR UPDATE" } else { "" };
    let ligne: Option<LignePiece> = tx.exec_first(
        format!("SELECT {COLONNES_PIECE} FROM piece WHERE caisse = ? AND numero = ?{fin}"),
        (caisse, numero),
    )?;
    let Some(ligne) = ligne else {
        return Ok(None);
    };
    let lignes: Vec<(String, u32, i64, u32)> = tx.exec(
        "SELECT nom, quantite, prix, taux_bp FROM piece_ligne \
         WHERE caisse = ? AND numero = ? ORDER BY rang",
        (caisse, numero),
    )?;
    let lignes = lignes.into_iter().map(ligne_vente).collect();
    Ok(Some(piece_depuis(ligne, lignes)?))
}

/// Une pièce quelconque de la base (n'importe quelle caisse) : le cœur s'en sert pour savoir
/// que la devise et le profil sont figés.
fn piece_quelconque(tx: &mut Transaction<'_>) -> Result<Option<Piece>, Echec> {
    let une: Option<(String, u64)> = tx.query_first("SELECT caisse, numero FROM piece LIMIT 1")?;
    match une {
        Some((caisse, numero)) => charger_piece(tx, &caisse, numero, false),
        None => Ok(None),
    }
}

/// Une pièce qui nomme ce produit (clé), si le produit a déjà été vendu : le cœur refuse alors de
/// le renommer ou de le supprimer.
fn piece_temoin(tx: &mut Transaction<'_>, cle_du_produit: &str) -> Result<Option<Piece>, Echec> {
    let une: Option<(String, u64)> = tx.exec_first(
        "SELECT caisse, numero FROM piece_ligne WHERE nom_cle = ? LIMIT 1",
        (cle_du_produit,),
    )?;
    match une {
        Some((caisse, numero)) => charger_piece(tx, &caisse, numero, false),
        None => Ok(None),
    }
}

/// Écrit les mouvements d'une opération (ceux du `Stock` partiel : tous sont nouveaux).
fn ecrire_mouvements(
    tx: &mut Transaction<'_>,
    caisse: &str,
    mouvements: &[Mouvement],
    piece_numero: Option<u64>,
) -> Result<(), Echec> {
    for mouvement in mouvements {
        let genre = match mouvement.genre {
            Genre::Entree => "entree",
            Genre::Sortie => "sortie",
        };
        tx.exec_drop(
            "INSERT INTO mouvement (produit_nom, produit_cle, genre, quantite, caisse, piece_numero) \
             VALUES (?, ?, ?, ?, ?, ?)",
            (
                &mouvement.nom,
                cle(&mouvement.nom),
                genre,
                mouvement.quantite,
                caisse,
                piece_numero,
            ),
        )?;
    }
    Ok(())
}

/// Une coupure de la communication : on ne sait pas si la dernière commande est arrivée.
fn est_coupure(e: &mysql::Error) -> bool {
    use mysql::Error::{CodecError, DriverError, IoError};
    matches!(e, IoError(_) | CodecError(_) | DriverError(_))
}

/// La tête du journal de ce poste, verrouillée : le dernier numéro et la dernière empreinte.
pub(super) fn verrouiller_la_tete(
    tx: &mut Transaction<'_>,
    caisse: &str,
) -> Result<(u64, String), Echec> {
    let tete: Option<(u64, String)> = tx.exec_first(
        "SELECT dernier_numero, dernier_hash FROM caisse WHERE code = ? FOR UPDATE",
        (caisse,),
    )?;
    tete.ok_or_else(|| {
        Echec::Regle(ErreurStock::Base(format!(
            "La caisse « {caisse} » n'existe pas dans la base."
        )))
    })
}

/// La dernière pièce du journal de ce poste (celle que dit la tête), ou `None` s'il est vide. Si
/// la tête et la pièce ne s'accordent pas, le journal a été modifié.
fn derniere_piece(
    tx: &mut Transaction<'_>,
    caisse: &str,
    numero: u64,
    hash: &str,
) -> Result<Option<Piece>, Echec> {
    if numero == 0 {
        return Ok(None);
    }
    match charger_piece(tx, caisse, numero, false)? {
        Some(piece) if piece.hash == hash => Ok(Some(piece)),
        _ => Err(Echec::Regle(ErreurStock::JournalAltere { numero })),
    }
}

/// Écrit les nouvelles quantités des produits que l'opération a changés.
fn ecrire_quantites(
    tx: &mut Transaction<'_>,
    avant: &[Produit],
    apres: &[Produit],
) -> Result<(), Echec> {
    for produit in apres {
        let inchange = avant
            .iter()
            .any(|p| cle(&p.nom) == cle(&produit.nom) && p.quantite == produit.quantite);
        if !inchange {
            tx.exec_drop(
                "UPDATE produit SET quantite = ? WHERE nom_cle = ?",
                (produit.quantite, cle(&produit.nom)),
            )?;
        }
    }
    Ok(())
}

/// Écrit une pièce du journal et ses lignes.
fn ecrire_piece(tx: &mut Transaction<'_>, caisse: &str, piece: &Piece) -> Result<(), Echec> {
    let (genre, annule) = match piece.genre {
        GenrePiece::Vente => ("V", None),
        GenrePiece::Annulation { de } => ("A", Some(de)),
    };
    tx.exec_drop(
        "INSERT INTO piece (caisse, numero, genre, annule_numero, utc, decalage_minutes, total, \
         moyen, recu, rendu, hash_precedent, hash) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        (
            caisse,
            piece.numero,
            genre,
            annule,
            piece.horodatage.utc,
            piece.horodatage.decalage_minutes,
            piece.total.0,
            piece.moyen.code(),
            piece.recu.0,
            piece.rendu.0,
            &piece.hash_precedent,
            &piece.hash,
        ),
    )?;
    for (rang, ligne) in piece.lignes.iter().enumerate() {
        tx.exec_drop(
            "INSERT INTO piece_ligne (caisse, numero, rang, nom, nom_cle, quantite, prix, taux_bp) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
            (
                caisse,
                piece.numero,
                rang as u64,
                &ligne.nom,
                cle(&ligne.nom),
                ligne.quantite,
                ligne.prix.0,
                ligne.taux_bp,
            ),
        )?;
    }
    Ok(())
}

/// Tout ce qu'une vente ou une annulation écrit : quantités, mouvements (liés à la pièce), la
/// pièce, puis la nouvelle tête du journal.
fn ecrire_les_effets(
    tx: &mut Transaction<'_>,
    caisse: &str,
    avant: &[Produit],
    stock: &Stock,
    piece: &Piece,
) -> Result<(), Echec> {
    ecrire_quantites(tx, avant, &stock.produits)?;
    ecrire_mouvements(tx, caisse, &stock.historique, Some(piece.numero))?;
    ecrire_piece(tx, caisse, piece)?;
    tx.exec_drop(
        "UPDATE caisse SET dernier_numero = ?, dernier_hash = ? WHERE code = ?",
        (piece.numero, &piece.hash, caisse),
    )?;
    Ok(())
}

/// Une pièce lue en base, avec son numéro : l'erreur est gardée pour que le vérificateur nomme la
/// pièce illisible.
type PieceLue = (u64, Result<Piece, ErreurStock>);

/// Jusqu'à `limite` pièces de la caisse après le numéro `apres`, avec leurs lignes. Une pièce
/// illisible est rendue avec son numéro et son erreur, pour que le vérificateur la signale.
fn lire_un_morceau(
    tx: &mut Transaction<'_>,
    caisse: &str,
    apres: u64,
    limite: u32,
) -> Result<Vec<PieceLue>, Echec> {
    let entetes: Vec<LignePiece> = tx.exec(
        format!(
            "SELECT {COLONNES_PIECE} FROM piece WHERE caisse = ? AND numero > ? \
             ORDER BY numero LIMIT ?"
        ),
        (caisse, apres, limite),
    )?;
    let Some(derniere) = entetes.last().map(|e| e.0) else {
        return Ok(Vec::new());
    };
    let lignes: Vec<(u64, String, u32, i64, u32)> = tx.exec(
        "SELECT numero, nom, quantite, prix, taux_bp FROM piece_ligne \
         WHERE caisse = ? AND numero > ? AND numero <= ? ORDER BY numero, rang",
        (caisse, apres, derniere),
    )?;
    let mut par_piece: BTreeMap<u64, Vec<LigneVente>> = BTreeMap::new();
    for (numero, nom, quantite, prix, taux_bp) in lignes {
        par_piece
            .entry(numero)
            .or_default()
            .push(ligne_vente((nom, quantite, prix, taux_bp)));
    }
    Ok(entetes
        .into_iter()
        .map(|entete| {
            let numero = entete.0;
            let lignes = par_piece.remove(&numero).unwrap_or_default();
            (numero, piece_depuis(entete, lignes))
        })
        .collect())
}

/// Une connexion à la base, avec la vérification que le schéma est de la version attendue et que
/// la caisse de ce poste existe (elle est créée à la première connexion, avec un journal vide).
pub(super) fn connexion_verifiee(config: &Configuration) -> Result<Conn, ErreurStock> {
    let mut connexion = connecter(config)?;
    match version_de(&mut connexion, config)? {
        None => {
            return Err(ErreurStock::Base(format!(
                "La base « {} » n'est pas encore créée : lancez « stock migrer ».",
                config.base
            )));
        }
        Some(v) if v < VERSION_COURANTE => {
            return Err(ErreurStock::Base(format!(
                "La base n'est pas à jour (version {v}, version attendue {VERSION_COURANTE}) : lancez « stock migrer »."
            )));
        }
        Some(v) if v > VERSION_COURANTE => {
            return Err(ErreurStock::Base(format!(
                "La base est plus récente que ce programme (version {v}, ce programme connaît la version {VERSION_COURANTE}) : mettez le programme à jour."
            )));
        }
        Some(_) => {}
    }
    connexion
        .exec_drop(
            "INSERT IGNORE INTO caisse (code) VALUES (?)",
            (&config.caisse,),
        )
        .map_err(|e| erreur_de_base(&e, config))?;
    Ok(connexion)
}

/// Recalcule toute la chaîne du journal de cette caisse, par morceaux de 1 000 pièces (jamais tout
/// en mémoire), et compare la tête de la caisse : `JournalAltere` à la première pièce qui ne
/// correspond plus. Sert à `verifier_journal` et, avant la validation, à l'import.
pub(super) fn verifier_la_chaine(tx: &mut Transaction<'_>, caisse: &str) -> Result<(), Echec> {
    let tete: Option<(u64, String)> = tx.exec_first(
        "SELECT dernier_numero, dernier_hash FROM caisse WHERE code = ?",
        (caisse,),
    )?;
    let (tete_numero, tete_hash) = tete.unwrap_or((0, HASH_INITIAL.to_string()));
    let mut attendu = HASH_INITIAL.to_string();
    let mut dernier = 0u64;
    loop {
        let morceau = lire_un_morceau(tx, caisse, dernier, 1000)?;
        if morceau.is_empty() {
            break;
        }
        for (numero, piece) in morceau {
            let altere = Echec::Regle(ErreurStock::JournalAltere { numero });
            let Ok(piece) = piece else {
                return Err(altere);
            };
            if piece.numero != dernier + 1
                || piece.hash_precedent != attendu
                || piece.hash != empreinte(&piece.hash_precedent, &piece)
            {
                return Err(altere);
            }
            attendu = piece.hash;
            dernier = piece.numero;
        }
    }
    if dernier != tete_numero || attendu != tete_hash {
        return Err(Echec::Regle(ErreurStock::JournalAltere {
            numero: dernier.max(tete_numero),
        }));
    }
    Ok(())
}

// ---- Le dépôt ---------------------------------------------------------------------------------

pub struct DepotMysql {
    config: Configuration,
    connexion: Conn,
    /// Pour les tests de coupure : appelé juste avant la validation d'une transaction, avec le
    /// numéro de la connexion.
    crochet: Option<Box<dyn FnMut(u64)>>,
}

impl DepotMysql {
    /// Se connecte, vérifie que le schéma est de la version attendue et que la caisse de ce poste
    /// existe (elle est créée à la première connexion, avec un journal vide).
    pub fn connecter(config: &Configuration) -> Result<DepotMysql, ErreurStock> {
        Ok(DepotMysql {
            config: config.clone(),
            connexion: connexion_verifiee(config)?,
            crochet: None,
        })
    }

    /// Une entrée ou une sortie de stock : `appliquer` est la règle du cœur, appliquée au produit
    /// verrouillé ; on écrit sa nouvelle quantité et le mouvement qu'elle a produit.
    fn mouvement(
        &mut self,
        nom: &str,
        appliquer: impl Fn(&mut Stock) -> Result<Produit, ErreurStock>,
    ) -> Result<Produit, ErreurStock> {
        let cle_du_nom = cle(nom);
        self.transaction(|tx, caisse| {
            // L'ORDRE DES VERROUS est toujours : la ligne de la caisse, puis les produits. Le
            // mouvement qu'on écrit référence la caisse (clé étrangère), donc le serveur la
            // verrouillerait de toute façon, mais APRÈS le produit : face à une vente qui tient
            // la caisse et attend le produit, c'est un interblocage. On la prend donc en premier.
            verrouiller_la_tete(tx, caisse)?;
            let produits = charger_produits(tx, std::slice::from_ref(&cle_du_nom))?;
            let mut stock = Stock::partiel(produits, Reglages::default(), Vec::new());
            let produit = appliquer(&mut stock)?;
            ecrire_mouvements(tx, caisse, &stock.historique, None)?;
            tx.exec_drop(
                "UPDATE produit SET quantite = ? WHERE nom_cle = ?",
                (produit.quantite, &cle_du_nom),
            )?;
            Ok(produit)
        })
    }

    /// Si la connexion est tombée depuis la dernière opération, en ouvre une nouvelle.
    fn assurer_la_connexion(&mut self) -> Result<(), ErreurStock> {
        if self.connexion.ping().is_err() {
            self.connexion = connecter(&self.config)?;
        }
        Ok(())
    }

    /// Exécute `operation` dans une transaction (verrous en lecture validée). Une règle qui
    /// refuse, ou une erreur du serveur, annule tout. Un blocage mutuel détecté par le serveur est
    /// rejoué (voir `ESSAIS`) : la transaction avait été annulée, rien n'a changé.
    fn transaction_brute<R>(
        &mut self,
        mut operation: impl FnMut(&mut Transaction<'_>, &str) -> Result<R, Echec>,
    ) -> Result<R, Echec> {
        for essai in 0..ESSAIS {
            self.assurer_la_connexion().map_err(Echec::Regle)?;
            let caisse = self.config.caisse.clone();
            let options =
                TxOpts::default().set_isolation_level(Some(IsolationLevel::ReadCommitted));
            let connexion = &mut self.connexion;
            let crochet = &mut self.crochet;
            let resultat = (|| -> Result<R, Echec> {
                let mut tx = connexion.start_transaction(options)?;
                let valeur = operation(&mut tx, &caisse)?;
                if let Some(crochet) = crochet.as_mut() {
                    let id: Option<u64> = tx.query_first("SELECT CONNECTION_ID()")?;
                    crochet(id.unwrap_or(0));
                }
                tx.commit()?;
                Ok(valeur)
            })();
            match resultat {
                Err(Echec::Serveur(e)) if essai + 1 < ESSAIS && est_blocage(&e) => continue,
                autre => return autre,
            }
        }
        unreachable!("la seconde tentative renvoie toujours un résultat")
    }

    fn transaction<R>(
        &mut self,
        operation: impl FnMut(&mut Transaction<'_>, &str) -> Result<R, Echec>,
    ) -> Result<R, ErreurStock> {
        match self.transaction_brute(operation) {
            Ok(valeur) => Ok(valeur),
            Err(Echec::Regle(e)) => Err(e),
            Err(Echec::Serveur(e)) => Err(erreur_de_base(&e, &self.config)),
        }
    }

    /// Une vente ou une annulation : une transaction qui écrit une pièce. Si la communication
    /// tombe, on ne réessaie JAMAIS d'écrire : on relit le journal pour savoir si la pièce est
    /// entrée (alors c'est un succès) ou non (alors rien n'a été enregistré, c'est un échec sûr).
    /// Si même cette relecture est impossible, on le dit, pour qu'on ne refasse pas la vente à
    /// l'aveugle.
    fn ecrire_une_piece(
        &mut self,
        mut operation: impl FnMut(&mut Transaction<'_>, &str) -> Result<Piece, Echec>,
    ) -> Result<Piece, ErreurStock> {
        let attendue: RefCell<Option<Piece>> = RefCell::new(None);
        let resultat = self.transaction_brute(|tx, caisse| {
            let piece = operation(tx, caisse)?;
            *attendue.borrow_mut() = Some(piece.clone());
            Ok(piece)
        });
        match resultat {
            Ok(piece) => Ok(piece),
            Err(Echec::Regle(e)) => Err(e),
            Err(Echec::Serveur(e)) if est_coupure(&e) => {
                let Some(attendue) = attendue.into_inner() else {
                    // La pièce n'a même pas été calculée : rien n'a pu être écrit.
                    return Err(erreur_de_base(&e, &self.config));
                };
                match self.piece_est_entree(&attendue) {
                    Ok(true) => Ok(attendue),
                    Ok(false) => Err(erreur_de_base(&e, &self.config)),
                    Err(_) => Err(ErreurStock::Base(
                        "Le serveur a cessé de répondre pendant l'enregistrement : la vente a peut-être été enregistrée. Vérifiez l'historique avant de la refaire."
                            .to_string(),
                    )),
                }
            }
            Err(Echec::Serveur(e)) => Err(erreur_de_base(&e, &self.config)),
        }
    }

    /// Vrai si cette pièce, telle quelle (mêmes lignes, mêmes empreintes), est dans le journal de
    /// ce poste. Se reconnecte au besoin.
    #[doc(hidden)]
    pub fn piece_est_entree(&mut self, piece: &Piece) -> Result<bool, ErreurStock> {
        self.assurer_la_connexion()?;
        let caisse = self.config.caisse.clone();
        let numero = piece.numero;
        let connexion = &mut self.connexion;
        let resultat = (|| -> Result<Option<Piece>, Echec> {
            let mut tx = connexion.start_transaction(TxOpts::default())?;
            let trouvee = charger_piece(&mut tx, &caisse, numero, false)?;
            tx.commit()?;
            Ok(trouvee)
        })();
        match resultat {
            Ok(trouvee) => Ok(trouvee.as_ref() == Some(piece)),
            Err(Echec::Regle(e)) => Err(e),
            Err(Echec::Serveur(e)) => Err(erreur_de_base(&e, &self.config)),
        }
    }

    /// Pour les tests de coupure : `crochet` est appelé juste avant la validation de chaque
    /// transaction, avec le numéro de la connexion (de quoi la tuer depuis l'extérieur).
    #[doc(hidden)]
    pub fn installer_un_crochet_avant_validation(&mut self, crochet: Box<dyn FnMut(u64)>) {
        self.crochet = Some(crochet);
    }

    #[doc(hidden)]
    pub fn retirer_le_crochet(&mut self) {
        self.crochet = None;
    }
}

impl Depot for DepotMysql {
    fn instantane(&mut self) -> Result<Stock, ErreurStock> {
        self.assurer_la_connexion()?;
        let caisse = self.config.caisse.clone();
        let options = TxOpts::default()
            .set_isolation_level(Some(IsolationLevel::RepeatableRead))
            .set_with_consistent_snapshot(true)
            .set_access_mode(Some(AccessMode::ReadOnly));
        let resultat = (|| -> Result<Stock, Echec> {
            let mut tx = self.connexion.start_transaction(options)?;
            let reglages = lire_reglages(&mut tx, false)?;
            let lignes: Vec<LigneProduit> =
                tx.query("SELECT nom, quantite, seuil, prix, taux_bp FROM produit ORDER BY id")?;
            let produits = lignes.into_iter().map(produit_depuis).collect();
            let mouvements: Vec<(String, String, u32)> =
                tx.query("SELECT genre, produit_nom, quantite FROM mouvement ORDER BY id")?;
            let historique = mouvements
                .into_iter()
                .map(|(genre, nom, quantite)| {
                    let genre = if genre == "entree" {
                        Genre::Entree
                    } else {
                        Genre::Sortie
                    };
                    Mouvement {
                        genre,
                        nom,
                        quantite,
                    }
                })
                .collect();
            let lignes_de_pieces: Vec<(u64, String, u32, i64, u32)> = tx.exec(
                "SELECT numero, nom, quantite, prix, taux_bp FROM piece_ligne \
                 WHERE caisse = ? ORDER BY numero, rang",
                (&caisse,),
            )?;
            let mut lignes_par_piece: BTreeMap<u64, Vec<LigneVente>> = BTreeMap::new();
            for (numero, nom, quantite, prix, taux_bp) in lignes_de_pieces {
                lignes_par_piece
                    .entry(numero)
                    .or_default()
                    .push(ligne_vente((nom, quantite, prix, taux_bp)));
            }
            let entetes: Vec<LignePiece> = tx.exec(
                format!("SELECT {COLONNES_PIECE} FROM piece WHERE caisse = ? ORDER BY numero"),
                (&caisse,),
            )?;
            let mut pieces = Vec::with_capacity(entetes.len());
            for entete in entetes {
                let lignes = lignes_par_piece.remove(&entete.0).unwrap_or_default();
                pieces.push(piece_depuis(entete, lignes)?);
            }
            tx.commit()?;
            Ok(Stock {
                produits,
                historique,
                reglages,
                pieces,
            })
        })();
        match resultat {
            Ok(stock) => Ok(stock),
            Err(Echec::Regle(e)) => Err(e),
            Err(Echec::Serveur(e)) => Err(erreur_de_base(&e, &self.config)),
        }
    }

    fn ajouter_produit(&mut self, nom: &str, seuil: u32) -> Result<(), ErreurStock> {
        let cle_du_nom = cle(nom);
        self.transaction(|tx, _| {
            let reglages = lire_reglages(tx, false)?;
            let produits = charger_produits(tx, std::slice::from_ref(&cle_du_nom))?;
            let mut stock = Stock::partiel(produits, reglages, Vec::new());
            stock.ajouter_produit(nom, seuil)?;
            let nouveau = stock
                .produits
                .last()
                .cloned()
                .expect("le produit vient d'être ajouté");
            verifier_longueur(&nouveau.nom)?;
            let cle_du_produit = cle(&nouveau.nom);
            let ecriture = tx.exec_drop(
                "INSERT INTO produit (nom, nom_cle, quantite, seuil, prix, taux_bp) \
                 VALUES (?, ?, 0, ?, NULL, NULL)",
                (&nouveau.nom, &cle_du_produit, nouveau.seuil),
            );
            match ecriture {
                // Un autre poste l'a créé entre-temps : l'index unique le garantit.
                Err(e) if est_doublon(&e) => {
                    let existant: Option<String> = tx.exec_first(
                        "SELECT nom FROM produit WHERE nom_cle = ?",
                        (&cle_du_produit,),
                    )?;
                    Err(Echec::Regle(ErreurStock::ProduitExistant(
                        existant.unwrap_or(nouveau.nom),
                    )))
                }
                autre => Ok(autre?),
            }
        })
    }

    fn definir_prix(
        &mut self,
        nom: &str,
        prix: Option<Montant>,
        taux_bp: Option<u32>,
    ) -> Result<(), ErreurStock> {
        let cle_du_nom = cle(nom);
        self.transaction(|tx, _| {
            let reglages = lire_reglages(tx, false)?;
            let produits = charger_produits(tx, std::slice::from_ref(&cle_du_nom))?;
            let mut stock = Stock::partiel(produits, reglages, Vec::new());
            stock.definir_prix(nom, prix, taux_bp)?;
            let produit = &stock.produits[0];
            tx.exec_drop(
                "UPDATE produit SET prix = ?, taux_bp = ? WHERE nom_cle = ?",
                (produit.prix.map(|m| m.0), produit.taux_bp, &cle_du_nom),
            )?;
            Ok(())
        })
    }

    fn definir_reglages(&mut self, reglages: Reglages) -> Result<(), ErreurStock> {
        self.transaction(|tx, _| {
            // Seule cette opération verrouille les réglages : les autres ne les lisent que pour
            // écrire un message, et ne doivent pas faire attendre les postes les uns les autres.
            let actuels = lire_reglages(tx, true)?;
            // Une pièce quelconque suffit : le cœur verrouille la devise et le profil dès qu'il
            // y en a une.
            let temoin = piece_quelconque(tx)?;
            let mut stock = Stock::partiel(Vec::new(), actuels, temoin.into_iter().collect());
            stock.definir_reglages(reglages)?;
            ecrire_reglages(tx, &stock.reglages)?;
            Ok(())
        })
    }

    fn enregistrer_entree(&mut self, nom: &str, quantite: u32) -> Result<Produit, ErreurStock> {
        self.mouvement(nom, |stock| {
            stock.enregistrer_entree(nom, quantite).cloned()
        })
    }

    fn enregistrer_sortie(&mut self, nom: &str, quantite: u32) -> Result<Produit, ErreurStock> {
        self.mouvement(nom, |stock| {
            stock.enregistrer_sortie(nom, quantite).cloned()
        })
    }

    fn renommer_produit(&mut self, ancien: &str, nouveau: &str) -> Result<(), ErreurStock> {
        let (cle_ancienne, cle_nouvelle) = (cle(ancien), cle(nouveau));
        self.transaction(|tx, _| {
            // L'ancien produit, et l'éventuel produit qui porte déjà le nouveau nom.
            let produits = charger_produits(tx, &[cle_ancienne.clone(), cle_nouvelle.clone()])?;
            let temoin = piece_temoin(tx, &cle_ancienne)?;
            let mut stock =
                Stock::partiel(produits, Reglages::default(), temoin.into_iter().collect());
            stock.renommer_produit(ancien, nouveau)?;
            let renomme = stock
                .produits
                .iter()
                .find(|p| cle(&p.nom) == cle_nouvelle)
                .expect("le produit renommé est dans le stock partiel");
            verifier_longueur(&renomme.nom)?;
            let nom = renomme.nom.clone();
            tx.exec_drop(
                "UPDATE produit SET nom = ?, nom_cle = ? WHERE nom_cle = ?",
                (&nom, &cle_nouvelle, &cle_ancienne),
            )?;
            // Le nouveau nom remplace l'ancien dans tout l'historique.
            tx.exec_drop(
                "UPDATE mouvement SET produit_nom = ?, produit_cle = ? WHERE produit_cle = ?",
                (&nom, &cle_nouvelle, &cle_ancienne),
            )?;
            Ok(())
        })
    }

    fn supprimer_produit(&mut self, nom: &str) -> Result<(), ErreurStock> {
        let cle_du_nom = cle(nom);
        self.transaction(|tx, _| {
            let produits = charger_produits(tx, std::slice::from_ref(&cle_du_nom))?;
            let temoin = piece_temoin(tx, &cle_du_nom)?;
            let mut stock =
                Stock::partiel(produits, Reglages::default(), temoin.into_iter().collect());
            stock.supprimer_produit(nom)?;
            // Les mouvements restent : l'historique n'oublie rien.
            tx.exec_drop("DELETE FROM produit WHERE nom_cle = ?", (&cle_du_nom,))?;
            Ok(())
        })
    }

    fn encaisser(
        &mut self,
        demande: &[(String, u32)],
        moyen: Moyen,
        recu: Montant,
        quand: Horodatage,
    ) -> Result<Piece, ErreurStock> {
        let cles: Vec<String> = demande.iter().map(|(nom, _)| cle(nom)).collect();
        self.ecrire_une_piece(|tx, caisse| {
            let (dernier_numero, dernier_hash) = verrouiller_la_tete(tx, caisse)?;
            let reglages = lire_reglages(tx, false)?;
            let produits = charger_produits(tx, &cles)?;
            let derniere = derniere_piece(tx, caisse, dernier_numero, &dernier_hash)?;
            let mut stock =
                Stock::partiel(produits.clone(), reglages, derniere.into_iter().collect());
            let piece = stock.encaisser(demande, moyen, recu, quand)?.clone();
            ecrire_les_effets(tx, caisse, &produits, &stock, &piece)?;
            Ok(piece)
        })
    }

    fn annuler(&mut self, numero: u64, quand: Horodatage) -> Result<Piece, ErreurStock> {
        self.ecrire_une_piece(|tx, caisse| {
            let (dernier_numero, dernier_hash) = verrouiller_la_tete(tx, caisse)?;
            let reglages = lire_reglages(tx, false)?;
            let derniere = derniere_piece(tx, caisse, dernier_numero, &dernier_hash)?;
            let origine = charger_piece(tx, caisse, numero, true)?;
            let numero_annulation: Option<u64> = tx.exec_first(
                "SELECT numero FROM piece WHERE caisse = ? AND annule_numero = ?",
                (caisse, numero),
            )?;
            let annulation = match numero_annulation {
                Some(n) => charger_piece(tx, caisse, n, true)?,
                None => None,
            };
            let cles: Vec<String> = origine
                .iter()
                .flat_map(|p| p.lignes.iter().map(|l| cle(&l.nom)))
                .collect();
            let mut pieces: Vec<Piece> = [origine, annulation, derniere]
                .into_iter()
                .flatten()
                .collect();
            pieces.sort_by_key(|p| p.numero);
            pieces.dedup_by_key(|p| p.numero);
            let produits = charger_produits(tx, &cles)?;
            let mut stock = Stock::partiel(produits.clone(), reglages, pieces);
            let piece = stock.annuler(numero, quand)?.clone();
            ecrire_les_effets(tx, caisse, &produits, &stock, &piece)?;
            Ok(piece)
        })
    }

    fn verifier_journal(&mut self) -> Result<(), ErreurStock> {
        self.assurer_la_connexion()?;
        let caisse = self.config.caisse.clone();
        let options = TxOpts::default()
            .set_isolation_level(Some(IsolationLevel::RepeatableRead))
            .set_with_consistent_snapshot(true)
            .set_access_mode(Some(AccessMode::ReadOnly));
        let connexion = &mut self.connexion;
        let resultat = (|| -> Result<(), Echec> {
            let mut tx = connexion.start_transaction(options)?;
            verifier_la_chaine(&mut tx, &caisse)?;
            tx.commit()?;
            Ok(())
        })();
        match resultat {
            Ok(()) => Ok(()),
            Err(Echec::Regle(e)) => Err(e),
            Err(Echec::Serveur(e)) => Err(erreur_de_base(&e, &self.config)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn la_cle_d_un_nom_est_nettoyee_et_en_minuscules_accents_gardes() {
        assert_eq!(cle("  Riz 5KG "), "riz 5kg");
        assert_eq!(cle("Thé"), "thé");
        assert_ne!(cle("Thé"), cle("the"));
    }

    #[test]
    fn la_longueur_d_un_nom_se_compte_en_caracteres() {
        assert!(verifier_longueur(&"é".repeat(190)).is_ok());
        assert!(verifier_longueur(&"é".repeat(191)).is_err());
        // 190 emoji : 190 caractères (mais 760 octets) : accepté.
        assert!(verifier_longueur(&"🥖".repeat(190)).is_ok());
    }
}
