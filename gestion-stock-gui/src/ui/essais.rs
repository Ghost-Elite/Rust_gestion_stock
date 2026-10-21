//! Aides communes aux tests de l'interface : boutiques d'exemple, fenêtre de test, relevé des
//! textes réellement peints (pour vérifier leur contraste).

use super::*;
use crate::etat::Etat;
use crate::palette::{self, Rgb, rapport_contraste};
use eframe::egui::accesskit::Role;
use eframe::egui::{Color32, Context, Pos2, RawInput, Rect, Shape, Vec2};
use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};
use gestion_stock::horodatage::Horodatage;
use gestion_stock::vente::Moyen;
use std::path::PathBuf;

pub(crate) fn chemin_temp(nom: &str) -> PathBuf {
    let chemin = std::env::temp_dir().join(format!(
        "gestion-stock-gui-ui-{}-{nom}.txt",
        std::process::id()
    ));
    let _ = std::fs::remove_file(&chemin);
    chemin
}

/// Une grande fenêtre : avec les cartes et l'espace du design, une fenêtre de 800 x 600 ne
/// montrerait pas tous les boutons, et ce qui est hors de l'écran n'existe pas pour le test.
pub(crate) fn harness(app: Application) -> Harness<'static, Application> {
    let mut h = Harness::builder()
        .with_size(Vec2::new(1100.0, 900.0))
        .build_ui_state(|ui, app: &mut Application| dessiner(ui, app), app);
    h.run();
    h
}

/// Tape dans un champ comme le ferait une personne : d'abord le focus, puis le texte.
pub(crate) fn saisir(h: &mut Harness<'static, Application>, libelle: &str, texte: &str) {
    h.get_by_label(libelle).focus();
    h.run();
    h.get_by_label(libelle).type_text(texte);
    h.run();
}

// ---- Ce qu'egui peint VRAIMENT, avec la vraie palette ----------------------------------

/// Un texte tel qu'egui le peint, avec la couleur du fond qui est derrière lui.
pub(crate) struct TextePeint {
    pub(crate) texte: String,
    pub(crate) couleur: Rgb,
    pub(crate) fond: Rgb,
    pub(crate) rect: Rect,
}

pub(crate) struct Releve {
    pub(crate) fonds: Vec<(Rect, Color32)>,
    pub(crate) fond_par_defaut: Color32,
    pub(crate) textes: Vec<TextePeint>,
}

pub(crate) fn rgb(c: Color32) -> Rgb {
    [c.r(), c.g(), c.b()]
}

/// Parcourt les formes dans l'ordre où egui les peint : un fond opaque peint avant un
/// texte, et qui le contient, est le fond de ce texte.
pub(crate) fn parcourir(forme: &Shape, releve: &mut Releve) {
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
pub(crate) fn textes_peints(app: &mut Application, taille: Vec2, zoom: f32) -> Vec<TextePeint> {
    let ctx = Context::default();
    palette::appliquer(&ctx);
    let entree = || RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, taille / zoom)),
        ..Default::default()
    };
    // Comme dans une vraie fenêtre, la mise en page se stabilise en quelques images.
    for _ in 0..4 {
        let _ = ctx.run_ui(entree(), |ui| dessiner(ui, app));
    }
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

pub(crate) const FIXE: Horodatage = Horodatage {
    utc: 1_700_000_000,
    decalage_minutes: 60,
};

/// Pain : 10 en stock à 1,20 € (TVA 5,5 %) ; Savon : 10 à 2,50 € et Lait : 10 à 0,90 € (TVA par
/// défaut, 20 %). L'horloge est fixe : 14/11/2023 à 23 h 13 heure locale.
pub(crate) fn etat_vendeur(nom: &str) -> Etat {
    let mut e = Etat::ouvrir(chemin_temp(nom))
        .unwrap()
        .avec_horloge(|| FIXE);
    assert!(e.ajouter_avec_prix("Pain", "0", "1,20", "5,5"));
    assert!(e.entree("Pain", "10"));
    assert!(e.ajouter_avec_prix("Savon", "0", "2,50", ""));
    assert!(e.entree("Savon", "10"));
    assert!(e.ajouter_avec_prix("Lait", "0", "0,90", ""));
    assert!(e.entree("Lait", "10"));
    e
}

/// La boutique d'exemple, sans produit bloqué : tous les boutons peuvent être actifs.
pub(crate) fn app_vendeuse(nom: &str) -> Application {
    Application::nouvelle(etat_vendeur(nom))
}

/// La même, avec un produit épuisé (« Sel ») et un produit sans prix (« Thé »).
pub(crate) fn app_vendeuse_avec_blocages(nom: &str) -> Application {
    let mut etat = etat_vendeur(nom);
    assert!(etat.ajouter_avec_prix("Sel", "0", "0,80", "")); // stock 0 : épuisé
    assert!(etat.ajouter("Thé", "0"));
    assert!(etat.entree("Thé", "5")); // pas de prix
    Application::nouvelle(etat)
}

/// Avec le VRAI style (polices, espacements) : ce que voit la personne.
pub(crate) fn harness_vente(app: Application, taille: Vec2) -> Harness<'static, Application> {
    let mut h = Harness::builder()
        .with_size(taille)
        .build_ui_state(|ui, app: &mut Application| dessiner(ui, app), app);
    // Comme dans la vraie fenêtre, la palette et les polices sont installées AVANT la première
    // image : egui refuse une famille de police qu'il ne connaît pas encore.
    palette::appliquer(&h.ctx);
    h.run();
    h
}

pub(crate) const FENETRE: Vec2 = Vec2::new(TAILLE_FENETRE[0], TAILLE_FENETRE[1]);

pub(crate) fn cliquer(h: &mut Harness<'static, Application>, libelle: &str) {
    h.get_by_role_and_label(Role::Button, libelle).click();
    h.run();
}

pub(crate) fn est_grise(h: &Harness<'static, Application>, libelle: &str) -> bool {
    h.get_by_role_and_label(Role::Button, libelle)
        .accesskit_node()
        .is_disabled()
}

pub(crate) fn nombre_de_pieces(h: &Harness<'static, Application>) -> usize {
    h.state().etat.as_ref().unwrap().pieces_recentes(100).len()
}

/// Sans les textes d'un contrôle grisé : les WCAG les dispensent de contraste (la raison est
/// écrite à côté).
pub(crate) fn sans_les_textes_grises(textes: Vec<TextePeint>) -> Vec<TextePeint> {
    textes
        .into_iter()
        .filter(|t| t.couleur != palette::GRISE_TEXTE)
        .collect()
}

pub(crate) fn verifier_contrastes(textes: &[TextePeint], nom_du_cas: &str) {
    assert!(
        textes.len() >= 6,
        "{nom_du_cas} : trop peu de textes : {}",
        textes.len()
    );
    for t in textes {
        let rapport = rapport_contraste(t.couleur, t.fond);
        assert!(
            rapport >= 4.5,
            "{nom_du_cas} : « {} » : {rapport:.2}:1 ({:?} sur {:?})",
            t.texte,
            t.couleur,
            t.fond
        );
    }
}

/// Une vente par carte (Pain × 3 = 3,60 €) puis une en espèces (Savon × 1 = 2,50 €),
/// affichées sur l'écran Historique.
pub(crate) fn app_avec_ventes(nom: &str) -> Application {
    let mut etat = etat_vendeur(nom);
    assert!(etat.encaisser(&[("Pain".to_string(), 3)], Moyen::Carte, ""));
    assert!(etat.encaisser(&[("Savon".to_string(), 1)], Moyen::Especes, "5"));
    etat.effacer_ticket();
    let mut app = Application::nouvelle(etat);
    app.ecran = Ecran::Historique;
    app
}

/// Un journal modifié à la main après une vente : le programme l'ouvre mais refuse de vendre.
pub(crate) fn app_journal_altere(nom: &str) -> Application {
    let chemin = chemin_temp(nom);
    {
        let mut e = etat_vendeur(&format!("{nom}-source"));
        assert!(e.encaisser(&[("Pain".to_string(), 1)], Moyen::Carte, ""));
        std::fs::copy(e.chemin(), &chemin).unwrap();
    }
    let contenu = std::fs::read_to_string(&chemin).unwrap();
    // Une modification à la main, dans le journal : le pain vendu passe de 1,20 à 0,10.
    let altere = contenu.replacen("L;1;Pain;1;120;", "L;1;Pain;1;10;", 1);
    assert_ne!(
        contenu, altere,
        "le fichier n'a pas le format attendu :\n{contenu}"
    );
    std::fs::write(&chemin, altere).unwrap();
    Application::nouvelle(Etat::ouvrir(chemin).unwrap().avec_horloge(|| FIXE))
}

pub(crate) fn est_selectionne(h: &Harness<'static, Application>, libelle: &str) -> Option<bool> {
    h.get_by_role_and_label(Role::Button, libelle)
        .accesskit_node()
        .toggled()
        .map(|t| t == eframe::egui::accesskit::Toggled::True)
}

pub(crate) fn quantite_en_stock(h: &Harness<'static, Application>, nom: &str) -> u32 {
    let etat = h.state().etat.as_ref().unwrap();
    etat.lignes(nom, false)[0].quantite
}

/// Tape un texte dans le champ de scan et valide par Entrée, comme une personne (ou un lecteur
/// de codes-barres, qui tape le code puis Entrée).
pub(crate) fn scanner(h: &mut Harness<'static, Application>, texte: &str) {
    saisir(h, CHAMP_SCAN, texte);
    h.key_press(eframe::egui::Key::Enter);
    h.run();
}
