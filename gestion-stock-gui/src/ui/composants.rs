//! Les petits éléments de dessin partagés par tous les écrans : cartes, champs, boutons, puces.
//! Les couleurs viennent de `palette`, où leur contraste est vérifié par des tests.

use crate::palette::{
    ACCENT, ACCENT_TEXTE, ALERTE_FOND, ALERTE_TEXTE, BORDURE_BOUTON, DANGER, FILET, FOCUS,
    FOND_DOUX, Rgb, SURFACE, TEXTE_DISCRET, VOILE, couleur,
};
use crate::police::{Graisse, famille_disponible, police_disponible};
use eframe::egui::accesskit::Role;
use eframe::egui::{
    self, Button, Color32, Context, CornerRadius, CursorIcon, FontId, Frame, Id, Margin, Pos2,
    Response, RichText, Sense, Stroke, StrokeKind, TextEdit, Ui, Vec2, WidgetInfo, WidgetType,
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
    bouton_large(ui, texte, style, actif, Vec2::new(0.0, 38.0))
}

/// Le même bouton avec une taille minimale au choix (les boutons des fenêtres sont larges).
pub(crate) fn bouton_large(
    ui: &mut Ui,
    texte: &str,
    style: Style,
    actif: bool,
    taille: Vec2,
) -> Response {
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
            .min_size(taille)
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
    cote: f32,
) -> Response {
    let reponse = ui.add_enabled(
        actif,
        Button::new(RichText::new(symbole).size(22.0))
            .min_size(Vec2::splat(cote))
            .stroke(Stroke::new(1.0, couleur(BORDURE_BOUTON)))
            .corner_radius(CornerRadius::same(8)),
    );
    anneau_de_focus(ui, &reponse, 8);
    reponse.widget_info(|| WidgetInfo::labeled(WidgetType::Button, actif, nom_accessible));
    reponse
}

/// Une ligne de texte d'un bouton à plusieurs lignes.
pub(crate) struct LigneBouton<'a> {
    pub texte: &'a str,
    pub taille: f32,
    pub graisse: Graisse,
    pub couleur: Rgb,
}

/// Un bouton dessiné à la main, de taille exacte, avec une ou plusieurs lignes de texte centrées
/// (un bouton egui centre mal un texte sur plusieurs lignes). C'est un vrai contrôle : focus clavier
/// avec son anneau, curseur main, survol, état grisé, et un nom complet pour les lecteurs d'écran.
#[allow(clippy::too_many_arguments)]
pub(crate) fn bouton_a_lignes(
    ui: &mut Ui,
    nom_accessible: &str,
    taille: Vec2,
    fond: Rgb,
    contour: Stroke,
    rayon: u8,
    lignes: &[LigneBouton],
    actif: bool,
) -> Response {
    let sense = if actif {
        Sense::click()
    } else {
        Sense::hover()
    };
    let (rect, reponse) = ui.allocate_exact_size(taille, sense);
    if ui.is_rect_visible(rect) {
        let galeries: Vec<_> = lignes
            .iter()
            .map(|ligne| {
                ui.painter().layout_no_wrap(
                    ligne.texte.to_string(),
                    police_disponible(ui.ctx(), ligne.graisse, ligne.taille),
                    couleur(ligne.couleur),
                )
            })
            .collect();
        let ecart = 2.0;
        let hauteur: f32 = galeries.iter().map(|g| g.size().y).sum::<f32>()
            + ecart * galeries.len().saturating_sub(1) as f32;
        let peintre = ui.painter();
        peintre.rect(
            rect,
            CornerRadius::same(rayon),
            couleur(fond),
            contour,
            StrokeKind::Inside,
        );
        if actif && reponse.hovered() {
            peintre.rect_filled(
                rect,
                CornerRadius::same(rayon),
                Color32::from_black_alpha(18),
            );
        }
        let mut y = rect.center().y - hauteur / 2.0;
        for (galerie, ligne) in galeries.into_iter().zip(lignes) {
            let x = rect.center().x - galerie.size().x / 2.0;
            let h = galerie.size().y;
            peintre.galley(Pos2::new(x, y), galerie, couleur(ligne.couleur));
            y += h + ecart;
        }
    }
    anneau_de_focus(ui, &reponse, rayon);
    reponse.widget_info(|| WidgetInfo::labeled(WidgetType::Button, actif, nom_accessible));
    if actif {
        reponse.on_hover_cursor(CursorIcon::PointingHand)
    } else {
        reponse
    }
}

/// Les initiales d'un produit (deux lettres au plus) : première lettre des deux premiers mots, ou
/// les deux premières lettres s'il n'y a qu'un mot utile.
pub(crate) fn initiales(nom: &str) -> String {
    let mots: Vec<&str> = nom.split_whitespace().collect();
    let mut lettres: Vec<char> = mots
        .iter()
        .filter_map(|mot| mot.chars().next())
        .filter(|c| c.is_alphabetic())
        .take(2)
        .collect();
    if lettres.len() < 2 {
        lettres = mots
            .first()
            .map(|mot| mot.chars().filter(|c| c.is_alphabetic()).take(2).collect())
            .unwrap_or_default();
    }
    lettres.iter().flat_map(|c| c.to_uppercase()).collect()
}

/// Le repère d'un produit (ses images n'existent pas encore) : un carré arrondi gris clair avec
/// ses initiales.
pub(crate) fn repere(ui: &mut Ui, nom: &str, cote: f32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(cote), Sense::hover());
    if !ui.is_rect_visible(rect) {
        return;
    }
    let galerie = ui.painter().layout_no_wrap(
        initiales(nom),
        police_disponible(ui.ctx(), Graisse::SemiGrasse, cote * 0.32),
        couleur(TEXTE_DISCRET),
    );
    let peintre = ui.painter();
    peintre.rect_filled(rect, CornerRadius::same(8), couleur(FOND_DOUX));
    peintre.galley(
        rect.center() - galerie.size() / 2.0,
        galerie,
        couleur(TEXTE_DISCRET),
    );
}

/// Une fenêtre modale : un voile sur tout l'écran, une carte blanche (coins de 16, marge de 28)
/// de la largeur donnée, annoncée aux lecteurs d'écran comme un dialogue nommé. Renvoie vrai si la
/// personne veut la fermer (touche Échap, ou clic sur le voile).
pub(crate) fn modale(
    ctx: &Context,
    id: &str,
    largeur: f32,
    nom_accessible: &str,
    contenu: impl FnOnce(&mut Ui),
) -> bool {
    let cadre = Frame::new()
        .fill(couleur(SURFACE))
        .corner_radius(CornerRadius::same(16))
        .inner_margin(Margin::same(28));
    // Sans fondu d'apparition : à la caisse la fenêtre doit être là tout de suite (et le relevé
    // de contraste des tests ne doit pas voir des fonds translucides).
    let reponse = egui::Modal::new(Id::new(id))
        .area(egui::Modal::default_area(Id::new(id)).fade_in(false))
        .backdrop_color(VOILE)
        .frame(cadre)
        .show(ctx, |ui| {
            ui.set_width(largeur);
            contenu(ui);
        });
    ctx.accesskit_node_builder(reponse.response.id, |noeud| {
        noeud.set_role(Role::Dialog);
        noeud.set_label(nom_accessible);
    });
    reponse.should_close()
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
    fn le_repere_d_un_produit_porte_ses_initiales() {
        assert_eq!(initiales("Riz parfumé 5 kg"), "RP");
        assert_eq!(initiales("thé vert"), "TV");
        assert_eq!(initiales("Baguette"), "BA");
        // Un seul mot utile : ses deux premières lettres.
        assert_eq!(initiales("Riz 5kg"), "RI");
        assert_eq!(initiales(""), "");
        assert_eq!(initiales("   "), "");
    }

    #[test]
    fn un_bouton_secondaire_garde_le_contour_clair_de_la_maquette() {
        let formes = dessiner_bouton(Style::Secondaire, false);
        assert!(a_un_contour(&formes, couleur(BORDURE_BOUTON), 1.0));
    }
}
