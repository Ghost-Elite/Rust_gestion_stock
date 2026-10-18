//! Tous les textes visibles de l'interface, au même endroit : pour traduire l'application
//! plus tard, il suffira de modifier ce fichier.

pub const TITRE_FENETRE: &str = "Gestion de stock";

pub const RECHERCHE: &str = "Rechercher un produit";
pub const ALERTES_SEULEMENT: &str = "Afficher seulement les produits en alerte";

pub const COLONNE_PRODUIT: &str = "Produit";
pub const COLONNE_STOCK: &str = "En stock";
pub const COLONNE_SEUIL: &str = "Seuil";
pub const COLONNE_ETAT: &str = "État";
pub const ETAT_ALERTE: &str = "ALERTE : stock bas";
pub const ETAT_CORRECT: &str = "Correct";
pub const AUCUN_PRODUIT: &str = "Aucun produit à afficher.";

pub const SECTION_AJOUT: &str = "Ajouter un produit";
pub const CHAMP_NOM: &str = "Nom du produit";
pub const CHAMP_SEUIL: &str = "Seuil d'alerte (facultatif)";
pub const BOUTON_AJOUTER: &str = "Ajouter le produit";

pub const AUCUNE_SELECTION: &str =
    "Aucun produit sélectionné. Choisissez un produit dans la liste.";
pub const CHAMP_QUANTITE: &str = "Quantité";
pub const BOUTON_ENTREE: &str = "Enregistrer une entrée";
pub const BOUTON_SORTIE: &str = "Enregistrer une sortie";
pub const CHAMP_NOUVEAU_NOM: &str = "Nouveau nom";
pub const BOUTON_RENOMMER: &str = "Renommer le produit";
pub const BOUTON_SUPPRIMER: &str = "Supprimer le produit";
pub const BOUTON_CONFIRMER_SUPPRESSION: &str = "Confirmer la suppression";
pub const BOUTON_ANNULER: &str = "Annuler";

pub const SECTION_HISTORIQUE: &str = "Derniers mouvements";
pub const AUCUN_MOUVEMENT: &str = "Aucun mouvement enregistré.";

pub const ERREUR_OUVERTURE_TITRE: &str = "Impossible d'ouvrir le fichier de données";
pub const ERREUR_OUVERTURE_AIDE: &str = "Rien n'a été modifié. Corrigez le fichier, ou indiquez un autre emplacement avec la variable STOCK_FICHIER, puis relancez le programme.";
pub const BOUTON_QUITTER: &str = "Quitter";

pub fn produit_selectionne(nom: &str, quantite: u32) -> String {
    format!("Produit sélectionné : {nom} ({quantite} en stock)")
}

pub fn confirmation_suppression(nom: &str, quantite: u32) -> String {
    format!("Supprimer « {nom} » ? Il reste {quantite} en stock. L'historique est conservé.")
}

pub fn pied_de_page_donnees(chemin: &str) -> String {
    format!("Données : {chemin}")
}
