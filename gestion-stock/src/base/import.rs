//! Importer un `stock.txt` dans la base : produits, mouvements, réglages et journal (pièces avec
//! leurs empreintes TELLES QUELLES), dans la caisse de ce poste, en UNE transaction.
//!
//! Le fichier est lu et sa chaîne d'empreintes vérifiée avant toute écriture ; la base doit être
//! vide ; la chaîne est revérifiée dans la base avant la validation. Si quoi que ce soit échoue,
//! rien n'est écrit. Le fichier n'est jamais modifié.

use super::depot_mysql::{
    Echec, cle, connexion_verifiee, ecrire_reglages, verifier_la_chaine, verifier_longueur,
    verrouiller_la_tete,
};
use super::{Configuration, erreur_de_base};
use crate::erreur::ErreurStock;
use crate::stock::Genre;
use crate::stockage::charger;
use crate::vente::GenrePiece;
use mysql::prelude::Queryable;
use mysql::{Transaction, TxOpts, Value};
use std::path::Path;

/// Ce que l'import a écrit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResumeImport {
    pub produits: usize,
    pub mouvements: usize,
    pub pieces: usize,
}

/// Au plus ce nombre de lignes par requête…
const LIGNES_PAR_REQUETE: usize = 500;
/// …et au plus ce nombre d'octets (estimés) : le serveur de XAMPP refuse les paquets de plus
/// de 1 Mo, et 500 noms de 190 caractères en feraient presque autant.
const OCTETS_PAR_REQUETE: usize = 200_000;

fn taille(valeur: &Value) -> usize {
    match valeur {
        Value::Bytes(octets) => octets.len() + 8,
        _ => 16,
    }
}

/// Insère `lignes` dans `table` par paquets (voir les deux constantes ci-dessus).
fn inserer(
    tx: &mut Transaction<'_>,
    table: &str,
    colonnes: &[&str],
    lignes: impl Iterator<Item = Vec<Value>>,
) -> Result<(), mysql::Error> {
    let marques = format!("({})", vec!["?"; colonnes.len()].join(", "));
    let envoyer = |tx: &mut Transaction<'_>, nombre: usize, valeurs: Vec<Value>| {
        let requete = format!(
            "INSERT INTO {table} ({}) VALUES {}",
            colonnes.join(", "),
            vec![marques.as_str(); nombre].join(", ")
        );
        tx.exec_drop(requete, valeurs)
    };
    let mut valeurs: Vec<Value> = Vec::new();
    let (mut nombre, mut octets) = (0, 0);
    for ligne in lignes {
        octets += ligne.iter().map(taille).sum::<usize>();
        valeurs.extend(ligne);
        nombre += 1;
        if nombre == LIGNES_PAR_REQUETE || octets >= OCTETS_PAR_REQUETE {
            envoyer(tx, nombre, std::mem::take(&mut valeurs))?;
            (nombre, octets) = (0, 0);
        }
    }
    if nombre > 0 {
        envoyer(tx, nombre, valeurs)?;
    }
    Ok(())
}

/// Importe `fichier` dans la base de `config`.
pub fn importer(config: &Configuration, fichier: &Path) -> Result<ResumeImport, ErreurStock> {
    if !fichier.exists() {
        return Err(ErreurStock::Base(format!(
            "Le fichier « {} » n'existe pas.",
            fichier.display()
        )));
    }
    let stock = charger(fichier)?;
    stock.verifier_journal()?;
    // Ce que la colonne ne pourrait pas garder entier est refusé AVANT d'écrire quoi que ce soit.
    let noms = stock
        .produits
        .iter()
        .map(|p| p.nom.as_str())
        .chain(stock.historique.iter().map(|m| m.nom.as_str()))
        .chain(
            stock
                .pieces
                .iter()
                .flat_map(|p| p.lignes.iter().map(|l| l.nom.as_str())),
        );
    for nom in noms {
        verifier_longueur(nom)?;
    }

    let mut connexion = connexion_verifiee(config)?;
    let caisse = config.caisse.as_str();
    let resultat = (|| -> Result<(), Echec> {
        let mut tx = connexion.start_transaction(TxOpts::default())?;
        verrouiller_la_tete(&mut tx, caisse)?;
        let occupe: u64 = tx
            .query_first(
                "SELECT (SELECT COUNT(*) FROM produit) + (SELECT COUNT(*) FROM mouvement) \
                 + (SELECT COUNT(*) FROM piece)",
            )?
            .unwrap_or(0);
        if occupe > 0 {
            return Err(Echec::Regle(ErreurStock::Base(
                "La base n'est pas vide : l'import ne se fait que dans une base sans produit, sans mouvement et sans pièce."
                    .to_string(),
            )));
        }
        ecrire_reglages(&mut tx, &stock.reglages)?;
        inserer(
            &mut tx,
            "produit",
            &["nom", "nom_cle", "quantite", "seuil", "prix", "taux_bp"],
            stock.produits.iter().map(|p| {
                vec![
                    p.nom.as_str().into(),
                    cle(&p.nom).into(),
                    p.quantite.into(),
                    p.seuil.into(),
                    p.prix.map(|m| m.0).into(),
                    p.taux_bp.into(),
                ]
            }),
        )?;
        inserer(
            &mut tx,
            "mouvement",
            &["produit_nom", "produit_cle", "genre", "quantite", "caisse"],
            stock.historique.iter().map(|m| {
                let genre = match m.genre {
                    Genre::Entree => "entree",
                    Genre::Sortie => "sortie",
                };
                vec![
                    m.nom.as_str().into(),
                    cle(&m.nom).into(),
                    genre.into(),
                    m.quantite.into(),
                    caisse.into(),
                ]
            }),
        )?;
        inserer(
            &mut tx,
            "piece",
            &[
                "caisse",
                "numero",
                "genre",
                "annule_numero",
                "utc",
                "decalage_minutes",
                "total",
                "moyen",
                "recu",
                "rendu",
                "hash_precedent",
                "hash",
            ],
            stock.pieces.iter().map(|p| {
                let (genre, annule) = match p.genre {
                    GenrePiece::Vente => ("V", None),
                    GenrePiece::Annulation { de } => ("A", Some(de)),
                };
                vec![
                    caisse.into(),
                    p.numero.into(),
                    genre.into(),
                    annule.into(),
                    p.horodatage.utc.into(),
                    p.horodatage.decalage_minutes.into(),
                    p.total.0.into(),
                    p.moyen.code().into(),
                    p.recu.0.into(),
                    p.rendu.0.into(),
                    p.hash_precedent.as_str().into(),
                    p.hash.as_str().into(),
                ]
            }),
        )?;
        inserer(
            &mut tx,
            "piece_ligne",
            &[
                "caisse", "numero", "rang", "nom", "nom_cle", "quantite", "prix", "taux_bp",
            ],
            stock.pieces.iter().flat_map(|p| {
                p.lignes.iter().enumerate().map(move |(rang, l)| {
                    vec![
                        caisse.into(),
                        p.numero.into(),
                        (rang as u64).into(),
                        l.nom.as_str().into(),
                        cle(&l.nom).into(),
                        l.quantite.into(),
                        l.prix.0.into(),
                        l.taux_bp.into(),
                    ]
                })
            }),
        )?;
        if let Some(derniere) = stock.pieces.last() {
            tx.exec_drop(
                "UPDATE caisse SET dernier_numero = ?, dernier_hash = ? WHERE code = ?",
                (derniere.numero, &derniere.hash, caisse),
            )?;
        }
        // La chaîne relue dans la base doit être aussi saine que celle du fichier : sinon on
        // annule tout (la transaction n'est pas validée).
        verifier_la_chaine(&mut tx, caisse)?;
        tx.commit()?;
        Ok(())
    })();
    match resultat {
        Ok(()) => Ok(ResumeImport {
            produits: stock.produits.len(),
            mouvements: stock.historique.len(),
            pieces: stock.pieces.len(),
        }),
        Err(Echec::Regle(e)) => Err(e),
        Err(Echec::Serveur(e)) => Err(erreur_de_base(&e, config)),
    }
}
