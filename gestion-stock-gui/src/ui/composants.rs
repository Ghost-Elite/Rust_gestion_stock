//! Les petits éléments de dessin partagés par tous les écrans : cartes, champs, boutons, puces.
//! Les couleurs viennent de `palette`, où leur contraste est vérifié par des tests.

use crate::palette::{
    ACCENT, ACCENT_TEXTE, ALERTE_FOND, ALERTE_TEXTE, BORDURE_BOUTON, DANGER, FILET, FOCUS, SURFACE,
    TEXTE_DISCRET, couleur,
};
use crate::police::{Graisse, famille_disponible};
use eframe::egui::{
    self, Button, CornerRadius, FontId, Frame, Margin, Response, RichText, Stroke, StrokeKind,
    TextEdit, Ui, Vec2, WidgetInfo, WidgetType,
};

// ---- Petits éléments de style -----------------------------------------------------------------

/// Une carte blanche aux coins arrondis, plate : un fin bord, pas d'ombre.
pub(crate) fn carte<R>(ui: &mut Ui, contenu: impl FnOnce(&mut Ui) -> R) -> R {
    carte_marge(ui, 16, contenu).inner
}

/// La même carte avec une marge au choix : le paiement, qui doit tenir à l'écran, en prend moins.
pub(crate) fn carte_marge<R>(
    ui: &mut Ui,
    marge: i8,
    contenu: impl FnOnce(&mut Ui) -> R,
) -> egui::InnerResponse<R> {
    Frame::new()
        .fill(couleur(SURFACE))
        .stroke(Stroke::new(1.0, couleur(FILET)))
        .corner_radius(CornerRadius::same(12))
        .inner_margin(Margin::same(marge))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            contenu(ui)
        })
}

/// Le titre d'une carte : un vrai titre pour les lecteurs d'écran, plus petit que celui de la page.
pub(crate) fn titre_de_carte(ui: &mut Ui, texte: &str) {
    ui.heading(RichText::new(texte).size(18.0));
}

/// Un texte secondaire, plus petit et plus gris (mais toujours >= 4,5:1).
pub(crate) fn discret(ui: &mut Ui, texte: &str) {
    ui.label(
        RichText::new(texte)
            .size(13.5)
            .color(couleur(TEXTE_DISCRET)),
    );
}

/// Une puce arrondie : l'état est toujours écrit dans la puce, la couleur n'est qu'un renfort.
pub(crate) fn puce(ui: &mut Ui, texte: &str, fond: [u8; 3], texte_couleur: [u8; 3]) {
    Frame::new()
        .fill(couleur(fond))
        .corner_radius(CornerRadius::same(12))
        .inner_margin(Margin::symmetric(10, 3))
        .show(ui, |ui| {
            ui.label(
                RichText::new(texte)
                    .size(13.5)
                    .color(couleur(texte_couleur)),
            );
        });
}

/// Un texte important sur fond rouge clair. La couleur n'est jamais seule : le texte dit tout.
pub(crate) fn bandeau_alerte(ui: &mut Ui, texte: &str) {
    Frame::new()
        .fill(couleur(ALERTE_FOND))
        .corner_radius(CornerRadius::same(8))
        .inner_margin(Margin::symmetric(10, 6))
        .show(ui, |ui| {
            ui.label(RichText::new(texte).color(couleur(ALERTE_TEXTE)));
        });
}

/// Un champ de saisie relié à son libellé, pour les lecteurs d'écran.
pub(crate) fn champ(ui: &mut Ui, libelle: &str, valeur: &mut String) -> Response {
    let etiquette = ui.label(
        RichText::new(libelle)
            .size(13.5)
            .color(couleur(TEXTE_DISCRET)),
    );
    ui.add(
        TextEdit::singleline(valeur)
            .desired_width(f32::INFINITY)
            .margin(Margin::symmetric(10, 8)),
    )
    .labelled_by(etiquette.id)
}

/// Un champ de saisie sur la même ligne que son libellé, pour gagner de la hauteur. On peut
/// y ajouter quelque chose à droite : `reserve` est la largeur à lui laisser.
pub(crate) fn champ_en_ligne(
    ui: &mut Ui,
    libelle: &str,
    valeur: &mut String,
    reserve: f32,
    a_droite: impl FnOnce(&mut Ui),
) -> Response {
    ui.horizontal(|ui| {
        let etiquette = ui.label(
            RichText::new(libelle)
                .size(13.5)
                .color(couleur(TEXTE_DISCRET)),
        );
        let largeur = (ui.available_width() - reserve - ui.spacing().item_spacing.x).max(60.0);
        let champ = ui
            .add(
                TextEdit::singleline(valeur)
                    .desired_width(largeur)
                    .margin(Margin::symmetric(10, 8)),
            )
            .labelled_by(etiquette.id);
        a_droite(ui);
        champ
    })
    .inner
}

#[derive(Clone, Copy)]
pub(crate) enum Style {
    /// Le bouton à mettre en avant : fond d'accent, texte blanc.
    Principal,
    /// Un bouton ordinaire : fond blanc, bordure, texte sombre.
    Secondaire,
    /// Une action qui supprime : texte et bordure rouges sur fond blanc.
    Danger,
    /// La confirmation d'une suppression : fond rouge, texte blanc.
    Confirmation,
}

/// Un bouton qui ne sert que si `actif` : sans cela, un double-clic enregistrerait l'action
/// une fois, puis afficherait « Erreur : quantité manquante » par-dessus le succès, et la
/// personne croirait que ça a échoué et recommencerait.
pub(crate) fn bouton(ui: &mut Ui, texte: &str, style: Style, actif: bool) -> Response {
    let bouton = match style {
        Style::Principal => Button::new(gras(ui, texte, 15.5).color(couleur(ACCENT_TEXTE)))
            .fill(couleur(ACCENT))
            .stroke(Stroke::NONE),
        Style::Secondaire => Button::new(texte).stroke(Stroke::new(1.0, couleur(BORDURE_BOUTON))),
        Style::Danger => Button::new(RichText::new(texte).color(couleur(ALERTE_TEXTE)))
            .fill(couleur(SURFACE))
            .stroke(Stroke::new(1.0, couleur(ALERTE_TEXTE))),
        Style::Confirmation => Button::new(gras(ui, texte, 15.5).color(couleur(ACCENT_TEXTE)))
            .fill(couleur(DANGER))
            .stroke(Stroke::NONE),
    };
    let reponse = ui.add_enabled(
        actif,
        bouton
            .min_size(Vec2::new(0.0, 38.0))
            .corner_radius(CornerRadius::same(10)),
    );
    anneau_de_focus(ui, &reponse, 10);
    reponse
}

/// Le contour de focus clavier (2, couleur de focus) : egui le dessine seul tant que le contour
/// du bouton suit son état ; un bouton au contour fixé (clair, rouge, absent) le perdrait.
pub(crate) fn anneau_de_focus(ui: &Ui, reponse: &Response, rayon: u8) {
    if reponse.has_focus() {
        ui.painter().rect_stroke(
            reponse.rect,
            CornerRadius::same(rayon),
            Stroke::new(2.0, couleur(FOCUS)),
            StrokeKind::Inside,
        );
    }
}

pub(crate) fn rempli(texte: &str) -> bool {
    !texte.trim().is_empty()
}

/// Un bouton « à bascule » : celui qui est choisi est plein et en accent, les autres sont
/// des boutons ordinaires.
pub(crate) fn bouton_choix(ui: &mut Ui, texte: &str, choisi: bool, largeur: f32) -> Response {
    let bouton = if choisi {
        Button::new(gras(ui, texte, 15.5).color(couleur(ACCENT_TEXTE)))
            .fill(couleur(ACCENT))
            .stroke(Stroke::NONE)
    } else {
        Button::new(texte)
    };
    ui.add(
        bouton
            .selected(choisi)
            .min_size(Vec2::new(largeur, 38.0))
            .corner_radius(CornerRadius::same(10)),
    )
}

/// Un texte en semi-gras : la graisse, plus que la taille, fait ressortir ce qui compte.
pub(crate) fn gras(ui: &Ui, texte: &str, taille: f32) -> RichText {
    RichText::new(texte).font(FontId::new(
        taille,
        famille_disponible(ui.ctx(), Graisse::SemiGrasse),
    ))
}

/// Un petit bouton carré (« − », « + »). Son texte n'est qu'un symbole : on lui donne un nom
/// complet pour les lecteurs d'écran (« Retirer une unité de Pain »).
pub(crate) fn bouton_symbole(
    ui: &mut Ui,
    symbole: &str,
    nom_accessible: &str,
    actif: bool,
) -> Response {
    let reponse = ui.add_enabled(
        actif,
        Button::new(RichText::new(symbole).size(18.0))
            .min_size(Vec2::splat(38.0))
            .corner_radius(CornerRadius::same(10)),
    );
    reponse.widget_info(|| WidgetInfo::labeled(WidgetType::Button, actif, nom_accessible));
    reponse
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::palette::{self, BORDURE_BOUTON, FOCUS};
    use eframe::egui::{Color32, Context, Id, Pos2, RawInput, Rect, Shape};

    /// Le premier rectangle de la liste des formes dont le contour a cette couleur et cette
    /// épaisseur au moins.
    fn a_un_contour(
        formes: &[eframe::egui::epaint::ClippedShape],
        c: Color32,
        epaisseur: f32,
    ) -> bool {
        fn cherche(forme: &Shape, c: Color32, epaisseur: f32) -> bool {
            match forme {
                Shape::Vec(v) => v.iter().any(|f| cherche(f, c, epaisseur)),
                Shape::Rect(r) => r.stroke.color == c && r.stroke.width >= epaisseur,
                _ => false,
            }
        }
        formes.iter().any(|f| cherche(&f.shape, c, epaisseur))
    }

    /// Dessine un bouton sur trois images ; si `focus`, le bouton reçoit le focus clavier.
    fn dessiner_bouton(style: Style, focus: bool) -> Vec<eframe::egui::epaint::ClippedShape> {
        let ctx = Context::default();
        palette::appliquer(&ctx);
        let entree = || RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 200.0))),
            ..Default::default()
        };
        let mut id = Id::NULL;
        let _ = ctx.run_ui(entree(), |ui| {
            id = bouton(ui, "Test", style, true).id;
        });
        if focus {
            ctx.memory_mut(|m| m.request_focus(id));
        }
        let _ = ctx.run_ui(entree(), |ui| {
            bouton(ui, "Test", style, true);
        });
        ctx.run_ui(entree(), |ui| {
            bouton(ui, "Test", style, true);
        })
        .shapes
    }

    #[test]
    fn le_focus_clavier_d_un_bouton_se_voit_meme_avec_un_contour_fixe() {
        let focus = couleur(FOCUS);
        for style in [
            Style::Principal,
            Style::Secondaire,
            Style::Danger,
            Style::Confirmation,
        ] {
            assert!(
                a_un_contour(&dessiner_bouton(style, true), focus, 2.0),
                "pas d'anneau de focus"
            );
            assert!(
                !a_un_contour(&dessiner_bouton(style, false), focus, 2.0),
                "anneau de focus sans focus"
            );
        }
    }

    #[test]
    fn un_bouton_secondaire_garde_le_contour_clair_de_la_maquette() {
        let formes = dessiner_bouton(Style::Secondaire, false);
        assert!(a_un_contour(&formes, couleur(BORDURE_BOUTON), 1.0));
    }
}
