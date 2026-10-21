//! Tous les textes visibles de l'interface, au même endroit : pour traduire l'application
//! plus tard, il suffira de modifier ce fichier.

use gestion_stock::reglages::Profil;

pub const TITRE_FENETRE: &str = "Kora Marché — Caisse";

/// Le magasin et le numéro de caisse affichés dans l'en-tête. Constantes à l'étape A ; ce sont des
/// réglages à l'étape B.
pub const NOM_MAGASIN: &str = "Kora Marché";
pub const NUMERO_CAISSE: u32 = 1;

/// `Caisse 01`.
pub fn libelle_caisse() -> String {
    format!("Caisse {NUMERO_CAISSE:02}")
}

/// Le numéro de ticket montré à la personne : numéro de caisse, tiret, numéro du journal
/// (`01-0186`). Le numéro interne reste celui du journal.
pub fn numero_de_ticket(numero: u64) -> String {
    format!("{NUMERO_CAISSE:02}-{numero:04}")
}

pub const BOUTON_RETOUR_VENTE: &str = "Retour à la vente";
pub const BOUTON_MENU: &str = "Menu";
pub const TITRE_MENU: &str = "Menu";
pub const ECRAN_HISTORIQUE: &str = "Historique";
pub const ECRAN_STOCK: &str = "Stock";
pub const ECRAN_REGLAGES: &str = "Réglages";
pub const BOUTON_FERMER: &str = "Fermer";
pub const RECHERCHE_TITRE: &str = "Rechercher un article";

// ---- L'écran de vente ---------------------------------------------------------------------------
/// Le texte d'invite du champ de scan, aussi son nom pour les lecteurs d'écran.
pub const CHAMP_SCAN: &str = "Scannez un article ou tapez son nom";
pub const TICKET_VIDE_TITRE: &str = "Prêt pour une nouvelle vente";
pub const TICKET_VIDE_AIDE: &str = "Scannez le premier article pour commencer.";
pub const LIBELLE_TOTAL_GRAND: &str = "TOTAL";
pub const BOUTON_ESPECES: &str = "Espèces";
pub const BOUTON_CARTE: &str = "Carte";
pub const BOUTON_AUTRE: &str = "Autre";
pub const FONCTION_RECHERCHER: &str = "Rechercher";
pub const FONCTION_SUPPRIMER: &str = "Supprimer l'article";
pub const FONCTION_ANNULER: &str = "Annuler la vente";
pub const FONCTION_MENU: &str = "Menu";
pub const FONCTION_MENU_DETAIL: &str = "Historique, stock…";
pub const RACCOURCI_RECHERCHER: &str = "F2";
pub const RACCOURCI_SUPPRIMER: &str = "Suppr";
pub const RACCOURCI_ANNULER: &str = "Échap";
pub const TOAST_AUCUNE_VENTE: &str = "Aucune vente en cours";
pub const TOAST_VENTE_ANNULEE: &str = "Vente annulée";
pub const OUI_ANNULER: &str = "Oui, annuler";
pub const OUI_SUPPRIMER: &str = "Oui, supprimer";
pub const NON_REVENIR: &str = "Non, revenir";
pub const INTROUVABLE_TITRE: &str = "Article introuvable";
pub const SCANNER_A_NOUVEAU: &str = "Scanner à nouveau";

/// `16 articles`, `1 article` (au singulier jusqu'à un).
pub fn articles_au_ticket(nombre: u32) -> String {
    if nombre > 1 {
        format!("{nombre} articles")
    } else {
        format!("{nombre} article")
    }
}

/// `2 × 1,20 €` : la quantité et le prix d'une unité, sous le nom d'une ligne.
pub fn detail_de_ligne(quantite: u32, prix: &str) -> String {
    format!("{quantite} × {prix}")
}

/// Le nom d'une case de la barre de fonctions : `Rechercher (F2)`.
pub fn fonction_nom(libelle: &str, raccourci: &str) -> String {
    format!("{libelle} ({raccourci})")
}

/// Le nom du contrôle qui choisit une ligne du ticket (la ligne entière est cliquable).
pub fn choisir_la_ligne(nom: &str) -> String {
    format!("Choisir {nom}")
}

// ---- Le paiement --------------------------------------------------------------------------------
pub const ESPECES_TITRE: &str = "Paiement en espèces";
pub const A_PAYER: &str = "À payer";
pub const QUICK_EXACT: &str = "Exact";
pub const MONNAIE_A_RENDRE: &str = "Monnaie à rendre";
pub const IL_MANQUE: &str = "Il manque";
pub const BOUTON_RETOUR: &str = "Retour";
pub const BOUTON_VALIDER: &str = "Valider";
pub const CARTE_TITRE: &str = "Paiement par carte";
pub const AUTRE_TITRE: &str = "Autre moyen de paiement";
pub const CARTE_ATTENTE: &str = "Présentez la carte sur le terminal";
pub const CARTE_AIDE: &str = "Sans contact, puce ou piste. Saisissez le montant sur le terminal, puis confirmez ici ce qu'il affiche.";
pub const AUTRE_ATTENTE: &str = "Le client règle autrement (mobile money, chèque, virement…).";
pub const AUTRE_AIDE: &str = "Confirmez ici quand le paiement est reçu.";
pub const BOUTON_ACCEPTE: &str = "Paiement accepté";
pub const BOUTON_REFUSE: &str = "Paiement refusé";
pub const BOUTON_RECU: &str = "Paiement reçu";
pub const CARTE_REFUSEE: &str = "Carte refusée";
pub const CARTE_REFUSEE_AIDE: &str =
    "Aucun montant n'a été débité. Proposez un autre moyen de paiement.";
pub const BOUTON_REESSAYER: &str = "Réessayer";
pub const BOUTON_PAYER_ESPECES: &str = "Payer en espèces";
pub const VENTE_TERMINEE: &str = "Vente terminée";
pub const BOUTON_VOIR_TICKET: &str = "Voir le ticket";
pub const BOUTON_CACHER_TICKET: &str = "Cacher le ticket";

/// `Carte · 3,60 € · ticket enregistré`.
pub fn vente_terminee_detail(moyen: &str, total: &str) -> String {
    format!("{moyen} · {total} · ticket enregistré")
}

/// `Ticket 01-0186`.
pub fn titre_du_ticket(ticket: &str) -> String {
    format!("Ticket {ticket}")
}

pub fn nom_du_moyen(moyen: gestion_stock::vente::Moyen) -> &'static str {
    use gestion_stock::vente::Moyen;
    match moyen {
        Moyen::Especes => MOYEN_ESPECES,
        Moyen::Carte => MOYEN_CARTE,
        Moyen::Autre => MOYEN_AUTRE,
    }
}

pub fn stock_limite(nom: &str, disponible: u32) -> String {
    format!("{nom} : il n'en reste que {disponible}.")
}

pub fn introuvable_texte(saisie: &str) -> String {
    format!(
        "Aucun article ne correspond à « {saisie} ». Scannez à nouveau ou recherchez l'article par son nom."
    )
}

pub fn confirmation_titre(question: crate::ui::Confirmation) -> &'static str {
    match question {
        crate::ui::Confirmation::AnnulerVente => "Annuler la vente ?",
        crate::ui::Confirmation::SupprimerArticle => "Supprimer cet article ?",
    }
}

pub fn confirmation_texte(question: crate::ui::Confirmation, articles: u32, nom: &str) -> String {
    match question {
        crate::ui::Confirmation::AnnulerVente => {
            format!("Tous les articles ({articles}) seront retirés du ticket.")
        }
        crate::ui::Confirmation::SupprimerArticle => format!("{nom} sera retiré du ticket."),
    }
}

/// `0 article`, `1 article`, `2 articles` (en français, zéro est au singulier).
pub fn compteur_articles(nombre: usize) -> String {
    if nombre > 1 {
        format!("{nombre} articles")
    } else {
        format!("{nombre} article")
    }
}

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
        Profil::Europe => format!("TVA {taux} incluse"),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn le_numero_de_ticket_s_ecrit_caisse_tiret_numero() {
        assert_eq!(numero_de_ticket(1), "01-0001");
        assert_eq!(numero_de_ticket(186), "01-0186");
        assert_eq!(numero_de_ticket(12_345), "01-12345");
        assert_eq!(libelle_caisse(), "Caisse 01");
    }

    #[test]
    fn le_compteur_d_articles_accorde_le_singulier_et_le_zero() {
        assert_eq!(compteur_articles(0), "0 article");
        assert_eq!(compteur_articles(1), "1 article");
        assert_eq!(compteur_articles(2), "2 articles");
    }

    #[test]
    fn la_tva_s_ecrit_incluse_en_europe_et_la_taxe_aux_etats_unis() {
        assert_eq!(
            libelle_taxe(Profil::Europe, "5,5\u{a0}%"),
            "TVA 5,5\u{a0}% incluse"
        );
        assert_eq!(libelle_taxe(Profil::EtatsUnis, "8\u{a0}%"), "Taxe 8\u{a0}%");
    }
}
