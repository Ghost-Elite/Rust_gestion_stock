//! Le dessin de la fenêtre avec egui. Aucune logique métier ici : tout passe par `Etat`.
//!
//! Règles d'accessibilité suivies partout : chaque champ est relié à son libellé
//! (`labelled_by`), chaque bouton porte son texte complet, et une alerte est toujours
//! écrite en toutes lettres (la couleur ne vient qu'en plus).
//!
//! Le style : un fond gris très clair, des cartes blanches aux coins arrondis, une seule
//! couleur d'accent, des puces pour les états, de l'espace. Les couleurs viennent toutes de
//! `palette`, où leur contraste est vérifié par des tests.

use crate::etat::{Etat, Retour, chemin_complet};
use crate::palette::{ENTETE_FOND, ENTETE_TEXTE, FOND, TEXTE, couleur};
use crate::textes::*;
use eframe::egui::{
    self, Align2, CentralPanel, CornerRadius, Frame, Id, Margin, Panel, RichText, Ui, Vec2,
};
use gestion_stock::monnaie::Devise;
use gestion_stock::reglages::Profil;
use gestion_stock::vente::Moyen;
use std::path::Path;
use std::time::Duration;

mod composants;
mod entete;
#[cfg(test)]
mod essais;
mod historique;
mod paiement;
mod recherche;
mod reglages;
mod stock;
mod ticket;
mod vente;

use composants::*;

/// La taille de la fenêtre au lancement et sa taille minimale, en points. Les tests les utilisent
/// aussi : ils vérifient que l'essentiel est à l'écran à la taille que voit vraiment la personne.
pub const TAILLE_FENETRE: [f32; 2] = [1366.0, 768.0];
pub const TAILLE_MINIMALE: [f32; 2] = [640.0, 420.0];

/// En dessous de cette largeur (en points, donc aussi quand le texte est agrandi), on passe
/// en une seule colonne : les commandes se placent sous la liste au lieu d'être à côté.
const LARGEUR_DEUX_COLONNES: f32 = 800.0;

/// En dessous de cette largeur, l'écran de vente passe en une seule colonne qui défile.
const LARGEUR_CAISSE_DEUX_COLONNES: f32 = 1100.0;

/// Les écrans de l'application. On passe de l'un à l'autre par le Menu et par l'en-tête : il n'y
/// a plus d'onglets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Ecran {
    /// L'écran de caisse : c'est celui que l'on ouvre en premier, la vente est le geste courant.
    #[default]
    Vente,
    /// La recherche d'un article à mettre au ticket.
    Recherche,
    /// Les tickets, où l'on rembourse une vente.
    Historique,
    Stock,
    Reglages,
}

/// Ce que l'on demande de confirmer avant de l'avoir fait.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Confirmation {
    AnnulerVente,
    SupprimerArticle,
}

/// Une fenêtre modale ouverte par-dessus l'écran (les autres variantes arrivent avec leurs
/// écrans).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Modale {
    Menu,
    Confirmer(Confirmation),
    /// Le texte tapé ne correspond à aucun article.
    Introuvable(String),
    /// Le paiement en espèces.
    Especes,
    /// Le paiement par carte ou autre moyen, confirmé à la main (aucun terminal n'est relié).
    Terminal {
        moyen: Moyen,
        refuse: bool,
    },
    /// La vente qui vient d'être enregistrée, avec le rendu de monnaie et (au choix) son ticket.
    VenteTerminee {
        ticket_visible: bool,
    },
    /// La demande de remboursement d'un ticket (par son numéro du journal).
    Rembourser(u64),
}

/// Ce que l'historique des tickets a en mémoire : le texte du champ « N° de ticket », le jour
/// affiché (`None` : aujourd'hui) et le ticket choisi (par son numéro du journal).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EtatHistorique {
    pub filtre: String,
    pub jour: Option<String>,
    pub choisi: Option<u64>,
}

/// Un message de quelques secondes, en bas de l'écran. `depuis` est l'heure d'egui du premier
/// affichage (posée à ce moment-là).
#[derive(Debug, Clone, PartialEq)]
pub struct Toast {
    pub texte: String,
    pub depuis: Option<f64>,
}

/// Combien de secondes un message éphémère reste affiché.
const DUREE_MESSAGE: f64 = 3.0;

fn toast_visible(depuis: f64, maintenant: f64) -> bool {
    maintenant - depuis < DUREE_MESSAGE
}

/// Les réglages en cours de modification : ils ne comptent qu'une fois enregistrés.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrouillonReglages {
    pub profil: Profil,
    pub devise: Devise,
    pub taux: String,
}

#[derive(Default)]
pub struct Application {
    pub ecran: Ecran,
    /// Ce qui est tapé dans le champ de scan de l'écran de vente.
    pub scan: String,
    /// La ligne du ticket sélectionnée (par son nom de produit).
    pub ligne_choisie: Option<String>,
    /// Vrai quand le champ de scan doit reprendre le focus à l'image suivante.
    pub focus_scan: bool,
    /// Ce qui est tapé dans la recherche d'articles.
    pub recherche_articles: String,
    /// L'article que l'on vient d'ajouter depuis la recherche (son bouton dit « Ajouté ✓ »).
    pub ajoute: Option<String>,
    /// Vrai quand le champ de la recherche doit prendre le focus à l'image suivante.
    pub focus_recherche: bool,
    /// La fenêtre modale ouverte, s'il y en a une.
    pub modale: Option<Modale>,
    /// Le message éphémère affiché en bas de l'écran.
    pub toast: Option<Toast>,
    /// Le panier en cours : (nom du produit, quantité).
    pub panier: Vec<(String, u32)>,
    /// Vrai quand le champ « Montant reçu » doit prendre le focus à l'image suivante.
    pub focus_recu: bool,
    /// Ce que le client a donné, tel que tapé (pour un paiement en espèces).
    pub saisie_recu: String,
    pub recherche_vente: String,
    /// L'historique des tickets : filtre, jour affiché, ticket choisi.
    pub historique: EtatHistorique,
    /// `None` : on montre les réglages enregistrés.
    pub reglages_saisis: Option<BrouillonReglages>,
    /// Le prix et le taux du formulaire « Ajouter un produit ».
    pub saisie_prix: String,
    pub saisie_taux: String,
    /// Le prix et le taux du produit sélectionné, pré-remplis à la sélection.
    pub saisie_prix_produit: String,
    pub saisie_taux_produit: String,
    /// `None` : le fichier de données n'a pas pu être ouvert, on affiche l'écran d'erreur.
    pub etat: Option<Etat>,
    pub erreur_ouverture: Option<String>,
    /// Le fichier qui n'a pas pu être ouvert, montré sur l'écran d'erreur.
    pub chemin_erreur: Option<String>,
    pub recherche: String,
    pub alertes_seulement: bool,
    /// Nom du produit sélectionné dans la liste.
    pub selection: Option<String>,
    pub saisie_nom: String,
    pub saisie_seuil: String,
    pub saisie_quantite: String,
    pub saisie_nouveau_nom: String,
    pub confirmer_suppression: bool,
}

impl Application {
    pub fn nouvelle(etat: Etat) -> Application {
        Application {
            etat: Some(etat),
            focus_scan: true,
            ..Application::default()
        }
    }

    /// `quantite` unités de plus de ce produit dans le panier.
    fn ajouter_au_panier(&mut self, nom: &str, quantite: u32) {
        match self.panier.iter_mut().find(|(n, _)| n == nom) {
            Some((_, deja)) => *deja = deja.saturating_add(quantite),
            None => self.panier.push((nom.to_string(), quantite)),
        }
    }

    /// Une unité de moins ; la ligne disparaît quand il n'en reste plus.
    fn retirer_du_panier_une_unite(&mut self, nom: &str) {
        if let Some((_, quantite)) = self.panier.iter_mut().find(|(n, _)| n == nom) {
            *quantite = quantite.saturating_sub(1);
        }
        self.panier.retain(|(_, quantite)| *quantite > 0);
    }

    /// Ouvre l'écran de recherche avec ce texte déjà tapé.
    pub fn ouvrir_la_recherche(&mut self, texte: String) {
        self.recherche_articles = texte;
        self.ajoute = None;
        self.focus_recherche = true;
        self.ecran = Ecran::Recherche;
    }

    /// Affiche un message de quelques secondes en bas de l'écran.
    pub fn message(&mut self, texte: impl Into<String>) {
        self.toast = Some(Toast {
            texte: texte.into(),
            depuis: None,
        });
    }

    pub fn en_erreur(message: String, chemin: &Path) -> Application {
        Application {
            erreur_ouverture: Some(message),
            chemin_erreur: Some(chemin_complet(chemin)),
            ..Application::default()
        }
    }
}

impl eframe::App for Application {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        dessiner(ui, self);
    }
}

/// Dessine toute la fenêtre dans le `Ui` donné (panneaux compris) : on peut donc la tester
/// sans ouvrir de fenêtre.
pub fn dessiner(ui: &mut Ui, app: &mut Application) {
    if app.etat.is_none() {
        ecran_erreur_ouverture(ui, app);
        return;
    }
    stock::abandonner_la_selection_perimee(app);
    succes_vers_message(app);
    Panel::top(Id::new("entete"))
        .frame(
            Frame::new()
                .fill(couleur(ENTETE_FOND))
                .inner_margin(Margin::symmetric(20, 0)),
        )
        .exact_size(56.0)
        .show_separator_line(false)
        .show(ui, |ui| entete::entete(ui, app));
    // Une erreur reste affichée sous l'en-tête jusqu'à la prochaine action réussie.
    if let Some(Retour::Erreur(message)) = app.etat.as_ref().and_then(|etat| etat.retour.clone()) {
        Panel::top(Id::new("bandeau_erreur"))
            .frame(
                Frame::new()
                    .fill(couleur(FOND))
                    .inner_margin(Margin::symmetric(16, 8)),
            )
            .show_separator_line(false)
            .show(ui, |ui| {
                bandeau_alerte(ui, &format!("Erreur : {message}"));
            });
    }
    match app.ecran {
        Ecran::Vente => vente::ecran_vente(ui, app),
        Ecran::Recherche => recherche::ecran_recherche(ui, app),
        Ecran::Historique => historique::ecran_historique(ui, app),
        Ecran::Stock => stock::ecran_stock(ui, app),
        Ecran::Reglages => reglages::ecran_reglages(ui, app),
    }
    match app.modale.clone() {
        Some(Modale::Menu) => entete::fenetre_menu(ui.ctx(), app),
        Some(Modale::Confirmer(question)) => vente::fenetre_confirmation(ui.ctx(), app, question),
        Some(Modale::Introuvable(saisie)) => vente::fenetre_introuvable(ui.ctx(), app, &saisie),
        Some(Modale::Especes) => paiement::fenetre_especes(ui.ctx(), app),
        Some(Modale::Terminal { moyen, refuse }) => {
            paiement::fenetre_terminal(ui.ctx(), app, moyen, refuse);
        }
        Some(Modale::VenteTerminee { ticket_visible }) => {
            paiement::fenetre_vente_terminee(ui.ctx(), app, ticket_visible);
        }
        Some(Modale::Rembourser(numero)) => {
            historique::fenetre_remboursement(ui.ctx(), app, numero)
        }
        None => {}
    }
    afficher_message(ui.ctx(), app);
}

/// Un message de succès de l'état devient un message éphémère : il n'y a plus de pied de page.
fn succes_vers_message(app: &mut Application) {
    let Some(etat) = app.etat.as_mut() else {
        return;
    };
    if let Some(Retour::Succes(texte)) = etat.retour.take_if(|r| matches!(r, Retour::Succes(_))) {
        app.message(texte);
    }
}

/// La pastille sombre du message éphémère, centrée au-dessus de la barre de fonctions.
fn afficher_message(ctx: &egui::Context, app: &mut Application) {
    let maintenant = ctx.input(|i| i.time);
    let Some(toast) = app.toast.as_mut() else {
        return;
    };
    let depuis = *toast.depuis.get_or_insert(maintenant);
    if !toast_visible(depuis, maintenant) {
        app.toast = None;
        return;
    }
    ctx.request_repaint_after(Duration::from_secs_f64(
        DUREE_MESSAGE - (maintenant - depuis),
    ));
    let texte = toast.texte.clone();
    egui::Area::new(Id::new("message_ephemere"))
        .anchor(Align2::CENTER_BOTTOM, Vec2::new(0.0, -104.0))
        .order(egui::Order::Tooltip)
        .fade_in(false)
        .interactable(false)
        .show(ctx, |ui| {
            Frame::new()
                .fill(couleur(TEXTE))
                .corner_radius(CornerRadius::same(10))
                .inner_margin(Margin::symmetric(18, 12))
                .show(ui, |ui| {
                    ui.add(
                        egui::Label::new(
                            RichText::new(texte).size(15.0).color(couleur(ENTETE_TEXTE)),
                        )
                        .wrap_mode(egui::TextWrapMode::Extend),
                    );
                });
        });
}

fn ecran_erreur_ouverture(ui: &mut Ui, app: &Application) {
    CentralPanel::default()
        .frame(
            Frame::new()
                .fill(couleur(FOND))
                .inner_margin(Margin::symmetric(24, 24)),
        )
        .show(ui, |ui| {
            carte(ui, |ui| {
                ui.heading(ERREUR_OUVERTURE_TITRE);
                if let Some(message) = &app.erreur_ouverture {
                    bandeau_alerte(ui, message);
                }
                if let Some(chemin) = &app.chemin_erreur {
                    ui.label(pied_de_page_donnees(chemin));
                }
                ui.label(ERREUR_OUVERTURE_AIDE);
                if bouton(ui, BOUTON_QUITTER, Style::Secondaire, true).clicked() {
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                }
            });
        });
}

#[cfg(test)]
mod tests {
    use super::essais::*;
    use super::*;

    use eframe::egui::accesskit::Role;

    use egui_kittest::kittest::Queryable;

    use std::path::Path;

    #[test]
    fn l_ecran_d_erreur_d_ouverture_est_lisible() {
        let h = harness(Application::en_erreur(
            "Fichier de données corrompu à la ligne 2.".into(),
            Path::new("/données/boutique/stock.txt"),
        ));
        h.get_by_label(ERREUR_OUVERTURE_TITRE);
        h.get_by_label_contains("ligne 2");
        // Après un double-clic, rien d'autre ne dit de quel fichier il s'agit.
        h.get_by_label(&pied_de_page_donnees("/données/boutique/stock.txt"));
        h.get_by_label(ERREUR_OUVERTURE_AIDE);
        h.get_by_label(BOUTON_QUITTER);
    }

    #[test]
    fn le_zoom_au_clavier_est_actif() {
        assert!(eframe::egui::Context::default().options(|o| o.zoom_with_keyboard));
    }

    // ---- Suggestions de produits : choisir au lieu de taper ----------------------------------

    // ---- L'écran de vente ----------------------------------------------------------------------

    // ---- Le cadre : en-tête, Menu, messages --------------------------------------------------

    #[test]
    fn l_en_tete_montre_le_magasin_la_caisse_et_l_heure() {
        let h = harness_vente(app_vendeuse("entete"), FENETRE);
        h.get_by_label(NOM_MAGASIN);
        h.get_by_label(&libelle_caisse());
        h.get_by_label("23:13"); // horloge fixe : 14/11/2023 23:13 heure locale
    }

    #[test]
    fn le_menu_mene_aux_ecrans_et_le_retour_ramene_a_la_vente() {
        let mut h = harness_vente(app_vendeuse("menu"), FENETRE);
        cliquer(&mut h, BOUTON_MENU); // bouton TEMPORAIRE de l'en-tête de vente (tâche 6)
        assert_eq!(h.state().modale, Some(Modale::Menu));
        for nom in [ECRAN_HISTORIQUE, ECRAN_STOCK, ECRAN_REGLAGES, BOUTON_FERMER] {
            h.get_by_role_and_label(Role::Button, nom);
        }
        cliquer(&mut h, ECRAN_STOCK);
        assert_eq!(h.state().ecran, Ecran::Stock);
        assert!(h.state().modale.is_none());
        cliquer(&mut h, BOUTON_RETOUR_VENTE);
        assert_eq!(h.state().ecran, Ecran::Vente);
        // « Fermer » referme le menu sans changer d'écran.
        cliquer(&mut h, BOUTON_MENU);
        cliquer(&mut h, BOUTON_FERMER);
        assert!(h.state().modale.is_none());
        assert_eq!(h.state().ecran, Ecran::Vente);
    }

    #[test]
    fn chaque_ecran_secondaire_a_un_retour_a_la_vente() {
        for ecran in [
            Ecran::Stock,
            Ecran::Reglages,
            Ecran::Historique,
            Ecran::Recherche,
        ] {
            let mut app = app_vendeuse("retour");
            app.ecran = ecran;
            let mut h = harness_vente(app, FENETRE);
            cliquer(&mut h, BOUTON_RETOUR_VENTE);
            assert_eq!(h.state().ecran, Ecran::Vente, "depuis {ecran:?}");
        }
    }

    #[test]
    fn la_fenetre_menu_est_un_dialogue_nomme_et_montre_le_fichier_de_donnees() {
        let mut app = app_vendeuse("dialogue");
        app.modale = Some(Modale::Menu);
        let h = harness_vente(app, FENETRE);
        h.get_by_role_and_label(Role::Dialog, TITRE_MENU);
        h.get_by_label_contains("Données : ");
    }

    #[test]
    fn un_succes_devient_un_message_ephemere_et_une_erreur_un_bandeau() {
        let mut h = harness_vente(app_avec_produits("messages"), FENETRE);
        h.get_by_label("Riz 5kg").click();
        h.run();
        saisir(&mut h, CHAMP_QUANTITE, "2");
        cliquer(&mut h, BOUTON_ENTREE);
        let texte = h
            .state()
            .toast
            .as_ref()
            .expect("un message éphémère")
            .texte
            .clone();
        h.get_by_label(&texte);
        assert!(h.state().etat.as_ref().unwrap().retour.is_none());

        // Une erreur reste affichée, en bandeau, image après image.
        saisir(&mut h, CHAMP_QUANTITE, "50");
        cliquer(&mut h, BOUTON_SORTIE);
        h.get_by_label_contains("Erreur :");
        h.run();
        h.get_by_label_contains("Erreur :");
        // Une action réussie la remplace par un message de succès.
        saisir(&mut h, CHAMP_QUANTITE, "1");
        cliquer(&mut h, BOUTON_ENTREE);
        assert!(h.query_by_label_contains("Erreur :").is_none());
    }

    #[test]
    fn un_message_ephemere_s_efface_apres_trois_secondes() {
        assert!(toast_visible(10.0, 10.0));
        assert!(toast_visible(10.0, 12.9));
        assert!(!toast_visible(10.0, 13.0));
    }

    #[test]
    fn tous_les_textes_peints_de_l_en_tete_et_du_menu_atteignent_4_5() {
        let mut app = app_vendeuse("contraste-entete");
        let textes = sans_les_textes_grises(textes_peints(&mut app, FENETRE, 1.0));
        for attendu in [NOM_MAGASIN, "Caisse 01", "23:13", FONCTION_MENU] {
            assert!(
                textes.iter().any(|t| t.texte == attendu),
                "« {attendu} » n'est pas peint"
            );
        }
        verifier_contrastes(&textes, "en-tête");
        app.modale = Some(Modale::Menu);
        let textes = sans_les_textes_grises(textes_peints(&mut app, FENETRE, 1.0));
        assert!(textes.iter().any(|t| t.texte == TITRE_MENU));
        verifier_contrastes(&textes, "menu");
        // Un message éphémère : texte blanc sur fond sombre.
        app.modale = None;
        app.message("Vente annulée");
        let textes = sans_les_textes_grises(textes_peints(&mut app, FENETRE, 1.0));
        assert!(textes.iter().any(|t| t.texte == "Vente annulée"));
        verifier_contrastes(&textes, "message");
    }

    #[test]
    fn l_application_s_ouvre_sur_la_vente() {
        let app = app_vendeuse("ouverture");
        assert_eq!(app.ecran, Ecran::Vente);
    }

    // ---- Ventes, réglages, prix ---------------------------------------------------------------

    #[test]
    fn a_200_pour_cent_l_historique_et_la_recherche_ne_debordent_pas_a_l_horizontale() {
        // Texte agrandi : la largeur disponible est divisée par deux, on passe en une colonne.
        let taille = Vec2::new(1366.0, 4000.0);
        let largeur = taille.x / 2.0;
        for (nom, ecran) in [
            ("historique", Ecran::Historique),
            ("recherche", Ecran::Recherche),
        ] {
            let mut app = app_avec_ventes(&format!("zoom-{nom}"));
            app.ecran = ecran;
            let textes = textes_peints(&mut app, taille, 2.0);
            assert!(textes.len() > 8, "{nom} : trop peu de textes");
            for t in &textes {
                assert!(
                    t.rect.left() >= -0.5 && t.rect.right() <= largeur + 0.5,
                    "{nom} : « {} » déborde : {:?} (largeur {largeur})",
                    t.texte,
                    t.rect
                );
            }
        }
    }
}
