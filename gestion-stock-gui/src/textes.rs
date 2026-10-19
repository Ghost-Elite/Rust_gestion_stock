//! Tous les textes visibles de l'interface, au même endroit : pour traduire l'application
//! plus tard, il suffira de modifier ce fichier.

use gestion_stock::reglages::Profil;

pub const TITRE_FENETRE: &str = "Gestion de stock";
pub const SOUS_TITRE: &str = "Suivez vos entrées, vos sorties et vos alertes de stock.";

pub const CHIFFRE_PRODUITS: &str = "Produits";
pub const CHIFFRE_ALERTES: &str = "En alerte";
pub const CHIFFRE_UNITES: &str = "Unités en stock";

pub const RECHERCHE: &str = "Rechercher un produit";
pub const ALERTES_SEULEMENT: &str = "Afficher seulement les produits en alerte";

pub const SECTION_LISTE: &str = "Vos produits";
pub const STOCK_UNITE: &str = "en stock";
pub const ETAT_ALERTE: &str = "ALERTE : stock bas";
pub const ETAT_CORRECT: &str = "Correct";
pub const AUCUN_PRODUIT: &str = "Aucun produit à afficher.";

pub const SECTION_AJOUT: &str = "Ajouter un produit";
pub const SECTION_ACTIONS: &str = "Actions sur le produit";
pub const CHAMP_NOM: &str = "Nom du produit";
pub const TITRE_SUGGESTIONS: &str = "Suggestions (cliquez pour choisir)";
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

pub const NAVIGATION: &str = "Sections de l'application";
pub const ONGLET_VENTE: &str = "Vente";
pub const ONGLET_STOCK: &str = "Stock";

pub const SECTION_PRODUITS_VENTE: &str = "Produits";
pub const RECHERCHE_VENTE: &str = "Rechercher un produit à vendre";
pub const AUCUN_PRODUIT_VENDABLE: &str = "Aucun produit ne correspond.";
pub const EPUISE: &str = "Épuisé";
pub const PRIX_MANQUANT: &str = "Prix manquant";
pub const TOUT_AU_PANIER: &str = "Tout le stock est au panier";

pub const SECTION_PANIER: &str = "Panier";
pub const PANIER_VIDE: &str = "Le panier est vide. Cliquez sur un produit pour l'ajouter.";
pub const BOUTON_VIDER: &str = "Vider le panier";
pub const BOUTON_RETIRER: &str = "Retirer";
pub const PANIER_INVALIDE: &str =
    "Un produit du panier n'existe plus ou n'a plus de prix. Retirez-le pour encaisser.";
pub const MOYEN_ESPECES: &str = "Espèces";
pub const MOYEN_CARTE: &str = "Carte";
pub const MOYEN_AUTRE: &str = "Autre";
pub const CHAMP_RECU: &str = "Montant reçu";
pub const BOUTON_MONTANT_JUSTE: &str = "Montant juste";
pub const RECU_ILLISIBLE: &str = "Montant illisible. Tapez par exemple 20 ou 12,50.";
pub const BOUTON_ENCAISSER: &str = "Encaisser";

pub const BOUTON_NOUVELLE_VENTE: &str = "Nouvelle vente";
pub const MENTION_NON_CERTIFIE: &str =
    "Ticket non fiscal : ce programme n'est pas une caisse certifiée.";
pub const LIBELLE_RENDU: &str = "Rendu de monnaie";

pub const ONGLET_VENTES: &str = "Ventes";
pub const ONGLET_REGLAGES: &str = "Réglages";

pub const SECTION_VENTES: &str = "Journal des ventes";
pub const AUCUNE_VENTE: &str = "Aucune vente enregistrée.";
pub const BOUTON_ANNULER_VENTE: &str = "Annuler la vente";
pub const BOUTON_CONFIRMER_ANNULATION: &str = "Confirmer l'annulation";
pub const BOUTON_GARDER_VENTE: &str = "Garder la vente";
pub const JOURNAL_ALTERE_SUITE: &str = "Les ventes et les annulations sont bloquées ; le stock reste utilisable. Restaurez une copie du fichier de données pour vendre à nouveau.";

pub const SECTION_REGLAGES: &str = "Réglages de la caisse";
pub const LIBELLE_PROFIL: &str = "Prix et taxes";
pub const PROFIL_EUROPE: &str = "Europe (prix TTC)";
pub const PROFIL_ETATS_UNIS: &str = "États-Unis (prix hors taxe)";
pub const LIBELLE_DEVISE: &str = "Devise";
pub const DEVISE_EUR: &str = "EUR (€)";
pub const DEVISE_USD: &str = "USD ($)";
pub const DEVISE_GBP: &str = "GBP (£)";
pub const DEVISE_XOF: &str = "XOF (FCFA)";
pub const CHAMP_TAUX_DEFAUT: &str = "Taux par défaut (%)";
pub const BOUTON_ENREGISTRER_REGLAGES: &str = "Enregistrer les réglages";
pub const RAPPEL_REGLAGES: &str = "Vérifiez ces réglages avant de vendre.";
pub const MENTION_CAISSE_NON_CERTIFIEE: &str = "Ce programme n'est pas une caisse certifiée.";
pub const REGLAGES_VERROUILLES: &str = "La devise et le mode de prix ne se changent plus après la première vente : le journal garde chaque montant dans sa devise d'origine. Le taux par défaut reste modifiable.";
pub const EXPLICATION_EUROPE: &str =
    "Les prix saisis sont TTC (TVA comprise) : le ticket montre la TVA contenue dans le total.";
pub const EXPLICATION_ETATS_UNIS: &str =
    "Les prix saisis sont hors taxe : la taxe de vente s'ajoute au total, par taux.";

pub const CHAMP_PRIX_AJOUT: &str = "Prix (facultatif)";
pub const CHAMP_TAUX_AJOUT: &str = "Taux de TVA ou de taxe, en % (facultatif)";
pub const CHAMP_PRIX: &str = "Prix du produit";
pub const CHAMP_TAUX: &str = "Taux de TVA ou de taxe du produit, en % (facultatif)";
pub const BOUTON_ENREGISTRER_PRIX: &str = "Enregistrer le prix";
pub const SANS_PRIX: &str = "Pas de prix";
pub const PRIX_VIDE_RETIRE: &str =
    "Un prix vide retire le prix : le produit ne pourra plus être vendu.";

pub const ERREUR_OUVERTURE_TITRE: &str = "Impossible d'ouvrir le fichier de données";
pub const ERREUR_OUVERTURE_AIDE: &str = "Rien n'a été modifié. Corrigez le fichier, ou indiquez un autre emplacement avec la variable STOCK_FICHIER, puis relancez le programme.";
pub const BOUTON_QUITTER: &str = "Quitter";

pub fn seuil_court(seuil: u32) -> String {
    format!("Seuil : {seuil}")
}

pub fn produit_selectionne(nom: &str, quantite: u32) -> String {
    format!("Produit sélectionné : {nom} ({quantite} en stock)")
}

pub fn confirmation_suppression(nom: &str, quantite: u32) -> String {
    format!("Supprimer « {nom} » ? Il reste {quantite} en stock. L'historique est conservé.")
}

pub fn pied_de_page_donnees(chemin: &str) -> String {
    format!("Données : {chemin}")
}

pub fn stock_restant(quantite: u32) -> String {
    format!("{quantite} en stock")
}

pub fn ajouter_au_panier(nom: &str, prix: &str) -> String {
    format!("Ajouter {nom} au panier ({prix})")
}

pub fn produit_non_vendable(nom: &str, raison: &str) -> String {
    format!("{nom} : {}", raison.to_lowercase())
}

pub fn retirer_une_unite(nom: &str) -> String {
    format!("Retirer une unité de {nom}")
}

pub fn ajouter_une_unite(nom: &str) -> String {
    format!("Ajouter une unité de {nom}")
}

pub fn retirer_du_panier(nom: &str) -> String {
    format!("Retirer {nom} du panier")
}

pub fn ligne_de_panier(prix: &str, quantite: u32, total: &str) -> String {
    format!("{prix} × {quantite} = {total}")
}

pub fn il_manque(montant: &str) -> String {
    format!("Il manque {montant}")
}

pub fn rendu_de_monnaie(montant: &str) -> String {
    format!("{LIBELLE_RENDU} : {montant}")
}

pub fn titre_ticket(numero: u64) -> String {
    format!("Ticket n° {numero}")
}

pub fn ligne_de_ticket(quantite: u32, nom: &str, total: &str) -> String {
    format!("{quantite} × {nom} : {total}")
}

pub fn moyen_du_ticket(moyen: &str) -> String {
    format!("Payé par : {moyen}")
}

pub fn recu_du_ticket(montant: &str) -> String {
    format!("Montant reçu : {montant}")
}

/// Le total à payer : en Europe les prix sont TTC, aux États-Unis la taxe s'ajoute.
pub fn libelle_total(profil: Profil) -> &'static str {
    match profil {
        Profil::Europe => "Total TTC",
        Profil::EtatsUnis => "Total",
    }
}

pub const LIBELLE_SOUS_TOTAL: &str = "Sous-total";

pub fn libelle_taxe(profil: Profil, taux: &str) -> String {
    match profil {
        Profil::Europe => format!("dont TVA {taux}"),
        Profil::EtatsUnis => format!("Taxe {taux}"),
    }
}

/// Un stock sous le seuil d'alerte s'écrit aussi en toutes lettres sur la tuile.
pub fn stock_bas(quantite: u32) -> String {
    format!("{quantite} en stock (bas)")
}

pub fn annuler_la_vente(numero: u64) -> String {
    format!("Annuler la vente n° {numero}")
}

pub fn confirmation_annulation(numero: u64, total: &str) -> String {
    format!(
        "Annuler la vente n° {numero} ({total}) ? Le stock sera remis et une écriture d'annulation sera ajoutée au journal : la vente reste visible, rien n'est effacé."
    )
}

pub fn titre_de_piece(numero: u64, libelle: &str) -> String {
    format!("n° {numero} : {libelle}")
}

pub fn detail_de_piece(quand: &str, moyen: &str) -> String {
    format!("{quand} · {moyen}")
}

pub fn prix_court(prix: &str) -> String {
    format!("Prix : {prix}")
}

pub fn journal_altere(message: &str) -> String {
    format!("{message} {JOURNAL_ALTERE_SUITE}")
}
