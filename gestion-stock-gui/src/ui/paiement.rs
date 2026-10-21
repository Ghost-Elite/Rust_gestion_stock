//! Les fenêtres de paiement : espèces, carte ou autre moyen, puis « Vente terminée ».
//!
//! Aucun terminal de paiement n'est relié au programme : pour la carte et les autres moyens, c'est
//! la personne à la caisse qui confirme ce que le terminal affiche (accepté ou refusé). Le montant
//! reçu, lui, se tape ici, et le rendu de monnaie se calcule en direct. Dimensions et couleurs :
//! celles de `maquette/2-especes.html` et `3-carte.html`.

use super::composants::{
    LigneBouton, Style, bandeau_alerte, bouton_a_lignes, bouton_large, bouton_pointille, discret,
    libelle_et_valeur, modale, modale_centree, zone,
};
use super::ticket::corps_du_ticket;
use super::{Application, Modale};
use crate::etat::{DetailTicket, Reglement, Retour};
use crate::palette::{
    ACCENT, ACCENT_DOUX, ACCENT_SURVOL, ACCENT_TEXTE, ALERTE_FOND, ALERTE_TEXTE, BORDURE_BOUTON,
    FOND_DOUX, GRISE_FOND, GRISE_TEXTE, SUCCES_FOND, SUCCES_TEXTE, TEXTE, TEXTE_SECONDAIRE,
    couleur,
};
use crate::police::{Graisse, police_disponible};
use crate::textes::*;
use eframe::egui::{
    Align, Context, CornerRadius, Frame, Key, Layout, Margin, RichText, ScrollArea, Sense, Stroke,
    TextEdit, Ui, Vec2,
    text::{CCursor, CCursorRange},
    text_edit::TextEditState,
};
use gestion_stock::monnaie::{Devise, Montant};
use gestion_stock::vente::Moyen;

/// Les pas d'arrondi des montants rapides, dans la plus petite unité de la devise : en FCFA
/// (sans centimes) 1 000 à 50 000 ; ailleurs, en centimes, 5 à 100.
fn pas_d_arrondi(devise: Devise) -> &'static [i64] {
    match devise {
        Devise::Xof => &[1_000, 5_000, 10_000, 20_000, 50_000],
        _ => &[500, 1_000, 2_000, 5_000, 10_000],
    }
}

/// Les montants que le client donne le plus souvent : le compte juste, puis jusqu'à trois
/// arrondis supérieurs distincts (le premier est toujours le total).
pub(crate) fn montants_rapides(devise: Devise, total: Montant) -> Vec<Montant> {
    let mut montants = vec![total];
    for pas in pas_d_arrondi(devise) {
        let Some(haut) = total.0.checked_add(pas - 1) else {
            continue;
        };
        let Some(arrondi) = (haut / pas).checked_mul(*pas) else {
            continue;
        };
        let arrondi = Montant(arrondi);
        if arrondi > total && !montants.contains(&arrondi) && montants.len() < 4 {
            montants.push(arrondi);
        }
    }
    montants
}

/// Regroupe par milliers la partie entière d'un montant tapé : « 20000 » devient « 20 000 » (avec
/// l'espace fine insécable de la maquette). Ce qui n'est pas un nombre reste tel quel, pour que
/// « Montant illisible » le dise.
pub(crate) fn regrouper(saisie: &str) -> String {
    let net: String = saisie
        .chars()
        .filter(|c| !matches!(c, ' ' | '\u{a0}' | '\u{202f}'))
        .collect();
    let (entier, reste) = match net.find([',', '.']) {
        Some(i) => net.split_at(i),
        None => (net.as_str(), ""),
    };
    let nombre = !entier.is_empty()
        && entier.chars().all(|c| c.is_ascii_digit())
        && reste.chars().skip(1).all(|c| c.is_ascii_digit());
    if !nombre {
        return saisie.to_string();
    }
    let mut groupe = String::new();
    for (i, c) in entier.chars().enumerate() {
        if i > 0 && (entier.len() - i) % 3 == 0 {
            groupe.push('\u{202f}');
        }
        groupe.push(c);
    }
    groupe + reste
}

fn erreur_en_cours(app: &Application) -> Option<String> {
    match app.etat.as_ref()?.retour.as_ref()? {
        Retour::Erreur(message) => Some(message.clone()),
        Retour::Succes(_) => None,
    }
}

fn total_a_payer(app: &Application) -> Montant {
    app.etat
        .as_ref()
        .and_then(|etat| etat.devis(&app.panier).ok())
        .map_or(Montant::ZERO, |devis| devis.total)
}

/// Encaisse le ticket, une seule fois. Si ça réussit, le ticket se vide et la fenêtre « Vente
/// terminée » s'ouvre ; sinon la fenêtre reste ouverte et l'erreur s'y affiche.
fn payer(app: &mut Application, moyen: Moyen) -> bool {
    let recu = app.saisie_recu.clone();
    let Some(etat) = app.etat.as_mut() else {
        return false;
    };
    if !etat.encaisser(&app.panier, moyen, &recu) {
        return false;
    }
    // Le message de succès est remplacé par la fenêtre « Vente terminée » : pas de doublon.
    etat.retour = None;
    app.panier.clear();
    app.saisie_recu.clear();
    app.ligne_choisie = None;
    app.modale = Some(Modale::VenteTerminee {
        ticket_visible: false,
    });
    true
}

fn fermer(app: &mut Application) {
    app.modale = None;
    app.focus_scan = true;
}

// ---- Espèces ------------------------------------------------------------------------------------

/// Une ligne « libellé à gauche, montant à droite » dans un bandeau coloré : 17 points et 32.
fn ligne_du_bandeau(ui: &mut Ui, libelle: &str, valeur: &str, couleur_texte: [u8; 3]) {
    libelle_et_valeur(
        ui,
        RichText::new(libelle)
            .font(police_disponible(ui.ctx(), Graisse::SemiGrasse, 17.0))
            .color(couleur(couleur_texte)),
        17.0,
        RichText::new(valeur)
            .font(police_disponible(ui.ctx(), Graisse::Grasse, 32.0))
            .color(couleur(couleur_texte)),
        32.0,
    );
}

pub(super) fn fenetre_especes(ctx: &Context, app: &mut Application) {
    let Some(etat) = &app.etat else {
        return;
    };
    let devise = etat.reglages().devise;
    let total = total_a_payer(app);
    let reglement = etat.reglement_especes(total, &app.saisie_recu);
    let valide = matches!(reglement, Reglement::Rendu(_))
        && etat.integrite().is_ok()
        && !app.panier.is_empty();
    let rapides = montants_rapides(devise, total);
    let erreur = erreur_en_cours(app);
    let focus = std::mem::take(&mut app.focus_recu);
    let (mut choix, mut retour, mut valider) = (None, false, false);
    let ferme = modale(ctx, "especes", 520.0, ESPECES_TITRE, |ui| {
        ui.spacing_mut().item_spacing.y = 18.0;
        ui.heading(ESPECES_TITRE);
        if let Some(message) = &erreur {
            bandeau_alerte(ui, &format!("Erreur : {message}"));
        }
        libelle_et_valeur(
            ui,
            RichText::new(A_PAYER)
                .size(17.0)
                .color(couleur(TEXTE_SECONDAIRE)),
            17.0,
            RichText::new(devise.formater(total))
                .font(police_disponible(ui.ctx(), Graisse::Grasse, 30.0))
                .color(couleur(TEXTE)),
            30.0,
        );
        // Le libellé et son champ se tiennent à 6 l'un de l'autre ; les blocs, eux, à 18.
        ui.scope(|ui| {
            ui.spacing_mut().item_spacing.y = 6.0;
            let etiquette = ui.label(
                RichText::new(CHAMP_RECU)
                    .size(15.0)
                    .color(couleur(TEXTE_SECONDAIRE)),
            );
            Frame::new()
                .stroke(Stroke::new(2.0, couleur(ACCENT)))
                .corner_radius(CornerRadius::same(10))
                .inner_margin(Margin::symmetric(16, 0))
                .show(ui, |ui| {
                    zone(
                        ui,
                        Vec2::new(ui.available_width(), 60.0),
                        Layout::left_to_right(Align::Center),
                        |ui| {
                            let champ = TextEdit::singleline(&mut app.saisie_recu)
                                .font(police_disponible(ui.ctx(), Graisse::SemiGrasse, 32.0))
                                .horizontal_align(Align::RIGHT)
                                .frame(Frame::NONE)
                                .desired_width(f32::INFINITY);
                            let reponse = ui.add(champ).labelled_by(etiquette.id);
                            if reponse.changed() {
                                let groupee = regrouper(&app.saisie_recu);
                                if groupee != app.saisie_recu {
                                    // Le curseur reste à la fin : on tape toujours à la suite.
                                    let fin = CCursor::new(groupee.chars().count());
                                    app.saisie_recu = groupee;
                                    if let Some(mut etat) =
                                        TextEditState::load(ui.ctx(), reponse.id)
                                    {
                                        etat.cursor.set_char_range(Some(CCursorRange::one(fin)));
                                        etat.store(ui.ctx(), reponse.id);
                                    }
                                }
                            }
                            if focus {
                                reponse.request_focus();
                            }
                            if reponse.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter)) {
                                valider = true;
                            }
                        },
                    );
                });
        });
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 8.0;
            let largeur = (ui.available_width() - 3.0 * 8.0) / 4.0;
            for (i, montant) in rapides.iter().enumerate() {
                let texte = if i == 0 {
                    QUICK_EXACT.to_string()
                } else {
                    devise.formater_nombre(*montant)
                };
                let reponse = bouton_a_lignes(
                    ui,
                    &texte,
                    Vec2::new(largeur, 52.0),
                    FOND_DOUX,
                    Stroke::new(1.0, couleur(BORDURE_BOUTON)),
                    10,
                    &[LigneBouton {
                        texte: &texte,
                        taille: 16.0,
                        graisse: Graisse::SemiGrasse,
                        couleur: TEXTE,
                    }],
                    true,
                );
                if reponse.clicked() {
                    choix = Some(*montant);
                }
            }
        });
        let (fond, texte) = match reglement {
            Reglement::Manque(_) | Reglement::Illisible => (ALERTE_FOND, ALERTE_TEXTE),
            Reglement::Attente | Reglement::Rendu(_) => (SUCCES_FOND, SUCCES_TEXTE),
        };
        Frame::new()
            .fill(couleur(fond))
            .corner_radius(CornerRadius::same(10))
            .inner_margin(Margin::same(16))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                match reglement {
                    Reglement::Illisible => {
                        ui.label(
                            RichText::new(RECU_ILLISIBLE)
                                .font(police_disponible(ui.ctx(), Graisse::SemiGrasse, 17.0))
                                .color(couleur(texte)),
                        );
                    }
                    Reglement::Manque(manque) => {
                        ligne_du_bandeau(ui, IL_MANQUE, &devise.formater(manque), texte);
                    }
                    Reglement::Rendu(rendu) => {
                        ligne_du_bandeau(ui, MONNAIE_A_RENDRE, &devise.formater(rendu), texte);
                    }
                    Reglement::Attente => ligne_du_bandeau(
                        ui,
                        MONNAIE_A_RENDRE,
                        &devise.formater(Montant::ZERO),
                        texte,
                    ),
                }
            });
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 10.0;
            let largeur = ui.available_width() - 10.0;
            retour = bouton_large(
                ui,
                BOUTON_RETOUR,
                Style::Secondaire,
                true,
                Vec2::new(largeur / 3.0, 60.0),
            )
            .clicked();
            let (fond, texte) = if valide {
                (ACCENT, ACCENT_TEXTE)
            } else {
                (GRISE_FOND, GRISE_TEXTE)
            };
            let reponse = bouton_a_lignes(
                ui,
                BOUTON_VALIDER,
                Vec2::new(largeur * 2.0 / 3.0, 60.0),
                fond,
                Stroke::NONE,
                10,
                &[LigneBouton {
                    texte: BOUTON_VALIDER,
                    taille: 18.0,
                    graisse: Graisse::Grasse,
                    couleur: texte,
                }],
                valide,
            );
            valider |= reponse.clicked();
        });
    });
    if let Some(montant) = choix {
        app.saisie_recu = regrouper(&devise.formater_brut(montant));
    }
    if valider && valide {
        payer(app, Moyen::Especes);
    } else if retour || ferme {
        fermer(app);
    }
}

// ---- Carte et autre moyen -----------------------------------------------------------------------

enum ActionTerminal {
    Accepte,
    Refuse,
    Reessayer,
    EnEspeces,
    Annuler,
}

pub(super) fn fenetre_terminal(ctx: &Context, app: &mut Application, moyen: Moyen, refuse: bool) {
    let Some(etat) = &app.etat else {
        return;
    };
    let total = etat.formater(total_a_payer(app));
    let erreur = erreur_en_cours(app);
    let titre = if moyen == Moyen::Carte {
        CARTE_TITRE
    } else {
        AUTRE_TITRE
    };
    let (message, aide, fond, texte) = match (moyen, refuse) {
        (_, true) => (CARTE_REFUSEE, CARTE_REFUSEE_AIDE, ALERTE_FOND, ALERTE_TEXTE),
        (Moyen::Carte, false) => (CARTE_ATTENTE, CARTE_AIDE, ACCENT_DOUX, ACCENT_SURVOL),
        (_, false) => (AUTRE_ATTENTE, AUTRE_AIDE, ACCENT_DOUX, ACCENT_SURVOL),
    };
    let mut action = None;
    let ferme = modale_centree(ctx, "terminal", 480.0, titre, |ui| {
        ui.spacing_mut().item_spacing.y = 16.0;
        ui.vertical_centered(|ui| {
            ui.heading(titre);
            if let Some(message) = &erreur {
                bandeau_alerte(ui, &format!("Erreur : {message}"));
            }
            ui.label(
                RichText::new(total)
                    .font(police_disponible(ui.ctx(), Graisse::Grasse, 44.0))
                    .color(couleur(TEXTE)),
            );
            Frame::new()
                .fill(couleur(fond))
                .corner_radius(CornerRadius::same(10))
                .inner_margin(Margin::same(16))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.vertical_centered(|ui| {
                        ui.label(
                            RichText::new(message)
                                .font(police_disponible(ui.ctx(), Graisse::SemiGrasse, 17.0))
                                .color(couleur(texte)),
                        );
                    });
                });
            ui.label(
                RichText::new(aide)
                    .size(15.0)
                    .color(couleur(TEXTE_SECONDAIRE)),
            );
            let largeur = ui.available_width();
            let grand = Vec2::new(largeur, 56.0);
            if refuse {
                ui.columns(2, |colonnes| {
                    let moitie = Vec2::new(colonnes[0].available_width(), 56.0);
                    if bouton_large(
                        &mut colonnes[0],
                        BOUTON_REESSAYER,
                        Style::Secondaire,
                        true,
                        moitie,
                    )
                    .clicked()
                    {
                        action = Some(ActionTerminal::Reessayer);
                    }
                    if bouton_large(
                        &mut colonnes[1],
                        BOUTON_PAYER_ESPECES,
                        Style::Principal,
                        true,
                        moitie,
                    )
                    .clicked()
                    {
                        action = Some(ActionTerminal::EnEspeces);
                    }
                });
            } else if moyen == Moyen::Carte {
                // Pas de terminal relié : le paiement se simule (boutons en pointillés).
                ui.columns(2, |colonnes| {
                    let moitie = Vec2::new(colonnes[0].available_width(), 48.0);
                    if bouton_pointille(&mut colonnes[0], BOUTON_ACCEPTE, moitie, 14.0).clicked() {
                        action = Some(ActionTerminal::Accepte);
                    }
                    if bouton_pointille(&mut colonnes[1], BOUTON_REFUSE, moitie, 14.0).clicked() {
                        action = Some(ActionTerminal::Refuse);
                    }
                });
            } else if bouton_pointille(ui, BOUTON_RECU, Vec2::new(largeur, 56.0), 15.0).clicked() {
                action = Some(ActionTerminal::Accepte);
            }
            if !refuse && bouton_large(ui, BOUTON_ANNULER, Style::Secondaire, true, grand).clicked()
            {
                action = Some(ActionTerminal::Annuler);
            }
        });
    });
    match action {
        Some(ActionTerminal::Accepte) => {
            payer(app, moyen);
        }
        Some(ActionTerminal::Refuse) => {
            app.modale = Some(Modale::Terminal {
                moyen,
                refuse: true,
            });
        }
        Some(ActionTerminal::Reessayer) => {
            app.modale = Some(Modale::Terminal {
                moyen,
                refuse: false,
            });
        }
        Some(ActionTerminal::EnEspeces) => {
            app.saisie_recu.clear();
            app.focus_recu = true;
            app.modale = Some(Modale::Especes);
        }
        Some(ActionTerminal::Annuler) => fermer(app),
        None if ferme => fermer(app),
        None => {}
    }
}

// ---- Vente terminée -----------------------------------------------------------------------------

/// La coche verte de la fenêtre « Vente terminée », dessinée (64 × 64).
fn coche(ui: &mut Ui) {
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(64.0), Sense::hover());
    let p = |x: f32, y: f32| rect.min + Vec2::new(x, y);
    let peintre = ui.painter();
    peintre.circle_filled(rect.center(), 32.0, couleur(SUCCES_FOND));
    let trait_ = Stroke::new(3.5, couleur(SUCCES_TEXTE));
    peintre.line_segment([p(19.0, 33.0), p(28.0, 42.0)], trait_);
    peintre.line_segment([p(28.0, 42.0), p(46.0, 23.0)], trait_);
}

pub(super) fn fenetre_vente_terminee(ctx: &Context, app: &mut Application, ticket_visible: bool) {
    let Some(etat) = &app.etat else {
        return;
    };
    let Some(numero) = etat.derniere_piece().map(|piece| piece.numero) else {
        app.modale = None;
        return;
    };
    let Some(detail) = etat.detail_ticket(numero) else {
        app.modale = None;
        return;
    };
    let (mut nouvelle, mut bascule) = (false, false);
    let ferme = modale_centree(ctx, "vente-terminee", 480.0, VENTE_TERMINEE, |ui| {
        ui.spacing_mut().item_spacing.y = 14.0;
        ui.vertical_centered(|ui| {
            coche(ui);
            ui.heading(VENTE_TERMINEE);
            ui.label(
                RichText::new(vente_terminee_detail(
                    nom_du_moyen(detail.moyen),
                    &detail.total,
                ))
                .size(16.0)
                .color(couleur(TEXTE_SECONDAIRE)),
            );
            if let Some(rendu) = &detail.rendu
                && detail.rendu != detail.recu
                && !est_nul(&detail)
            {
                ui.label(
                    RichText::new(MONNAIE_A_RENDRE)
                        .size(16.0)
                        .color(couleur(TEXTE_SECONDAIRE)),
                );
                ui.label(
                    RichText::new(rendu)
                        .font(police_disponible(ui.ctx(), Graisse::Grasse, 44.0))
                        .color(couleur(SUCCES_TEXTE)),
                );
            }
            let largeur = ui.available_width();
            let texte = if ticket_visible {
                BOUTON_CACHER_TICKET
            } else {
                BOUTON_VOIR_TICKET
            };
            bascule = bouton_large(ui, texte, Style::Secondaire, true, Vec2::new(largeur, 44.0))
                .clicked();
            if ticket_visible {
                ScrollArea::vertical().max_height(180.0).show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    corps_du_ticket(ui, &detail, false);
                });
                // Hors de la zone qui défile : la mention légale reste toujours visible.
                discret(ui, MENTION_NON_CERTIFIE);
            }
            nouvelle = bouton_large(
                ui,
                BOUTON_NOUVELLE_VENTE,
                Style::Principal,
                true,
                Vec2::new(largeur, 60.0),
            )
            .clicked();
        });
    });
    if bascule {
        app.modale = Some(Modale::VenteTerminee {
            ticket_visible: !ticket_visible,
        });
    }
    if nouvelle || ferme {
        if let Some(etat) = app.etat.as_mut() {
            etat.effacer_ticket();
        }
        fermer(app);
    }
}

/// Vrai si le rendu de monnaie est nul (le client a payé juste) : on ne l'affiche pas.
fn est_nul(detail: &DetailTicket) -> bool {
    detail
        .rendu
        .as_deref()
        .is_some_and(|rendu| rendu.chars().all(|c| !c.is_ascii_digit() || c == '0'))
}

#[cfg(test)]
mod tests {
    use super::super::essais::*;
    use super::super::*;
    use super::montants_rapides;
    use eframe::egui::Key;
    use eframe::egui::accesskit::Role;
    use egui_kittest::kittest::{NodeT, Queryable};
    use gestion_stock::monnaie::Montant;

    /// Un ticket de 4,90 € : Pain × 2 (2,40 €) et Savon × 1 (2,50 €).
    fn a_payer(nom: &str) -> Application {
        let mut app = app_vendeuse(nom);
        app.panier = vec![("Pain".to_string(), 2), ("Savon".to_string(), 1)];
        app
    }

    fn nbsp(texte: &str) -> String {
        texte.replace("EUR", "\u{a0}€")
    }

    #[test]
    fn la_saisie_du_montant_recu_regroupe_les_milliers() {
        use super::regrouper;
        assert_eq!(regrouper("20000"), "20\u{202f}000");
        assert_eq!(regrouper("1234,5"), "1\u{202f}234,5");
        assert_eq!(regrouper("1234.50"), "1\u{202f}234.50");
        assert_eq!(regrouper("1 234 567"), "1\u{202f}234\u{202f}567");
        assert_eq!(regrouper("20\u{202f}0000"), "200\u{202f}000");
        assert_eq!(regrouper("12"), "12");
        assert_eq!(regrouper(""), "");
        assert_eq!(regrouper(",5"), ",5");
        // Ce qui n'est pas un nombre reste tel quel : « Montant illisible » le dira.
        for illisible in ["abc", "1e5", "-5", "12,345,6", "1 2a"] {
            assert_eq!(regrouper(illisible), illisible);
        }
    }

    #[test]
    fn taper_un_montant_le_regroupe_et_le_rendu_se_calcule() {
        let mut h = harness_vente(a_payer("recu-groupe"), FENETRE);
        cliquer(&mut h, BOUTON_ESPECES);
        saisir(&mut h, CHAMP_RECU, "1000");
        assert_eq!(h.state().saisie_recu, "1\u{202f}000");
        h.get_by_label(MONNAIE_A_RENDRE);
        h.get_by_label(&nbsp("995,10EUR"));
        cliquer(&mut h, BOUTON_VALIDER);
        assert_eq!(nombre_de_pieces(&h), 1);
    }

    #[test]
    fn les_montants_rapides_sont_exact_puis_trois_arrondis_distincts() {
        assert_eq!(
            montants_rapides(Devise::Xof, Montant(17_025)),
            [17_025, 18_000, 20_000, 50_000].map(Montant)
        );
        assert_eq!(
            montants_rapides(Devise::Eur, Montant(490)),
            [490, 500, 1_000, 2_000].map(Montant)
        );
        // Un total déjà rond n'est pas répété, et la liste reste strictement croissante.
        let ronds = montants_rapides(Devise::Xof, Montant(1_000));
        assert_eq!(ronds[0], Montant(1_000));
        assert!(ronds.windows(2).all(|w| w[0] < w[1]), "{ronds:?}");
        assert!(ronds.len() <= 4);
    }

    /// La largeur des fenêtres de la maquette (520 pour les espèces, 480 pour la carte) COMPREND la
    /// marge de 28 : avant, la marge s'ajoutait et les fenêtres faisaient 56 de trop.
    #[test]
    fn les_fenetres_de_paiement_ont_la_largeur_de_la_maquette_marges_comprises() {
        let largeur = |modale: Modale, titre: &str| {
            let mut app = a_payer("largeur-fenetre");
            app.modale = Some(modale);
            let h = harness_fenetre(app, FENETRE);
            let rect = h.get_by_role_and_label(Role::Dialog, titre).rect();
            assert_eq!(rect.center().x, FENETRE.x / 2.0, "{titre} : centrée");
            rect.width()
        };
        assert_eq!(largeur(Modale::Especes, ESPECES_TITRE), 520.0);
        let carte = Modale::Terminal {
            moyen: Moyen::Carte,
            refuse: false,
        };
        assert_eq!(largeur(carte, CARTE_TITRE), 480.0);
    }

    /// Les écarts de `maquette/2-especes.html` : 18 entre les blocs, 6 entre « Montant reçu » et
    /// son champ, un bandeau de monnaie de 74 (16 + une ligne de 42 + 16). Mises bout à bout, les
    /// côtes donnent une fenêtre de 489 de haut (28 + 28,6 + 18 + 39 + 18 + 19,5 + 6 + 64 + 18 +
    /// 52 + 18 + 74 + 18 + 60 + 28), comme dans la maquette.
    #[test]
    fn la_fenetre_especes_suit_l_espacement_de_la_maquette() {
        let mut app = a_payer("espacement-especes");
        app.saisie_recu = "10".to_string();
        app.modale = Some(Modale::Especes);
        let h = harness_fenetre(app, FENETRE);
        let rect = |role, nom: &str| h.get_by_role_and_label(role, nom).rect();
        let exact = rect(Role::Button, QUICK_EXACT);
        let valider = rect(Role::Button, BOUTON_VALIDER);
        let ecart = valider.top() - exact.bottom();
        assert!(
            (ecart - (18.0 + 74.0 + 18.0)).abs() < 0.5,
            "le bandeau de monnaie (74) et ses deux écarts de 18 font {ecart}, pas 110"
        );
        let fenetre = rect(Role::Dialog, ESPECES_TITRE);
        assert!(
            (fenetre.height() - 489.1).abs() < 1.0,
            "la fenêtre fait {} de haut au lieu de 489",
            fenetre.height()
        );
    }

    /// `maquette/3-carte.html` : 32 de marge en haut, le total à payer en 44 points.
    #[test]
    fn la_fenetre_carte_a_32_de_marge_haute_et_son_total_en_44_points() {
        let mut app = a_payer("carte-cotes");
        app.modale = Some(Modale::Terminal {
            moyen: Moyen::Carte,
            refuse: false,
        });
        let h = harness_fenetre(app, FENETRE);
        let fenetre = h.get_by_role_and_label(Role::Dialog, CARTE_TITRE).rect();
        // Le titre porte le même nom que la fenêtre : c'est l'autre nœud, le plus petit.
        let titre = h
            .get_all_by_label(CARTE_TITRE)
            .map(|n| n.rect())
            .min_by(|a, b| a.height().total_cmp(&b.height()))
            .unwrap();
        assert_eq!(titre.top() - fenetre.top(), 32.0);
        // Le même montant est écrit derrière la fenêtre (sous-total, total en 64 points) :
        // on cherche celui qui est dans la fenêtre.
        let total = h
            .get_all_by_label(&nbsp("4,90EUR"))
            .map(|n| n.rect())
            .find(|r| fenetre.contains_rect(*r))
            .expect("le total à payer est dans la fenêtre");
        assert!(
            (total.height() - 44.0 * 1.3).abs() < 0.5,
            "total de {} de haut",
            total.height()
        );
    }

    /// `maquette/3-carte.html` : il n'y a pas de terminal relié, le paiement se SIMULE avec deux
    /// boutons en pointillés.
    #[test]
    fn la_carte_et_l_autre_moyen_se_simulent_comme_dans_la_maquette() {
        let mut app = a_payer("simule-carte");
        app.modale = Some(Modale::Terminal {
            moyen: Moyen::Carte,
            refuse: false,
        });
        let h = harness_vente(app, FENETRE);
        h.get_by_role_and_label(Role::Button, "Simuler : acceptée");
        h.get_by_role_and_label(Role::Button, "Simuler : refusée");
        h.get_by_label("Sans contact, puce ou piste. Le montant est déjà envoyé au terminal.");

        let mut app = a_payer("simule-autre");
        app.modale = Some(Modale::Terminal {
            moyen: Moyen::Autre,
            refuse: false,
        });
        let h = harness_vente(app, FENETRE);
        h.get_by_role_and_label(Role::Button, "Simuler : paiement accepté");
        assert!(
            h.query_by_role_and_label(Role::Button, "Simuler : refusée")
                .is_none()
        );
    }

    #[test]
    fn payer_en_especes_montre_le_rendu_puis_la_vente_terminee() {
        let mut h = harness_vente(a_payer("especes"), FENETRE);
        cliquer(&mut h, BOUTON_ESPECES);
        h.get_by_role_and_label(Role::Dialog, ESPECES_TITRE);
        saisir(&mut h, CHAMP_RECU, "10");
        h.get_by_label(MONNAIE_A_RENDRE);
        h.get_by_label(&nbsp("5,10EUR"));
        cliquer(&mut h, BOUTON_VALIDER);

        assert_eq!(nombre_de_pieces(&h), 1);
        assert!(matches!(
            h.state().modale,
            Some(Modale::VenteTerminee { .. })
        ));
        assert!(h.state().panier.is_empty());
        assert!(h.state().saisie_recu.is_empty());
        h.get_by_role_and_label(Role::Dialog, VENTE_TERMINEE);
        h.get_by_label(&nbsp("5,10EUR")); // la monnaie à rendre, en grand
        let etat = h.state().etat.as_ref().unwrap();
        assert_eq!(etat.lignes("pain", false)[0].quantite, 8);

        cliquer(&mut h, BOUTON_NOUVELLE_VENTE);
        assert!(h.state().modale.is_none());
        assert!(h.state().etat.as_ref().unwrap().derniere_piece().is_none());
        assert_eq!(
            nombre_de_pieces(&h),
            1,
            "fermer le ticket ne supprime pas la vente"
        );
        h.get_by_label(TICKET_VIDE_TITRE);
    }

    #[test]
    fn valider_est_grise_si_le_recu_ne_couvre_pas_le_total() {
        let mut h = harness_vente(a_payer("recu-court"), FENETRE);
        cliquer(&mut h, BOUTON_ESPECES);
        // Aucun montant : rien n'est écrit en rouge, mais on ne peut pas valider.
        assert!(est_grise(&h, BOUTON_VALIDER));
        saisir(&mut h, CHAMP_RECU, "2");
        h.get_by_label(IL_MANQUE);
        h.get_by_label(&nbsp("2,90EUR"));
        assert!(est_grise(&h, BOUTON_VALIDER));
        h.get_by_role_and_label(Role::Button, BOUTON_VALIDER)
            .click();
        h.run();
        assert_eq!(nombre_de_pieces(&h), 0);
        assert_eq!(h.state().modale, Some(Modale::Especes));
    }

    #[test]
    fn un_montant_absurde_est_signale_sans_rien_ecrire() {
        let mut h = harness_vente(a_payer("absurde"), FENETRE);
        cliquer(&mut h, BOUTON_ESPECES);
        for saisie in ["abc", "1e5", "-5", "12,345", "99999999999999999999"] {
            h.state_mut().saisie_recu = saisie.to_string();
            h.run();
            h.get_by_label_contains("Montant illisible");
            assert!(est_grise(&h, BOUTON_VALIDER), "{saisie}");
            h.get_by_role_and_label(Role::Button, BOUTON_VALIDER)
                .click();
            h.run();
            assert_eq!(nombre_de_pieces(&h), 0, "{saisie}");
        }
    }

    #[test]
    fn un_double_clic_sur_valider_n_enregistre_qu_une_vente() {
        let mut app = app_vendeuse("double-vente");
        app.panier = vec![("Pain".to_string(), 1)];
        let mut h = harness_vente(app, FENETRE);
        cliquer(&mut h, BOUTON_ESPECES);
        saisir(&mut h, CHAMP_RECU, "2");
        cliquer(&mut h, BOUTON_VALIDER);
        // Le second clic tombe sur un bouton qui n'existe plus : rien ne se passe.
        if let Some(bouton) = h.query_by_role_and_label(Role::Button, BOUTON_VALIDER) {
            bouton.click();
            h.run();
        }
        assert_eq!(nombre_de_pieces(&h), 1);
        let etat = h.state().etat.as_ref().unwrap();
        assert_eq!(etat.lignes("pain", false)[0].quantite, 9);
    }

    #[test]
    fn un_montant_rapide_remplit_le_recu() {
        let mut h = harness_vente(a_payer("rapide"), FENETRE);
        cliquer(&mut h, BOUTON_ESPECES);
        cliquer(&mut h, QUICK_EXACT);
        assert_eq!(h.state().saisie_recu, "4,90");
        h.get_by_label(&nbsp("0,00EUR"));
        assert!(!est_grise(&h, BOUTON_VALIDER));
    }

    #[test]
    fn par_carte_le_caissier_confirme_a_la_main() {
        let mut app = app_vendeuse("carte");
        app.panier = vec![("Savon".to_string(), 1)];
        let mut h = harness_vente(app, FENETRE);
        cliquer(&mut h, BOUTON_CARTE);
        h.get_by_role_and_label(Role::Dialog, CARTE_TITRE);
        h.get_by_label(CARTE_ATTENTE);
        h.get_by_label(CARTE_AIDE);
        assert!(
            h.query_by_label(CHAMP_RECU).is_none(),
            "pas de montant reçu par carte"
        );
        assert_eq!(
            nombre_de_pieces(&h),
            0,
            "rien n'est vendu avant la confirmation"
        );
        cliquer(&mut h, BOUTON_ACCEPTE);
        assert_eq!(nombre_de_pieces(&h), 1);
        let etat = h.state().etat.as_ref().unwrap();
        assert_eq!(etat.derniere_piece().unwrap().moyen, Moyen::Carte);
        assert!(matches!(
            h.state().modale,
            Some(Modale::VenteTerminee { .. })
        ));
        h.get_by_label(&vente_terminee_detail("Carte", &nbsp("2,50EUR")));
    }

    #[test]
    fn une_carte_refusee_ne_vend_rien_et_propose_de_reessayer_ou_d_encaisser_en_especes() {
        let mut app = app_vendeuse("refusee");
        app.panier = vec![("Savon".to_string(), 1)];
        let mut h = harness_vente(app, FENETRE);
        cliquer(&mut h, BOUTON_CARTE);
        cliquer(&mut h, BOUTON_REFUSE);
        h.get_by_label(CARTE_REFUSEE);
        h.get_by_label(CARTE_REFUSEE_AIDE);
        assert_eq!(nombre_de_pieces(&h), 0);
        cliquer(&mut h, BOUTON_REESSAYER);
        h.get_by_label(CARTE_ATTENTE);
        cliquer(&mut h, BOUTON_REFUSE);
        cliquer(&mut h, BOUTON_PAYER_ESPECES);
        assert_eq!(h.state().modale, Some(Modale::Especes));
        assert_eq!(h.state().panier.len(), 1, "le ticket est conservé");
    }

    #[test]
    fn autre_moyen_se_confirme_par_paiement_recu() {
        let mut app = app_vendeuse("autre");
        app.panier = vec![("Savon".to_string(), 1)];
        let mut h = harness_vente(app, FENETRE);
        cliquer(&mut h, BOUTON_AUTRE);
        h.get_by_role_and_label(Role::Dialog, AUTRE_TITRE);
        h.get_by_label(AUTRE_ATTENTE);
        assert!(
            h.query_by_role_and_label(Role::Button, BOUTON_REFUSE)
                .is_none()
        );
        cliquer(&mut h, BOUTON_RECU);
        let etat = h.state().etat.as_ref().unwrap();
        assert_eq!(etat.derniere_piece().unwrap().moyen, Moyen::Autre);
    }

    #[test]
    fn echap_retour_et_annuler_ferment_la_fenetre_sans_rien_enregistrer() {
        let mut h = harness_vente(a_payer("fermer"), FENETRE);
        cliquer(&mut h, BOUTON_ESPECES);
        h.key_press(Key::Escape);
        h.run();
        assert!(h.state().modale.is_none());
        cliquer(&mut h, BOUTON_ESPECES);
        cliquer(&mut h, BOUTON_RETOUR);
        assert!(h.state().modale.is_none());
        cliquer(&mut h, BOUTON_CARTE);
        cliquer(&mut h, BOUTON_ANNULER);
        assert!(h.state().modale.is_none());
        assert_eq!(nombre_de_pieces(&h), 0);
        assert_eq!(h.state().panier.len(), 2);
        assert!(h.get_by_label(CHAMP_SCAN).accesskit_node().is_focused());
    }

    #[test]
    fn une_erreur_de_la_bibliotheque_s_affiche_dans_la_fenetre() {
        let mut h = harness_vente(a_payer("erreur-fenetre"), FENETRE);
        cliquer(&mut h, BOUTON_ESPECES);
        saisir(&mut h, CHAMP_RECU, "10");
        // Le stock baisse entre-temps : il n'en reste qu'un, on en vend deux.
        assert!(h.state_mut().etat.as_mut().unwrap().sortie("Pain", "9"));
        cliquer(&mut h, BOUTON_VALIDER);
        assert_eq!(nombre_de_pieces(&h), 0);
        assert_eq!(h.state().panier.len(), 2, "le ticket est conservé");
        assert_eq!(h.state().modale, Some(Modale::Especes));
        // Une fois sous l'en-tête, une fois dans la fenêtre : on la voit sous le voile.
        assert!(h.get_all_by_label_contains("Erreur :").count() >= 2);
    }

    #[test]
    fn le_journal_altere_bloque_le_paiement_et_le_dit() {
        let mut app = app_journal_altere("altere-paiement");
        app.panier = vec![("Savon".to_string(), 1)];
        let h = harness_vente(app, FENETRE);
        h.get_by_label_contains("Le journal de caisse a été modifié");
        for nom in [BOUTON_ESPECES, BOUTON_CARTE, BOUTON_AUTRE] {
            assert!(est_grise(&h, nom), "{nom}");
        }
    }

    #[test]
    fn voir_le_ticket_detaille_la_tva_par_taux_et_la_mention_non_fiscal() {
        let mut app = app_vendeuse("voir-ticket");
        app.panier = vec![("Pain".to_string(), 3), ("Savon".to_string(), 1)];
        let mut h = harness_vente(app, FENETRE);
        cliquer(&mut h, BOUTON_CARTE);
        cliquer(&mut h, BOUTON_ACCEPTE);
        assert!(
            h.query_by_label(MENTION_NON_CERTIFIE).is_none(),
            "ticket caché au départ"
        );
        cliquer(&mut h, BOUTON_VOIR_TICKET);
        h.get_by_label(&titre_du_ticket("01-0001"));
        h.get_by_label_contains("14/11/2023 23:13");
        for ligne in [
            "Total TTC : 6,10\u{a0}€",
            "TVA 5,5\u{a0}% incluse : 0,19\u{a0}€",
            "TVA 20\u{a0}% incluse : 0,42\u{a0}€",
        ] {
            h.get_by_label(ligne);
        }
        h.get_by_label(&moyen_du_ticket("Carte"));
        h.get_by_label(MENTION_NON_CERTIFIE);
        cliquer(&mut h, BOUTON_CACHER_TICKET);
        assert!(h.query_by_label(MENTION_NON_CERTIFIE).is_none());
    }

    #[test]
    fn voir_le_ticket_detaille_la_taxe_aux_etats_unis() {
        let mut etat = Etat::ouvrir(chemin_temp("ticket-us"))
            .unwrap()
            .avec_horloge(|| FIXE);
        assert!(etat.definir_reglages(Profil::EtatsUnis, Devise::Usd, "8"));
        assert!(etat.ajouter_avec_prix("Pain", "0", "1.20", ""));
        assert!(etat.entree("Pain", "10"));
        let mut app = Application::nouvelle(etat);
        app.panier = vec![("Pain".to_string(), 2)];
        let mut h = harness_vente(app, FENETRE);
        cliquer(&mut h, BOUTON_CARTE);
        cliquer(&mut h, BOUTON_ACCEPTE);
        cliquer(&mut h, BOUTON_VOIR_TICKET);
        // 2,40 $ hors taxe + 8 % (19,2 cents arrondis à 19) = 2,59 $.
        for ligne in [
            "Sous-total : $2.40",
            "Taxe 8\u{a0}% : $0.19",
            "Total : $2.59",
        ] {
            h.get_by_label(ligne);
        }
    }

    #[test]
    fn tous_les_textes_peints_des_fenetres_de_paiement_atteignent_4_5() {
        // Espèces : reçu suffisant, puis insuffisant, puis illisible.
        for recu in ["10", "2", "x"] {
            let mut app = a_payer("contraste-especes");
            app.saisie_recu = recu.to_string();
            app.modale = Some(Modale::Especes);
            let textes = sans_les_textes_grises(textes_peints(&mut app, FENETRE, 1.0));
            assert!(textes.iter().any(|t| t.texte == ESPECES_TITRE));
            verifier_contrastes(&textes, &format!("espèces, reçu « {recu} »"));
        }
        // Carte : en attente, refusée ; autre moyen.
        for modale in [
            Modale::Terminal {
                moyen: Moyen::Carte,
                refuse: false,
            },
            Modale::Terminal {
                moyen: Moyen::Carte,
                refuse: true,
            },
            Modale::Terminal {
                moyen: Moyen::Autre,
                refuse: false,
            },
        ] {
            let mut app = a_payer("contraste-terminal");
            app.modale = Some(modale.clone());
            let textes = textes_peints(&mut app, FENETRE, 1.0);
            verifier_contrastes(&textes, &format!("{modale:?}"));
        }
        // Vente terminée, avec le ticket affiché.
        let mut app = a_payer("contraste-terminee");
        let etat = app.etat.as_mut().unwrap();
        assert!(etat.encaisser(&app.panier, Moyen::Especes, "10"));
        app.panier.clear();
        app.modale = Some(Modale::VenteTerminee {
            ticket_visible: true,
        });
        // Derrière la fenêtre, le ticket est vidé : les boutons de paiement sont grisés.
        let textes = sans_les_textes_grises(textes_peints(&mut app, FENETRE, 1.0));
        assert!(textes.iter().any(|t| t.texte == MENTION_NON_CERTIFIE));
        verifier_contrastes(&textes, "vente terminée");
    }
}
