//! Couleurs de l'interface et calcul du contraste (WCAG 2.1).
//!
//! Les valeurs sont fixes et vérifiées par les tests en bas de fichier : texte >= 4,5:1
//! (critère 1.4.3) et contours / focus >= 3:1 (critère 1.4.11).

use eframe::egui::{
    Color32, Context, CornerRadius, FontFamily, FontId, Stroke, TextStyle, Theme, ThemePreference,
    Vec2, Visuals, epaint::Shadow,
};

pub type Rgb = [u8; 3];

// ---- Fonds et textes ------------------------------------------------------------------------
/// Le fond de la fenêtre : un gris très clair, sur lequel les cartes blanches se détachent.
pub const FOND: Rgb = [246, 247, 251];
/// Le fond des cartes, des champs de saisie et de l'en-tête.
pub const SURFACE: Rgb = [255, 255, 255];
pub const FOND_DOUX: Rgb = [243, 244, 246];
pub const TEXTE: Rgb = [17, 24, 39];
/// Pour les libellés secondaires : reste lisible (>= 4,5:1) sur le fond comme sur les cartes.
pub const TEXTE_DISCRET: Rgb = [75, 85, 99];
/// Les contours des composants (champs, boutons) : au moins 3:1, critère WCAG 1.4.11.
pub const BORDURE: Rgb = [120, 130, 145];
/// Les filets décoratifs (bords de cartes, séparateurs) : volontairement discrets.
pub const FILET: Rgb = [226, 230, 236];

// ---- Couleur d'accent -----------------------------------------------------------------------
pub const ACCENT: Rgb = [67, 56, 202];
pub const ACCENT_SURVOL: Rgb = [55, 48, 163];
pub const ACCENT_TEXTE: Rgb = [255, 255, 255];
pub const ACCENT_DOUX: Rgb = [238, 242, 255];
pub const FOCUS: Rgb = ACCENT;

// Les boutons ordinaires sont CLAIRS avec un texte sombre : dans egui, la couleur de texte d'un
// widget interactif est aussi celle du texte saisi, des libellés de cases et des noms de
// produits. Un texte blanc sur bouton foncé deviendrait blanc sur blanc partout ailleurs.
// Les boutons principaux (fond d'accent, texte blanc) fixent leurs couleurs un par un.
pub const BOUTON_FOND: Rgb = SURFACE;
pub const BOUTON_SURVOL_FOND: Rgb = ACCENT_DOUX;
pub const BOUTON_ACTIF_FOND: Rgb = [199, 210, 254];
pub const BOUTON_TEXTE: Rgb = TEXTE;

pub const SELECTION_FOND: Rgb = ACCENT_DOUX;
pub const SELECTION_TEXTE: Rgb = TEXTE;

// ---- États ----------------------------------------------------------------------------------
pub const ALERTE_FOND: Rgb = [254, 226, 226];
pub const ALERTE_TEXTE: Rgb = [153, 27, 27];
pub const SUCCES_FOND: Rgb = [220, 252, 231];
pub const SUCCES_TEXTE: Rgb = [22, 101, 52];
/// Le mot « Sortie » dans l'historique (le mot « Entrée » prend la couleur du succès).
pub const SORTIE_TEXTE: Rgb = [146, 64, 14];
/// Le fond du bouton « Confirmer la suppression » (texte blanc dessus).
pub const DANGER: Rgb = [185, 28, 28];

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

/// Une ombre légère, pour que les cartes se détachent du fond.
pub fn ombre() -> Shadow {
    Shadow {
        offset: [0, 2],
        blur: 12,
        spread: 0,
        color: Color32::from_black_alpha(20),
    }
}

/// Le style egui complet, construit à partir des constantes ci-dessus.
/// On n'utilise pas `override_text_color` : il écraserait le texte blanc des boutons principaux.
pub fn visuals() -> Visuals {
    let mut v = Visuals::light();
    v.panel_fill = couleur(FOND);
    v.window_fill = couleur(SURFACE);
    v.extreme_bg_color = couleur(SURFACE); // fond des champs de saisie
    v.faint_bg_color = couleur(FOND_DOUX);
    v.hyperlink_color = couleur(FOCUS);
    v.window_corner_radius = CornerRadius::same(14);
    v.menu_corner_radius = CornerRadius::same(10);
    v.window_stroke = Stroke::new(1.0, couleur(FILET));
    v.window_shadow = ombre();
    v.popup_shadow = ombre();

    let arrondi = CornerRadius::same(8);
    let contour = Stroke::new(1.0, couleur(BORDURE));
    let contour_focus = Stroke::new(2.0, couleur(FOCUS));

    // Libellés et cadres : texte sombre sur fond clair, filets discrets.
    let w = &mut v.widgets.noninteractive;
    w.bg_fill = couleur(SURFACE);
    w.weak_bg_fill = couleur(SURFACE);
    w.bg_stroke = Stroke::new(1.0, couleur(FILET));
    w.fg_stroke = Stroke::new(1.0, couleur(TEXTE));
    w.corner_radius = arrondi;

    // Boutons et autres widgets interactifs : texte sombre sur fond clair, et une bordure
    // qui les distingue du fond ; egui peint les boutons avec `weak_bg_fill`.
    for (w, fond, trait_) in [
        (&mut v.widgets.inactive, BOUTON_FOND, contour),
        (&mut v.widgets.hovered, BOUTON_SURVOL_FOND, contour_focus),
        (&mut v.widgets.active, BOUTON_ACTIF_FOND, contour_focus),
        (&mut v.widgets.open, BOUTON_SURVOL_FOND, contour_focus),
    ] {
        w.bg_fill = couleur(fond);
        w.weak_bg_fill = couleur(fond);
        w.bg_stroke = trait_;
        w.fg_stroke = Stroke::new(1.0, couleur(BOUTON_TEXTE));
        w.corner_radius = arrondi;
    }

    // Ligne sélectionnée : texte sombre sur fond d'accent très clair (egui s'en sert aussi
    // pour le contour d'un champ de saisie qui a le focus).
    v.selection.bg_fill = couleur(SELECTION_FOND);
    v.selection.stroke = Stroke::new(1.5, couleur(FOCUS));
    v
}

/// À appeler au démarrage, avant le premier affichage. On impose le thème clair ET on y
/// range la palette : `Context::set_visuals` écrit dans le thème « courant », et avant le
/// premier affichage egui ignore encore celui du système (il suppose le sombre) ; la palette
/// atterrirait dans le mauvais thème et ne serait jamais utilisée sur un Mac en mode clair.
/// On règle aussi l'espacement et les tailles de texte : de l'air, de grandes cibles, une vraie
/// hiérarchie entre titre, texte et détail.
pub fn appliquer(ctx: &Context) {
    ctx.set_fonts(crate::police::definitions());
    ctx.set_theme(ThemePreference::Light);
    ctx.style_mut_of(Theme::Light, |style| {
        style.visuals = visuals();
        style.spacing.item_spacing = Vec2::new(10.0, 10.0);
        style.spacing.button_padding = Vec2::new(16.0, 9.0);
        style.spacing.interact_size.y = 36.0;
        style.text_styles = [
            (TextStyle::Heading, crate::police::semi_gras(24.0)),
            (TextStyle::Body, FontId::new(15.5, FontFamily::Proportional)),
            (
                TextStyle::Button,
                FontId::new(15.5, FontFamily::Proportional),
            ),
            (
                TextStyle::Small,
                FontId::new(13.5, FontFamily::Proportional),
            ),
            (
                TextStyle::Monospace,
                FontId::new(14.0, FontFamily::Monospace),
            ),
        ]
        .into();
    });
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
        assert!(rapport_contraste([0x76; 3], SURFACE) >= 4.5);
        assert!(rapport_contraste([0x77; 3], SURFACE) < 4.5);
    }

    #[test]
    fn les_textes_de_la_palette_atteignent_4_5() {
        let paires = [
            ("texte/fond", TEXTE, FOND),
            ("texte/carte", TEXTE, SURFACE),
            ("texte/fond doux", TEXTE, FOND_DOUX),
            ("discret/fond", TEXTE_DISCRET, FOND),
            ("discret/carte", TEXTE_DISCRET, SURFACE),
            ("bouton", BOUTON_TEXTE, BOUTON_FOND),
            ("bouton survolé", BOUTON_TEXTE, BOUTON_SURVOL_FOND),
            ("bouton pressé", BOUTON_TEXTE, BOUTON_ACTIF_FOND),
            ("bouton principal", ACCENT_TEXTE, ACCENT),
            ("bouton principal survolé", ACCENT_TEXTE, ACCENT_SURVOL),
            ("bouton de suppression", ACCENT_TEXTE, DANGER),
            ("alerte", ALERTE_TEXTE, ALERTE_FOND),
            ("alerte sur carte", ALERTE_TEXTE, SURFACE),
            ("correct", SUCCES_TEXTE, SUCCES_FOND),
            ("entrée dans l'historique", SUCCES_TEXTE, SURFACE),
            ("sortie dans l'historique", SORTIE_TEXTE, SURFACE),
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
            ("bordure/carte", BORDURE, SURFACE),
            ("focus/fond", FOCUS, FOND),
            ("focus/carte", FOCUS, SURFACE),
            // Les boutons secondaires sont clairs : c'est leur bordure qui les distingue.
            ("bordure/bouton", BORDURE, BOUTON_FOND),
            ("bouton principal/carte", ACCENT, SURFACE),
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
        // egui réutilise la couleur de texte des widgets interactifs SANS cadre : texte saisi
        // dans un champ, libellé d'une case, nom de produit non sélectionné (état « inactive »)
        // et titres du tableau (`strong`, état « active »). Elle doit donc passer aussi sur
        // les fonds de fenêtre, de panneau, de champ et de ligne alternée.
        for fond in [
            v.panel_fill,
            v.window_fill,
            v.extreme_bg_color,
            v.faint_bg_color,
        ] {
            assert!(rapport_contraste(rgb(v.widgets.inactive.fg_stroke.color), rgb(fond)) >= 4.5);
        }
        for fond in [v.panel_fill, v.window_fill, v.faint_bg_color] {
            assert!(rapport_contraste(rgb(v.widgets.active.fg_stroke.color), rgb(fond)) >= 4.5);
        }
    }

    #[test]
    fn le_style_est_arrondi_aere_et_lisible() {
        use eframe::egui::{Context, TextStyle};
        let v = visuals();
        assert!(v.widgets.inactive.corner_radius.nw >= 6, "boutons arrondis");
        assert!(v.window_corner_radius.nw >= 10, "fenêtres arrondies");

        let ctx = Context::default();
        appliquer(&ctx);
        let style = ctx.style_of(eframe::egui::Theme::Light);
        // Des cibles assez grandes (critère WCAG 2.5.8 : au moins 24 px) et de l'air.
        assert!(
            style.spacing.interact_size.y >= 34.0,
            "cibles de 34 px au moins"
        );
        assert!(
            style.spacing.item_spacing.y >= 8.0,
            "espacement vertical aéré"
        );
        assert!(style.spacing.button_padding.x >= 14.0);
        // Une vraie hiérarchie de textes : titre > texte > détail, et un texte courant à 15 px+.
        let taille = |t: &TextStyle| style.text_styles[t].size;
        assert_eq!(
            style.text_styles[&TextStyle::Heading].family,
            crate::police::famille_semi_grasse(),
            "les titres sont en semi-gras"
        );
        assert!(taille(&TextStyle::Heading) >= 22.0);
        assert!(taille(&TextStyle::Body) >= 15.0);
        assert!(taille(&TextStyle::Heading) > taille(&TextStyle::Body));
        assert!(taille(&TextStyle::Body) > taille(&TextStyle::Small));
        assert!(
            taille(&TextStyle::Small) >= 12.5,
            "le plus petit texte reste lisible"
        );
    }

    #[test]
    fn appliquer_impose_la_palette_quel_que_soit_le_theme_du_systeme() {
        use eframe::egui::{Context, RawInput, Theme};
        // Au démarrage, `appliquer` est appelé AVANT le premier affichage, quand egui ne
        // connaît pas encore le thème du système : la palette doit quand même s'imposer.
        for systeme in [Some(Theme::Light), Some(Theme::Dark), None] {
            let ctx = Context::default();
            appliquer(&ctx);
            let entree = RawInput {
                system_theme: systeme,
                ..Default::default()
            };
            let _ = ctx.run_ui(entree, |_ui| {});
            assert!(
                ctx.global_style().visuals == visuals(),
                "thème du système : {systeme:?}"
            );
        }
    }
}
