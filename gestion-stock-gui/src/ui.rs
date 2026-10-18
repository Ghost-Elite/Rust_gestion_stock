//! Le dessin de la fenêtre avec egui. Aucune logique métier ici : tout passe par `Etat`.
//!
//! Règles d'accessibilité suivies partout : chaque champ est relié à son libellé
//! (`labelled_by`), chaque bouton porte son texte complet, et une alerte est toujours
//! écrite en toutes lettres (la couleur ne vient qu'en plus).

use crate::etat::{Etat, LigneProduit, Retour, chemin_complet};
use crate::palette::{ALERTE_FOND, ALERTE_TEXTE, couleur};
use crate::textes::*;
use eframe::egui::{
    self, Button, CentralPanel, Frame, Grid, Id, Margin, Panel, Response, RichText, ScrollArea, Ui,
};
use std::path::Path;

/// En dessous de cette largeur (en points, donc aussi quand le texte est agrandi), on passe
/// en une seule colonne : les commandes se placent sous la liste au lieu d'être à côté.
const LARGEUR_DEUX_COLONNES: f32 = 720.0;

#[derive(Default)]
pub struct Application {
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
            ..Application::default()
        }
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
    abandonner_la_selection_perimee(app);
    Panel::bottom(Id::new("pied_de_page")).show(ui, |ui| pied_de_page(ui, app));
    if ui.available_width() >= LARGEUR_DEUX_COLONNES {
        Panel::right(Id::new("commandes"))
            .default_size(320.0)
            .min_size(260.0)
            .show(ui, |ui| {
                ScrollArea::vertical().show(ui, |ui| contenu_commandes(ui, app));
            });
        CentralPanel::default_margins().show(ui, |ui| {
            ScrollArea::vertical()
                .auto_shrink(false)
                .show(ui, |ui| contenu_liste(ui, app));
        });
    } else {
        // Fenêtre étroite ou texte très agrandi : une seule colonne, rien n'est rogné.
        CentralPanel::default_margins().show(ui, |ui| {
            ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
                contenu_liste(ui, app);
                ui.separator();
                contenu_commandes(ui, app);
            });
        });
    }
}

/// Si le produit sélectionné n'existe plus (supprimé, renommé), on n'agit jamais dessus.
fn abandonner_la_selection_perimee(app: &mut Application) {
    let Some(etat) = &app.etat else { return };
    let existe = app
        .selection
        .as_ref()
        .is_none_or(|nom| etat.lignes("", false).iter().any(|l| &l.nom == nom));
    if !existe {
        app.selection = None;
        app.confirmer_suppression = false;
    }
}

/// Un champ de saisie relié à son libellé, pour les lecteurs d'écran.
fn champ(ui: &mut Ui, libelle: &str, valeur: &mut String) -> Response {
    let etiquette = ui.label(libelle);
    ui.text_edit_singleline(valeur).labelled_by(etiquette.id)
}

/// Un texte important sur fond rouge clair. La couleur n'est jamais seule : le texte dit tout.
fn bandeau_alerte(ui: &mut Ui, texte: &str) {
    Frame::new()
        .fill(couleur(ALERTE_FOND))
        .inner_margin(Margin::same(6))
        .show(ui, |ui| {
            ui.label(RichText::new(texte).color(couleur(ALERTE_TEXTE)).strong());
        });
}

fn pied_de_page(ui: &mut Ui, app: &Application) {
    let Some(etat) = &app.etat else { return };
    match &etat.retour {
        Some(Retour::Succes(message)) => {
            ui.label(message.as_str());
        }
        Some(Retour::Erreur(message)) => bandeau_alerte(ui, &format!("Erreur : {message}")),
        None => {}
    }
    ui.label(pied_de_page_donnees(&chemin_complet(etat.chemin())));
}

fn contenu_liste(ui: &mut Ui, app: &mut Application) {
    champ(ui, RECHERCHE, &mut app.recherche);
    ui.checkbox(&mut app.alertes_seulement, ALERTES_SEULEMENT);
    ui.add_space(8.0);
    tableau(ui, app);
    ui.add_space(12.0);
    historique(ui, app);
}

fn tableau(ui: &mut Ui, app: &mut Application) {
    let Some(etat) = &app.etat else { return };
    let lignes = etat.lignes(&app.recherche, app.alertes_seulement);
    if lignes.is_empty() {
        ui.label(AUCUN_PRODUIT);
        return;
    }
    let mut clic: Option<String> = None;
    // Un tableau est dispensé de « reflow » (WCAG 1.4.10) : si vraiment la place manque, on
    // défile horizontalement. L'état vient juste après le nom, et un nom très long passe à la
    // ligne au lieu de repousser l'alerte hors de l'écran.
    ScrollArea::horizontal().show(ui, |ui| {
        Grid::new("tableau_produits")
            .striped(true)
            .num_columns(4)
            .spacing([16.0, 6.0])
            .max_col_width(260.0)
            .show(ui, |ui| {
                for titre in [COLONNE_PRODUIT, COLONNE_ETAT, COLONNE_STOCK, COLONNE_SEUIL] {
                    ui.strong(titre);
                }
                ui.end_row();
                for ligne in &lignes {
                    let choisi = app.selection.as_deref() == Some(ligne.nom.as_str());
                    if ui.selectable_label(choisi, ligne.nom.as_str()).clicked() {
                        clic = Some(ligne.nom.clone());
                    }
                    if ligne.en_alerte {
                        bandeau_alerte(ui, ETAT_ALERTE);
                    } else {
                        ui.label(ETAT_CORRECT);
                    }
                    ui.label(ligne.quantite.to_string());
                    ui.label(ligne.seuil.to_string());
                    ui.end_row();
                }
            });
    });
    if let Some(nom) = clic {
        app.selection = Some(nom);
        app.saisie_nouveau_nom.clear();
        app.confirmer_suppression = false;
    }
}

fn historique(ui: &mut Ui, app: &Application) {
    let Some(etat) = &app.etat else { return };
    ui.heading(SECTION_HISTORIQUE);
    let lignes = etat.historique(20);
    if lignes.is_empty() {
        ui.label(AUCUN_MOUVEMENT);
    }
    for ligne in lignes {
        ui.label(ligne);
    }
}

fn contenu_commandes(ui: &mut Ui, app: &mut Application) {
    formulaire_ajout(ui, app);
    ui.separator();
    commandes_du_produit(ui, app);
}

/// Un bouton qui ne sert que si son champ est rempli : sans cela, un double-clic enregistrerait
/// l'action une fois, puis afficherait « Erreur : quantité manquante » par-dessus le succès, et
/// la personne croirait que ça a échoué et recommencerait.
fn bouton(ui: &mut Ui, texte: &str, champ_rempli: &str) -> Response {
    ui.add_enabled(!champ_rempli.trim().is_empty(), Button::new(texte))
}

fn formulaire_ajout(ui: &mut Ui, app: &mut Application) {
    ui.heading(SECTION_AJOUT);
    champ(ui, CHAMP_NOM, &mut app.saisie_nom);
    champ(ui, CHAMP_SEUIL, &mut app.saisie_seuil);
    if bouton(ui, BOUTON_AJOUTER, &app.saisie_nom).clicked()
        && let Some(etat) = app.etat.as_mut()
        && etat.ajouter(&app.saisie_nom, &app.saisie_seuil)
    {
        app.saisie_nom.clear();
        app.saisie_seuil.clear();
    }
}

fn commandes_du_produit(ui: &mut Ui, app: &mut Application) {
    let ligne = app.selection.as_ref().and_then(|nom| {
        let etat = app.etat.as_ref()?;
        etat.lignes("", false).into_iter().find(|l| &l.nom == nom)
    });
    let Some(ligne) = ligne else {
        ui.label(AUCUNE_SELECTION);
        return;
    };

    ui.label(produit_selectionne(&ligne.nom, ligne.quantite));
    champ(ui, CHAMP_QUANTITE, &mut app.saisie_quantite);
    ui.horizontal_wrapped(|ui| {
        if bouton(ui, BOUTON_ENTREE, &app.saisie_quantite).clicked()
            && let Some(etat) = app.etat.as_mut()
            && etat.entree(&ligne.nom, &app.saisie_quantite)
        {
            app.saisie_quantite.clear();
        }
        if bouton(ui, BOUTON_SORTIE, &app.saisie_quantite).clicked()
            && let Some(etat) = app.etat.as_mut()
            && etat.sortie(&ligne.nom, &app.saisie_quantite)
        {
            app.saisie_quantite.clear();
        }
    });

    ui.separator();
    champ(ui, CHAMP_NOUVEAU_NOM, &mut app.saisie_nouveau_nom);
    if bouton(ui, BOUTON_RENOMMER, &app.saisie_nouveau_nom).clicked() {
        let nouveau = app.saisie_nouveau_nom.clone();
        if let Some(etat) = app.etat.as_mut()
            && etat.renommer(&ligne.nom, &nouveau)
        {
            app.selection = Some(nouveau.trim().to_string());
            app.saisie_nouveau_nom.clear();
        }
    }

    ui.separator();
    suppression(ui, app, &ligne);
}

/// Supprimer demande toujours une confirmation : le premier clic ne supprime rien.
fn suppression(ui: &mut Ui, app: &mut Application, ligne: &LigneProduit) {
    if app.confirmer_suppression {
        ui.label(confirmation_suppression(&ligne.nom, ligne.quantite));
        ui.horizontal_wrapped(|ui| {
            if ui.button(BOUTON_CONFIRMER_SUPPRESSION).clicked() {
                if let Some(etat) = app.etat.as_mut()
                    && etat.supprimer(&ligne.nom)
                {
                    app.selection = None;
                }
                app.confirmer_suppression = false;
            }
            if ui.button(BOUTON_ANNULER).clicked() {
                app.confirmer_suppression = false;
            }
        });
    } else if ui.button(BOUTON_SUPPRIMER).clicked() {
        app.confirmer_suppression = true;
    }
}

fn ecran_erreur_ouverture(ui: &mut Ui, app: &Application) {
    CentralPanel::default_margins().show(ui, |ui| {
        ui.heading(ERREUR_OUVERTURE_TITRE);
        if let Some(message) = &app.erreur_ouverture {
            bandeau_alerte(ui, message);
        }
        if let Some(chemin) = &app.chemin_erreur {
            ui.label(pied_de_page_donnees(chemin));
        }
        ui.label(ERREUR_OUVERTURE_AIDE);
        if ui.button(BOUTON_QUITTER).clicked() {
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::etat::Etat;
    use crate::palette::{self, Rgb, rapport_contraste};
    use eframe::egui::accesskit::Role;
    use eframe::egui::{Color32, Context, Pos2, RawInput, Rect, Shape, Vec2};
    use egui_kittest::{Harness, kittest::Queryable};
    use std::path::{Path, PathBuf};

    fn chemin_temp(nom: &str) -> PathBuf {
        let chemin = std::env::temp_dir().join(format!(
            "gestion-stock-gui-ui-{}-{nom}.txt",
            std::process::id()
        ));
        let _ = std::fs::remove_file(&chemin);
        chemin
    }

    /// « Riz 5kg » (seuil 5, stock 3, en alerte) et « Sucre » (seuil 0, stock 10, correct).
    fn app_avec_produits(nom: &str) -> Application {
        let mut etat = Etat::ouvrir(chemin_temp(nom)).unwrap();
        assert!(etat.ajouter("Riz 5kg", "5"));
        assert!(etat.entree("Riz 5kg", "3"));
        assert!(etat.ajouter("Sucre", "0"));
        assert!(etat.entree("Sucre", "10"));
        Application::nouvelle(etat)
    }

    fn harness(app: Application) -> Harness<'static, Application> {
        let mut h = Harness::new_ui_state(|ui, app: &mut Application| dessiner(ui, app), app);
        h.run();
        h
    }

    /// Tape dans un champ comme le ferait une personne : d'abord le focus, puis le texte.
    fn saisir(h: &mut Harness<'static, Application>, libelle: &str, texte: &str) {
        h.get_by_label(libelle).focus();
        h.run();
        h.get_by_label(libelle).type_text(texte);
        h.run();
    }

    #[test]
    fn les_commandes_sans_selection_ont_un_nom_accessible() {
        let h = harness(app_avec_produits("sans-selection"));
        // On interroge le RÔLE autant que le nom : un champ sans `labelled_by` ne serait
        // trouvé que comme simple libellé, jamais comme champ de saisie.
        for nom in [RECHERCHE, CHAMP_NOM, CHAMP_SEUIL] {
            h.get_by_role_and_label(Role::TextInput, nom);
        }
        h.get_by_role_and_label(Role::CheckBox, ALERTES_SEULEMENT);
        h.get_by_role_and_label(Role::Button, BOUTON_AJOUTER);
        assert!(h.query_by_label(BOUTON_ENTREE).is_none());
        h.get_by_label(AUCUNE_SELECTION);
    }

    #[test]
    fn choisir_un_produit_affiche_ses_commandes() {
        let mut h = harness(app_avec_produits("selection"));
        h.get_by_label("Riz 5kg").click();
        h.run();
        assert_eq!(h.state().selection.as_deref(), Some("Riz 5kg"));
        for nom in [CHAMP_QUANTITE, CHAMP_NOUVEAU_NOM] {
            h.get_by_role_and_label(Role::TextInput, nom);
        }
        for nom in [
            BOUTON_ENTREE,
            BOUTON_SORTIE,
            BOUTON_RENOMMER,
            BOUTON_SUPPRIMER,
        ] {
            h.get_by_role_and_label(Role::Button, nom);
        }
        h.get_by_label(&produit_selectionne("Riz 5kg", 3));
    }

    #[test]
    fn ajouter_un_produit_par_l_interface() {
        let mut h = harness(app_avec_produits("ajout"));
        saisir(&mut h, CHAMP_NOM, "Café 250g");
        saisir(&mut h, CHAMP_SEUIL, "2");
        h.get_by_label(BOUTON_AJOUTER).click();
        h.run();
        let lignes = h.state().etat.as_ref().unwrap().lignes("café", false);
        assert_eq!((lignes[0].nom.as_str(), lignes[0].seuil), ("Café 250g", 2));
        assert!(h.state().saisie_nom.is_empty() && h.state().saisie_seuil.is_empty());
    }

    #[test]
    fn une_sortie_trop_grande_affiche_une_erreur_lisible_et_garde_la_saisie() {
        let mut h = harness(app_avec_produits("sortie-erreur"));
        h.get_by_label("Riz 5kg").click();
        h.run();
        saisir(&mut h, CHAMP_QUANTITE, "5");
        h.get_by_label(BOUTON_SORTIE).click();
        h.run();
        h.get_by_label_contains(
            "Erreur : Stock insuffisant pour « Riz 5kg » : 3 disponible(s), 5 demandé(s).",
        );
        assert_eq!(h.state().saisie_quantite, "5");
    }

    #[test]
    fn l_alerte_est_ecrite_et_pas_seulement_coloree() {
        let h = harness(app_avec_produits("alerte"));
        assert_eq!(h.get_all_by_label(ETAT_ALERTE).count(), 1); // Riz 5kg
        assert_eq!(h.get_all_by_label(ETAT_CORRECT).count(), 1); // Sucre
    }

    #[test]
    fn la_suppression_demande_une_confirmation() {
        let mut h = harness(app_avec_produits("suppression"));
        h.get_by_label("Riz 5kg").click();
        h.run();
        h.get_by_label(BOUTON_SUPPRIMER).click();
        h.run();
        h.get_by_label_contains("Supprimer « Riz 5kg » ? Il reste 3 en stock.");
        assert_eq!(h.state().etat.as_ref().unwrap().lignes("", false).len(), 2);
        h.get_by_label(BOUTON_ANNULER).click();
        h.run();
        assert_eq!(h.state().etat.as_ref().unwrap().lignes("", false).len(), 2);
        h.get_by_label(BOUTON_SUPPRIMER).click();
        h.run();
        h.get_by_label(BOUTON_CONFIRMER_SUPPRESSION).click();
        h.run();
        assert_eq!(h.state().etat.as_ref().unwrap().lignes("", false).len(), 1);
        assert_eq!(h.state().selection, None);
    }

    #[test]
    fn une_selection_disparue_est_abandonnee_sans_planter() {
        let mut app = app_avec_produits("fantome");
        app.selection = Some("Fantôme".into());
        let h = harness(app);
        assert_eq!(h.state().selection, None);
        h.get_by_label(AUCUNE_SELECTION);
    }

    #[test]
    fn la_recherche_qui_masque_le_produit_selectionne_ne_plante_pas() {
        let mut h = harness(app_avec_produits("recherche"));
        h.get_by_label("Riz 5kg").click();
        h.run();
        saisir(&mut h, RECHERCHE, "sucre");
        assert!(h.query_by_label("Riz 5kg").is_none());
        h.get_by_label(BOUTON_ENTREE); // la sélection (masquée mais existante) reste utilisable
    }

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

    #[test]
    fn un_double_clic_sur_sortie_n_enregistre_qu_une_sortie_et_garde_le_message_de_succes() {
        let mut h = harness(app_avec_produits("double-clic"));
        h.get_by_label("Sucre").click();
        h.run();
        saisir(&mut h, CHAMP_QUANTITE, "2");
        h.get_by_label(BOUTON_SORTIE).click();
        h.run();
        h.get_by_label(BOUTON_SORTIE).click(); // le champ vient d'être vidé
        h.run();
        let etat = h.state().etat.as_ref().unwrap();
        assert_eq!(etat.lignes("sucre", false)[0].quantite, 8);
        assert!(
            matches!(etat.retour, Some(Retour::Succes(_))),
            "{:?}",
            etat.retour
        );
    }

    // ---- Ce qu'egui peint VRAIMENT, avec la vraie palette ----------------------------------

    /// Un texte tel qu'egui le peint, avec la couleur du fond qui est derrière lui.
    struct TextePeint {
        texte: String,
        couleur: Rgb,
        fond: Rgb,
        rect: Rect,
    }

    struct Releve {
        fonds: Vec<(Rect, Color32)>,
        fond_par_defaut: Color32,
        textes: Vec<TextePeint>,
    }

    fn rgb(c: Color32) -> Rgb {
        [c.r(), c.g(), c.b()]
    }

    /// Parcourt les formes dans l'ordre où egui les peint : un fond opaque peint avant un
    /// texte, et qui le contient, est le fond de ce texte.
    fn parcourir(forme: &Shape, releve: &mut Releve) {
        match forme {
            Shape::Vec(formes) => formes.iter().for_each(|f| parcourir(f, releve)),
            Shape::Rect(r) if r.fill.a() == 255 => releve.fonds.push((r.rect, r.fill)),
            Shape::Text(t) => {
                let mut couleur = t
                    .override_text_color
                    .or(t.galley.job.sections.first().map(|s| s.format.color))
                    .unwrap_or(t.fallback_color);
                if couleur == Color32::PLACEHOLDER {
                    couleur = t.fallback_color;
                }
                let rect = t.galley.rect.translate(t.pos.to_vec2());
                let fond = releve
                    .fonds
                    .iter()
                    .rev()
                    .find(|(r, _)| r.contains(rect.center()))
                    .map_or(releve.fond_par_defaut, |(_, c)| *c);
                if !t.galley.job.text.trim().is_empty() {
                    releve.textes.push(TextePeint {
                        texte: t.galley.job.text.clone(),
                        couleur: rgb(couleur),
                        fond: rgb(fond),
                        rect,
                    });
                }
            }
            _ => {}
        }
    }

    /// Dessine l'application avec la VRAIE palette (`palette::appliquer`) puis relève chaque
    /// texte peint. `taille` : la fenêtre ; `zoom` : le texte agrandi (Cmd/Ctrl +), qui
    /// réduit d'autant la place disponible, en points.
    fn textes_peints(app: &mut Application, taille: Vec2, zoom: f32) -> Vec<TextePeint> {
        let ctx = Context::default();
        palette::appliquer(&ctx);
        let entree = || RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, taille / zoom)),
            ..Default::default()
        };
        let _ = ctx.run_ui(entree(), |ui| dessiner(ui, app)); // première image : mise en page
        let sortie = ctx.run_ui(entree(), |ui| dessiner(ui, app));
        let mut releve = Releve {
            fonds: Vec::new(),
            fond_par_defaut: ctx.global_style().visuals.panel_fill,
            textes: Vec::new(),
        };
        for forme in &sortie.shapes {
            parcourir(&forme.shape, &mut releve);
        }
        releve.textes
    }

    /// Toutes les saisies remplies : aucun bouton n'est grisé (un contrôle inactif est
    /// dispensé de contraste), et un produit est sélectionné.
    fn app_toute_remplie(nom: &str) -> Application {
        let mut app = app_avec_produits(nom);
        app.selection = Some("Riz 5kg".into());
        app.saisie_nom = "Thé vert".into();
        app.saisie_seuil = "3".into();
        app.saisie_quantite = "2".into();
        app.saisie_nouveau_nom = "Riz basmati".into();
        app
    }

    #[test]
    fn tous_les_textes_peints_atteignent_4_5_sur_leur_fond() {
        for confirmation in [false, true] {
            let mut app = app_toute_remplie("contraste");
            app.confirmer_suppression = confirmation;
            let textes = textes_peints(&mut app, Vec2::new(960.0, 640.0), 1.0);
            assert!(textes.len() > 20, "trop peu de textes : {}", textes.len());
            for t in &textes {
                let rapport = rapport_contraste(t.couleur, t.fond);
                assert!(
                    rapport >= 4.5,
                    "« {} » : {rapport:.2}:1 ({:?} sur {:?})",
                    t.texte,
                    t.couleur,
                    t.fond
                );
            }
        }
    }

    #[test]
    fn l_alerte_reste_visible_avec_un_texte_agrandi_ou_une_petite_fenetre() {
        let cas = [
            (
                "texte agrandi à 200 %",
                Vec2::new(960.0, 640.0),
                2.0,
                "Riz 5kg",
            ),
            (
                "petite fenêtre, nom très long",
                Vec2::new(640.0, 420.0),
                1.0,
                "Huile d'arachide Lesieur bouteille plastique 1 litre (carton de 12)",
            ),
        ];
        for (nom_du_cas, taille, zoom, produit) in cas {
            let mut etat = Etat::ouvrir(chemin_temp("visible")).unwrap();
            assert!(etat.ajouter(produit, "5")); // stock 0 <= seuil 5 : en alerte
            let mut app = Application::nouvelle(etat);
            let ecran = Rect::from_min_size(Pos2::ZERO, taille / zoom);
            let textes = textes_peints(&mut app, taille, zoom);
            let alerte = textes
                .iter()
                .find(|t| t.texte == ETAT_ALERTE)
                .unwrap_or_else(|| panic!("{nom_du_cas} : « {ETAT_ALERTE} » n'est pas peint"));
            assert!(
                ecran.contains_rect(alerte.rect),
                "{nom_du_cas} : l'alerte déborde de l'écran : {:?} hors de {ecran:?}",
                alerte.rect
            );
        }
    }
}
