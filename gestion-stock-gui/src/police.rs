//! La police de l'interface : IBM Plex Sans, avec quatre graisses (normale, moyenne, semi-grasse,
//! grasse) pour créer une vraie hiérarchie entre titres, montants et texte courant.
//!
//! IBM Plex Sans est sous licence SIL Open Font License 1.1 (copyright 2017 IBM Corp., avec le
//! nom réservé « Plex ») ; la licence est dans `assets/polices/LICENCE-IBM-Plex-OFL-1.1.txt`. Les
//! fichiers viennent de la version 1.1.0 officielle
//! (https://github.com/IBM/plex/releases/tag/%40ibm%2Fplex-sans%401.1.0, archive
//! `ibm-plex-sans.zip`, SHA-256 fb365d910566e6d199cc2c15579a7dd9a267128e18431a394ed81f1970c69200),
//! dossier `fonts/complete/ttf`. SHA-256 des fichiers :
//!
//! - `IBMPlexSans-Regular.ttf` : 975dcda37d80f038dcd143c22e33ca2d97a0cc5a929aace1c749153b0fe1afa5
//! - `IBMPlexSans-Medium.ttf` : 331c8639d7598b2cde62a911a71db195e30cb655cd6bdf2e324a7e984955f907
//! - `IBMPlexSans-SemiBold.ttf` : a20caf8286023a6a7a85e40b1d2a4ae9fc3e3b1f9eda8f4c542dd4986af67bb1
//! - `IBMPlexSans-Bold.ttf` : 9e6c74a889a700d707613d24548fe4ffa6bc59559a0689d2cf9e133bdcdafb2f

use eframe::egui::{Context, FontData, FontDefinitions, FontFamily, FontId};
use std::sync::Arc;

pub const PLEX_REGULAR: &[u8] = include_bytes!("../assets/polices/IBMPlexSans-Regular.ttf");
pub const PLEX_MEDIUM: &[u8] = include_bytes!("../assets/polices/IBMPlexSans-Medium.ttf");
pub const PLEX_SEMI_BOLD: &[u8] = include_bytes!("../assets/polices/IBMPlexSans-SemiBold.ttf");
pub const PLEX_BOLD: &[u8] = include_bytes!("../assets/polices/IBMPlexSans-Bold.ttf");

/// Le nom de la police normale, aussi celui de la famille par défaut d'egui une fois installée.
const PLEX: &str = "IBM Plex Sans";

/// Les graisses de l'interface : 400, 500, 600 et 700.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Graisse {
    Normale,
    Moyenne,
    SemiGrasse,
    Grasse,
}

/// Le nom de la famille d'une graisse (la normale est la famille `Proportional`).
fn nom_de_famille(graisse: Graisse) -> Option<&'static str> {
    match graisse {
        Graisse::Normale => None,
        Graisse::Moyenne => Some("Plex-Medium"),
        Graisse::SemiGrasse => Some("Plex-SemiBold"),
        Graisse::Grasse => Some("Plex-Bold"),
    }
}

/// La famille d'une graisse.
pub fn famille(graisse: Graisse) -> FontFamily {
    match nom_de_famille(graisse) {
        None => FontFamily::Proportional,
        Some(nom) => FontFamily::Name(nom.into()),
    }
}

/// La famille de la graisse demandée si elle est installée, sinon la police normale : demander à
/// egui une famille qu'il ne connaît pas le fait planter, et ce ne doit jamais arriver (par
/// exemple pendant la toute première image, avant que les polices soient chargées).
pub fn famille_disponible(ctx: &Context, graisse: Graisse) -> FontFamily {
    let voulue = famille(graisse);
    if ctx.fonts(|f| f.families().contains(&voulue)) {
        voulue
    } else {
        FontFamily::Proportional
    }
}

/// Une police de la graisse et de la taille demandées.
pub fn police(graisse: Graisse, taille: f32) -> FontId {
    FontId::new(taille, famille(graisse))
}

/// Plex devient la police par défaut ; les polices d'egui restent derrière elle, pour les
/// caractères qu'elle n'a pas (emoji, symboles).
pub fn definitions() -> FontDefinitions {
    let mut polices = FontDefinitions::default();
    for (cle, octets) in [
        (PLEX, PLEX_REGULAR),
        ("Plex-Medium", PLEX_MEDIUM),
        ("Plex-SemiBold", PLEX_SEMI_BOLD),
        ("Plex-Bold", PLEX_BOLD),
    ] {
        polices
            .font_data
            .insert(cle.to_owned(), Arc::new(FontData::from_static(octets)));
    }
    let secours = polices
        .families
        .entry(FontFamily::Proportional)
        .or_default();
    secours.insert(0, PLEX.to_owned());
    let secours = secours.clone();
    for graisse in [Graisse::Moyenne, Graisse::SemiGrasse, Graisse::Grasse] {
        if let Some(nom) = nom_de_famille(graisse) {
            let mut chaine = vec![nom.to_owned()];
            chaine.extend(secours.iter().cloned());
            polices.families.insert(famille(graisse), chaine);
        }
    }
    polices
}

#[cfg(test)]
mod tests {
    use super::*;
    use eframe::egui::{Context, FontFamily, Pos2, RawInput, Rect, Vec2};

    #[test]
    fn plex_devient_la_police_par_defaut_et_garde_les_polices_d_egui_en_secours() {
        let d = definitions();
        let proportionnelle = &d.families[&FontFamily::Proportional];
        assert_eq!(proportionnelle[0], "IBM Plex Sans");
        // Les polices d'egui (emoji, symboles) restent derrière, pour les caractères absents.
        assert!(
            proportionnelle.len() > 1,
            "polices de secours : {proportionnelle:?}"
        );
        assert!(d.font_data.contains_key("IBM Plex Sans"));
    }

    #[test]
    fn chaque_graisse_a_sa_famille_qui_retombe_sur_plex_regular() {
        let d = definitions();
        for graisse in [Graisse::Moyenne, Graisse::SemiGrasse, Graisse::Grasse] {
            let famille = &d.families[&famille(graisse)];
            assert!(famille.len() >= 2, "{graisse:?} : {famille:?}");
            assert!(famille.contains(&"IBM Plex Sans".to_string()));
            assert!(d.font_data.contains_key(&famille[0]));
        }
        // La graisse normale est la famille par défaut.
        assert_eq!(super::famille(Graisse::Normale), FontFamily::Proportional);
    }

    #[test]
    fn les_quatre_polices_sont_de_vraies_polices_truetype() {
        // Un fichier TrueType commence par 0x00010000 (ou « true » / « OTTO »).
        for octets in [PLEX_REGULAR, PLEX_MEDIUM, PLEX_SEMI_BOLD, PLEX_BOLD] {
            assert_eq!(&octets[..4], &[0x00, 0x01, 0x00, 0x00]);
            assert!(octets.len() > 100_000);
        }
    }

    #[test]
    fn les_chiffres_ont_tous_la_meme_largeur() {
        // La maquette demande des chiffres de largeur fixe, pour aligner les montants.
        let ctx = Context::default();
        ctx.set_fonts(definitions());
        let entree = || RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 300.0))),
            ..Default::default()
        };
        let _ = ctx.run_ui(entree(), |_| {}); // les polices sont prises en compte à l'image suivante
        // Garde-fou : sans les polices de Plex installées, le test ne prouverait rien.
        assert!(ctx.fonts(|f| f.families().contains(&famille(Graisse::Grasse))));
        for graisse in [Graisse::Normale, Graisse::SemiGrasse, Graisse::Grasse] {
            let mut largeurs = Vec::new();
            let _ = ctx.run_ui(entree(), |ui| {
                for chiffre in '0'..='9' {
                    let galerie = ui.painter().layout_no_wrap(
                        chiffre.to_string(),
                        police(graisse, 20.0),
                        eframe::egui::Color32::BLACK,
                    );
                    largeurs.push(galerie.size().x);
                }
            });
            assert_eq!(largeurs.len(), 10);
            assert!(
                largeurs.iter().all(|l| (l - largeurs[0]).abs() < 0.01),
                "{graisse:?} : largeurs des chiffres {largeurs:?}"
            );
        }
    }
}
