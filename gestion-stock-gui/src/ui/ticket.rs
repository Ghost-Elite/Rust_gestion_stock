//! L'affichage d'un ticket (la vente qui vient d'être faite) : lignes avec leur repère, taxes par
//! taux, total, moyen de paiement, et la mention « Ticket non fiscal ». Les prix et les taux sont
//! ceux du moment de la vente.

use super::composants::{discret, repere};
use crate::etat::DetailTicket;
use crate::palette::{TEXTE, TEXTE_SECONDAIRE, couleur};
use crate::police::{Graisse, police_disponible};
use crate::textes::*;
use eframe::egui::{Align, Label, Layout, RichText, Ui};

fn texte_gras(ui: &Ui, texte: String, taille: f32) -> Label {
    Label::new(
        RichText::new(texte)
            .font(police_disponible(ui.ctx(), Graisse::SemiGrasse, taille))
            .color(couleur(TEXTE)),
    )
    .selectable(false)
}

pub(super) fn corps_du_ticket(ui: &mut Ui, detail: &DetailTicket, avec_mention: bool) {
    ui.spacing_mut().item_spacing.y = 6.0;
    ui.add(texte_gras(ui, titre_du_ticket(&detail.ticket), 18.0));
    discret(ui, &detail.quand);
    ui.separator();
    for ligne in &detail.lignes {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 12.0;
            repere(ui, &ligne.nom, 40.0);
            ui.label(
                RichText::new(format!("{} × {}", ligne.quantite, ligne.nom))
                    .size(16.0)
                    .color(couleur(TEXTE)),
            );
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.add(texte_gras(ui, ligne.total.clone(), 16.0));
            });
        });
    }
    ui.separator();
    if let Some(sous_total) = &detail.sous_total {
        ui.label(format!("{LIBELLE_SOUS_TOTAL} : {sous_total}"));
    }
    for taxe in &detail.taxes {
        ui.label(
            RichText::new(taxe)
                .size(14.0)
                .color(couleur(TEXTE_SECONDAIRE)),
        );
    }
    ui.add(texte_gras(
        ui,
        format!("{} : {}", detail.libelle_total, detail.total),
        18.0,
    ));
    ui.label(moyen_du_ticket(nom_du_moyen(detail.moyen)));
    if let (Some(recu), Some(rendu)) = (&detail.recu, &detail.rendu) {
        ui.label(recu_du_ticket(recu));
        ui.add(texte_gras(ui, rendu_de_monnaie(rendu), 17.0));
    }
    if avec_mention {
        discret(ui, MENTION_NON_CERTIFIE);
    }
}
