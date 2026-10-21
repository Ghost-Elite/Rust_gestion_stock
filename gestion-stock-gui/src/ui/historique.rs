//! L'historique des tickets : les ventes d'une journée (ou, en tapant un numéro de ticket, celles
//! de n'importe quel jour), le détail du ticket choisi et son remboursement. Dimensions et
//! couleurs : celles de `maquette/5-historique.html`.

use super::composants::{
    LigneBouton, Style, bandeau_alerte, bouton_a_lignes, bouton_large, bouton_symbole, discret,
    modale, repere, zone,
};
use super::{Application, LARGEUR_CAISSE_DEUX_COLONNES, Modale};
use crate::etat::{DetailTicket, LigneHistorique, Retour, StatutPiece};
use crate::palette::{
    ACCENT_DOUX, ACCENT_TEXTE, AVERTISSEMENT_TEXTE, BORDURE, DANGER, FILET, FILET_LIGNE, FOND,
    GRISE_FOND, GRISE_TEXTE, Rgb, SUCCES_TEXTE, SURFACE, TEXTE, TEXTE_DISCRET, TEXTE_SECONDAIRE,
    couleur,
};
use crate::police::{Graisse, police_disponible};
use crate::textes::*;
use eframe::egui::{
    Align, CentralPanel, Context, CornerRadius, FontId, Frame, Id, Label, Layout, Margin, Panel,
    RichText, ScrollArea, Sense, Stroke, TextEdit, Ui, UiBuilder, Vec2, WidgetInfo, WidgetType,
};

const HAUTEUR_LIGNE: f32 = 56.0;
const LARGEUR_PANNEAU: f32 = 420.0;
const HAUTEUR_BLOC_ETROIT: f32 = 420.0;

/// Les tickets à montrer : ceux du jour, ou, si un numéro est tapé, ceux de tous les jours dont
/// le numéro le contient (on retrouve ainsi un ticket ancien).
pub(crate) fn filtrer<'a>(
    lignes: &'a [LigneHistorique],
    jour: &str,
    recherche: &str,
) -> Vec<&'a LigneHistorique> {
    let recherche = recherche.trim();
    if recherche.is_empty() {
        lignes.iter().filter(|l| l.jour == jour).collect()
    } else {
        lignes
            .iter()
            .filter(|l| l.ticket.contains(recherche))
            .collect()
    }
}

/// Le jour le plus proche qui en précède (ou en suit) un autre, parmi ceux de la liste.
pub(crate) fn jour_voisin(jours: &[String], courant: &str, avant: bool) -> Option<String> {
    if avant {
        jours.iter().filter(|j| j.as_str() < courant).max().cloned()
    } else {
        jours.iter().filter(|j| j.as_str() > courant).min().cloned()
    }
}

/// Tout ce que l'écran affiche, calculé avant de dessiner.
struct Vue {
    jour: String,
    aujourd_hui: String,
    /// Les jours où l'on peut aller : ceux qui ont des pièces, et aujourd'hui.
    jours: Vec<String>,
    visibles: Vec<LigneHistorique>,
    courant: Option<u64>,
    detail: Option<DetailTicket>,
    integrite: Result<(), String>,
    aucune_vente: bool,
}

fn calculer(app: &Application) -> Option<Vue> {
    let etat = app.etat.as_ref()?;
    let aujourd_hui = etat.jour_courant();
    let jour = app
        .historique
        .jour
        .clone()
        .unwrap_or_else(|| aujourd_hui.clone());
    let lignes = etat.historique_ventes();
    let mut jours = etat.jours_avec_pieces();
    if !jours.contains(&aujourd_hui) {
        jours.push(aujourd_hui.clone());
    }
    let visibles: Vec<LigneHistorique> = filtrer(&lignes, &jour, &app.historique.filtre)
        .into_iter()
        .cloned()
        .collect();
    let courant = app
        .historique
        .choisi
        .filter(|n| visibles.iter().any(|l| l.numero == *n))
        .or_else(|| visibles.first().map(|l| l.numero));
    Some(Vue {
        detail: courant.and_then(|n| etat.detail_ticket(n)),
        aucune_vente: lignes.is_empty(),
        integrite: etat.integrite(),
        jour,
        aujourd_hui,
        jours,
        visibles,
        courant,
    })
}

enum Action {
    Choisir(u64),
    Aller(String),
}

pub(super) fn ecran_historique(ui: &mut Ui, app: &mut Application) {
    let Some(vue) = calculer(app) else {
        return;
    };
    let mut action = None;
    let mut demande = None;
    if ui.available_width() >= LARGEUR_CAISSE_DEUX_COLONNES {
        Panel::right(Id::new("detail_du_ticket"))
            .frame(Frame::new().fill(couleur(FOND)).inner_margin(Margin {
                left: 8,
                right: 16,
                top: 16,
                bottom: 16,
            }))
            .exact_size(LARGEUR_PANNEAU + 8.0 + 16.0)
            .resizable(false)
            .show_separator_line(false)
            .show(ui, |ui| demande = panneau_detail(ui, &vue, None));
        CentralPanel::default()
            .frame(Frame::new().fill(couleur(FOND)).inner_margin(Margin {
                left: 16,
                right: 8,
                top: 16,
                bottom: 16,
            }))
            .show(ui, |ui| action = carte_liste(ui, app, &vue, None));
    } else {
        // Fenêtre étroite : la liste, puis le détail du ticket, dans une seule colonne.
        CentralPanel::default()
            .frame(
                Frame::new()
                    .fill(couleur(FOND))
                    .inner_margin(Margin::same(16)),
            )
            .show(ui, |ui| {
                ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
                    ui.spacing_mut().item_spacing.y = 12.0;
                    action = carte_liste(ui, app, &vue, Some(HAUTEUR_BLOC_ETROIT));
                    demande = panneau_detail(ui, &vue, Some(HAUTEUR_BLOC_ETROIT));
                });
            });
    }
    match action {
        Some(Action::Choisir(numero)) => app.historique.choisi = Some(numero),
        Some(Action::Aller(jour)) => {
            app.historique.jour = (jour != vue.aujourd_hui).then_some(jour);
            app.historique.choisi = None;
        }
        None => {}
    }
    if let Some(numero) = demande {
        app.historique.choisi = Some(numero);
        app.modale = Some(Modale::Rembourser(numero));
    }
}

// ---- La liste -----------------------------------------------------------------------------------

fn carte_liste(
    ui: &mut Ui,
    app: &mut Application,
    vue: &Vue,
    hauteur: Option<f32>,
) -> Option<Action> {
    let mut action = None;
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
            if let Some(a) = en_tete_de_liste(ui, app, vue) {
                action = Some(a);
            }
            if let Err(message) = &vue.integrite {
                Frame::new().inner_margin(Margin::same(14)).show(ui, |ui| {
                    bandeau_alerte(ui, &journal_altere(message));
                });
            }
            ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    if vue.visibles.is_empty() {
                        let texte = if vue.aucune_vente {
                            AUCUNE_VENTE
                        } else if app.historique.filtre.trim().is_empty() {
                            AUCUNE_VENTE_JOUR
                        } else {
                            AUCUN_TICKET
                        };
                        ui.add_space(32.0);
                        ui.vertical_centered(|ui| discret(ui, texte));
                    }
                    for ligne in &vue.visibles {
                        let choisie = vue.courant == Some(ligne.numero);
                        if ligne_de_ticket(ui, ligne, choisie) {
                            action = Some(Action::Choisir(ligne.numero));
                        }
                    }
                });
        });
    action
}

/// Le titre, le choix du jour, et le champ « N° de ticket ».
fn en_tete_de_liste(ui: &mut Ui, app: &mut Application, vue: &Vue) -> Option<Action> {
    let mut action = None;
    let cherche = !app.historique.filtre.trim().is_empty();
    let titre = if cherche {
        HISTORIQUE_TITRE_RECHERCHE.to_string()
    } else if vue.jour == vue.aujourd_hui {
        HISTORIQUE_TITRE.to_string()
    } else {
        historique_titre_jour(&vue.jour)
    };
    let zone_titre = Frame::new()
        .inner_margin(Margin::symmetric(20, 16))
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.x = 8.0;
            ui.horizontal(|ui| {
                ui.heading(RichText::new(titre).size(22.0));
                if !cherche {
                    let avant = jour_voisin(&vue.jours, &vue.jour, true);
                    let apres = jour_voisin(&vue.jours, &vue.jour, false);
                    if bouton_symbole(ui, "\u{2039}", JOUR_PRECEDENT, avant.is_some(), 44.0)
                        .clicked()
                        && let Some(jour) = avant
                    {
                        action = Some(Action::Aller(jour));
                    }
                    if bouton_symbole(ui, "\u{203a}", JOUR_SUIVANT, apres.is_some(), 44.0).clicked()
                        && let Some(jour) = apres
                    {
                        action = Some(Action::Aller(jour));
                    }
                }
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    Frame::new()
                        .stroke(Stroke::new(1.0, couleur(BORDURE)))
                        .corner_radius(CornerRadius::same(10))
                        .inner_margin(Margin::symmetric(14, 0))
                        .show(ui, |ui| {
                            zone(
                                ui,
                                Vec2::new(212.0, 46.0),
                                Layout::left_to_right(Align::Center),
                                |ui| {
                                    let champ = TextEdit::singleline(&mut app.historique.filtre)
                                        .hint_text(
                                            RichText::new(CHAMP_TICKET)
                                                .size(16.0)
                                                .color(couleur(TEXTE_DISCRET)),
                                        )
                                        .font(FontId::proportional(16.0))
                                        .frame(Frame::NONE)
                                        .desired_width(f32::INFINITY);
                                    let reponse = ui.add(champ);
                                    ui.ctx().accesskit_node_builder(reponse.id, |noeud| {
                                        noeud.set_label(CHAMP_TICKET);
                                    });
                                },
                            );
                        });
                });
            });
        })
        .response
        .rect;
    ui.painter().hline(
        zone_titre.x_range(),
        zone_titre.bottom(),
        Stroke::new(1.0, couleur(FILET)),
    );
    action
}

fn couleur_du_statut(statut: StatutPiece) -> Rgb {
    match statut {
        StatutPiece::Payee => SUCCES_TEXTE,
        StatutPiece::Remboursee | StatutPiece::Remboursement => AVERTISSEMENT_TEXTE,
    }
}

fn texte_de_colonne(ui: &mut Ui, texte: &str, taille: f32, graisse: Graisse, c: Rgb) {
    ui.add(
        Label::new(
            RichText::new(texte)
                .font(police_disponible(ui.ctx(), graisse, taille))
                .color(couleur(c)),
        )
        .truncate()
        .selectable(false),
    );
}

/// Une ligne de la liste : numéro, heure, moyen, total, statut. Toute la ligne se clique.
fn ligne_de_ticket(ui: &mut Ui, ligne: &LigneHistorique, choisie: bool) -> bool {
    let (rect, _) = ui.allocate_exact_size(
        Vec2::new(ui.available_width(), HAUTEUR_LIGNE),
        Sense::hover(),
    );
    let nom = nom_de_ligne(
        &ligne.ticket,
        &ligne.heure,
        nom_du_moyen(ligne.moyen),
        &ligne.total,
        ligne.statut,
    );
    let reponse = ui.interact(rect, ui.id().with(("ticket", ligne.numero)), Sense::click());
    reponse.widget_info(|| WidgetInfo::selected(WidgetType::Button, true, choisie, &nom));
    let fond = if choisie { ACCENT_DOUX } else { SURFACE };
    let peintre = ui.painter();
    peintre.rect_filled(rect, 0.0, couleur(fond));
    peintre.hline(
        rect.x_range(),
        rect.bottom() - 0.5,
        Stroke::new(1.0, couleur(FILET_LIGNE)),
    );
    super::composants::anneau_de_focus(ui, &reponse, 0);
    let interieur = rect.shrink2(Vec2::new(20.0, 0.0));
    let mut enfant = ui.new_child(
        UiBuilder::new()
            .max_rect(interieur)
            .layout(Layout::left_to_right(Align::Center)),
    );
    enfant.spacing_mut().item_spacing.x = 12.0;
    let flexible = (interieur.width() - 120.0 - 80.0 - 140.0 - 150.0 - 4.0 * 12.0).max(60.0);
    let ligne_de = Layout::left_to_right(Align::Center);
    zone(
        &mut enfant,
        Vec2::new(120.0, HAUTEUR_LIGNE),
        ligne_de,
        |ui| {
            texte_de_colonne(ui, &ligne.ticket, 16.0, Graisse::SemiGrasse, TEXTE);
        },
    );
    zone(
        &mut enfant,
        Vec2::new(80.0, HAUTEUR_LIGNE),
        ligne_de,
        |ui| {
            texte_de_colonne(ui, &ligne.heure, 16.0, Graisse::Normale, TEXTE_DISCRET);
        },
    );
    zone(
        &mut enfant,
        Vec2::new(flexible, HAUTEUR_LIGNE),
        ligne_de,
        |ui| {
            texte_de_colonne(
                ui,
                nom_du_moyen(ligne.moyen),
                16.0,
                Graisse::Normale,
                TEXTE_DISCRET,
            );
        },
    );
    let a_droite = Layout::right_to_left(Align::Center);
    zone(
        &mut enfant,
        Vec2::new(140.0, HAUTEUR_LIGNE),
        a_droite,
        |ui| {
            texte_de_colonne(ui, &ligne.total, 16.0, Graisse::SemiGrasse, TEXTE);
        },
    );
    zone(
        &mut enfant,
        Vec2::new(150.0, HAUTEUR_LIGNE),
        a_droite,
        |ui| {
            texte_de_colonne(
                ui,
                nom_du_statut(ligne.statut),
                14.0,
                Graisse::SemiGrasse,
                couleur_du_statut(ligne.statut),
            );
        },
    );
    reponse.clicked()
}

// ---- Le ticket choisi ---------------------------------------------------------------------------

/// Le détail du ticket choisi, avec le bouton « Rembourser ». Renvoie le numéro du ticket dont
/// on vient de demander le remboursement.
fn panneau_detail(ui: &mut Ui, vue: &Vue, hauteur: Option<f32>) -> Option<u64> {
    let mut demande = None;
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
            let (Some(detail), Some(numero)) = (&vue.detail, vue.courant) else {
                ui.add_space(32.0);
                ui.vertical_centered(|ui| discret(ui, AUCUN_TICKET_A_MONTRER));
                return;
            };
            let zone_titre = Frame::new()
                .inner_margin(Margin::symmetric(20, 18))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.heading(RichText::new(titre_du_ticket(&detail.ticket)).size(20.0));
                    ui.label(
                        RichText::new(meta_du_ticket(
                            &heure_du_ticket(&detail.quand),
                            nom_du_moyen(detail.moyen),
                            detail.articles,
                            detail.statut,
                        ))
                        .size(15.0)
                        .color(couleur(TEXTE_DISCRET)),
                    );
                })
                .response
                .rect;
            ui.painter().hline(
                zone_titre.x_range(),
                zone_titre.bottom(),
                Stroke::new(1.0, couleur(FILET)),
            );
            let reste = (ui.available_height() - 168.0).max(80.0);
            ScrollArea::vertical()
                .auto_shrink([false, false])
                .max_height(reste)
                .show(ui, |ui| {
                    Frame::new()
                        .inner_margin(Margin::symmetric(20, 8))
                        .show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            for ligne in &detail.lignes {
                                ligne_du_detail(ui, ligne.quantite, &ligne.nom, &ligne.total);
                            }
                        });
                });
            pied_du_detail(ui, detail, vue, numero, &mut demande);
        });
    demande
}

/// `14/11/2023 23:13` → `23:13`.
fn heure_du_ticket(quand: &str) -> String {
    quand.rsplit(' ').next().unwrap_or(quand).to_string()
}

fn ligne_du_detail(ui: &mut Ui, quantite: u32, nom: &str, total: &str) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 56.0), Sense::hover());
    ui.painter().hline(
        rect.x_range(),
        rect.bottom() - 0.5,
        Stroke::new(1.0, couleur(FILET_LIGNE)),
    );
    let mut enfant = ui.new_child(
        UiBuilder::new()
            .max_rect(rect.shrink2(Vec2::new(0.0, 8.0)))
            .layout(Layout::left_to_right(Align::Center)),
    );
    enfant.spacing_mut().item_spacing.x = 12.0;
    repere(&mut enfant, nom, 40.0);
    let largeur = (rect.width() - 40.0 - 100.0 - 2.0 * 12.0).max(60.0);
    zone(
        &mut enfant,
        Vec2::new(largeur, 40.0),
        Layout::left_to_right(Align::Center),
        |ui| {
            texte_de_colonne(
                ui,
                &format!("{quantite} × {nom}"),
                16.0,
                Graisse::Normale,
                TEXTE,
            );
        },
    );
    zone(
        &mut enfant,
        Vec2::new(100.0, 40.0),
        Layout::right_to_left(Align::Center),
        |ui| texte_de_colonne(ui, total, 16.0, Graisse::SemiGrasse, TEXTE),
    );
}

/// Le total et le bouton « Rembourser » (actif seulement pour un ticket payé, journal intact).
fn pied_du_detail(
    ui: &mut Ui,
    detail: &DetailTicket,
    vue: &Vue,
    numero: u64,
    demande: &mut Option<u64>,
) {
    let zone_total = Frame::new()
        .inner_margin(Margin::symmetric(20, 16))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(LIBELLE_TOTAL_TICKET)
                        .font(police_disponible(ui.ctx(), Graisse::SemiGrasse, 17.0))
                        .color(couleur(TEXTE_SECONDAIRE)),
                );
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.label(
                        RichText::new(&detail.total)
                            .font(police_disponible(ui.ctx(), Graisse::Grasse, 28.0))
                            .color(couleur(TEXTE)),
                    );
                });
            });
        })
        .response
        .rect;
    ui.painter().hline(
        zone_total.x_range(),
        zone_total.top(),
        Stroke::new(1.0, couleur(FILET)),
    );
    let possible = detail.statut == StatutPiece::Payee && vue.integrite.is_ok();
    Frame::new()
        .inner_margin(Margin {
            left: 20,
            right: 20,
            top: 0,
            bottom: 20,
        })
        .show(ui, |ui| {
            let (fond, texte) = if possible {
                (DANGER, ACCENT_TEXTE)
            } else {
                (GRISE_FOND, GRISE_TEXTE)
            };
            let reponse = bouton_a_lignes(
                ui,
                &rembourser_le_ticket(&detail.ticket),
                Vec2::new(ui.available_width(), 56.0),
                fond,
                Stroke::NONE,
                10,
                &[LigneBouton {
                    texte: BOUTON_REMBOURSER,
                    taille: 16.0,
                    graisse: Graisse::SemiGrasse,
                    couleur: texte,
                }],
                possible,
            );
            if reponse.clicked() {
                *demande = Some(numero);
            }
        });
}

// ---- La fenêtre de remboursement ----------------------------------------------------------------

/// « Rembourser le ticket … ? » : le premier clic ne change rien. Rembourser ajoute au journal
/// une écriture inverse (le ticket d'origine n'est pas touché) et remet le stock.
pub(super) fn fenetre_remboursement(ctx: &Context, app: &mut Application, numero: u64) {
    let Some(detail) = app.etat.as_ref().and_then(|e| e.detail_ticket(numero)) else {
        app.modale = None;
        return;
    };
    let erreur = match app.etat.as_ref().and_then(|e| e.retour.as_ref()) {
        Some(Retour::Erreur(message)) => Some(message.clone()),
        _ => None,
    };
    let (mut oui, mut non) = (false, false);
    let ferme = modale(
        ctx,
        "remboursement",
        460.0,
        &rembourser_titre(&detail.ticket),
        |ui| {
            ui.spacing_mut().item_spacing.y = 16.0;
            ui.heading(rembourser_titre(&detail.ticket));
            if let Some(message) = &erreur {
                bandeau_alerte(ui, &format!("Erreur : {message}"));
            }
            ui.label(
                RichText::new(rembourser_texte(&detail.total))
                    .size(16.0)
                    .color(couleur(TEXTE_SECONDAIRE)),
            );
            ui.columns(2, |colonnes| {
                let largeur = colonnes[0].available_width();
                let taille = Vec2::new(largeur, 56.0);
                non = bouton_large(
                    &mut colonnes[0],
                    BOUTON_ANNULER,
                    Style::Secondaire,
                    true,
                    taille,
                )
                .clicked();
                oui = bouton_large(
                    &mut colonnes[1],
                    BOUTON_REMBOURSER,
                    Style::Confirmation,
                    true,
                    taille,
                )
                .clicked();
            });
        },
    );
    if oui {
        if let Some(etat) = app.etat.as_mut()
            && etat.annuler(numero)
        {
            etat.effacer_ticket();
            app.modale = None;
        }
    } else if non || ferme {
        app.modale = None;
    }
}

#[cfg(test)]
mod tests {
    use super::super::essais::*;
    use super::super::*;
    use super::{filtrer, jour_voisin};
    use crate::etat::{LigneHistorique, StatutPiece};
    use eframe::egui::accesskit::Role;
    use egui_kittest::kittest::Queryable;
    use gestion_stock::vente::Moyen;

    fn ligne(numero: u64, jour: &str) -> LigneHistorique {
        LigneHistorique {
            numero,
            ticket: numero_de_ticket(numero),
            jour: jour.to_string(),
            heure: "10:00".to_string(),
            moyen: Moyen::Carte,
            total: "1,00\u{a0}€".to_string(),
            statut: StatutPiece::Payee,
            articles: 1,
        }
    }

    #[test]
    fn filtrer_montre_le_jour_ou_cherche_dans_tous_les_jours() {
        let lignes = [
            ligne(3, "2023-11-16"),
            ligne(2, "2023-11-14"),
            ligne(1, "2023-11-14"),
        ];
        let numeros = |l: Vec<&LigneHistorique>| l.iter().map(|x| x.numero).collect::<Vec<_>>();
        assert_eq!(numeros(filtrer(&lignes, "2023-11-14", "")), [2, 1]);
        assert!(filtrer(&lignes, "2023-11-15", "").is_empty());
        // Une recherche de numéro ignore le jour affiché : on retrouve un ancien ticket.
        assert_eq!(numeros(filtrer(&lignes, "2023-11-15", "0002")), [2]);
        assert_eq!(numeros(filtrer(&lignes, "2023-11-14", " 01-0003 ")), [3]);
        // Des espaces seuls ne sont pas une recherche.
        assert_eq!(numeros(filtrer(&lignes, "2023-11-14", "   ")), [2, 1]);
        assert!(filtrer(&lignes, "2023-11-14", "9999").is_empty());
    }

    #[test]
    fn le_jour_voisin_est_le_plus_proche_avec_des_pieces() {
        let jours: Vec<String> = ["2023-11-14", "2023-11-16"].map(String::from).to_vec();
        assert_eq!(
            jour_voisin(&jours, "2023-11-15", true).as_deref(),
            Some("2023-11-14")
        );
        assert_eq!(
            jour_voisin(&jours, "2023-11-15", false).as_deref(),
            Some("2023-11-16")
        );
        assert_eq!(jour_voisin(&jours, "2023-11-14", true), None);
        assert_eq!(jour_voisin(&jours, "2023-11-16", false), None);
        assert_eq!(jour_voisin(&[], "2023-11-16", true), None);
    }

    /// Le nom accessible de la ligne du ticket `numero`, tel que l'écran le donne.
    fn nom_ligne(h: &Harness<'static, Application>, numero: u64) -> String {
        let l = h
            .state()
            .etat
            .as_ref()
            .unwrap()
            .historique_ventes()
            .into_iter()
            .find(|l| l.numero == numero)
            .expect("le ticket");
        nom_de_ligne(
            &l.ticket,
            &l.heure,
            nom_du_moyen(l.moyen),
            &l.total,
            l.statut,
        )
    }

    fn choisir(h: &mut Harness<'static, Application>, numero: u64) {
        let nom = nom_ligne(h, numero);
        cliquer(h, &nom);
    }

    use egui_kittest::Harness;

    #[test]
    fn la_liste_montre_les_tickets_du_jour_avec_leur_statut() {
        let mut app = app_avec_ventes("liste");
        let textes = textes_peints(&mut app, FENETRE, 1.0);
        for attendu in [
            HISTORIQUE_TITRE,
            "01-0002",
            "01-0001",
            "23:13",
            "Carte",
            "Espèces",
            "2,50\u{a0}€",
            "3,60\u{a0}€",
            STATUT_PAYEE,
            BOUTON_REMBOURSER,
        ] {
            assert!(
                textes.iter().any(|t| t.texte == attendu),
                "« {attendu} » n'est pas peint"
            );
        }
        let h = harness_vente(app, FENETRE);
        // Le ticket le plus récent est choisi : son détail est à droite.
        assert_eq!(est_selectionne(&h, &nom_ligne(&h, 2)), Some(true));
        assert_eq!(est_selectionne(&h, &nom_ligne(&h, 1)), Some(false));
        h.get_by_label(&titre_du_ticket("01-0002"));
        h.get_by_label("1 × Savon");
        // La plus récente d'abord.
        assert!(
            h.get_by_role_and_label(Role::Button, &nom_ligne(&h, 2))
                .rect()
                .top()
                < h.get_by_role_and_label(Role::Button, &nom_ligne(&h, 1))
                    .rect()
                    .top()
        );
    }

    #[test]
    fn rembourser_demande_confirmation_puis_ajoute_une_ecriture_inverse() {
        let mut h = harness_vente(app_avec_ventes("remboursement"), FENETRE);
        choisir(&mut h, 1);
        h.get_by_label(&titre_du_ticket("01-0001"));
        cliquer(&mut h, &rembourser_le_ticket("01-0001"));
        h.get_by_role_and_label(Role::Dialog, &rembourser_titre("01-0001"));
        h.get_by_label(&rembourser_texte("3,60\u{a0}€"));
        assert_eq!(
            nombre_de_pieces(&h),
            2,
            "rien n'est remboursé avant la confirmation"
        );
        cliquer(&mut h, BOUTON_ANNULER);
        assert!(h.state().modale.is_none());
        assert_eq!(nombre_de_pieces(&h), 2);
        assert_eq!(quantite_en_stock(&h, "Pain"), 7);

        cliquer(&mut h, &rembourser_le_ticket("01-0001"));
        cliquer(&mut h, BOUTON_REMBOURSER);
        // Une écriture inverse s'ajoute, le stock est remis, la vente reste visible.
        assert_eq!(nombre_de_pieces(&h), 3);
        assert_eq!(quantite_en_stock(&h, "Pain"), 10);
        assert_eq!(
            h.state().toast.as_ref().unwrap().texte,
            "Ticket 01-0001 remboursé : stock remis."
        );
        let statuts: Vec<(u64, StatutPiece)> = h
            .state()
            .etat
            .as_ref()
            .unwrap()
            .historique_ventes()
            .iter()
            .map(|l| (l.numero, l.statut))
            .collect();
        assert_eq!(
            statuts,
            [
                (3, StatutPiece::Remboursement),
                (2, StatutPiece::Payee),
                (1, StatutPiece::Remboursee)
            ]
        );
        // Le ticket remboursé et l'écriture de remboursement n'ont plus de bouton actif ;
        // l'autre ticket reste remboursable.
        assert!(est_grise(&h, &rembourser_le_ticket("01-0001")));
        choisir(&mut h, 3);
        assert!(est_grise(&h, &rembourser_le_ticket("01-0003")));
        choisir(&mut h, 2);
        assert!(!est_grise(&h, &rembourser_le_ticket("01-0002")));
    }

    #[test]
    fn un_double_clic_sur_rembourser_ne_cree_qu_une_ecriture() {
        let mut h = harness_vente(app_avec_ventes("double-remboursement"), FENETRE);
        choisir(&mut h, 2);
        cliquer(&mut h, &rembourser_le_ticket("01-0002"));
        h.get_by_role_and_label(Role::Button, BOUTON_REMBOURSER)
            .click();
        h.run();
        // Le second clic tombe sur un bouton qui n'existe plus : rien ne se passe.
        if let Some(bouton) = h.query_by_role_and_label(Role::Button, BOUTON_REMBOURSER) {
            bouton.click();
            h.run();
        }
        assert_eq!(nombre_de_pieces(&h), 3);
        assert_eq!(quantite_en_stock(&h, "Savon"), 10);
    }

    #[test]
    fn un_journal_altere_grise_rembourser_et_le_dit() {
        let mut app = app_journal_altere("altere-historique");
        app.ecran = Ecran::Historique;
        let h = harness_vente(app, FENETRE);
        h.get_by_label_contains("Le journal de caisse a été modifié");
        h.get_by_label_contains("Les ventes et les annulations sont bloquées");
        assert!(est_grise(&h, &rembourser_le_ticket("01-0001")));
    }

    #[test]
    fn un_jour_sans_vente_le_dit_et_reste_navigable() {
        let (mut etat, horloge) = etat_sur_deux_jours("jour-vide");
        assert!(etat.encaisser(&[("Pain".to_string(), 2)], Moyen::Carte, ""));
        horloge.set(FIXE.utc + 86_400); // le lendemain : aucune vente aujourd'hui
        let mut app = Application::nouvelle(etat);
        app.ecran = Ecran::Historique;
        let mut h = harness_vente(app, FENETRE);
        h.get_by_label(HISTORIQUE_TITRE);
        h.get_by_label(AUCUNE_VENTE_JOUR);
        assert!(!est_grise(&h, JOUR_PRECEDENT));
        assert!(est_grise(&h, JOUR_SUIVANT));
        cliquer(&mut h, JOUR_PRECEDENT);
        h.get_by_label(&historique_titre_jour("2023-11-14"));
        h.get_by_role_and_label(Role::Button, &nom_ligne(&h, 1));
        assert!(!est_grise(&h, JOUR_SUIVANT));
        cliquer(&mut h, JOUR_SUIVANT);
        h.get_by_label(HISTORIQUE_TITRE);
    }

    #[test]
    fn un_ticket_ancien_reste_remboursable_par_son_numero() {
        let (mut etat, horloge) = etat_sur_deux_jours("ancien-ticket");
        assert!(etat.encaisser(&[("Pain".to_string(), 2)], Moyen::Carte, ""));
        horloge.set(FIXE.utc + 86_400);
        let mut app = Application::nouvelle(etat);
        app.ecran = Ecran::Historique;
        let mut h = harness_vente(app, FENETRE);
        assert!(h.query_by_label(&titre_du_ticket("01-0001")).is_none());
        saisir(&mut h, CHAMP_TICKET, "0001");
        h.get_by_label(&titre_du_ticket("01-0001"));
        cliquer(&mut h, &rembourser_le_ticket("01-0001"));
        cliquer(&mut h, BOUTON_REMBOURSER);
        assert_eq!(nombre_de_pieces(&h), 2);
        assert_eq!(quantite_en_stock(&h, "Pain"), 20);
        // Une recherche sans résultat le dit.
        saisir(&mut h, CHAMP_TICKET, "9");
        h.get_by_label(AUCUN_TICKET);
    }

    #[test]
    fn sans_aucune_vente_la_liste_le_dit() {
        let mut app = app_vendeuse("aucune-vente");
        app.ecran = Ecran::Historique;
        let h = harness_vente(app, FENETRE);
        h.get_by_label(AUCUNE_VENTE);
        // Pas de ticket à montrer : le bouton de remboursement n'existe pas.
        assert!(
            h.query_by_role_and_label(Role::Button, BOUTON_REMBOURSER)
                .is_none()
        );
    }

    #[test]
    fn tous_les_textes_peints_de_l_historique_atteignent_4_5() {
        // Deux ventes puis le remboursement de la première.
        let mut app = app_avec_ventes("contraste-historique");
        assert!(app.etat.as_mut().unwrap().annuler(1));
        app.etat.as_mut().unwrap().effacer_ticket();
        app.ecran = Ecran::Historique;
        app.historique.choisi = Some(1);
        let textes = sans_les_textes_grises(textes_peints(&mut app, FENETRE, 1.0));
        for attendu in [
            STATUT_PAYEE,
            STATUT_REMBOURSEE,
            STATUT_REMBOURSEMENT,
            "-3,60\u{a0}€",
        ] {
            assert!(
                textes.iter().any(|t| t.texte == attendu),
                "« {attendu} » n'est pas peint"
            );
        }
        verifier_contrastes(&textes, "historique");
        // Le ticket payé : le bouton rouge est actif ; puis la fenêtre de remboursement.
        app.historique.choisi = Some(2);
        let textes = textes_peints(&mut app, FENETRE, 1.0);
        verifier_contrastes(&textes, "ticket payé");
        app.modale = Some(Modale::Rembourser(2));
        let textes = textes_peints(&mut app, FENETRE, 1.0);
        verifier_contrastes(&textes, "remboursement");
        // Le journal altéré : le bandeau d'alerte.
        let mut app = app_journal_altere("contraste-historique-altere");
        app.ecran = Ecran::Historique;
        let textes = sans_les_textes_grises(textes_peints(&mut app, FENETRE, 1.0));
        assert!(
            textes
                .iter()
                .any(|t| t.texte.contains("Les ventes et les annulations"))
        );
        verifier_contrastes(&textes, "journal altéré");
    }

    #[test]
    fn l_historique_s_empile_en_fenetre_etroite_sans_rien_perdre() {
        use eframe::egui::{Pos2, Rect, Vec2};
        let mut app = app_avec_ventes("etroit-historique");
        app.ecran = Ecran::Historique;
        let taille = Vec2::new(640.0, 2000.0);
        let h = harness_vente(app, taille);
        let ecran = Rect::from_min_size(Pos2::ZERO, taille);
        for nom in [
            nom_ligne(&h, 1),
            nom_ligne(&h, 2),
            rembourser_le_ticket("01-0002"),
        ] {
            let rect = h.get_by_role_and_label(Role::Button, &nom).rect();
            assert!(ecran.contains_rect(rect), "« {nom} » : {rect:?}");
        }
    }
}
