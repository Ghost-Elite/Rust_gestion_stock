//! L'en-tête sombre (magasin, caisse, heure, retour à la vente) et la fenêtre Menu.

use super::composants::{Style, bouton_large, discret, modale};
use super::{Application, Ecran};
use crate::etat::chemin_complet;
use crate::palette::{ENTETE_DISCRET, ENTETE_TEXTE, SURFACE, TEXTE, couleur};
use crate::police::{Graisse, police_disponible};
use crate::textes::*;
use eframe::egui::{
    Align, Button, Context, CornerRadius, Layout, RichText, Stroke, StrokeKind, Ui, Vec2,
};
use std::time::Duration;

/// Au bout de combien de temps l'heure affichée est rafraîchie.
const RAFRAICHIR_HEURE: Duration = Duration::from_secs(10);

/// Le bouton blanc de l'en-tête (retour à la vente). Son anneau de
/// focus est blanc : l'accent bleu ne ferait pas 3:1 sur le fond sombre.
fn bouton_entete(ui: &mut Ui, texte: &str) -> bool {
    let etiquette =
        RichText::new(texte).font(police_disponible(ui.ctx(), Graisse::SemiGrasse, 15.0));
    let reponse = ui.add(
        Button::new(etiquette.color(couleur(TEXTE)))
            .fill(couleur(SURFACE))
            .stroke(Stroke::NONE)
            .min_size(Vec2::new(0.0, 40.0))
            .corner_radius(CornerRadius::same(8)),
    );
    if reponse.has_focus() {
        ui.painter().rect_stroke(
            reponse.rect.expand(2.0),
            CornerRadius::same(10),
            Stroke::new(2.0, couleur(ENTETE_TEXTE)),
            StrokeKind::Outside,
        );
    }
    reponse.clicked()
}

/// Le contenu de l'en-tête : le magasin et la caisse à gauche ; à droite l'heure et, selon
/// l'écran, « Retour à la vente » (la vente, elle, a son « Menu » dans la barre de fonctions).
pub(super) fn entete(ui: &mut Ui, app: &mut Application) {
    let heure = app
        .etat
        .as_ref()
        .map(|etat| etat.heure_locale())
        .unwrap_or_default();
    ui.ctx().request_repaint_after(RAFRAICHIR_HEURE);
    ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
        ui.spacing_mut().item_spacing.x = 20.0;
        ui.label(
            RichText::new(NOM_MAGASIN)
                .font(police_disponible(ui.ctx(), Graisse::Grasse, 18.0))
                .color(couleur(ENTETE_TEXTE)),
        );
        ui.label(
            RichText::new(libelle_caisse())
                .size(14.0)
                .color(couleur(ENTETE_DISCRET)),
        );
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if app.ecran != Ecran::Vente && bouton_entete(ui, BOUTON_RETOUR_VENTE) {
                app.ecran = Ecran::Vente;
            }
            if app.ecran != Ecran::Historique {
                ui.label(
                    RichText::new(heure)
                        .font(police_disponible(ui.ctx(), Graisse::SemiGrasse, 18.0))
                        .color(couleur(ENTETE_TEXTE)),
                );
            }
        });
    });
}

/// La fenêtre Menu : les écrans qui ne sont pas la vente, et le fichier de données utilisé.
pub(super) fn fenetre_menu(ctx: &Context, app: &mut Application) {
    let ferme = modale(ctx, "menu", 460.0, TITRE_MENU, |ui| {
        ui.heading(TITRE_MENU);
        for (ecran, texte) in [
            (Ecran::Historique, ECRAN_HISTORIQUE),
            (Ecran::Stock, ECRAN_STOCK),
            (Ecran::Reglages, ECRAN_REGLAGES),
        ] {
            let largeur = ui.available_width();
            if bouton_large(ui, texte, Style::Secondaire, true, Vec2::new(largeur, 56.0)).clicked()
            {
                app.ecran = ecran;
                app.modale = None;
            }
        }
        if let Some(etat) = &app.etat {
            discret(ui, &pied_de_page_donnees(&chemin_complet(etat.chemin())));
        }
        let largeur = ui.available_width();
        if bouton_large(
            ui,
            BOUTON_FERMER,
            Style::Secondaire,
            true,
            Vec2::new(largeur, 48.0),
        )
        .clicked()
        {
            app.modale = None;
        }
    });
    if ferme {
        app.modale = None;
    }
}
