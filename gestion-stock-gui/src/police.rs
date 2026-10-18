//! La police de l'interface : Inter, une police pensée pour les écrans, avec deux graisses
//! (normale et semi-grasse) pour créer une vraie hiérarchie entre titres et texte.
//!
//! Inter est sous licence SIL Open Font License 1.1 (copyright The Inter Project Authors) ;
//! la licence est dans `assets/polices/LICENCE-Inter-OFL-1.1.txt`. Les fichiers viennent de
//! la version 4.0 officielle (https://github.com/rsms/inter/releases/tag/v4.0, archive
//! `Inter-4.0.zip`, SHA-256 ff970a5d4561a04f102a7cb781adbd6ac4e9b6c460914c7a101f15acb7f7d1a4).

use eframe::egui::{FontData, FontDefinitions, FontFamily, FontId};
use std::sync::Arc;

pub const INTER_REGULAR: &[u8] = include_bytes!("../assets/polices/Inter-Regular.ttf");
pub const INTER_SEMI_BOLD: &[u8] = include_bytes!("../assets/polices/Inter-SemiBold.ttf");

/// Le nom de la famille semi-grasse (pour `RichText::family`).
pub const SEMI_GRAS: &str = "Inter-SemiBold";

/// La famille semi-grasse.
pub fn famille_semi_grasse() -> FontFamily {
    FontFamily::Name(SEMI_GRAS.into())
}

/// La famille semi-grasse si elle est installée, sinon la police normale : demander à egui une
/// famille qu'il ne connaît pas le fait planter, et ce ne doit jamais arriver (par exemple
/// pendant la toute première image, avant que les polices soient chargées).
pub fn famille_disponible(ctx: &eframe::egui::Context) -> FontFamily {
    let semi_grasse = famille_semi_grasse();
    if ctx.fonts(|f| f.families().contains(&semi_grasse)) {
        semi_grasse
    } else {
        FontFamily::Proportional
    }
}

pub fn semi_gras(taille: f32) -> FontId {
    FontId::new(taille, famille_semi_grasse())
}

/// Inter devient la police par défaut ; les polices d'egui restent derrière elle, pour les
/// caractères qu'Inter n'a pas (emoji, symboles).
pub fn definitions() -> FontDefinitions {
    let mut polices = FontDefinitions::default();
    polices.font_data.insert(
        "Inter".to_owned(),
        Arc::new(FontData::from_static(INTER_REGULAR)),
    );
    polices.font_data.insert(
        SEMI_GRAS.to_owned(),
        Arc::new(FontData::from_static(INTER_SEMI_BOLD)),
    );
    let secours = polices
        .families
        .entry(FontFamily::Proportional)
        .or_default();
    secours.insert(0, "Inter".to_owned());
    let mut semi_gras = vec![SEMI_GRAS.to_owned()];
    semi_gras.extend(secours.iter().cloned());
    polices.families.insert(famille_semi_grasse(), semi_gras);
    polices
}

#[cfg(test)]
mod tests {
    use super::*;
    use eframe::egui::FontFamily;

    #[test]
    fn inter_devient_la_police_par_defaut_et_garde_les_polices_d_egui_en_secours() {
        let d = definitions();
        let proportionnelle = &d.families[&FontFamily::Proportional];
        assert_eq!(proportionnelle[0], "Inter");
        // Les polices d'egui (emoji, symboles) restent derrière, pour les caractères absents.
        assert!(
            proportionnelle.len() > 1,
            "polices de secours : {proportionnelle:?}"
        );
        assert!(d.font_data.contains_key("Inter"));
    }

    #[test]
    fn la_famille_semi_grasse_existe_et_retombe_sur_inter() {
        let d = definitions();
        let famille = &d.families[&FontFamily::Name(SEMI_GRAS.into())];
        assert_eq!(famille[0], "Inter-SemiBold");
        assert!(famille.contains(&"Inter".to_string()));
        assert!(d.font_data.contains_key("Inter-SemiBold"));
    }

    #[test]
    fn les_polices_integrees_sont_de_vraies_polices_truetype() {
        // Un fichier TrueType commence par 0x00010000 (ou « true » / « OTTO »).
        for octets in [INTER_REGULAR, INTER_SEMI_BOLD] {
            assert_eq!(&octets[..4], &[0x00, 0x01, 0x00, 0x00]);
            assert!(octets.len() > 100_000);
        }
    }
}
