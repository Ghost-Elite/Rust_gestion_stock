//! Couleurs de l'interface et calcul du contraste (WCAG 2.1).
//!
//! Les valeurs sont fixes et vérifiées par les tests en bas de fichier : texte >= 4,5:1
//! (critère 1.4.3) et contours / focus >= 3:1 (critère 1.4.11).

use eframe::egui::{Color32, Stroke, Visuals};

pub type Rgb = [u8; 3];

pub const FOND: Rgb = [255, 255, 255];
pub const FOND_PANNEAU: Rgb = [243, 244, 246];
pub const TEXTE: Rgb = [17, 24, 39];
pub const BORDURE: Rgb = [107, 114, 128];
pub const BOUTON_FOND: Rgb = [30, 64, 175];
pub const BOUTON_SURVOL_FOND: Rgb = [23, 37, 84];
pub const BOUTON_TEXTE: Rgb = [255, 255, 255];
pub const ALERTE_FOND: Rgb = [254, 226, 226];
pub const ALERTE_TEXTE: Rgb = [153, 27, 27];
pub const SELECTION_FOND: Rgb = [219, 234, 254];
pub const SELECTION_TEXTE: Rgb = [17, 24, 39];
pub const FOCUS: Rgb = [37, 99, 235];

pub fn couleur(c: Rgb) -> Color32 {
    Color32::from_rgb(c[0], c[1], c[2])
}

/// Luminance relative d'une couleur sRGB, de 0,0 (noir) à 1,0 (blanc).
fn luminance(c: Rgb) -> f64 {
    let lineaire = |canal: u8| {
        let c = f64::from(canal) / 255.0;
        if c <= 0.03928 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * lineaire(c[0]) + 0.7152 * lineaire(c[1]) + 0.0722 * lineaire(c[2])
}

/// Rapport de contraste WCAG entre deux couleurs : de 1,0 à 21,0.
pub fn rapport_contraste(a: Rgb, b: Rgb) -> f64 {
    let (la, lb) = (luminance(a), luminance(b));
    let (clair, sombre) = if la >= lb { (la, lb) } else { (lb, la) };
    (clair + 0.05) / (sombre + 0.05)
}

/// Le style egui complet, construit à partir des constantes ci-dessus.
/// On n'utilise pas `override_text_color` : il écraserait le texte blanc des boutons.
pub fn visuals() -> Visuals {
    let mut v = Visuals::light();
    v.panel_fill = couleur(FOND);
    v.window_fill = couleur(FOND);
    v.extreme_bg_color = couleur(FOND); // fond des champs de saisie
    v.faint_bg_color = couleur(FOND_PANNEAU);
    v.hyperlink_color = couleur(BOUTON_FOND);

    let contour = Stroke::new(1.0, couleur(BORDURE));
    let contour_focus = Stroke::new(2.0, couleur(FOCUS));

    // Libellés et cadres : texte sombre sur fond clair.
    let w = &mut v.widgets.noninteractive;
    w.bg_fill = couleur(FOND_PANNEAU);
    w.weak_bg_fill = couleur(FOND_PANNEAU);
    w.bg_stroke = contour;
    w.fg_stroke = Stroke::new(1.0, couleur(TEXTE));

    // Boutons : texte blanc sur fond bleu foncé ; egui peint les boutons avec `weak_bg_fill`.
    for (w, fond, trait_) in [
        (&mut v.widgets.inactive, BOUTON_FOND, contour),
        (&mut v.widgets.hovered, BOUTON_SURVOL_FOND, contour_focus),
        (&mut v.widgets.active, BOUTON_SURVOL_FOND, contour_focus),
        (&mut v.widgets.open, BOUTON_SURVOL_FOND, contour_focus),
    ] {
        w.bg_fill = couleur(fond);
        w.weak_bg_fill = couleur(fond);
        w.bg_stroke = trait_;
        w.fg_stroke = Stroke::new(1.0, couleur(BOUTON_TEXTE));
    }

    // Ligne sélectionnée : texte sombre sur fond bleu clair (egui s'en sert aussi pour le
    // contour d'un champ de saisie qui a le focus).
    v.selection.bg_fill = couleur(SELECTION_FOND);
    v.selection.stroke = Stroke::new(1.0, couleur(SELECTION_TEXTE));
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rgb(c: eframe::egui::Color32) -> Rgb {
        [c.r(), c.g(), c.b()]
    }

    #[test]
    fn contraste_noir_sur_blanc_vaut_21() {
        assert!((rapport_contraste([0, 0, 0], [255, 255, 255]) - 21.0).abs() < 0.01);
    }

    #[test]
    fn contraste_est_symetrique() {
        let (a, b) = ([30, 64, 175], [254, 226, 226]);
        assert!((rapport_contraste(a, b) - rapport_contraste(b, a)).abs() < 1e-9);
    }

    #[test]
    fn le_gris_767676_passe_le_seuil_aa_et_le_gris_777777_echoue() {
        assert!(rapport_contraste([0x76; 3], FOND) >= 4.5);
        assert!(rapport_contraste([0x77; 3], FOND) < 4.5);
    }

    #[test]
    fn les_textes_de_la_palette_atteignent_4_5() {
        let paires = [
            ("texte/fond", TEXTE, FOND),
            ("texte/panneau", TEXTE, FOND_PANNEAU),
            ("bouton", BOUTON_TEXTE, BOUTON_FOND),
            ("bouton survolé", BOUTON_TEXTE, BOUTON_SURVOL_FOND),
            ("alerte", ALERTE_TEXTE, ALERTE_FOND),
            ("sélection", SELECTION_TEXTE, SELECTION_FOND),
        ];
        for (nom, texte, fond) in paires {
            assert!(
                rapport_contraste(texte, fond) >= 4.5,
                "{nom}: {}",
                rapport_contraste(texte, fond)
            );
        }
    }

    #[test]
    fn les_contours_et_le_focus_de_la_palette_atteignent_3() {
        let paires = [
            ("bordure/fond", BORDURE, FOND),
            ("bordure/panneau", BORDURE, FOND_PANNEAU),
            ("focus/fond", FOCUS, FOND),
            ("focus/panneau", FOCUS, FOND_PANNEAU),
            ("bouton/fond", BOUTON_FOND, FOND),
        ];
        for (nom, a, b) in paires {
            assert!(
                rapport_contraste(a, b) >= 3.0,
                "{nom}: {}",
                rapport_contraste(a, b)
            );
        }
    }

    #[test]
    fn le_style_egui_respecte_les_seuils_dans_tous_les_etats() {
        let v = visuals();
        // Texte des libellés sur les fonds de fenêtre, de panneau et de champ de saisie.
        for fond in [v.panel_fill, v.window_fill, v.extreme_bg_color] {
            assert!(
                rapport_contraste(rgb(v.widgets.noninteractive.fg_stroke.color), rgb(fond)) >= 4.5
            );
        }
        // Boutons : texte sur fond, dans les trois états. egui peint les boutons avec
        // `weak_bg_fill` (et les curseurs avec `bg_fill`) : les deux doivent passer.
        for w in [v.widgets.inactive, v.widgets.hovered, v.widgets.active] {
            assert!(rapport_contraste(rgb(w.fg_stroke.color), rgb(w.bg_fill)) >= 4.5);
            assert!(rapport_contraste(rgb(w.fg_stroke.color), rgb(w.weak_bg_fill)) >= 4.5);
        }
        // Ligne sélectionnée : texte sur fond de sélection.
        assert!(rapport_contraste(rgb(v.selection.stroke.color), rgb(v.selection.bg_fill)) >= 4.5);
        // Bordures des composants sur le fond de panneau.
        assert!(
            rapport_contraste(rgb(v.widgets.inactive.bg_stroke.color), rgb(v.panel_fill)) >= 3.0
        );
    }
}
