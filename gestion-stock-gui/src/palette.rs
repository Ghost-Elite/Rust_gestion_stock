//! Couleurs de l'interface et calcul du contraste (WCAG 2.1).
//!
//! Les valeurs viennent de la maquette « POS Kora Marché » et sont fixes ; les tests en bas de
//! fichier vérifient : texte >= 4,5:1 (critère 1.4.3) et contours de champs / focus >= 3:1
//! (critère 1.4.11).
//!
//! Écarts avec la maquette, voulus : le contour des champs de saisie passe de `#AEB5BF`
//! (2,07:1 sur blanc) à `#848B96` (3,07:1 au moins, sur le fond de page comme sur les cartes) ;
//! les contours des boutons secondaires gardent le `#D5DAE0` clair de la maquette, ces boutons
//! étant identifiés par leur texte et non par leur bord.

use eframe::egui::{
    Color32, Context, CornerRadius, FontFamily, FontId, Stroke, TextStyle, Theme, ThemePreference,
    Vec2, Visuals, epaint::Shadow,
};

pub type Rgb = [u8; 3];

// ---- Fonds et textes ------------------------------------------------------------------------
/// Le fond de la fenêtre : un gris très clair, sur lequel les cartes blanches se détachent.
pub const FOND: Rgb = [242, 243, 245];
/// Le fond des cartes, des fenêtres modales et des champs de saisie.
pub const SURFACE: Rgb = [255, 255, 255];
/// Les petits fonds de remplacement (repère de produit, montants rapides).
pub const FOND_DOUX: Rgb = [245, 246, 248];
pub const TEXTE: Rgb = [22, 25, 29];
/// Les libellés de second rang : pour les montants et les détails.
pub const TEXTE_SECONDAIRE: Rgb = [74, 81, 91];
/// Pour les libellés discrets : reste lisible (>= 4,5:1) sur le fond comme sur les cartes.
pub const TEXTE_DISCRET: Rgb = [91, 99, 110];
/// Le contour des CHAMPS de saisie : au moins 3:1, critère WCAG 1.4.11.
pub const BORDURE: Rgb = [132, 139, 150];
/// Le contour (décoratif) des boutons secondaires : clair, comme dans la maquette.
pub const BORDURE_BOUTON: Rgb = [213, 218, 224];
/// Les bords de cartes et les séparateurs : volontairement discrets.
pub const FILET: Rgb = [225, 228, 232];
/// Le filet entre deux lignes d'une liste.
pub const FILET_LIGNE: Rgb = [238, 240, 243];

// ---- L'en-tête sombre -----------------------------------------------------------------------
pub const ENTETE_FOND: Rgb = [22, 25, 29];
pub const ENTETE_TEXTE: Rgb = [255, 255, 255];
pub const ENTETE_DISCRET: Rgb = [196, 202, 211];

// ---- Couleur d'accent -----------------------------------------------------------------------
pub const ACCENT: Rgb = [30, 86, 200];
pub const ACCENT_SURVOL: Rgb = [22, 63, 148];
pub const ACCENT_TEXTE: Rgb = [255, 255, 255];
/// La ligne choisie, le survol d'un bouton clair.
pub const ACCENT_DOUX: Rgb = [234, 240, 252];
pub const FOCUS: Rgb = ACCENT;

// Les boutons ordinaires sont CLAIRS avec un texte sombre : dans egui, la couleur de texte d'un
// widget interactif est aussi celle du texte saisi, des libellés de cases et des noms de
// produits. Un texte blanc sur bouton foncé deviendrait blanc sur blanc partout ailleurs.
// Les boutons principaux (fond d'accent, texte blanc) fixent leurs couleurs un par un.
pub const BOUTON_FOND: Rgb = SURFACE;
pub const BOUTON_SURVOL_FOND: Rgb = ACCENT_DOUX;
pub const BOUTON_ACTIF_FOND: Rgb = [214, 226, 248];
pub const BOUTON_TEXTE: Rgb = TEXTE;

pub const SELECTION_FOND: Rgb = ACCENT_DOUX;
pub const SELECTION_TEXTE: Rgb = TEXTE;

// ---- États ----------------------------------------------------------------------------------
pub const ALERTE_FOND: Rgb = [253, 236, 234];
pub const ALERTE_TEXTE: Rgb = [166, 27, 18];
pub const ALERTE_BORD: Rgb = [232, 180, 174];
pub const SUCCES_FOND: Rgb = [232, 245, 238];
pub const SUCCES_TEXTE: Rgb = [18, 101, 63];
/// Le fond d'un bouton « validé » (texte blanc dessus).
pub const SUCCES_BOUTON: Rgb = [18, 120, 74];
/// Un état à surveiller : stock bas, ticket remboursé, mot « Sortie » dans l'historique.
pub const AVERTISSEMENT_TEXTE: Rgb = [138, 65, 6];
/// Le fond du bouton de suppression ou de remboursement (texte blanc dessus).
pub const DANGER: Rgb = [180, 35, 24];
/// Un contrôle grisé : son texte est dispensé de contraste (WCAG 1.4.3), la raison s'écrit à côté.
pub const GRISE_FOND: Rgb = [227, 230, 234];
pub const GRISE_TEXTE: Rgb = [107, 114, 128];

/// Le voile derrière une fenêtre modale : noir-bleu à moitié transparent.
/// (Valeurs prémultipliées : 22, 25, 29 à 128/255, car `from_rgba_unmultiplied` n'est pas `const`.)
pub const VOILE: Color32 = Color32::from_rgba_premultiplied(11, 13, 15, 128);

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
/// On n'utilise pas `override_text_color` : il écraserait le texte blanc des boutons principaux.
pub fn visuals() -> Visuals {
    let mut v = Visuals::light();
    v.panel_fill = couleur(FOND);
    v.window_fill = couleur(SURFACE);
    v.extreme_bg_color = couleur(SURFACE); // fond des champs de saisie
    v.faint_bg_color = couleur(FOND_DOUX);
    v.hyperlink_color = couleur(FOCUS);
    v.window_corner_radius = CornerRadius::same(16);
    v.menu_corner_radius = CornerRadius::same(10);
    v.window_stroke = Stroke::new(1.0, couleur(FILET));
    v.window_shadow = Shadow::NONE;
    v.popup_shadow = Shadow::NONE;

    let arrondi = CornerRadius::same(10);
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
    // qui les distingue du fond ; egui peint les boutons avec `weak_bg_fill`. Cette bordure est
    // celle des CHAMPS ; les boutons secondaires fixent la leur (`BORDURE_BOUTON`).
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
        style.spacing.button_padding = Vec2::new(16.0, 10.0);
        style.spacing.interact_size.y = 48.0;
        style.text_styles = [
            (
                TextStyle::Heading,
                crate::police::police(crate::police::Graisse::SemiGrasse, 22.0),
            ),
            (TextStyle::Body, FontId::new(16.0, FontFamily::Proportional)),
            (
                TextStyle::Button,
                FontId::new(16.0, FontFamily::Proportional),
            ),
            (
                TextStyle::Small,
                FontId::new(14.0, FontFamily::Proportional),
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
            ("secondaire/carte", TEXTE_SECONDAIRE, SURFACE),
            ("secondaire/fond", TEXTE_SECONDAIRE, FOND),
            ("discret/fond", TEXTE_DISCRET, FOND),
            ("discret/carte", TEXTE_DISCRET, SURFACE),
            ("discret/sélection", TEXTE_DISCRET, ACCENT_DOUX),
            ("en-tête", ENTETE_TEXTE, ENTETE_FOND),
            ("en-tête discret", ENTETE_DISCRET, ENTETE_FOND),
            ("bouton", BOUTON_TEXTE, BOUTON_FOND),
            ("bouton survolé", BOUTON_TEXTE, BOUTON_SURVOL_FOND),
            ("bouton pressé", BOUTON_TEXTE, BOUTON_ACTIF_FOND),
            ("accent/carte", ACCENT, SURFACE),
            ("accent/sélection", ACCENT, ACCENT_DOUX),
            ("bouton principal", ACCENT_TEXTE, ACCENT),
            ("bouton principal survolé", ACCENT_TEXTE, ACCENT_SURVOL),
            ("bouton de suppression", ACCENT_TEXTE, DANGER),
            ("bouton validé", ACCENT_TEXTE, SUCCES_BOUTON),
            ("alerte", ALERTE_TEXTE, ALERTE_FOND),
            ("alerte sur carte", ALERTE_TEXTE, SURFACE),
            ("correct", SUCCES_TEXTE, SUCCES_FOND),
            ("succès sur carte", SUCCES_TEXTE, SURFACE),
            ("avertissement", AVERTISSEMENT_TEXTE, SURFACE),
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
    fn les_valeurs_de_la_maquette_sont_reprises() {
        assert_eq!(FOND, [242, 243, 245]);
        assert_eq!(SURFACE, [255, 255, 255]);
        assert_eq!(ACCENT, [30, 86, 200]);
        assert_eq!(ENTETE_FOND, [22, 25, 29]);
        assert_eq!(SUCCES_BOUTON, [18, 120, 74]);
        assert_eq!(ALERTE_BORD, [232, 180, 174]);
        assert_eq!(GRISE_FOND, [227, 230, 234]);
        assert_eq!(VOILE.a(), 128);
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
            crate::police::famille(crate::police::Graisse::SemiGrasse),
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
