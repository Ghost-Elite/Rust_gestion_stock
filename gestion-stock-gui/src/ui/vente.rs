//! L'écran de vente : le champ de scan, le ticket en cours, le total, les boutons de paiement et
//! la barre de fonctions. Les fenêtres de paiement sont dans `paiement`.
//!
//! Dimensions et couleurs : celles de `maquette/1-vente.html` (en-tête 56, marges de 16, colonne
//! de paiement de 420, lignes de 68, boutons de paiement de 84 et 72, cases de 64).

use super::composants::{
    LigneBouton, Style, bandeau_alerte, bloc_de_deux_lignes, bouton_a_lignes, bouton_large,
    bouton_symbole, carte_marge, discret, modale, repere, zone, zone_fixe,
};
use super::{
    Application, Confirmation, LARGEUR_CAISSE_DEUX_COLONNES, LARGEUR_DEUX_COLONNES, Modale,
};
use crate::palette::{
    ACCENT, ACCENT_DOUX, ACCENT_TEXTE, ALERTE_BORD, ALERTE_TEXTE, BORDURE_BOUTON, FILET,
    FILET_LIGNE, FOND, FOND_DOUX, GRISE_FOND, GRISE_TEXTE, SURFACE, TEXTE, TEXTE_DISCRET,
    TEXTE_SECONDAIRE, couleur,
};
use crate::police::{Graisse, police_disponible};
use crate::scan::{ResultatScan, analyser, texte_recherche};
use crate::textes::*;
use eframe::egui::{
    Align, CentralPanel, Context, CornerRadius, FontId, Frame, Id, Key, Label, Layout, Margin,
    Modifiers, Panel, RichText, ScrollArea, Sense, Stroke, TextEdit, TextFormat, Ui, UiBuilder,
    Vec2, WidgetInfo, WidgetType, text::LayoutJob,
};
use gestion_stock::erreur::ErreurStock;
use gestion_stock::monnaie::{Devise, Montant};
use gestion_stock::reglages::Profil;
use gestion_stock::vente::{Devis, Moyen};

const LARGEUR_COLONNE_PAIEMENT: f32 = 420.0;
/// 52 de repère + 2 × 8 de marge + 1 de filet.
const HAUTEUR_LIGNE: f32 = 69.0;
const HAUTEUR_PIED: f32 = 46.0;
const HAUTEUR_CASE: f32 = 64.0;
const HAUTEUR_TICKET_ETROIT: f32 = 420.0;

/// Le devis du ticket : `None` s'il est vide.
type DevisTicket = Option<Result<Devis, ErreurStock>>;

fn devis_du_ticket(app: &Application) -> DevisTicket {
    if app.panier.is_empty() {
        return None;
    }
    app.etat.as_ref().map(|etat| etat.devis(&app.panier))
}

// ---- Ce que l'on peut faire du ticket -----------------------------------------------------------

/// Met `quantite` unités du produit au ticket, si on peut le vendre : il lui faut un prix, du
/// stock, et pas plus que ce qu'il en reste en comptant ce qui est déjà au ticket. Sinon, un
/// message dit pourquoi et rien ne change. Renvoie vrai si c'est ajouté.
pub(super) fn ajouter_au_ticket(app: &mut Application, nom: &str, quantite: u32) -> bool {
    let Some(etat) = app.etat.as_mut() else {
        return false;
    };
    let Some(ligne) = etat.lignes("", false).into_iter().find(|l| l.nom == nom) else {
        return false;
    };
    let raison = if ligne.prix.is_none() {
        Some(PRIX_MANQUANT)
    } else if ligne.quantite == 0 {
        Some(EPUISE)
    } else {
        None
    };
    if let Some(raison) = raison {
        app.message(produit_non_vendable(nom, raison));
        return false;
    }
    let deja = app
        .panier
        .iter()
        .find(|(n, _)| n == nom)
        .map_or(0, |(_, q)| *q);
    if deja.saturating_add(quantite) > ligne.quantite {
        app.message(stock_limite(nom, ligne.quantite));
        return false;
    }
    // Choisir un produit commence une nouvelle vente : le ticket précédent se ferme.
    if let Some(etat) = app.etat.as_mut() {
        etat.effacer_ticket();
    }
    app.ajouter_au_panier(nom, quantite);
    app.ligne_choisie = Some(nom.to_string());
    true
}

/// Valide ce qui est tapé dans le champ de scan.
fn valider_scan(app: &mut Application) {
    let saisie = std::mem::take(&mut app.scan);
    let Some(etat) = &app.etat else {
        return;
    };
    let produits = etat.lignes("", false);
    match analyser(&saisie, &produits) {
        ResultatScan::Vide => {}
        ResultatScan::Ajouter { nom, quantite } => {
            ajouter_au_ticket(app, &nom, quantite);
        }
        ResultatScan::Plusieurs => {
            app.ouvrir_la_recherche(texte_recherche(&saisie));
        }
        ResultatScan::Introuvable(texte) => app.modale = Some(Modale::Introuvable(texte)),
    }
}

fn demander_l_annulation(app: &mut Application) {
    if app.panier.is_empty() {
        app.message(TOAST_AUCUNE_VENTE);
    } else {
        app.modale = Some(Modale::Confirmer(Confirmation::AnnulerVente));
    }
}

fn nombre_d_articles(app: &Application) -> u32 {
    app.panier.iter().map(|(_, quantite)| *quantite).sum()
}

/// Les raccourcis clavier : F2 (rechercher), Suppr (supprimer l'article choisi) et Échap (effacer
/// le champ, puis annuler la vente). Suppr ne compte que si le champ de scan est vide : sinon c'est
/// une touche d'édition, et supprimer un article par erreur serait grave.
pub(super) fn raccourcis(ui: &mut Ui, app: &mut Application) {
    let scan_vide = app.scan.is_empty();
    let (f2, suppr, echap) = ui.input_mut(|i| {
        (
            i.consume_key(Modifiers::NONE, Key::F2),
            scan_vide && i.consume_key(Modifiers::NONE, Key::Delete),
            i.consume_key(Modifiers::NONE, Key::Escape),
        )
    });
    if f2 {
        app.ouvrir_la_recherche(String::new());
    }
    if suppr && app.ligne_choisie.is_some() {
        app.modale = Some(Modale::Confirmer(Confirmation::SupprimerArticle));
    }
    if echap {
        if app.scan.is_empty() {
            demander_l_annulation(app);
        } else {
            app.scan.clear();
        }
    }
}

// ---- L'écran ------------------------------------------------------------------------------------

pub(super) fn ecran_vente(ui: &mut Ui, app: &mut Application) {
    if app.modale.is_none() {
        raccourcis(ui, app);
    }
    if let Some(nom) = &app.ligne_choisie
        && !app.panier.iter().any(|(n, _)| n == nom)
    {
        app.ligne_choisie = None;
    }
    let devis = devis_du_ticket(app);
    if ui.available_width() >= LARGEUR_CAISSE_DEUX_COLONNES {
        Panel::bottom(Id::new("fonctions"))
            .frame(
                Frame::new()
                    .fill(couleur(FOND))
                    .inner_margin(Margin::same(16)),
            )
            .show_separator_line(false)
            .show(ui, |ui| barre_de_fonctions(ui, app));
        // Largeur de la colonne + ses marges (8 à gauche, 16 à droite) ; 8 + 8 entre les deux.
        Panel::right(Id::new("paiement"))
            .frame(Frame::new().fill(couleur(FOND)).inner_margin(Margin {
                left: 8,
                right: 16,
                top: 16,
                bottom: 0,
            }))
            .exact_size(LARGEUR_COLONNE_PAIEMENT + 8.0 + 16.0)
            .resizable(false)
            .show_separator_line(false)
            .show(ui, |ui| {
                // Dans une fenêtre basse, la colonne défile d'un bloc : rien n'est jamais rogné.
                ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| colonne_paiement(ui, app, &devis));
            });
        CentralPanel::default()
            .frame(Frame::new().fill(couleur(FOND)).inner_margin(Margin {
                left: 16,
                right: 8,
                top: 16,
                bottom: 0,
            }))
            .show(ui, |ui| carte_ticket(ui, app, &devis, None));
    } else {
        // Fenêtre étroite ou texte agrandi : une seule colonne qui défile, le scan d'abord.
        CentralPanel::default()
            .frame(
                Frame::new()
                    .fill(couleur(FOND))
                    .inner_margin(Margin::same(16)),
            )
            .show(ui, |ui| {
                ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
                    ui.spacing_mut().item_spacing.y = 12.0;
                    carte_ticket(ui, app, &devis, Some(HAUTEUR_TICKET_ETROIT));
                    colonne_paiement(ui, app, &devis);
                    barre_de_fonctions(ui, app);
                });
            });
    }
}

// ---- Le ticket ----------------------------------------------------------------------------------

fn carte_ticket(ui: &mut Ui, app: &mut Application, devis: &DevisTicket, hauteur: Option<f32>) {
    Frame::new()
        .fill(couleur(SURFACE))
        .stroke(Stroke::new(1.0, couleur(FILET)))
        .corner_radius(CornerRadius::same(12))
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing = Vec2::ZERO;
            ui.set_width(ui.available_width());
            match hauteur {
                Some(h) => ui.set_height(h),
                None => ui.set_min_height(ui.available_height()),
            }
            champ_de_scan(ui, app);
            let reste = (ui.available_height() - HAUTEUR_PIED).max(HAUTEUR_LIGNE);
            ScrollArea::vertical()
                .auto_shrink([false, false])
                .max_height(reste)
                .show(ui, |ui| lignes_du_ticket(ui, app, devis));
            pied_du_ticket(ui, app);
        });
}

/// Le champ de scan : un lecteur de codes-barres tape le code puis Entrée, comme une personne qui
/// tape un nom. Il garde le focus : au démarrage et après chaque action.
fn champ_de_scan(ui: &mut Ui, app: &mut Application) {
    let zone = Frame::new()
        .inner_margin(Margin::same(14))
        .show(ui, |ui| {
            Frame::new()
                .stroke(Stroke::new(2.0, couleur(ACCENT)))
                .corner_radius(CornerRadius::same(10))
                .inner_margin(Margin::symmetric(16, 0))
                .show(ui, |ui| {
                    zone(
                        ui,
                        Vec2::new(ui.available_width(), 52.0),
                        Layout::left_to_right(Align::Center),
                        |ui| {
                            ui.spacing_mut().item_spacing.x = 10.0;
                            icone_code_barres(ui);
                            let champ = TextEdit::singleline(&mut app.scan)
                                .hint_text(
                                    RichText::new(CHAMP_SCAN)
                                        .size(18.0)
                                        .color(couleur(TEXTE_DISCRET)),
                                )
                                .font(FontId::proportional(18.0))
                                .frame(Frame::NONE)
                                .desired_width(f32::INFINITY);
                            let reponse = ui.add(champ);
                            // egui n'utilise pas le texte d'invite comme nom : on le donne.
                            ui.ctx().accesskit_node_builder(reponse.id, |noeud| {
                                noeud.set_label(CHAMP_SCAN);
                            });
                            if app.focus_scan && app.modale.is_none() {
                                reponse.request_focus();
                                app.focus_scan = false;
                            }
                            if reponse.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter)) {
                                valider_scan(app);
                                app.focus_scan = true;
                            }
                        },
                    );
                });
        })
        .response
        .rect;
    ui.painter().hline(
        zone.x_range(),
        zone.bottom(),
        Stroke::new(1.0, couleur(FILET)),
    );
}

/// Un petit code-barres entre quatre coins, dessiné (24 × 24, couleur d'accent).
fn icone_code_barres(ui: &mut Ui) {
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(24.0), Sense::hover());
    let trait_ = Stroke::new(1.8, couleur(ACCENT));
    let p = |x: f32, y: f32| rect.min + Vec2::new(x, y);
    let peintre = ui.painter();
    for (a, b, c) in [
        (p(3.0, 8.0), p(3.0, 3.0), p(8.0, 3.0)),
        (p(16.0, 3.0), p(21.0, 3.0), p(21.0, 8.0)),
        (p(21.0, 16.0), p(21.0, 21.0), p(16.0, 21.0)),
        (p(8.0, 21.0), p(3.0, 21.0), p(3.0, 16.0)),
    ] {
        peintre.line_segment([a, b], trait_);
        peintre.line_segment([b, c], trait_);
    }
    for x in [8.0, 12.0, 16.0] {
        peintre.line_segment([p(x, 7.0), p(x, 17.0)], trait_);
    }
}

enum ActionLigne {
    Choisir(String),
    Plus(String),
    Moins(String),
}

fn lignes_du_ticket(ui: &mut Ui, app: &mut Application, devis: &DevisTicket) {
    if app.panier.is_empty() {
        ticket_vide(ui);
        return;
    }
    if let Some(Err(erreur)) = devis {
        Frame::new().inner_margin(Margin::same(14)).show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 6.0;
            bandeau_alerte(ui, PANIER_INVALIDE);
            discret(ui, &erreur.to_string());
        });
    }
    let Some(etat) = &app.etat else {
        return;
    };
    let produits = etat.lignes("", false);
    let mut action = None;
    for (nom, quantite) in &app.panier {
        let stock = produits.iter().find(|p| &p.nom == nom).map(|p| p.quantite);
        let montants = match devis {
            Some(Ok(devis)) => devis.lignes.iter().find(|l| &l.nom == nom).and_then(|l| {
                let total = l.prix.multiplier(*quantite).ok()?;
                Some((
                    detail_de_ligne(*quantite, &etat.formater(l.prix)),
                    etat.formater(total),
                ))
            }),
            _ => None,
        };
        let choisie = app.ligne_choisie.as_deref() == Some(nom.as_str());
        let detail = montants.as_ref().map(|(detail, _)| detail.as_str());
        let total = montants.as_ref().map(|(_, total)| total.as_str());
        if let Some(a) = ligne_du_ticket(ui, nom, *quantite, stock, detail, total, choisie) {
            action = Some(a);
        }
    }
    match action {
        Some(ActionLigne::Choisir(nom)) => app.ligne_choisie = Some(nom),
        Some(ActionLigne::Plus(nom)) => app.ajouter_au_panier(&nom, 1),
        Some(ActionLigne::Moins(nom)) => app.retirer_du_panier_une_unite(&nom),
        None => {}
    }
}

fn ticket_vide(ui: &mut Ui) {
    ui.vertical_centered(|ui| {
        ui.spacing_mut().item_spacing.y = 8.0;
        ui.add_space((ui.available_height() / 2.0 - 40.0).max(20.0));
        ui.label(
            RichText::new(TICKET_VIDE_TITRE)
                .font(police_disponible(ui.ctx(), Graisse::SemiGrasse, 22.0))
                .color(couleur(TEXTE)),
        );
        ui.label(
            RichText::new(TICKET_VIDE_AIDE)
                .size(16.0)
                .color(couleur(TEXTE_DISCRET)),
        );
    });
}

/// Une ligne du ticket : repère, nom et détail, (si elle est choisie) « − » quantité « + », et
/// total. Toute la ligne se clique pour la choisir ; le nom long est coupé par des points de
/// suspension.
fn ligne_du_ticket(
    ui: &mut Ui,
    nom: &str,
    quantite: u32,
    stock: Option<u32>,
    detail: Option<&str>,
    total: Option<&str>,
    choisie: bool,
) -> Option<ActionLigne> {
    let (rect, _) = ui.allocate_exact_size(
        Vec2::new(ui.available_width(), HAUTEUR_LIGNE),
        Sense::hover(),
    );
    // La ligne est déclarée AVANT ses boutons : ceux-ci, déclarés après, gardent leurs clics.
    let id = ui.id().with(("ligne_du_ticket", nom));
    let reponse = ui.interact(rect, id, Sense::click());
    reponse.widget_info(|| {
        WidgetInfo::selected(WidgetType::Button, true, choisie, choisir_la_ligne(nom))
    });
    let fond = if choisie { ACCENT_DOUX } else { SURFACE };
    let peintre = ui.painter();
    peintre.rect_filled(rect, 0.0, couleur(fond));
    peintre.hline(
        rect.x_range(),
        rect.bottom() - 0.5,
        Stroke::new(1.0, couleur(FILET_LIGNE)),
    );
    super::composants::anneau_de_focus(ui, &reponse, 0);

    let interieur = rect.shrink2(Vec2::new(20.0, 8.0));
    let mut action = if reponse.clicked() {
        Some(ActionLigne::Choisir(nom.to_string()))
    } else {
        None
    };
    let mut enfant = ui.new_child(
        UiBuilder::new()
            .max_rect(interieur)
            .layout(Layout::left_to_right(Align::Center)),
    );
    enfant.spacing_mut().item_spacing.x = 16.0;
    repere(&mut enfant, nom, 52.0);
    let largeur_controles = if choisie {
        44.0 * 2.0 + 36.0 + 6.0 * 2.0
    } else {
        0.0
    };
    let reserve_controles = if choisie {
        largeur_controles + 16.0
    } else {
        0.0
    };
    let largeur_nom =
        (interieur.width() - 52.0 - 16.0 - 120.0 - 16.0 - reserve_controles).max(80.0);
    let police_nom = police_disponible(enfant.ctx(), Graisse::Moyenne, 17.0);
    zone(
        &mut enfant,
        Vec2::new(largeur_nom, 52.0),
        Layout::top_down(Align::Min),
        |ui| {
            let detail_police = FontId::proportional(14.0);
            bloc_de_deux_lignes(ui, 52.0, 2.0, &police_nom, &detail_police, |ui| {
                ui.add(
                    Label::new(
                        RichText::new(nom)
                            .font(police_nom.clone())
                            .color(couleur(TEXTE)),
                    )
                    .truncate()
                    .selectable(false),
                );
                if let Some(detail) = detail {
                    ui.add(
                        Label::new(
                            RichText::new(detail)
                                .size(14.0)
                                .color(couleur(TEXTE_DISCRET)),
                        )
                        .truncate()
                        .selectable(false),
                    );
                }
            });
        },
    );
    if choisie {
        enfant.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 6.0;
            let place = stock.is_some_and(|stock| quantite < stock);
            if bouton_symbole(ui, "\u{2212}", &retirer_une_unite(nom), quantite > 1, 44.0).clicked()
            {
                action = Some(ActionLigne::Moins(nom.to_string()));
            }
            ui.add_sized(
                [36.0, 44.0],
                Label::new(
                    RichText::new(quantite.to_string())
                        .font(police_disponible(ui.ctx(), Graisse::SemiGrasse, 18.0))
                        .color(couleur(TEXTE)),
                )
                .selectable(false),
            );
            if bouton_symbole(ui, "+", &ajouter_une_unite(nom), place, 44.0).clicked() {
                action = Some(ActionLigne::Plus(nom.to_string()));
            }
        });
    }
    zone(
        &mut enfant,
        Vec2::new(120.0, 52.0),
        Layout::right_to_left(Align::Center),
        |ui| {
            if let Some(total) = total {
                ui.add(
                    Label::new(
                        RichText::new(total)
                            .font(police_disponible(ui.ctx(), Graisse::SemiGrasse, 18.0))
                            .color(couleur(TEXTE)),
                    )
                    .selectable(false),
                );
            }
        },
    );
    action
}

fn pied_du_ticket(ui: &mut Ui, app: &Application) {
    let zone = Frame::new()
        .inner_margin(Margin::symmetric(20, 12))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(
                RichText::new(articles_au_ticket(nombre_d_articles(app)))
                    .size(15.0)
                    .color(couleur(TEXTE_DISCRET)),
            );
        })
        .response
        .rect;
    ui.painter()
        .hline(zone.x_range(), zone.top(), Stroke::new(1.0, couleur(FILET)));
}

// ---- Le total et les boutons de paiement --------------------------------------------------------

fn colonne_paiement(ui: &mut Ui, app: &mut Application, devis: &DevisTicket) {
    ui.spacing_mut().item_spacing.y = 12.0;
    let integrite = app.etat.as_ref().map_or(Ok(()), |etat| etat.integrite());
    if let Err(message) = &integrite {
        bandeau_alerte(ui, &journal_altere(message));
    }
    carte_totaux(ui, app, devis);
    let actif = matches!(devis, Some(Ok(_))) && integrite.is_ok();
    boutons_de_paiement(ui, app, actif);
}

/// La hauteur d'une ligne de texte de 16 points dans la maquette (« line-height: normal »).
const HAUTEUR_TEXTE: f32 = 20.0;

fn ligne_montant(ui: &mut Ui, libelle: &str, valeur: &str) {
    zone_fixe(
        ui,
        Vec2::new(ui.available_width(), HAUTEUR_TEXTE),
        Layout::left_to_right(Align::Center),
        |ui| {
            ui.label(
                RichText::new(libelle)
                    .size(16.0)
                    .color(couleur(TEXTE_SECONDAIRE)),
            );
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.label(
                    RichText::new(valeur)
                        .size(16.0)
                        .color(couleur(TEXTE_SECONDAIRE)),
                );
            });
        },
    );
}

/// Le total en grand : 64 points gras, interlettrage de −0,02 em, sur une ligne de 64 de haut,
/// puis l'unité en 24 points moyens et gris, à droite et sur la même ligne de base.
///
/// Dans la maquette (« line-height: 1 »), le texte de 1,3 em déborde de la ligne d'autant en haut
/// qu'en bas ; egui n'a pas ce demi-interligne négatif, on décale donc le texte à la main.
fn total_en_grand(ui: &mut Ui, devise: Devise, total: Montant) {
    const HAUTEUR: f32 = 64.0;
    const HAUTEUR_DU_TEXTE: f32 = 64.0 * 1.3;
    let nombre = devise.formater_nombre(total);
    let format_nombre = TextFormat {
        font_id: police_disponible(ui.ctx(), Graisse::Grasse, 64.0),
        color: couleur(TEXTE),
        extra_letter_spacing: -0.02 * 64.0,
        ..Default::default()
    };
    let format_unite = TextFormat {
        font_id: police_disponible(ui.ctx(), Graisse::Moyenne, 24.0),
        color: couleur(TEXTE_DISCRET),
        ..Default::default()
    };
    let mut texte = LayoutJob::default();
    if devise.symbole_avant() {
        texte.append(devise.symbole(), 0.0, format_unite);
        texte.append(&nombre, 0.0, format_nombre);
    } else {
        texte.append(&nombre, 0.0, format_nombre.clone());
        texte.append("\u{a0}", 0.0, format_nombre);
        texte.append(devise.symbole(), 0.0, format_unite);
    }
    let (rect, _) =
        ui.allocate_exact_size(Vec2::new(ui.available_width(), HAUTEUR), Sense::hover());
    let decalage = (HAUTEUR_DU_TEXTE - HAUTEUR) / 2.0;
    let mut enfant = ui.new_child(
        UiBuilder::new()
            .max_rect(rect.translate(Vec2::new(0.0, -decalage)))
            .layout(Layout::right_to_left(Align::Min)),
    );
    enfant.add(Label::new(texte).selectable(false));
}

/// Le filet sous les sous-totaux : 1 de haut, avec 4 de marge de chaque côté (en plus de l'écart).
fn filet_du_total(ui: &mut Ui) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 9.0), Sense::hover());
    ui.painter().hline(
        rect.x_range(),
        rect.top() + 4.5,
        Stroke::new(1.0, couleur(FILET)),
    );
}

/// Le total : sous-total, (aux États-Unis) la taxe par taux, le total en grand, et (en Europe) la
/// TVA incluse par taux. La remise de la maquette arrive à l'étape B.
fn carte_totaux(ui: &mut Ui, app: &Application, devis: &DevisTicket) {
    let Some(etat) = &app.etat else {
        return;
    };
    let reglages = etat.reglages();
    let (devise, profil) = (reglages.devise, reglages.profil);
    let (total, sous_total, taxes) = match devis {
        Some(Ok(devis)) => {
            let sous_total = match profil {
                Profil::Europe => devis.total,
                Profil::EtatsUnis => Montant(devis.groupes.iter().map(|g| g.base.0).sum()),
            };
            let taxes: Vec<(String, Montant)> = devis
                .groupes
                .iter()
                .map(|g| (devise.formater_taux(g.taux_bp), g.taxe))
                .collect();
            (devis.total, sous_total, taxes)
        }
        _ => (Montant::ZERO, Montant::ZERO, Vec::new()),
    };
    carte_marge(ui, 20, |ui| {
        ui.spacing_mut().item_spacing.y = 10.0;
        ligne_montant(ui, LIBELLE_SOUS_TOTAL, &devise.formater(sous_total));
        if profil == Profil::EtatsUnis {
            for (taux, taxe) in &taxes {
                ligne_montant(ui, &libelle_taxe(profil, taux), &devise.formater(*taxe));
            }
        }
        filet_du_total(ui);
        zone_fixe(
            ui,
            Vec2::new(ui.available_width(), HAUTEUR_TEXTE),
            Layout::left_to_right(Align::Center),
            |ui| {
                ui.label(
                    RichText::new(LIBELLE_TOTAL_GRAND)
                        .font(police_disponible(ui.ctx(), Graisse::SemiGrasse, 16.0))
                        .color(couleur(TEXTE_SECONDAIRE)),
                );
            },
        );
        total_en_grand(ui, devise, total);
        if profil == Profil::Europe {
            for (taux, taxe) in &taxes {
                zone_fixe(
                    ui,
                    Vec2::new(ui.available_width(), 16.0),
                    Layout::right_to_left(Align::Center),
                    |ui| {
                        ui.label(
                            RichText::new(format!(
                                "{} : {}",
                                libelle_taxe(profil, taux),
                                devise.formater(*taxe)
                            ))
                            .size(13.0)
                            .color(couleur(TEXTE_DISCRET)),
                        );
                    },
                );
            }
        }
    });
}

fn boutons_de_paiement(ui: &mut Ui, app: &mut Application, actif: bool) {
    let largeur = ui.available_width();
    let (fond, texte) = if actif {
        (ACCENT, ACCENT_TEXTE)
    } else {
        (GRISE_FOND, GRISE_TEXTE)
    };
    let especes = bouton_a_lignes(
        ui,
        BOUTON_ESPECES,
        Vec2::new(largeur, 84.0),
        fond,
        Stroke::NONE,
        12,
        &[LigneBouton {
            texte: BOUTON_ESPECES,
            taille: 22.0,
            graisse: Graisse::Normale,
            couleur: texte,
        }],
        actif,
    );
    if especes.clicked() {
        app.saisie_recu.clear();
        app.focus_recu = true;
        app.modale = Some(Modale::Especes);
    }
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 12.0;
        let moitie = (largeur - 12.0) / 2.0;
        let (fond, contour, texte) = if actif {
            (SURFACE, Stroke::new(2.0, couleur(ACCENT)), ACCENT)
        } else {
            (
                FOND_DOUX,
                Stroke::new(1.0, couleur(GRISE_FOND)),
                GRISE_TEXTE,
            )
        };
        for (nom, moyen) in [(BOUTON_CARTE, Moyen::Carte), (BOUTON_AUTRE, Moyen::Autre)] {
            let reponse = bouton_a_lignes(
                ui,
                nom,
                Vec2::new(moitie, 72.0),
                fond,
                contour,
                12,
                &[LigneBouton {
                    texte: nom,
                    taille: 19.0,
                    graisse: Graisse::SemiGrasse,
                    couleur: texte,
                }],
                actif,
            );
            if reponse.clicked() {
                app.modale = Some(Modale::Terminal {
                    moyen,
                    refuse: false,
                });
            }
        }
    });
}

// ---- La barre de fonctions ----------------------------------------------------------------------

#[derive(Clone, Copy)]
enum Fonction {
    Rechercher,
    Supprimer,
    Annuler,
    Menu,
}

fn barre_de_fonctions(ui: &mut Ui, app: &mut Application) {
    let colonnes = if ui.available_width() >= LARGEUR_DEUX_COLONNES - 240.0 {
        4
    } else {
        2
    };
    let ecart = 10.0;
    let largeur = (ui.available_width() - ecart * (colonnes as f32 - 1.0)) / colonnes as f32;
    ui.spacing_mut().item_spacing.y = ecart;
    let cases = [
        Fonction::Rechercher,
        Fonction::Supprimer,
        Fonction::Annuler,
        Fonction::Menu,
    ];
    for rangee in cases.chunks(colonnes) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = ecart;
            for fonction in rangee {
                case_de_fonction(ui, app, *fonction, largeur);
            }
        });
    }
}

fn case_de_fonction(ui: &mut Ui, app: &mut Application, fonction: Fonction, largeur: f32) {
    let (libelle, detail, nom, actif, couleur_texte, bord) = match fonction {
        Fonction::Rechercher => (
            FONCTION_RECHERCHER,
            RACCOURCI_RECHERCHER,
            fonction_nom(FONCTION_RECHERCHER, RACCOURCI_RECHERCHER),
            true,
            TEXTE,
            BORDURE_BOUTON,
        ),
        Fonction::Supprimer => (
            FONCTION_SUPPRIMER,
            RACCOURCI_SUPPRIMER,
            fonction_nom(FONCTION_SUPPRIMER, RACCOURCI_SUPPRIMER),
            app.ligne_choisie.is_some(),
            if app.ligne_choisie.is_some() {
                TEXTE
            } else {
                GRISE_TEXTE
            },
            BORDURE_BOUTON,
        ),
        Fonction::Annuler => (
            FONCTION_ANNULER,
            RACCOURCI_ANNULER,
            fonction_nom(FONCTION_ANNULER, RACCOURCI_ANNULER),
            true,
            ALERTE_TEXTE,
            ALERTE_BORD,
        ),
        Fonction::Menu => (
            FONCTION_MENU,
            FONCTION_MENU_DETAIL,
            FONCTION_MENU.to_string(),
            true,
            TEXTE,
            BORDURE_BOUTON,
        ),
    };
    let detail_couleur = if actif { TEXTE_DISCRET } else { GRISE_TEXTE };
    let reponse = bouton_a_lignes(
        ui,
        &nom,
        Vec2::new(largeur, HAUTEUR_CASE),
        SURFACE,
        Stroke::new(1.0, couleur(bord)),
        10,
        &[
            LigneBouton {
                texte: libelle,
                taille: 15.0,
                graisse: Graisse::SemiGrasse,
                couleur: couleur_texte,
            },
            LigneBouton {
                texte: detail,
                taille: 12.0,
                graisse: Graisse::Moyenne,
                couleur: detail_couleur,
            },
        ],
        actif,
    );
    if !reponse.clicked() {
        return;
    }
    match fonction {
        Fonction::Rechercher => app.ouvrir_la_recherche(String::new()),
        Fonction::Supprimer => {
            app.modale = Some(Modale::Confirmer(Confirmation::SupprimerArticle));
        }
        Fonction::Annuler => demander_l_annulation(app),
        Fonction::Menu => app.modale = Some(Modale::Menu),
    }
}

// ---- Les fenêtres de l'écran de vente -----------------------------------------------------------

/// « Annuler la vente ? » et « Supprimer cet article ? » : le premier clic ne change rien.
pub(super) fn fenetre_confirmation(ctx: &Context, app: &mut Application, question: Confirmation) {
    let articles = nombre_d_articles(app);
    let nom = app.ligne_choisie.clone().unwrap_or_default();
    let oui_texte = match question {
        Confirmation::AnnulerVente => OUI_ANNULER,
        Confirmation::SupprimerArticle => OUI_SUPPRIMER,
    };
    let (mut oui, mut non) = (false, false);
    let ferme = modale(
        ctx,
        "confirmation",
        460.0,
        confirmation_titre(question),
        |ui| {
            ui.spacing_mut().item_spacing.y = 14.0;
            ui.heading(confirmation_titre(question));
            ui.label(
                RichText::new(confirmation_texte(question, articles, &nom))
                    .size(16.0)
                    .color(couleur(TEXTE_SECONDAIRE)),
            );
            ui.add_space(6.0);
            ui.columns(2, |colonnes| {
                let largeur = colonnes[0].available_width();
                non = bouton_large(
                    &mut colonnes[0],
                    NON_REVENIR,
                    Style::Secondaire,
                    true,
                    Vec2::new(largeur, 56.0),
                )
                .clicked();
                oui = bouton_large(
                    &mut colonnes[1],
                    oui_texte,
                    Style::Confirmation,
                    true,
                    Vec2::new(largeur, 56.0),
                )
                .clicked();
            });
        },
    );
    if oui {
        match question {
            Confirmation::AnnulerVente => {
                app.panier.clear();
                app.ligne_choisie = None;
                app.saisie_recu.clear();
                app.message(TOAST_VENTE_ANNULEE);
            }
            Confirmation::SupprimerArticle => {
                if let Some(nom) = app.ligne_choisie.take() {
                    app.panier.retain(|(n, _)| *n != nom);
                }
            }
        }
    }
    if oui || non || ferme {
        app.modale = None;
        app.focus_scan = true;
    }
}

/// « Article introuvable » : scanner à nouveau, ou chercher par le nom.
pub(super) fn fenetre_introuvable(ctx: &Context, app: &mut Application, saisie: &str) {
    let (mut encore, mut chercher) = (false, false);
    let ferme = modale(ctx, "introuvable", 460.0, INTROUVABLE_TITRE, |ui| {
        ui.spacing_mut().item_spacing.y = 14.0;
        ui.heading(INTROUVABLE_TITRE);
        ui.label(
            RichText::new(introuvable_texte(saisie))
                .size(16.0)
                .color(couleur(TEXTE_SECONDAIRE)),
        );
        ui.add_space(6.0);
        ui.columns(2, |colonnes| {
            let largeur = colonnes[0].available_width();
            encore = bouton_large(
                &mut colonnes[0],
                SCANNER_A_NOUVEAU,
                Style::Secondaire,
                true,
                Vec2::new(largeur, 56.0),
            )
            .clicked();
            chercher = bouton_large(
                &mut colonnes[1],
                FONCTION_RECHERCHER,
                Style::Principal,
                true,
                Vec2::new(largeur, 56.0),
            )
            .clicked();
        });
    });
    if chercher {
        app.ouvrir_la_recherche(saisie.to_string());
    }
    if encore || chercher || ferme {
        app.modale = None;
        app.focus_scan = true;
    }
}

#[cfg(test)]
mod tests {
    use super::super::essais::*;
    use super::super::*;
    use eframe::egui::accesskit::Role;
    use eframe::egui::{Key, Pos2, Rect, Vec2};
    use egui_kittest::{
        Harness,
        kittest::{NodeT, Queryable},
    };

    fn acheteur(nom: &str, panier: &[(&str, u32)]) -> Application {
        let mut app = app_vendeuse(nom);
        app.panier = panier.iter().map(|(n, q)| (n.to_string(), *q)).collect();
        app
    }

    fn peint(textes: &[TextePeint], texte: &str) -> bool {
        textes.iter().any(|t| t.texte == texte)
    }

    #[test]
    fn scanner_un_nom_ajoute_une_unite_choisit_la_ligne_et_vide_le_champ() {
        let mut h = harness_vente(app_vendeuse("scan"), FENETRE);
        scanner(&mut h, "pain");
        assert_eq!(h.state().panier, [("Pain".to_string(), 1)]);
        assert_eq!(h.state().ligne_choisie.as_deref(), Some("Pain"));
        assert!(h.state().scan.is_empty());
        assert_eq!(
            nombre_de_pieces(&h),
            0,
            "rien n'est vendu avant le paiement"
        );
    }

    #[test]
    fn le_champ_de_scan_a_le_focus_au_demarrage_et_le_garde() {
        let mut h = harness_vente(app_vendeuse("focus"), FENETRE);
        assert!(h.get_by_label(CHAMP_SCAN).accesskit_node().is_focused());
        scanner(&mut h, "pain");
        assert!(h.get_by_label(CHAMP_SCAN).accesskit_node().is_focused());
        // Après une fenêtre fermée, le champ reprend le focus.
        scanner(&mut h, "zzz");
        assert!(matches!(h.state().modale, Some(Modale::Introuvable(_))));
        cliquer(&mut h, SCANNER_A_NOUVEAU);
        assert!(h.get_by_label(CHAMP_SCAN).accesskit_node().is_focused());
    }

    #[test]
    fn un_prefixe_de_quantite_multiplie() {
        let mut h = harness_vente(app_vendeuse("quantite"), FENETRE);
        scanner(&mut h, "3 x pain");
        assert_eq!(h.state().panier, [("Pain".to_string(), 3)]);
        scanner(&mut h, "2*pain");
        assert_eq!(h.state().panier, [("Pain".to_string(), 5)]);
    }

    #[test]
    fn un_article_inconnu_ouvre_la_fenetre_introuvable() {
        let mut h = harness_vente(app_vendeuse("introuvable"), FENETRE);
        scanner(&mut h, "zzz");
        assert_eq!(
            h.state().modale,
            Some(Modale::Introuvable("zzz".to_string()))
        );
        h.get_by_label(&introuvable_texte("zzz"));
        cliquer(&mut h, SCANNER_A_NOUVEAU);
        assert!(h.state().modale.is_none());
        scanner(&mut h, "zzz");
        cliquer(&mut h, FONCTION_RECHERCHER);
        assert_eq!(h.state().ecran, Ecran::Recherche);
        assert!(h.state().modale.is_none());
    }

    #[test]
    fn plusieurs_correspondances_ouvrent_la_recherche_avec_le_texte() {
        let mut app = app_vendeuse("plusieurs");
        let etat = app.etat.as_mut().unwrap();
        assert!(etat.ajouter_avec_prix("Laitue", "0", "1", ""));
        assert!(etat.entree("Laitue", "4"));
        let mut h = harness_vente(app, FENETRE);
        scanner(&mut h, "2 x lai");
        assert_eq!(h.state().ecran, Ecran::Recherche);
        assert_eq!(h.state().recherche_articles, "lai");
        assert!(h.state().panier.is_empty());
    }

    #[test]
    fn un_produit_epuise_ou_sans_prix_ne_s_ajoute_pas_et_dit_pourquoi() {
        let mut h = harness_vente(app_vendeuse_avec_blocages("bloques"), FENETRE);
        scanner(&mut h, "sel");
        assert!(h.state().panier.is_empty());
        let toast = h.state().toast.clone().expect("un message");
        assert_eq!(toast.texte, produit_non_vendable("Sel", EPUISE));
        scanner(&mut h, "thé");
        assert!(h.state().panier.is_empty());
        assert_eq!(
            h.state().toast.as_ref().unwrap().texte,
            produit_non_vendable("Thé", PRIX_MANQUANT)
        );
    }

    #[test]
    fn on_ne_peut_pas_ajouter_plus_que_le_stock() {
        let mut h = harness_vente(app_vendeuse("limite"), FENETRE);
        for saisie in ["11 x pain", "999 x pain"] {
            scanner(&mut h, saisie);
            assert!(h.state().panier.is_empty(), "{saisie}");
            assert_eq!(
                h.state().toast.as_ref().unwrap().texte,
                stock_limite("Pain", 10)
            );
        }
        // Tout le stock est déjà au ticket : une unité de plus est refusée.
        let mut h = harness_vente(acheteur("limite-2", &[("Savon", 10)]), FENETRE);
        scanner(&mut h, "savon");
        assert_eq!(h.state().panier, [("Savon".to_string(), 10)]);
        assert_eq!(
            h.state().toast.as_ref().unwrap().texte,
            stock_limite("Savon", 10)
        );
    }

    #[test]
    fn les_lignes_montrent_nom_detail_et_total() {
        let mut app = acheteur("lignes", &[("Pain", 2)]);
        let textes = textes_peints(&mut app, FENETRE, 1.0);
        assert!(peint(&textes, "Pain"));
        assert!(peint(&textes, &detail_de_ligne(2, "1,20\u{a0}€")));
        assert!(peint(&textes, "2,40\u{a0}€"));
        assert!(peint(&textes, &articles_au_ticket(2)));
    }

    /// Dans la maquette, le bloc « nom + détail » est centré en hauteur sur l'image de la ligne.
    #[test]
    fn le_nom_et_le_detail_d_une_ligne_sont_centres_sur_son_repere() {
        let mut app = acheteur("ligne-centree", &[("Pain", 2)]);
        let textes = textes_peints(&mut app, FENETRE, 1.0);
        let rect = |texte: &str| textes.iter().find(|t| t.texte == texte).unwrap().rect;
        let bloc_centre =
            (rect("Pain").top() + rect(&detail_de_ligne(2, "1,20\u{a0}€")).bottom()) / 2.0;
        let repere = rect("PA").center().y;
        assert!(
            (bloc_centre - repere).abs() <= 1.0,
            "bloc centré en {bloc_centre}, repère en {repere}"
        );
    }

    #[test]
    fn une_seule_ligne_est_choisie_et_montre_moins_et_plus() {
        let mut h = harness_vente(acheteur("choix", &[("Pain", 2), ("Savon", 1)]), FENETRE);
        assert!(
            h.query_by_role_and_label(Role::Button, &ajouter_une_unite("Pain"))
                .is_none()
        );
        h.get_by_label("Pain").click();
        h.run();
        assert_eq!(h.state().ligne_choisie.as_deref(), Some("Pain"));
        assert_eq!(est_selectionne_ligne(&h, "Pain"), Some(true));
        assert!(
            h.query_by_role_and_label(Role::Button, &ajouter_une_unite("Savon"))
                .is_none()
        );
        cliquer(&mut h, &ajouter_une_unite("Pain"));
        assert_eq!(h.state().panier[0], ("Pain".to_string(), 3));
        cliquer(&mut h, &retirer_une_unite("Pain"));
        cliquer(&mut h, &retirer_une_unite("Pain"));
        assert_eq!(h.state().panier[0], ("Pain".to_string(), 1));
        // À une unité, « − » est grisé : on supprime la ligne par « Supprimer l'article ».
        assert!(est_grise(&h, &retirer_une_unite("Pain")));
    }

    fn est_selectionne_ligne(h: &Harness<'static, Application>, nom: &str) -> Option<bool> {
        est_selectionne(h, &choisir_la_ligne(nom))
    }

    #[test]
    fn le_plus_est_grise_quand_tout_le_stock_est_au_ticket() {
        let mut app = acheteur("plein", &[("Savon", 10)]);
        app.ligne_choisie = Some("Savon".to_string());
        let h = harness_vente(app, FENETRE);
        assert!(est_grise(&h, &ajouter_une_unite("Savon")));
        assert!(!est_grise(&h, &retirer_une_unite("Savon")));
    }

    #[test]
    fn le_total_en_grand_la_tva_incluse_et_le_sous_total() {
        let mut app = acheteur("totaux", &[("Pain", 3), ("Savon", 1)]);
        let textes = textes_peints(&mut app, FENETRE, 1.0);
        for attendu in [LIBELLE_TOTAL_GRAND, LIBELLE_SOUS_TOTAL, "6,10\u{a0}€"] {
            assert!(peint(&textes, attendu), "« {attendu} » n'est pas peint");
        }
        // Le total en grand est un seul texte (« 6,10 € »), d'une ligne de 64 points : plus haut
        // que le sous-total, qui porte les mêmes mots.
        assert!(
            textes
                .iter()
                .any(|t| t.texte == "6,10\u{a0}€" && t.rect.height() > 64.0),
            "le total n'est pas peint en grand"
        );
        let h = harness_vente(app, FENETRE);
        h.get_by_label("TVA 5,5\u{a0}% incluse : 0,19\u{a0}€");
        h.get_by_label("TVA 20\u{a0}% incluse : 0,42\u{a0}€");
    }

    #[test]
    fn aux_etats_unis_la_taxe_s_ajoute_avant_le_total() {
        let mut etat = Etat::ouvrir(chemin_temp("taxe-us"))
            .unwrap()
            .avec_horloge(|| FIXE);
        assert!(etat.definir_reglages(Profil::EtatsUnis, Devise::Usd, "8"));
        assert!(etat.ajouter_avec_prix("Pain", "0", "1.20", ""));
        assert!(etat.entree("Pain", "10"));
        let mut app = Application::nouvelle(etat);
        app.panier = vec![("Pain".to_string(), 2)];
        let textes = textes_peints(&mut app, FENETRE, 1.0);
        // 2,40 $ hors taxe + 8 % (19,2 cents arrondis à 19) = 2,59 $.
        for attendu in [
            LIBELLE_SOUS_TOTAL,
            "$2.40",
            "Taxe 8\u{a0}%",
            "$0.19",
            "$2.59",
        ] {
            assert!(peint(&textes, attendu), "« {attendu} » n'est pas peint");
        }
    }

    #[test]
    fn les_boutons_de_paiement_sont_grises_sans_article_et_actifs_avec() {
        let mut h = harness_vente(app_vendeuse("paiement-grise"), FENETRE);
        for nom in [BOUTON_ESPECES, BOUTON_CARTE, BOUTON_AUTRE] {
            assert!(est_grise(&h, nom), "{nom} devrait être grisé");
        }
        scanner(&mut h, "pain");
        for nom in [BOUTON_ESPECES, BOUTON_CARTE, BOUTON_AUTRE] {
            assert!(!est_grise(&h, nom), "{nom} devrait être actif");
        }
        cliquer(&mut h, BOUTON_ESPECES);
        assert_eq!(h.state().modale, Some(Modale::Especes));
    }

    #[test]
    fn un_ticket_devenu_invalide_bloque_le_paiement_et_le_dit() {
        let h = harness_vente(acheteur("invalide", &[("Fantôme", 1)]), FENETRE);
        h.get_by_label(PANIER_INVALIDE);
        for nom in [BOUTON_ESPECES, BOUTON_CARTE, BOUTON_AUTRE] {
            assert!(est_grise(&h, nom), "{nom}");
        }
    }

    #[test]
    fn la_barre_de_fonctions_a_quatre_cases_avec_leur_raccourci() {
        let h = harness_vente(app_vendeuse("barre"), FENETRE);
        for nom in [
            fonction_nom(FONCTION_RECHERCHER, RACCOURCI_RECHERCHER),
            fonction_nom(FONCTION_SUPPRIMER, RACCOURCI_SUPPRIMER),
            fonction_nom(FONCTION_ANNULER, RACCOURCI_ANNULER),
            FONCTION_MENU.to_string(),
        ] {
            h.get_by_role_and_label(Role::Button, &nom);
        }
        assert!(est_grise(
            &h,
            &fonction_nom(FONCTION_SUPPRIMER, RACCOURCI_SUPPRIMER)
        ));
    }

    #[test]
    fn f2_ouvre_la_recherche_et_le_menu_ouvre_le_menu() {
        let mut h = harness_vente(app_vendeuse("f2"), FENETRE);
        h.key_press(Key::F2);
        h.run();
        assert_eq!(h.state().ecran, Ecran::Recherche);
        let mut h = harness_vente(app_vendeuse("menu-barre"), FENETRE);
        cliquer(&mut h, FONCTION_MENU);
        assert_eq!(h.state().modale, Some(Modale::Menu));
    }

    #[test]
    fn suppr_ne_supprime_un_article_que_si_le_champ_de_scan_est_vide() {
        let mut app = acheteur("suppr", &[("Pain", 2)]);
        app.ligne_choisie = Some("Pain".to_string());
        let mut h = harness_vente(app, FENETRE);
        saisir(&mut h, CHAMP_SCAN, "ab");
        h.key_press(Key::Delete);
        h.run();
        assert_eq!(h.state().panier, [("Pain".to_string(), 2)]);
        assert!(h.state().modale.is_none());
        h.state_mut().scan.clear();
        h.run();
        h.key_press(Key::Delete);
        h.run();
        assert_eq!(
            h.state().modale,
            Some(Modale::Confirmer(Confirmation::SupprimerArticle))
        );
    }

    #[test]
    fn echap_efface_d_abord_la_saisie_puis_propose_d_annuler_la_vente() {
        let mut h = harness_vente(acheteur("echap", &[("Pain", 2)]), FENETRE);
        saisir(&mut h, CHAMP_SCAN, "ab");
        h.key_press(Key::Escape);
        h.run();
        assert!(h.state().scan.is_empty());
        assert!(h.state().modale.is_none());
        h.key_press(Key::Escape);
        h.run();
        assert_eq!(
            h.state().modale,
            Some(Modale::Confirmer(Confirmation::AnnulerVente))
        );
        // Dans la fenêtre, Échap la ferme sans rien annuler.
        h.key_press(Key::Escape);
        h.run();
        assert!(h.state().modale.is_none());
        assert_eq!(h.state().panier, [("Pain".to_string(), 2)]);
    }

    #[test]
    fn annuler_une_vente_sans_article_le_dit() {
        let mut h = harness_vente(app_vendeuse("aucune-vente"), FENETRE);
        h.key_press(Key::Escape);
        h.run();
        assert!(h.state().modale.is_none());
        assert_eq!(h.state().toast.as_ref().unwrap().texte, TOAST_AUCUNE_VENTE);
    }

    #[test]
    fn annuler_la_vente_vide_le_ticket_apres_confirmation_et_n_ecrit_rien_au_journal() {
        let mut h = harness_vente(acheteur("annuler", &[("Pain", 2), ("Savon", 1)]), FENETRE);
        cliquer(&mut h, &fonction_nom(FONCTION_ANNULER, RACCOURCI_ANNULER));
        h.get_by_role_and_label(Role::Dialog, confirmation_titre(Confirmation::AnnulerVente));
        h.get_by_label(&confirmation_texte(Confirmation::AnnulerVente, 3, ""));
        cliquer(&mut h, NON_REVENIR);
        assert_eq!(h.state().panier.len(), 2);
        cliquer(&mut h, &fonction_nom(FONCTION_ANNULER, RACCOURCI_ANNULER));
        cliquer(&mut h, OUI_ANNULER);
        assert!(h.state().panier.is_empty());
        assert_eq!(nombre_de_pieces(&h), 0);
        assert_eq!(h.state().toast.as_ref().unwrap().texte, TOAST_VENTE_ANNULEE);
        h.get_by_label(TICKET_VIDE_TITRE);
    }

    #[test]
    fn supprimer_l_article_retire_seulement_la_ligne_choisie() {
        let mut h = harness_vente(acheteur("supprimer", &[("Pain", 2), ("Savon", 1)]), FENETRE);
        h.get_by_label("Savon").click();
        h.run();
        cliquer(
            &mut h,
            &fonction_nom(FONCTION_SUPPRIMER, RACCOURCI_SUPPRIMER),
        );
        h.get_by_label(&confirmation_texte(
            Confirmation::SupprimerArticle,
            3,
            "Savon",
        ));
        cliquer(&mut h, OUI_SUPPRIMER);
        assert_eq!(h.state().panier, [("Pain".to_string(), 2)]);
        assert!(h.state().ligne_choisie.is_none());
    }

    #[test]
    fn un_nom_de_produit_tres_long_ne_chevauche_pas_le_total() {
        let nom = "Huile d'arachide Lesieur bouteille plastique 1 litre (carton de douze) édition spéciale fêtes";
        let mut etat = etat_vendeur("long");
        assert!(etat.ajouter_avec_prix(nom, "0", "1", ""));
        assert!(etat.entree(nom, "5"));
        let mut app = Application::nouvelle(etat);
        app.panier = vec![(nom.to_string(), 1)];
        app.ligne_choisie = Some(nom.to_string());
        let h = harness_vente(app, FENETRE);
        let rect_nom = h.get_by_label(nom).rect();
        let total = h
            .get_all_by_label("1,00\u{a0}€")
            .find(|n| {
                (n.rect().center().y - rect_nom.center().y).abs() < 30.0
                    && n.rect().left() > rect_nom.left()
            })
            .expect("le total de la ligne")
            .rect();
        assert!(
            rect_nom.right() <= total.left(),
            "le nom ({rect_nom:?}) recouvre le total ({total:?})"
        );
    }

    #[test]
    fn en_fenetre_etroite_le_scan_et_les_paiements_sont_a_l_ecran() {
        let taille = Vec2::new(640.0, 2200.0);
        let h = harness_vente(acheteur("etroit", &[("Pain", 2)]), taille);
        let ecran = Rect::from_min_size(Pos2::ZERO, taille);
        for (role, nom) in [
            (Role::TextInput, CHAMP_SCAN),
            (Role::Button, BOUTON_ESPECES),
            (Role::Button, BOUTON_CARTE),
            (Role::Button, BOUTON_AUTRE),
        ] {
            let rect = h.get_by_role_and_label(role, nom).rect();
            assert!(ecran.contains_rect(rect), "« {nom} » : {rect:?}");
        }
    }

    #[test]
    fn tous_les_textes_peints_de_l_ecran_vente_atteignent_4_5() {
        let mut app = acheteur("contraste-vente", &[("Pain", 2), ("Savon", 1), ("Lait", 3)]);
        app.ligne_choisie = Some("Savon".to_string());
        let textes = textes_peints(&mut app, FENETRE, 1.0);
        for attendu in [
            CHAMP_SCAN,
            LIBELLE_TOTAL_GRAND,
            BOUTON_ESPECES,
            BOUTON_CARTE,
            BOUTON_AUTRE,
            FONCTION_RECHERCHER,
            RACCOURCI_RECHERCHER,
        ] {
            assert!(peint(&textes, attendu), "« {attendu} » n'est pas peint");
        }
        verifier_contrastes(&textes, "ticket rempli");

        // Ticket vide : les boutons grisés sont dispensés de contraste.
        app.panier.clear();
        app.ligne_choisie = None;
        let textes = sans_les_textes_grises(textes_peints(&mut app, FENETRE, 1.0));
        assert!(peint(&textes, TICKET_VIDE_TITRE));
        verifier_contrastes(&textes, "ticket vide");

        // Les fenêtres de confirmation et d'article introuvable.
        app.panier = vec![("Pain".to_string(), 2)];
        for modale in [
            Modale::Confirmer(Confirmation::AnnulerVente),
            Modale::Confirmer(Confirmation::SupprimerArticle),
            Modale::Introuvable("zzz".to_string()),
        ] {
            app.modale = Some(modale.clone());
            let textes = textes_peints(&mut app, FENETRE, 1.0);
            verifier_contrastes(&textes, &format!("{modale:?}"));
        }
    }

    // ---- Taille de fenêtre, zoom et clavier -------------------------------------------------------

    /// Un ticket de six lignes, comme dans la maquette.
    fn ticket_de_six_lignes(nom: &str) -> Application {
        let mut etat = etat_vendeur(nom);
        for produit in ["Riz", "Huile", "Sucre"] {
            assert!(etat.ajouter_avec_prix(produit, "0", "2", ""));
            assert!(etat.entree(produit, "10"));
        }
        let mut app = Application::nouvelle(etat);
        app.panier = ["Pain", "Savon", "Lait", "Riz", "Huile", "Sucre"]
            .iter()
            .enumerate()
            .map(|(i, n)| (n.to_string(), i as u32 % 3 + 1))
            .collect();
        app
    }

    /// Les côtes de `maquette/1-vente.html` à 1366 × 768, sans remise (étape B) : la carte du total
    /// fait 211 de haut, puis viennent Espèces (84) et Carte / Autre (72), à 12 d'écart.
    #[test]
    fn la_colonne_de_paiement_a_les_cotes_de_la_maquette() {
        let mut app = app_vendeuse("cotes-colonne");
        app.panier = vec![("Savon".to_string(), 1)]; // un seul taux de TVA : une seule ligne
        let h = harness_fenetre(app, FENETRE);
        let rect = |nom: &str| h.get_by_role_and_label(Role::Button, nom).rect();
        let (especes, carte, autre) =
            (rect(BOUTON_ESPECES), rect(BOUTON_CARTE), rect(BOUTON_AUTRE));
        assert_eq!(
            (especes.left(), especes.width(), especes.height()),
            (930.0, 420.0, 84.0)
        );
        assert_eq!(especes.top(), 295.0, "sous la carte du total de 211");
        assert_eq!((carte.top(), carte.height()), (391.0, 72.0));
        assert_eq!((carte.width(), autre.width()), (204.0, 204.0));
        assert_eq!(autre.left(), 1146.0);
    }

    #[test]
    fn a_1366_par_768_l_essentiel_de_la_vente_est_a_l_ecran() {
        let h = harness_vente(ticket_de_six_lignes("ecran-defaut"), FENETRE);
        let ecran = Rect::from_min_size(Pos2::ZERO, FENETRE);
        for (role, nom) in [
            (Role::TextInput, CHAMP_SCAN.to_string()),
            (Role::Button, BOUTON_ESPECES.to_string()),
            (Role::Button, BOUTON_CARTE.to_string()),
            (Role::Button, BOUTON_AUTRE.to_string()),
            (
                Role::Button,
                fonction_nom(FONCTION_RECHERCHER, RACCOURCI_RECHERCHER),
            ),
            (
                Role::Button,
                fonction_nom(FONCTION_SUPPRIMER, RACCOURCI_SUPPRIMER),
            ),
            (
                Role::Button,
                fonction_nom(FONCTION_ANNULER, RACCOURCI_ANNULER),
            ),
            (Role::Button, FONCTION_MENU.to_string()),
        ] {
            let rect = h.get_by_role_and_label(role, &nom).rect();
            assert!(
                ecran.contains_rect(rect),
                "« {nom} » est hors de l'écran : {rect:?} hors de {ecran:?}"
            );
        }
    }

    #[test]
    fn dans_une_fenetre_etroite_le_scan_passe_avant_le_ticket_et_les_paiements() {
        let taille = Vec2::new(640.0, 2400.0);
        let h = harness_vente(ticket_de_six_lignes("etroit-ordre"), taille);
        let scan = h.get_by_role_and_label(Role::TextInput, CHAMP_SCAN).rect();
        let premiere_ligne = h.get_by_label("Pain").rect();
        let especes = h.get_by_role_and_label(Role::Button, BOUTON_ESPECES).rect();
        let menu = h.get_by_role_and_label(Role::Button, FONCTION_MENU).rect();
        assert!(scan.top() < premiere_ligne.top());
        assert!(premiere_ligne.top() < especes.top());
        assert!(especes.top() < menu.top());
        let ecran = Rect::from_min_size(Pos2::ZERO, taille);
        assert!(ecran.contains_rect(menu), "{menu:?}");
    }

    #[test]
    fn dans_une_fenetre_basse_la_colonne_de_paiement_defile_et_les_paiements_restent_atteignables()
    {
        // La hauteur minimale : le total et les trois boutons ne tiennent plus à côté du ticket
        // (le dernier, « Autre », finit à 497 sur 420).
        let taille = Vec2::new(TAILLE_FENETRE[0], TAILLE_MINIMALE[1]);
        let mut h = harness_vente(ticket_de_six_lignes("fenetre-basse"), taille);
        let ecran = Rect::from_min_size(Pos2::ZERO, taille);
        let autre = |h: &Harness<'static, Application>| {
            h.get_by_role_and_label(Role::Button, BOUTON_AUTRE).rect()
        };
        // Hors de l'écran au départ : sinon ce test ne prouverait rien.
        assert!(!ecran.contains_rect(autre(&h)), "{:?}", autre(&h));
        h.get_by_role_and_label(Role::Button, BOUTON_AUTRE)
            .scroll_to_me();
        h.run();
        assert!(ecran.contains_rect(autre(&h)), "{:?}", autre(&h));
        cliquer(&mut h, BOUTON_AUTRE);
        assert!(matches!(
            h.state().modale,
            Some(Modale::Terminal {
                moyen: Moyen::Autre,
                ..
            })
        ));
    }

    #[test]
    fn la_barre_de_fonctions_passe_sur_deux_lignes_en_fenetre_etroite() {
        let taille = Vec2::new(520.0, 2400.0);
        let h = harness_vente(app_vendeuse("barre-etroite"), taille);
        let noms = [
            fonction_nom(FONCTION_RECHERCHER, RACCOURCI_RECHERCHER),
            fonction_nom(FONCTION_SUPPRIMER, RACCOURCI_SUPPRIMER),
            fonction_nom(FONCTION_ANNULER, RACCOURCI_ANNULER),
            FONCTION_MENU.to_string(),
        ];
        let rects: Vec<Rect> = noms
            .iter()
            .map(|n| h.get_by_role_and_label(Role::Button, n).rect())
            .collect();
        let ecran = Rect::from_min_size(Pos2::ZERO, taille);
        for (nom, rect) in noms.iter().zip(&rects) {
            assert!(ecran.contains_rect(*rect), "« {nom} » : {rect:?}");
            assert!(rect.height() >= 44.0, "« {nom} » trop basse : {rect:?}");
        }
        assert_eq!(
            rects[0].top(),
            rects[1].top(),
            "une première rangée de deux"
        );
        assert_eq!(rects[2].top(), rects[3].top(), "une seconde rangée de deux");
        assert!(rects[2].top() > rects[0].bottom());
    }

    #[test]
    fn a_200_pour_cent_la_vente_ne_deborde_pas_a_l_horizontale() {
        // Texte agrandi : la largeur disponible est divisée par deux, on passe en une colonne.
        let taille = Vec2::new(1366.0, 4000.0);
        let largeur = taille.x / 2.0;
        for (nom, mut app) in [
            ("avec ticket", ticket_de_six_lignes("zoom-ticket")),
            ("avec blocages", app_vendeuse_avec_blocages("zoom-blocages")),
        ] {
            let textes = textes_peints(&mut app, taille, 2.0);
            assert!(textes.len() > 8, "{nom} : trop peu de textes");
            for t in &textes {
                assert!(
                    t.rect.left() >= -0.5 && t.rect.right() <= largeur + 0.5,
                    "{nom} : « {} » déborde : {:?} (largeur {largeur})",
                    t.texte,
                    t.rect
                );
            }
        }
    }

    #[test]
    fn on_met_un_produit_au_ticket_puis_on_paie_sans_la_souris() {
        let mut h = harness_vente(app_vendeuse("clavier"), FENETRE);
        // Le champ de scan a déjà le focus : on tape le nom puis Entrée.
        h.get_by_label(CHAMP_SCAN).type_text("pain");
        h.run();
        h.key_press(Key::Enter);
        h.run();
        assert_eq!(h.state().panier, [("Pain".to_string(), 1)]);
        // On atteint « Carte » au clavier et on l'active avec la barre d'espace.
        h.get_by_role_and_label(Role::Button, BOUTON_CARTE).focus();
        h.run();
        h.key_press(Key::Space);
        h.run();
        assert!(matches!(
            h.state().modale,
            Some(Modale::Terminal {
                moyen: Moyen::Carte,
                refuse: false
            })
        ));
        // Puis « Paiement accepté » avec Entrée : la vente est enregistrée.
        h.get_by_role_and_label(Role::Button, BOUTON_ACCEPTE)
            .focus();
        h.run();
        h.key_press(Key::Enter);
        h.run();
        assert_eq!(nombre_de_pieces(&h), 1);
        assert!(matches!(
            h.state().modale,
            Some(Modale::VenteTerminee { .. })
        ));
    }
}
