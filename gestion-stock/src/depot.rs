//! Où vivent les données : un trait `Depot` et sa réalisation par fichier. (La réalisation par
//! base MySQL est dans `base::depot_mysql`.)
//!
//! Chaque méthode est une opération complète du métier : elle réussit en entier ou ne change
//! rien. Les règles sont celles du cœur (`Stock`) ; un dépôt ne fait que les appliquer à un
//! endroit donné et les rendre durables.

use crate::erreur::ErreurStock;
use crate::horodatage::Horodatage;
use crate::monnaie::Montant;
use crate::reglages::Reglages;
use crate::stock::{Produit, Stock};
use crate::stockage::{charger, sauvegarder};
use crate::vente::{Moyen, Piece};
use std::path::{Path, PathBuf};

pub trait Depot {
    /// Une copie complète (réglages, produits, mouvements, pièces de la caisse de ce dépôt) pour
    /// les écrans de lecture. Ce n'est qu'une photo : elle ne suit pas les changements suivants.
    fn instantane(&mut self) -> Result<Stock, ErreurStock>;

    fn ajouter_produit(&mut self, nom: &str, seuil: u32) -> Result<(), ErreurStock>;

    fn definir_prix(
        &mut self,
        nom: &str,
        prix: Option<Montant>,
        taux_bp: Option<u32>,
    ) -> Result<(), ErreurStock>;

    fn definir_reglages(&mut self, reglages: Reglages) -> Result<(), ErreurStock>;

    fn enregistrer_entree(&mut self, nom: &str, quantite: u32) -> Result<Produit, ErreurStock>;

    fn enregistrer_sortie(&mut self, nom: &str, quantite: u32) -> Result<Produit, ErreurStock>;

    fn renommer_produit(&mut self, ancien: &str, nouveau: &str) -> Result<(), ErreurStock>;

    fn supprimer_produit(&mut self, nom: &str) -> Result<(), ErreurStock>;

    fn encaisser(
        &mut self,
        demande: &[(String, u32)],
        moyen: Moyen,
        recu: Montant,
        quand: Horodatage,
    ) -> Result<Piece, ErreurStock>;

    fn annuler(&mut self, numero: u64, quand: Horodatage) -> Result<Piece, ErreurStock>;

    /// Recalcule la chaîne d'empreintes du journal : `JournalAltere` à la première pièce qui ne
    /// correspond plus.
    fn verifier_journal(&mut self) -> Result<(), ErreurStock>;
}

/// Le stock dans un fichier texte, comme avant : lu en entier à l'ouverture, réécrit en entier
/// (de façon atomique) à chaque opération.
pub struct DepotFichier {
    chemin: PathBuf,
    stock: Stock,
}

impl DepotFichier {
    /// Un fichier absent donne un stock vide ; il n'est créé qu'à la première opération.
    pub fn ouvrir(chemin: &Path) -> Result<DepotFichier, ErreurStock> {
        Ok(DepotFichier {
            chemin: chemin.to_path_buf(),
            stock: charger(chemin)?,
        })
    }

    /// Applique `operation` à une COPIE du stock ; la copie n'est enregistrée puis adoptée que
    /// si tout réussit, donc une opération refusée (ou une sauvegarde qui échoue) ne change rien.
    fn appliquer<R>(
        &mut self,
        operation: impl FnOnce(&mut Stock) -> Result<R, ErreurStock>,
    ) -> Result<R, ErreurStock> {
        let mut copie = self.stock.clone();
        let resultat = operation(&mut copie)?;
        sauvegarder(&copie, &self.chemin)?;
        self.stock = copie;
        Ok(resultat)
    }
}

impl Depot for DepotFichier {
    fn instantane(&mut self) -> Result<Stock, ErreurStock> {
        Ok(self.stock.clone())
    }

    fn ajouter_produit(&mut self, nom: &str, seuil: u32) -> Result<(), ErreurStock> {
        self.appliquer(|s| s.ajouter_produit(nom, seuil))
    }

    fn definir_prix(
        &mut self,
        nom: &str,
        prix: Option<Montant>,
        taux_bp: Option<u32>,
    ) -> Result<(), ErreurStock> {
        self.appliquer(|s| s.definir_prix(nom, prix, taux_bp))
    }

    fn definir_reglages(&mut self, reglages: Reglages) -> Result<(), ErreurStock> {
        self.appliquer(|s| s.definir_reglages(reglages))
    }

    fn enregistrer_entree(&mut self, nom: &str, quantite: u32) -> Result<Produit, ErreurStock> {
        self.appliquer(|s| s.enregistrer_entree(nom, quantite).cloned())
    }

    fn enregistrer_sortie(&mut self, nom: &str, quantite: u32) -> Result<Produit, ErreurStock> {
        self.appliquer(|s| s.enregistrer_sortie(nom, quantite).cloned())
    }

    fn renommer_produit(&mut self, ancien: &str, nouveau: &str) -> Result<(), ErreurStock> {
        self.appliquer(|s| s.renommer_produit(ancien, nouveau))
    }

    fn supprimer_produit(&mut self, nom: &str) -> Result<(), ErreurStock> {
        self.appliquer(|s| s.supprimer_produit(nom))
    }

    fn encaisser(
        &mut self,
        demande: &[(String, u32)],
        moyen: Moyen,
        recu: Montant,
        quand: Horodatage,
    ) -> Result<Piece, ErreurStock> {
        self.appliquer(|s| s.encaisser(demande, moyen, recu, quand).cloned())
    }

    fn annuler(&mut self, numero: u64, quand: Horodatage) -> Result<Piece, ErreurStock> {
        self.appliquer(|s| s.annuler(numero, quand).cloned())
    }

    fn verifier_journal(&mut self) -> Result<(), ErreurStock> {
        self.stock.verifier_journal()
    }
}
