//! État de l'application : toute la logique, sans aucune fenêtre (donc testable).
//!
//! Règle commune à toutes les actions : on travaille sur une COPIE du stock ; si l'action
//! réussit on sauvegarde la copie, et seulement si la sauvegarde réussit la copie remplace
//! le stock. Ainsi la liste affichée ne diffère jamais du fichier, même si l'écriture échoue.

use crate::catalogue::{CATALOGUE, Suggestion, suggerer};
use crate::horloge;
use gestion_stock::erreur::ErreurStock;
use gestion_stock::horodatage::Horodatage;
use gestion_stock::monnaie::{Devise, Montant, lire_taux};
use gestion_stock::reglages::{Profil, Reglages};
use gestion_stock::stock::{Genre, Stock};
use gestion_stock::stockage::{charger, sauvegarder};
use gestion_stock::vente::{Devis, GenrePiece, GroupeTaux, Moyen, Piece, detail_taxes};
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

/// Le message affiché en bas de la fenêtre après chaque action.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Retour {
    Succes(String),
    Erreur(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LigneProduit {
    pub nom: String,
    pub quantite: u32,
    pub seuil: u32,
    pub en_alerte: bool,
    /// `None` : le produit n'a pas de prix, on ne peut pas le vendre.
    pub prix: Option<Montant>,
    /// `None` : le produit suit le taux par défaut des réglages.
    pub taux_bp: Option<u32>,
}

/// Où en est un règlement en espèces, pour l'afficher pendant la saisie.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reglement {
    /// Rien n'est encore saisi (ou il n'y a rien à payer) : on n'affiche rien.
    Attente,
    /// Ce qui est saisi n'est pas un montant.
    Illisible,
    /// Le client n'a pas donné assez : il manque cette somme.
    Manque(Montant),
    /// Le compte est bon (0) ou il y a cette monnaie à rendre.
    Rendu(Montant),
}

/// Une ligne de la liste des ventes, déjà écrite pour l'affichage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LignePiece {
    pub numero: u64,
    /// La date et l'heure locales : `14/11/2023 23:13`.
    pub quand: String,
    /// `Vente`, ou `Annulation de la vente n° 3`.
    pub libelle: String,
    pub total: String,
    pub moyen: String,
    /// Vrai pour une vente qui n'a pas encore été annulée.
    pub annulable: bool,
}

/// Les trois chiffres du haut de la fenêtre.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Resume {
    pub produits: usize,
    pub en_alerte: usize,
    /// Total des quantités en stock, tous produits confondus.
    pub unites: u64,
}

/// La façon de lire l'heure : la vraie par défaut, une heure fixe dans les tests.
struct Horloge(Box<dyn Fn() -> Horodatage>);

impl std::fmt::Debug for Horloge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Horloge")
    }
}

#[derive(Debug)]
pub struct Etat {
    stock: Stock,
    chemin: PathBuf,
    pub retour: Option<Retour>,
    horloge: Horloge,
    /// Le numéro de la pièce dont on affiche le ticket.
    ticket: Option<u64>,
    /// L'état du journal de caisse, vérifié à l'ouverture et après chaque action.
    integrite: Result<(), String>,
}

const JOURNAL_BLOQUE: &str = "Le journal de caisse a été modifié en dehors du programme : les ventes sont bloquées. Le stock reste utilisable ; restaurez une copie du fichier de données pour vendre à nouveau.";

fn libelle_moyen(moyen: Moyen) -> &'static str {
    match moyen {
        Moyen::Especes => "Espèces",
        Moyen::Carte => "Carte",
        Moyen::Autre => "Autre",
    }
}

fn libelle_profil(profil: Profil) -> &'static str {
    match profil {
        Profil::Europe => "Europe (prix TTC)",
        Profil::EtatsUnis => "États-Unis (prix hors taxe)",
    }
}

/// Un prix vide veut dire « pas de prix ».
fn lire_prix(devise: Devise, texte: &str) -> Result<Option<Montant>, ErreurStock> {
    match texte.trim() {
        "" => Ok(None),
        saisie => devise.lire(saisie).map(Some),
    }
}

/// Un taux vide veut dire « le taux par défaut des réglages ».
fn lire_taux_facultatif(texte: &str) -> Result<Option<u32>, ErreurStock> {
    match texte.trim() {
        "" => Ok(None),
        saisie => lire_taux(saisie).map(Some),
    }
}

/// Le montant donné par le client en espèces : obligatoire.
fn lire_recu(devise: Devise, texte: &str) -> Result<Montant, ErreurStock> {
    if texte.trim().is_empty() {
        return Err(ErreurStock::Utilisation(
            "Montant reçu manquant. Saisissez la somme donnée par le client, par exemple 20."
                .to_string(),
        ));
    }
    devise.lire(texte)
}

fn lire_quantite(texte: &str) -> Result<u32, ErreurStock> {
    let texte = texte.trim();
    if texte.is_empty() {
        return Err(ErreurStock::Utilisation(
            "Quantité manquante. Saisissez un nombre entier positif, par exemple 12.".to_string(),
        ));
    }
    texte.parse().map_err(|_| {
        ErreurStock::Utilisation(format!(
            "Quantité invalide : « {texte} ». Saisissez un nombre entier positif, par exemple 12."
        ))
    })
}

/// Un seuil vide veut dire 0.
fn lire_seuil(texte: &str) -> Result<u32, ErreurStock> {
    let texte = texte.trim();
    if texte.is_empty() {
        return Ok(0);
    }
    texte.parse().map_err(|_| {
        ErreurStock::Utilisation(format!(
            "Seuil d'alerte invalide : « {texte} ». Saisissez un nombre entier positif, par exemple 5, ou laissez vide pour 0."
        ))
    })
}

/// Le chemin tel qu'on l'affiche : toujours absolu, pour que la personne sache exactement
/// quel fichier est utilisé, même si `STOCK_FICHIER` est un chemin relatif.
pub fn chemin_complet(chemin: &Path) -> String {
    std::path::absolute(chemin)
        .unwrap_or_else(|_| chemin.to_path_buf())
        .display()
        .to_string()
}

const FICHIER_MODIFIE: &str = "Le fichier de données a été modifié par un autre programme ou une autre fenêtre : rien n'a été enregistré. Fermez puis rouvrez le programme pour voir le stock à jour.";

impl Etat {
    /// Crée le dossier du fichier s'il manque. Un fichier absent donne un stock vide,
    /// et on le dit ; un fichier abîmé est une erreur (il n'est jamais écrasé).
    pub fn ouvrir(chemin: PathBuf) -> Result<Etat, ErreurStock> {
        if let Some(dossier) = chemin.parent().filter(|d| !d.as_os_str().is_empty()) {
            std::fs::create_dir_all(dossier).map_err(|e| {
                let raison = match e.kind() {
                    ErrorKind::PermissionDenied => "accès refusé".to_string(),
                    _ => e.to_string(),
                };
                ErreurStock::Io(format!(
                    "impossible de créer le dossier « {} » : {raison}.",
                    dossier.display()
                ))
            })?;
        }
        let existait = chemin.exists();
        let stock = charger(&chemin)?;
        let integrite = stock.verifier_journal().map_err(|e| e.to_string());
        let retour = (!existait).then(|| {
            Retour::Succes(
                "Aucun fichier de données trouvé : un stock vide est utilisé. Le fichier sera créé au premier enregistrement."
                    .to_string(),
            )
        });
        Ok(Etat {
            stock,
            chemin,
            retour,
            horloge: Horloge(Box::new(horloge::maintenant)),
            ticket: None,
            integrite,
        })
    }

    /// Remplace l'horloge (pour les tests : une heure fixe).
    pub fn avec_horloge(mut self, horloge: impl Fn() -> Horodatage + 'static) -> Etat {
        self.horloge = Horloge(Box::new(horloge));
        self
    }

    pub fn chemin(&self) -> &Path {
        &self.chemin
    }

    /// L'heure locale, `23:13`, pour l'en-tête.
    pub fn heure_locale(&self) -> String {
        let maintenant = (self.horloge.0)().local();
        format!("{:02}:{:02}", maintenant.heure, maintenant.minute)
    }

    /// Produits triés par nom ; `recherche` : texte contenu dans le nom, sans casse
    /// ni espaces autour.
    pub fn lignes(&self, recherche: &str, alertes_seulement: bool) -> Vec<LigneProduit> {
        let cherche = recherche.trim().to_lowercase();
        self.stock
            .produits()
            .into_iter()
            .filter(|p| p.nom.to_lowercase().contains(&cherche))
            .filter(|p| !alertes_seulement || p.en_alerte())
            .map(|p| LigneProduit {
                nom: p.nom.clone(),
                quantite: p.quantite,
                seuil: p.seuil,
                en_alerte: p.en_alerte(),
                prix: p.prix,
                taux_bp: p.taux_bp,
            })
            .collect()
    }

    /// Des produits courants à proposer pour ce qui est tapé (voir `catalogue`), sans ceux qui
    /// sont déjà en stock.
    pub fn suggestions(&self, saisie: &str, limite: usize) -> Vec<&'static Suggestion> {
        let deja: Vec<String> = self
            .stock
            .produits()
            .iter()
            .map(|p| p.nom.clone())
            .collect();
        suggerer(CATALOGUE, saisie, &deja, limite)
    }

    pub fn resume(&self) -> Resume {
        let produits = self.stock.produits();
        Resume {
            produits: produits.len(),
            en_alerte: produits.iter().filter(|p| p.en_alerte()).count(),
            unites: produits.iter().map(|p| u64::from(p.quantite)).sum(),
        }
    }

    /// Du plus récent au plus ancien.
    pub fn historique(&self, limite: usize) -> Vec<String> {
        self.stock
            .historique()
            .iter()
            .rev()
            .take(limite)
            .map(|m| match m.genre {
                Genre::Entree => format!("Entrée +{} {}", m.quantite, m.nom),
                Genre::Sortie => format!("Sortie -{} {}", m.quantite, m.nom),
            })
            .collect()
    }

    /// Refuse d'enregistrer si le fichier n'est plus tel que cet écran l'a chargé : sinon on
    /// l'écraserait avec un stock périmé et les mouvements faits ailleurs (autre fenêtre,
    /// programme en ligne de commande) seraient perdus sans un mot.
    fn verifier_fichier_inchange(&self) -> Result<(), ErreurStock> {
        let sur_disque = charger(&self.chemin)?;
        if sur_disque.produits() == self.stock.produits()
            && sur_disque.historique() == self.stock.historique()
            && sur_disque.pieces() == self.stock.pieces()
            && sur_disque.reglages() == self.stock.reglages()
        {
            Ok(())
        } else {
            Err(ErreurStock::Utilisation(FICHIER_MODIFIE.to_string()))
        }
    }

    /// Vérifie le fichier, applique l'action sur une copie, sauvegarde, puis adopte la copie
    /// (voir l'en-tête).
    fn appliquer(
        &mut self,
        action: impl FnOnce(&mut Stock) -> Result<String, ErreurStock>,
    ) -> bool {
        let resultat = self.verifier_fichier_inchange().and_then(|()| {
            let mut copie = self.stock.clone();
            let message = action(&mut copie)?;
            sauvegarder(&copie, &self.chemin)?;
            Ok((copie, message))
        });
        match resultat {
            Ok((copie, message)) => {
                self.stock = copie;
                self.integrite = self.stock.verifier_journal().map_err(|e| e.to_string());
                self.retour = Some(Retour::Succes(message));
                true
            }
            Err(e) => {
                self.retour = Some(Retour::Erreur(e.to_string()));
                false
            }
        }
    }

    /// Un seuil vide veut dire 0.
    pub fn ajouter(&mut self, nom: &str, seuil: &str) -> bool {
        self.ajouter_avec_prix(nom, seuil, "", "")
    }

    /// Ajoute un produit avec son prix et son taux, en une seule opération : si l'un des champs
    /// est refusé, le produit n'est pas ajouté. Prix et taux vides : pas de prix, taux par défaut.
    pub fn ajouter_avec_prix(&mut self, nom: &str, seuil: &str, prix: &str, taux: &str) -> bool {
        self.appliquer(|stock| {
            let devise = stock.reglages().devise;
            let seuil = lire_seuil(seuil)?;
            let prix = lire_prix(devise, prix)?;
            let taux = lire_taux_facultatif(taux)?;
            stock.ajouter_produit(nom, seuil)?;
            if prix.is_some() || taux.is_some() {
                stock.definir_prix(nom, prix, taux)?;
            }
            let mut message = format!(
                "Produit « {} » ajouté (seuil d'alerte : {seuil}).",
                nom.trim()
            );
            if let Some(prix) = prix {
                message.push_str(&format!(" Prix : {}.", devise.formater(prix)));
            }
            Ok(message)
        })
    }

    pub fn entree(&mut self, nom: &str, quantite: &str) -> bool {
        self.appliquer(|stock| {
            let quantite = lire_quantite(quantite)?;
            let produit = stock.enregistrer_entree(nom, quantite)?;
            Ok(format!(
                "Entrée enregistrée : +{quantite} {}, stock actuel : {}.",
                produit.nom, produit.quantite
            ))
        })
    }

    pub fn sortie(&mut self, nom: &str, quantite: &str) -> bool {
        self.appliquer(|stock| {
            let quantite = lire_quantite(quantite)?;
            let produit = stock.enregistrer_sortie(nom, quantite)?;
            let mut texte = format!(
                "Sortie enregistrée : -{quantite} {}, stock actuel : {}.",
                produit.nom, produit.quantite
            );
            if produit.en_alerte() {
                texte.push_str(&format!(
                    " ATTENTION : stock bas (seuil d'alerte : {}).",
                    produit.seuil
                ));
            }
            Ok(texte)
        })
    }

    pub fn renommer(&mut self, ancien: &str, nouveau: &str) -> bool {
        self.appliquer(|stock| {
            stock.renommer_produit(ancien, nouveau)?;
            Ok(format!(
                "Produit renommé : « {} » devient « {} ».",
                ancien.trim(),
                nouveau.trim()
            ))
        })
    }

    pub fn supprimer(&mut self, nom: &str) -> bool {
        self.appliquer(|stock| {
            stock.supprimer_produit(nom)?;
            Ok(format!(
                "Produit « {} » supprimé (l'historique est conservé).",
                nom.trim()
            ))
        })
    }

    // ---- Réglages et prix --------------------------------------------------------------------

    pub fn reglages(&self) -> Reglages {
        self.stock.reglages()
    }

    /// Les réglages ne se changent plus après la première vente : le journal garde les montants
    /// dans la devise d'origine.
    pub fn reglages_verrouilles(&self) -> bool {
        !self.stock.pieces().is_empty()
    }

    /// Le prix et le taux d'un produit écrits comme dans un champ de saisie (vides s'il n'y en
    /// a pas), pour pré-remplir le formulaire : réenregistrer sans rien changer ne change rien.
    pub fn saisies_du_prix(&self, nom: &str) -> (String, String) {
        let devise = self.stock.reglages().devise;
        self.lignes("", false)
            .into_iter()
            .find(|ligne| ligne.nom == nom)
            .map_or_else(Default::default, |ligne| {
                (
                    ligne
                        .prix
                        .map(|p| devise.formater_brut(p))
                        .unwrap_or_default(),
                    ligne
                        .taux_bp
                        .map(|t| devise.formater_taux_brut(t))
                        .unwrap_or_default(),
                )
            })
    }

    /// Un montant écrit avec la devise des réglages.
    pub fn formater(&self, montant: Montant) -> String {
        self.stock.reglages().devise.formater(montant)
    }

    pub fn definir_reglages(&mut self, profil: Profil, devise: Devise, taux: &str) -> bool {
        self.appliquer(|stock| {
            let taux_defaut_bp = lire_taux(taux)?;
            stock.definir_reglages(Reglages {
                profil,
                devise,
                taux_defaut_bp,
            })?;
            Ok(format!(
                "Réglages enregistrés : {}, {}, taux par défaut {}.",
                libelle_profil(profil),
                devise.code(),
                devise.formater_taux(taux_defaut_bp)
            ))
        })
    }

    /// Prix et taux vides : le produit n'a plus de prix (on ne peut plus le vendre) et reprend
    /// le taux par défaut.
    pub fn definir_prix(&mut self, nom: &str, prix: &str, taux: &str) -> bool {
        self.appliquer(|stock| {
            let devise = stock.reglages().devise;
            let prix = lire_prix(devise, prix)?;
            let taux = lire_taux_facultatif(taux)?;
            stock.definir_prix(nom, prix, taux)?;
            Ok(match prix {
                None => format!(
                    "Prix de « {} » retiré : le produit ne peut plus être vendu.",
                    nom.trim()
                ),
                Some(prix) => {
                    let taux = taux.map_or(String::new(), |t| {
                        format!(" (taux : {})", devise.formater_taux(t))
                    });
                    format!(
                        "Prix de « {} » : {}{taux}.",
                        nom.trim(),
                        devise.formater(prix)
                    )
                }
            })
        })
    }

    // ---- La vente ----------------------------------------------------------------------------

    /// Le devis d'un panier, pour afficher les totaux en direct. Ne change rien.
    pub fn devis(&self, panier: &[(String, u32)]) -> Result<Devis, ErreurStock> {
        self.stock.devis(panier)
    }

    /// Refuse de vendre (et le dit) si le journal a été modifié en dehors du programme.
    fn ventes_autorisees(&mut self) -> bool {
        if self.integrite.is_err() {
            self.retour = Some(Retour::Erreur(JOURNAL_BLOQUE.to_string()));
            return false;
        }
        true
    }

    /// Encaisse le panier. `recu` n'est lu que pour un paiement en espèces.
    pub fn encaisser(&mut self, panier: &[(String, u32)], moyen: Moyen, recu: &str) -> bool {
        if !self.ventes_autorisees() {
            return false;
        }
        let quand = (self.horloge.0)();
        let mut numero = None;
        let reussi = self.appliquer(|stock| {
            let devise = stock.reglages().devise;
            let recu = match moyen {
                Moyen::Especes => lire_recu(devise, recu)?,
                // La bibliothèque ignore le montant reçu pour la carte et les autres moyens.
                Moyen::Carte | Moyen::Autre => Montant::ZERO,
            };
            let piece = stock.encaisser(panier, moyen, recu, quand)?;
            numero = Some(piece.numero);
            Ok(format!(
                "Vente n° {} enregistrée : {}.",
                piece.numero,
                devise.formater(piece.total)
            ))
        });
        if reussi {
            self.ticket = numero;
        }
        reussi
    }

    /// Annule une vente en entier : le stock est remis et une écriture inverse est ajoutée.
    pub fn annuler(&mut self, numero: u64) -> bool {
        if !self.ventes_autorisees() {
            return false;
        }
        let quand = (self.horloge.0)();
        let mut annulation = None;
        let reussi = self.appliquer(|stock| {
            annulation = Some(stock.annuler(numero, quand)?.numero);
            Ok(format!("Vente n° {numero} annulée : stock remis."))
        });
        if reussi {
            self.ticket = annulation;
        }
        reussi
    }

    /// La pièce dont on affiche le ticket : celle de la dernière vente ou annulation faite ici.
    pub fn derniere_piece(&self) -> Option<&Piece> {
        let numero = self.ticket?;
        self.stock.pieces().iter().find(|p| p.numero == numero)
    }

    /// Le détail par taux d'une pièce, pour son ticket. Calculé depuis les lignes de la pièce
    /// (qui gardent leur prix et leur taux du moment), donc indépendant des prix d'aujourd'hui.
    pub fn detail_taxes(&self, piece: &Piece) -> Vec<GroupeTaux> {
        detail_taxes(&piece.lignes, self.stock.reglages().profil)
            .map(|(groupes, _)| groupes)
            .unwrap_or_default()
    }

    /// Ce que vaut le montant tapé pour payer `total` en espèces. Ne change rien : c'est
    /// `encaisser` qui décide, ceci sert seulement à renseigner la personne en direct.
    pub fn reglement_especes(&self, total: Montant, recu: &str) -> Reglement {
        if total <= Montant::ZERO || recu.trim().is_empty() {
            return Reglement::Attente;
        }
        match self.stock.reglages().devise.lire(recu) {
            Ok(recu) if recu < Montant::ZERO => Reglement::Illisible,
            Ok(recu) if recu < total => Reglement::Manque(Montant(total.0 - recu.0)),
            Ok(recu) => Reglement::Rendu(Montant(recu.0 - total.0)),
            Err(_) => Reglement::Illisible,
        }
    }

    /// Ferme le ticket affiché : la personne prépare la vente suivante.
    pub fn effacer_ticket(&mut self) {
        self.ticket = None;
    }

    /// Les pièces du journal, de la plus récente à la plus ancienne.
    pub fn pieces_recentes(&self, limite: usize) -> Vec<LignePiece> {
        let devise = self.stock.reglages().devise;
        let pieces = self.stock.pieces();
        pieces
            .iter()
            .rev()
            .take(limite)
            .map(|piece| {
                let (libelle, annulable) = match piece.genre {
                    GenrePiece::Vente => (
                        "Vente".to_string(),
                        !pieces
                            .iter()
                            .any(|p| p.genre == GenrePiece::Annulation { de: piece.numero }),
                    ),
                    GenrePiece::Annulation { de } => {
                        (format!("Annulation de la vente n° {de}"), false)
                    }
                };
                LignePiece {
                    numero: piece.numero,
                    quand: piece.horodatage.local().en_francais(),
                    libelle,
                    total: devise.formater(piece.total),
                    moyen: libelle_moyen(piece.moyen).to_string(),
                    annulable,
                }
            })
            .collect()
    }

    /// L'état du journal de caisse : `Err` avec le message à afficher s'il a été modifié.
    pub fn integrite(&self) -> Result<(), String> {
        self.integrite.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chemin_temp(nom: &str) -> PathBuf {
        let chemin = std::env::temp_dir().join(format!(
            "gestion-stock-gui-{}-{nom}.txt",
            std::process::id()
        ));
        let _ = std::fs::remove_file(&chemin);
        chemin
    }

    /// « Riz 5kg », seuil 5, stock 12.
    fn etat_avec_riz(nom: &str) -> Etat {
        let mut e = Etat::ouvrir(chemin_temp(nom)).unwrap();
        assert!(e.ajouter("Riz 5kg", "5"));
        assert!(e.entree("Riz 5kg", "12"));
        e
    }

    #[test]
    fn ouvrir_un_fichier_absent_donne_un_stock_vide_et_le_dit() {
        let e = Etat::ouvrir(chemin_temp("absent")).unwrap();
        assert!(e.lignes("", false).is_empty());
        assert_eq!(
            e.retour,
            Some(Retour::Succes(
                "Aucun fichier de données trouvé : un stock vide est utilisé. Le fichier sera créé au premier enregistrement."
                    .into()
            ))
        );
    }

    #[test]
    fn ouvrir_cree_le_dossier_des_donnees() {
        let dossier =
            std::env::temp_dir().join(format!("gestion-stock-gui-{}-dossier", std::process::id()));
        let _ = std::fs::remove_dir_all(&dossier);
        let mut e = Etat::ouvrir(dossier.join("stock.txt")).unwrap();
        assert!(e.ajouter("Riz", ""));
        assert!(dossier.join("stock.txt").exists());
        std::fs::remove_dir_all(&dossier).unwrap();
    }

    #[test]
    fn ouvrir_un_fichier_abime_renvoie_l_erreur_sans_toucher_au_fichier() {
        let c = chemin_temp("abime");
        std::fs::write(&c, "P;Riz;12;5\nn'importe quoi\n").unwrap();
        assert_eq!(
            Etat::ouvrir(c.clone()).unwrap_err(),
            ErreurStock::FichierCorrompu { ligne: 2 }
        );
        assert_eq!(
            std::fs::read_to_string(&c).unwrap(),
            "P;Riz;12;5\nn'importe quoi\n"
        );
    }

    #[test]
    fn ajouter_valide_sauvegarde_et_confirme() {
        let c = chemin_temp("ajout");
        let mut e = Etat::ouvrir(c.clone()).unwrap();
        assert!(e.ajouter("  Riz 5kg ", "5"));
        assert_eq!(
            e.retour,
            Some(Retour::Succes(
                "Produit « Riz 5kg » ajouté (seuil d'alerte : 5).".into()
            ))
        );
        assert_eq!(
            e.lignes("", false),
            [LigneProduit {
                nom: "Riz 5kg".into(),
                quantite: 0,
                seuil: 5,
                en_alerte: true,
                prix: None,
                taux_bp: None
            }]
        );
        assert!(
            std::fs::read_to_string(&c)
                .unwrap()
                .contains("P;Riz 5kg;0;5")
        );
    }

    #[test]
    fn ajouter_sans_seuil_utilise_zero() {
        let mut e = Etat::ouvrir(chemin_temp("sans-seuil")).unwrap();
        assert!(e.ajouter("Sel", "  "));
        assert_eq!(e.lignes("", false)[0].seuil, 0);
    }

    #[test]
    fn saisies_invalides_de_l_ajout_ne_changent_rien() {
        for (nom, seuil) in [
            ("", "5"),
            ("a;b", "5"),
            ("Riz", "abc"),
            ("Riz", "-1"),
            ("Riz", "2,5"),
        ] {
            let mut e = Etat::ouvrir(chemin_temp("invalide")).unwrap();
            assert!(!e.ajouter(nom, seuil), "{nom:?} {seuil:?}");
            assert!(matches!(e.retour, Some(Retour::Erreur(_))));
            assert!(e.lignes("", false).is_empty());
        }
        let mut e = Etat::ouvrir(chemin_temp("seuil")).unwrap();
        e.ajouter("Riz", "abc");
        assert_eq!(
            e.retour,
            Some(Retour::Erreur(
                "Seuil d'alerte invalide : « abc ». Saisissez un nombre entier positif, par exemple 5, ou laissez vide pour 0."
                    .into()
            ))
        );
    }

    #[test]
    fn entree_et_sortie_confirment_et_signalent_le_stock_bas() {
        let mut e = etat_avec_riz("mouvements");
        assert!(e.entree("riz 5kg", "3"));
        assert_eq!(
            e.retour,
            Some(Retour::Succes(
                "Entrée enregistrée : +3 Riz 5kg, stock actuel : 15.".into()
            ))
        );
        assert!(e.sortie("Riz 5kg", "2"));
        assert_eq!(
            e.retour,
            Some(Retour::Succes(
                "Sortie enregistrée : -2 Riz 5kg, stock actuel : 13.".into()
            ))
        );
        assert!(e.sortie("Riz 5kg", "9"));
        assert_eq!(
            e.retour,
            Some(Retour::Succes(
                "Sortie enregistrée : -9 Riz 5kg, stock actuel : 4. ATTENTION : stock bas (seuil d'alerte : 5)."
                    .into()
            ))
        );
    }

    #[test]
    fn quantites_invalides_donnent_un_message_clair() {
        let mut e = etat_avec_riz("quantites");
        for q in ["", "  ", "abc", "-3", "2,5", "99999999999"] {
            assert!(!e.entree("Riz 5kg", q), "{q:?}");
            assert!(!e.sortie("Riz 5kg", q), "{q:?}");
            assert!(
                matches!(&e.retour, Some(Retour::Erreur(m)) if m.starts_with("Quantité ")),
                "{q:?}"
            );
        }
        e.entree("Riz 5kg", "");
        assert_eq!(
            e.retour,
            Some(Retour::Erreur(
                "Quantité manquante. Saisissez un nombre entier positif, par exemple 12.".into()
            ))
        );
        e.entree("Riz 5kg", "abc");
        assert_eq!(
            e.retour,
            Some(Retour::Erreur(
                "Quantité invalide : « abc ». Saisissez un nombre entier positif, par exemple 12."
                    .into()
            ))
        );
        assert!(!e.sortie("Riz 5kg", "0"));
        assert_eq!(
            e.retour,
            Some(Retour::Erreur(
                "La quantité doit être supérieure à zéro.".into()
            ))
        );
        assert_eq!(e.lignes("", false)[0].quantite, 12);
    }

    #[test]
    fn sortie_trop_grande_refusee_sans_toucher_au_fichier() {
        let mut e = etat_avec_riz("trop-grande");
        let avant = std::fs::read_to_string(e.chemin()).unwrap();
        assert!(!e.sortie("Riz 5kg", "99"));
        assert_eq!(
            e.retour,
            Some(Retour::Erreur(
                "Stock insuffisant pour « Riz 5kg » : 12 disponible(s), 99 demandé(s).".into()
            ))
        );
        assert_eq!(e.lignes("", false)[0].quantite, 12);
        assert_eq!(std::fs::read_to_string(e.chemin()).unwrap(), avant);
    }

    #[test]
    fn echec_de_sauvegarde_laisse_la_liste_affichee_inchangee() {
        let c = chemin_temp("sauvegarde-impossible");
        let mut e = Etat::ouvrir(c.clone()).unwrap();
        // Un DOSSIER à la place du fichier temporaire : l'écriture est impossible.
        let bloc = PathBuf::from(format!("{}.tmp", c.display()));
        std::fs::create_dir(&bloc).unwrap();
        assert!(!e.ajouter("Sel", "1"));
        assert!(
            matches!(&e.retour, Some(Retour::Erreur(m)) if m.contains(&c.display().to_string()))
        );
        assert!(e.lignes("", false).is_empty());
        std::fs::remove_dir(&bloc).unwrap();
    }

    #[test]
    fn renommer_met_a_jour_la_liste_et_l_historique() {
        let mut e = etat_avec_riz("renommer");
        assert!(e.renommer("riz 5kg", " Riz basmati "));
        assert_eq!(
            e.retour,
            Some(Retour::Succes(
                "Produit renommé : « riz 5kg » devient « Riz basmati ».".into()
            ))
        );
        assert_eq!(e.lignes("", false)[0].nom, "Riz basmati");
        assert_eq!(e.historique(10)[0], "Entrée +12 Riz basmati");
        assert!(!e.renommer("Sel", "Sel fin"));
        assert!(matches!(e.retour, Some(Retour::Erreur(_))));
    }

    #[test]
    fn supprimer_retire_le_produit_et_garde_l_historique() {
        let mut e = etat_avec_riz("supprimer");
        assert!(e.supprimer("RIZ 5KG"));
        assert_eq!(
            e.retour,
            Some(Retour::Succes(
                "Produit « RIZ 5KG » supprimé (l'historique est conservé).".into()
            ))
        );
        assert!(e.lignes("", false).is_empty());
        assert_eq!(e.historique(10).len(), 1);
        assert!(!e.supprimer("Riz 5kg"));
    }

    #[test]
    fn lignes_filtrent_par_recherche_et_par_alerte() {
        let mut e = etat_avec_riz("filtres"); // Riz 5kg : 12, seuil 5 (pas en alerte)
        e.ajouter("Riz basmati", "5"); // 0, seuil 5 (en alerte)
        e.ajouter("Sucre", "0");
        e.entree("Sucre", "4");
        assert_eq!(e.lignes("  RIZ ", false).len(), 2);
        assert_eq!(e.lignes("sucre", false).len(), 1);
        assert_eq!(e.lignes("zzz", false).len(), 0);
        let alertes: Vec<String> = e.lignes("", true).into_iter().map(|l| l.nom).collect();
        assert_eq!(alertes, ["Riz basmati"]);
    }

    #[test]
    fn historique_du_plus_recent_au_plus_ancien_avec_limite() {
        let mut e = etat_avec_riz("historique");
        e.sortie("Riz 5kg", "2");
        assert_eq!(
            e.historique(10),
            ["Sortie -2 Riz 5kg", "Entrée +12 Riz 5kg"]
        );
        assert_eq!(e.historique(1), ["Sortie -2 Riz 5kg"]);
    }

    #[test]
    fn deux_fenetres_sur_le_meme_fichier_ne_perdent_aucun_mouvement() {
        let mut a = etat_avec_riz("deux-fenetres"); // le fichier contient Riz 5kg : 12
        let mut b = Etat::ouvrir(a.chemin().to_path_buf()).unwrap();
        assert!(a.sortie("Riz 5kg", "5")); // fichier : 7
        // B montre encore 12 : si elle enregistrait, elle effacerait la sortie de A.
        assert!(!b.sortie("Riz 5kg", "2"));
        assert!(
            matches!(&b.retour, Some(Retour::Erreur(m)) if m.contains("modifié par un autre")),
            "{:?}",
            b.retour
        );
        let sur_disque = Etat::ouvrir(a.chemin().to_path_buf()).unwrap();
        assert_eq!(sur_disque.lignes("", false)[0].quantite, 7);
        assert_eq!(b.lignes("", false)[0].quantite, 12); // B n'a rien changé
    }

    #[test]
    fn le_chemin_affiche_est_toujours_absolu() {
        let affiche = chemin_complet(Path::new("stock.txt"));
        assert!(Path::new(&affiche).is_absolute(), "{affiche}");
        assert!(affiche.ends_with("stock.txt"));
    }

    #[test]
    fn le_resume_compte_les_produits_les_alertes_et_les_unites() {
        let mut e = etat_avec_riz("resume"); // Riz 5kg : 12, seuil 5 (correct)
        e.ajouter("Savon", "3"); // 0 <= 3 : en alerte
        e.ajouter("Sucre", "0");
        e.entree("Sucre", "30");
        assert_eq!(
            e.resume(),
            Resume {
                produits: 3,
                en_alerte: 1,
                unites: 42
            }
        );
    }

    #[test]
    fn le_resume_d_un_stock_vide_est_a_zero() {
        let e = Etat::ouvrir(chemin_temp("resume-vide")).unwrap();
        assert_eq!(
            e.resume(),
            Resume {
                produits: 0,
                en_alerte: 0,
                unites: 0
            }
        );
    }

    #[test]
    fn les_suggestions_excluent_les_produits_deja_en_stock_et_suivent_la_saisie() {
        let mut e = Etat::ouvrir(chemin_temp("suggestions")).unwrap();
        assert!(e.ajouter("CAFÉ MOULU 250G", "5")); // casse et accents ne comptent pas
        let tous: Vec<&str> = e.suggestions("", 100).iter().map(|s| s.nom).collect();
        assert!(!tous.contains(&"Café moulu 250g"));
        assert!(tous.contains(&"Sel fin 1kg"));
        let huile: Vec<&str> = e.suggestions("huile", 10).iter().map(|s| s.nom).collect();
        assert!(!huile.is_empty());
        assert!(huile.iter().all(|n| n.to_lowercase().contains("huile")));
    }

    // ---- La vente ----------------------------------------------------------------------------

    use gestion_stock::horodatage::Horodatage;
    use gestion_stock::monnaie::{Devise, Montant};
    use gestion_stock::reglages::{Profil, Reglages};
    use gestion_stock::vente::{GenrePiece, Moyen};

    const FIXE: Horodatage = Horodatage {
        utc: 1_700_000_000,
        decalage_minutes: 60,
    };

    /// Pain : 10 en stock à 1,20 € (TVA 5,5 %) ; Savon : 10 en stock à 2,50 € (TVA par défaut,
    /// 20 %). L'horloge est fixe : 14/11/2023 à 23 h 13 heure locale.
    fn etat_vendeur(nom: &str) -> Etat {
        let mut e = Etat::ouvrir(chemin_temp(nom))
            .unwrap()
            .avec_horloge(|| FIXE);
        assert!(e.ajouter_avec_prix("Pain", "0", "1,20", "5,5"));
        assert!(e.entree("Pain", "10"));
        assert!(e.ajouter_avec_prix("Savon", "0", "2,50", ""));
        assert!(e.entree("Savon", "10"));
        e
    }

    fn panier(articles: &[(&str, u32)]) -> Vec<(String, u32)> {
        articles.iter().map(|(n, q)| (n.to_string(), *q)).collect()
    }

    fn ligne(e: &Etat, nom: &str) -> LigneProduit {
        e.lignes("", false)
            .into_iter()
            .find(|l| l.nom == nom)
            .unwrap()
    }

    #[test]
    fn encaisser_par_la_fenetre_retire_le_stock_et_confirme() {
        let mut e = etat_vendeur("vente");
        assert!(e.encaisser(&panier(&[("Pain", 3)]), Moyen::Carte, ""));
        assert_eq!(
            e.retour,
            Some(Retour::Succes(
                "Vente n° 1 enregistrée : 3,60\u{a0}€.".into()
            ))
        );
        assert_eq!(ligne(&e, "Pain").quantite, 7);
        assert_eq!(e.derniere_piece().unwrap().numero, 1);
        // Le fichier a été écrit : on retrouve la vente en rouvrant.
        let relu = Etat::ouvrir(e.chemin().to_path_buf()).unwrap();
        assert_eq!(relu.pieces_recentes(10).len(), 1);
        assert_eq!(ligne(&relu, "Pain").quantite, 7);
        assert_eq!(relu.integrite(), Ok(()));
    }

    #[test]
    fn especes_donnent_le_rendu_ou_un_message_clair_sans_rien_changer() {
        let mut e = etat_vendeur("especes");
        assert!(e.encaisser(&panier(&[("Pain", 3)]), Moyen::Especes, "5"));
        assert_eq!(e.derniere_piece().unwrap().rendu, Montant(140));

        let avant = e.pieces_recentes(10).len();
        assert!(!e.encaisser(&panier(&[("Pain", 3)]), Moyen::Especes, "2"));
        assert!(
            matches!(&e.retour, Some(Retour::Erreur(m)) if m.contains("3,60\u{a0}€") && m.contains("2,00\u{a0}€")),
            "{:?}",
            e.retour
        );
        assert_eq!(ligne(&e, "Pain").quantite, 7);
        assert_eq!(e.pieces_recentes(10).len(), avant);
    }

    #[test]
    fn saisies_de_montant_invalides_donnent_un_message_clair() {
        let mut e = etat_vendeur("montants-invalides");
        for saisie in ["abc", "-3", "12,505", "", "  "] {
            assert!(
                !e.encaisser(&panier(&[("Pain", 1)]), Moyen::Especes, saisie),
                "{saisie:?}"
            );
            assert!(matches!(e.retour, Some(Retour::Erreur(_))), "{saisie:?}");
        }
        assert!(
            matches!(&e.retour, Some(Retour::Erreur(m)) if m.contains("manquant")),
            "{:?}",
            e.retour
        );
        assert_eq!(ligne(&e, "Pain").quantite, 10);
        assert!(e.pieces_recentes(10).is_empty());
    }

    #[test]
    fn un_panier_vide_ou_sans_prix_est_refuse_clairement() {
        let mut e = etat_vendeur("refus");
        assert!(e.ajouter("Riz", "0"));
        assert!(e.entree("Riz", "5"));
        assert!(!e.encaisser(&[], Moyen::Carte, ""));
        assert_eq!(e.retour, Some(Retour::Erreur("Le panier est vide.".into())));
        assert!(!e.encaisser(&panier(&[("Riz", 1)]), Moyen::Carte, ""));
        assert!(matches!(&e.retour, Some(Retour::Erreur(m)) if m.contains("n'a pas de prix")));
        assert!(!e.encaisser(&panier(&[("Pain", 99)]), Moyen::Carte, ""));
        assert!(matches!(&e.retour, Some(Retour::Erreur(m)) if m.contains("Stock insuffisant")));
    }

    #[test]
    fn le_devis_en_direct_donne_le_total() {
        let e = etat_vendeur("devis");
        let d = e.devis(&panier(&[("Pain", 3), ("Savon", 1)])).unwrap();
        assert_eq!(d.total, Montant(610));
        assert_eq!(e.formater(d.total), "6,10\u{a0}€");
    }

    #[test]
    fn annuler_remet_le_stock_et_ajoute_une_ligne_aux_pieces_recentes() {
        let mut e = etat_vendeur("annulation");
        assert!(e.encaisser(&panier(&[("Pain", 3)]), Moyen::Carte, ""));
        assert!(e.annuler(1));
        assert_eq!(
            e.retour,
            Some(Retour::Succes("Vente n° 1 annulée : stock remis.".into()))
        );
        assert_eq!(ligne(&e, "Pain").quantite, 10);
        assert_eq!(
            e.derniere_piece().unwrap().genre,
            GenrePiece::Annulation { de: 1 }
        );
        let pieces = e.pieces_recentes(10);
        assert_eq!(pieces[0].libelle, "Annulation de la vente n° 1");
        assert!(!pieces[0].annulable);
        assert_eq!(pieces[1].libelle, "Vente");
        assert!(!pieces[1].annulable, "une vente déjà annulée ne l'est plus");
        assert!(!e.annuler(1));
        assert!(matches!(&e.retour, Some(Retour::Erreur(m)) if m.contains("déjà été annulée")));
    }

    #[test]
    fn les_pieces_recentes_sont_les_plus_recentes_d_abord_et_bien_ecrites() {
        let mut e = etat_vendeur("pieces");
        assert!(e.encaisser(&panier(&[("Pain", 3)]), Moyen::Carte, ""));
        assert!(e.encaisser(&panier(&[("Savon", 1)]), Moyen::Especes, "5"));
        let pieces = e.pieces_recentes(10);
        assert_eq!(
            pieces[1],
            LignePiece {
                numero: 1,
                quand: "14/11/2023 23:13".into(),
                libelle: "Vente".into(),
                total: "3,60\u{a0}€".into(),
                moyen: "Carte".into(),
                annulable: true
            }
        );
        assert_eq!((pieces[0].numero, pieces[0].moyen.as_str()), (2, "Espèces"));
        assert_eq!(e.pieces_recentes(1).len(), 1);
        assert_eq!(e.pieces_recentes(1)[0].numero, 2);
    }

    #[test]
    fn deux_fenetres_ne_perdent_aucune_vente() {
        let mut a = etat_vendeur("deux-fenetres-vente");
        let mut b = Etat::ouvrir(a.chemin().to_path_buf())
            .unwrap()
            .avec_horloge(|| FIXE);
        assert!(a.encaisser(&panier(&[("Pain", 1)]), Moyen::Carte, ""));
        // B montre encore le stock d'avant : si elle vendait, elle effacerait la vente de A.
        assert!(!b.encaisser(&panier(&[("Pain", 2)]), Moyen::Carte, ""));
        assert!(
            matches!(&b.retour, Some(Retour::Erreur(m)) if m.contains("modifié par un autre")),
            "{:?}",
            b.retour
        );
        let disque = Etat::ouvrir(a.chemin().to_path_buf()).unwrap();
        assert_eq!(disque.pieces_recentes(10).len(), 1);
        assert_eq!(ligne(&disque, "Pain").quantite, 9);
    }

    #[test]
    fn des_reglages_changes_ailleurs_sont_aussi_detectes() {
        let mut a = etat_vendeur("reglages-ailleurs");
        let mut b = Etat::ouvrir(a.chemin().to_path_buf()).unwrap();
        assert!(a.definir_reglages(Profil::Europe, Devise::Eur, "10"));
        assert!(!b.definir_prix("Pain", "1", ""));
        assert!(matches!(&b.retour, Some(Retour::Erreur(m)) if m.contains("modifié par un autre")));
    }

    #[test]
    fn un_journal_altere_bloque_les_ventes_mais_reste_lisible() {
        let mut e = etat_vendeur("journal-altere");
        assert!(e.encaisser(&panier(&[("Pain", 3)]), Moyen::Carte, ""));
        let chemin = e.chemin().to_path_buf();
        let texte = std::fs::read_to_string(&chemin).unwrap();
        // On « corrige » à la main le total de la vente n° 1 (3,60 € -> 0,01 €).
        let modifie = texte.replacen("1700000000;60;360;", "1700000000;60;1;", 1);
        assert_ne!(modifie, texte);
        std::fs::write(&chemin, modifie).unwrap();

        let mut e = Etat::ouvrir(chemin).unwrap().avec_horloge(|| FIXE);
        assert!(e.integrite().is_err());
        assert_eq!(e.lignes("", false).len(), 2); // toujours lisible
        assert!(!e.encaisser(&panier(&[("Pain", 1)]), Moyen::Carte, ""));
        assert!(
            matches!(&e.retour, Some(Retour::Erreur(m)) if m.contains("ventes sont bloquées")),
            "{:?}",
            e.retour
        );
        assert!(!e.annuler(1));
        assert!(matches!(&e.retour, Some(Retour::Erreur(m)) if m.contains("ventes sont bloquées")));
        // Le stock, lui, reste utilisable.
        assert!(e.entree("Pain", "1"));
        assert!(e.integrite().is_err());
    }

    #[test]
    fn definir_prix_par_la_fenetre() {
        let mut e = etat_vendeur("prix");
        assert!(e.definir_prix("Pain", "3,5", "5,5"));
        assert_eq!(ligne(&e, "Pain").prix, Some(Montant(350)));
        assert!(e.definir_prix("Pain", "", ""));
        assert_eq!(ligne(&e, "Pain").prix, None);
        assert!(matches!(&e.retour, Some(Retour::Succes(m)) if m.contains("retiré")));
        assert!(!e.definir_prix("Pain", "abc", ""));
        assert!(!e.definir_prix("Pain", "1", "abc"));
        assert!(!e.definir_prix("Sel", "1", ""));
        assert_eq!(ligne(&e, "Pain").prix, None);
    }

    #[test]
    fn ajouter_avec_prix_est_tout_ou_rien() {
        let mut e = Etat::ouvrir(chemin_temp("ajout-prix")).unwrap();
        assert!(e.ajouter_avec_prix("Café", "5", "3,20", "5,5"));
        let cafe = ligne(&e, "Café");
        assert_eq!((cafe.seuil, cafe.prix), (5, Some(Montant(320))));
        for (nom, seuil, prix, taux) in [
            ("A", "x", "1", ""),
            ("B", "1", "abc", ""),
            ("C", "1", "1", "abc"),
            ("", "1", "1", ""),
        ] {
            assert!(!e.ajouter_avec_prix(nom, seuil, prix, taux), "{nom:?}");
        }
        assert_eq!(e.lignes("", false).len(), 1);
    }

    #[test]
    fn les_reglages_se_changent_se_conservent_et_se_verrouillent_apres_une_vente() {
        let mut e = etat_vendeur("reglages");
        assert!(e.definir_reglages(Profil::EtatsUnis, Devise::Usd, "8,25"));
        assert_eq!(e.formater(Montant(1250)), "$12.50");
        let voulus = Reglages {
            profil: Profil::EtatsUnis,
            devise: Devise::Usd,
            taux_defaut_bp: 825,
        };
        assert_eq!(e.reglages(), voulus);
        assert_eq!(
            Etat::ouvrir(e.chemin().to_path_buf()).unwrap().reglages(),
            voulus
        );
        assert!(!e.definir_reglages(Profil::EtatsUnis, Devise::Usd, "abc"));
        assert!(matches!(&e.retour, Some(Retour::Erreur(m)) if m.contains("Taux invalide")));

        assert!(e.encaisser(&panier(&[("Pain", 1)]), Moyen::Carte, ""));
        assert!(!e.definir_reglages(Profil::Europe, Devise::Eur, "20"));
        assert!(
            matches!(&e.retour, Some(Retour::Erreur(m)) if m.contains("ne peuvent plus changer"))
        );
        assert!(e.definir_reglages(Profil::EtatsUnis, Devise::Usd, "6"));
    }

    #[test]
    fn un_stock_neuf_a_un_journal_integre() {
        assert_eq!(etat_vendeur("integre").integrite(), Ok(()));
    }

    #[test]
    fn effacer_le_ticket_prepare_la_vente_suivante() {
        let mut e = etat_vendeur("ticket");
        assert!(e.derniere_piece().is_none());
        assert!(e.encaisser(&panier(&[("Pain", 1)]), Moyen::Carte, ""));
        assert!(e.derniere_piece().is_some());
        e.effacer_ticket();
        assert!(e.derniere_piece().is_none());
    }

    #[test]
    fn le_detail_des_taxes_du_ticket_vient_des_lignes_de_la_piece() {
        let mut e = etat_vendeur("detail-taxes");
        assert!(e.encaisser(&panier(&[("Pain", 3), ("Savon", 1)]), Moyen::Carte, ""));
        let piece = e.derniere_piece().unwrap().clone();
        let taxes = |e: &Etat| -> Vec<(u32, i64)> {
            e.detail_taxes(&piece)
                .iter()
                .map(|g| (g.taux_bp, g.taxe.0))
                .collect()
        };
        assert_eq!(taxes(&e), [(550, 19), (2000, 42)]);
        // Un changement de prix après coup ne change pas le détail de l'ancien ticket.
        assert!(e.definir_prix("Pain", "9", "5,5"));
        assert_eq!(taxes(&e), [(550, 19), (2000, 42)]);
    }

    #[test]
    fn le_reglement_en_especes_se_calcule_en_direct_sans_rien_enregistrer() {
        let e = etat_vendeur("reglement");
        let total = Montant(360);
        assert_eq!(e.reglement_especes(total, ""), Reglement::Attente);
        assert_eq!(e.reglement_especes(total, "   "), Reglement::Attente);
        assert_eq!(e.reglement_especes(total, "abc"), Reglement::Illisible);
        assert_eq!(e.reglement_especes(total, "-5"), Reglement::Illisible);
        assert_eq!(
            e.reglement_especes(total, "3,10"),
            Reglement::Manque(Montant(50))
        );
        assert_eq!(
            e.reglement_especes(total, "3.60"),
            Reglement::Rendu(Montant(0))
        );
        assert_eq!(
            e.reglement_especes(total, "5"),
            Reglement::Rendu(Montant(140))
        );
        // Un panier vide n'a rien à encaisser.
        assert_eq!(e.reglement_especes(Montant(0), "5"), Reglement::Attente);
        assert!(e.derniere_piece().is_none());
    }

    #[test]
    fn le_prix_et_le_taux_d_un_produit_se_redonnent_comme_on_les_a_tapes() {
        let mut e = etat_vendeur("saisies-prix");
        assert!(e.ajouter("Thé", "0"));
        assert_eq!(
            e.saisies_du_prix("Pain"),
            ("1,20".to_string(), "5,5".to_string())
        );
        // Pas de taux propre : le champ reste vide, le taux par défaut s'applique.
        assert_eq!(
            e.saisies_du_prix("Savon"),
            ("2,50".to_string(), String::new())
        );
        assert_eq!(e.saisies_du_prix("Thé"), (String::new(), String::new()));
        assert_eq!(e.saisies_du_prix("Inconnu"), (String::new(), String::new()));
        // Ce qu'on relit est accepté tel quel par le formulaire : rien ne change au réenregistrement.
        let (prix, taux) = e.saisies_du_prix("Pain");
        assert!(e.definir_prix("Pain", &prix, &taux));
        assert_eq!(e.saisies_du_prix("Pain"), (prix, taux));
        assert_eq!(ligne(&e, "Pain").taux_bp, Some(550));
        assert_eq!(ligne(&e, "Savon").taux_bp, None);
    }

    #[test]
    fn les_reglages_sont_verrouilles_des_la_premiere_piece() {
        let mut e = etat_vendeur("verrou");
        assert!(!e.reglages_verrouilles());
        assert!(e.encaisser(&panier(&[("Pain", 1)]), Moyen::Carte, ""));
        assert!(e.reglages_verrouilles());
    }

    #[test]
    fn l_heure_locale_s_ecrit_hh_mm() {
        let e = Etat::ouvrir(chemin_temp("heure"))
            .unwrap()
            .avec_horloge(|| Horodatage {
                utc: 1_700_000_000,
                decalage_minutes: 60,
            });
        assert_eq!(e.heure_locale(), "23:13");
        let e = e.avec_horloge(|| Horodatage {
            utc: 1_700_000_000 + 7 * 3600 + 120,
            decalage_minutes: 0,
        });
        // 2023-11-14 22:13 UTC + 7 h 02 = 2023-11-15 05:15 UTC, avec un zéro devant l'heure.
        assert_eq!(e.heure_locale(), "05:15");
    }
}
