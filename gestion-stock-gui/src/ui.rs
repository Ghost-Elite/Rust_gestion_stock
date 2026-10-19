//! Le dessin de la fenêtre avec egui. Aucune logique métier ici : tout passe par `Etat`.
//!
//! Règles d'accessibilité suivies partout : chaque champ est relié à son libellé
//! (`labelled_by`), chaque bouton porte son texte complet, et une alerte est toujours
//! écrite en toutes lettres (la couleur ne vient qu'en plus).
//!
//! Le style : un fond gris très clair, des cartes blanches aux coins arrondis, une seule
//! couleur d'accent, des puces pour les états, de l'espace. Les couleurs viennent toutes de
//! `palette`, où leur contraste est vérifié par des tests.

use crate::etat::{Etat, LigneProduit, Resume, Retour, chemin_complet};
use crate::palette::{
    ACCENT, ACCENT_DOUX, ACCENT_TEXTE, ALERTE_FOND, ALERTE_TEXTE, DANGER, FILET, FOND,
    SORTIE_TEXTE, SUCCES_FOND, SUCCES_TEXTE, SURFACE, TEXTE, TEXTE_DISCRET, couleur, ombre,
};
use crate::police::famille_disponible;
use crate::textes::*;
use eframe::egui::{
    self, Align, Button, CentralPanel, CornerRadius, FontId, Frame, Id, Layout, Margin, Panel,
    Response, RichText, ScrollArea, Sense, Stroke, TextEdit, Ui, Vec2,
};
use std::path::Path;

/// En dessous de cette largeur (en points, donc aussi quand le texte est agrandi), on passe
/// en une seule colonne : les commandes se placent sous la liste au lieu d'être à côté.
const LARGEUR_DEUX_COLONNES: f32 = 800.0;

#[derive(Default)]
pub struct Application {
    /// `None` : le fichier de données n'a pas pu être ouvert, on affiche l'écran d'erreur.
    pub etat: Option<Etat>,
    pub erreur_ouverture: Option<String>,
    /// Le fichier qui n'a pas pu être ouvert, montré sur l'écran d'erreur.
    pub chemin_erreur: Option<String>,
    pub recherche: String,
    pub alertes_seulement: bool,
    /// Nom du produit sélectionné dans la liste.
    pub selection: Option<String>,
    pub saisie_nom: String,
    pub saisie_seuil: String,
    pub saisie_quantite: String,
    pub saisie_nouveau_nom: String,
    pub confirmer_suppression: bool,
}

impl Application {
    pub fn nouvelle(etat: Etat) -> Application {
        Application {
            etat: Some(etat),
            ..Application::default()
        }
    }

    pub fn en_erreur(message: String, chemin: &Path) -> Application {
        Application {
            erreur_ouverture: Some(message),
            chemin_erreur: Some(chemin_complet(chemin)),
            ..Application::default()
        }
    }
}

impl eframe::App for Application {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        dessiner(ui, self);
    }
}

/// Dessine toute la fenêtre dans le `Ui` donné (panneaux compris) : on peut donc la tester
/// sans ouvrir de fenêtre.
pub fn dessiner(ui: &mut Ui, app: &mut Application) {
    if app.etat.is_none() {
        ecran_erreur_ouverture(ui, app);
        return;
    }
    abandonner_la_selection_perimee(app);
    Panel::top(Id::new("entete"))
        .frame(
            Frame::new()
                .fill(couleur(SURFACE))
                .inner_margin(Margin::symmetric(24, 16)),
        )
        .show(ui, entete);
    Panel::bottom(Id::new("pied_de_page"))
        .frame(
            Frame::new()
                .fill(couleur(SURFACE))
                .inner_margin(Margin::symmetric(24, 10)),
        )
        .show(ui, |ui| pied_de_page(ui, app));
    if ui.available_width() >= LARGEUR_DEUX_COLONNES {
        Panel::right(Id::new("commandes"))
            .frame(Frame::new().fill(couleur(FOND)).inner_margin(Margin {
                left: 4,
                right: 24,
                top: 20,
                bottom: 20,
            }))
            .show_separator_line(false)
            .default_size(350.0)
            .min_size(290.0)
            .show(ui, |ui| {
                ScrollArea::vertical().show(ui, |ui| contenu_commandes(ui, app));
            });
        CentralPanel::default()
            .frame(Frame::new().fill(couleur(FOND)).inner_margin(Margin {
                left: 24,
                right: 4,
                top: 20,
                bottom: 20,
            }))
            .show(ui, |ui| {
                ScrollArea::vertical()
                    .auto_shrink(false)
                    .show(ui, |ui| contenu_liste(ui, app));
            });
    } else {
        // Fenêtre étroite ou texte très agrandi : une seule colonne, rien n'est rogné.
        CentralPanel::default()
            .frame(
                Frame::new()
                    .fill(couleur(FOND))
                    .inner_margin(Margin::symmetric(16, 14)),
            )
            .show(ui, |ui| {
                ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
                    contenu_liste(ui, app);
                    ui.add_space(4.0);
                    contenu_commandes(ui, app);
                });
            });
    }
}

/// Si le produit sélectionné n'existe plus (supprimé, renommé), on n'agit jamais dessus.
fn abandonner_la_selection_perimee(app: &mut Application) {
    let Some(etat) = &app.etat else { return };
    let existe = app
        .selection
        .as_ref()
        .is_none_or(|nom| etat.lignes("", false).iter().any(|l| &l.nom == nom));
    if !existe {
        app.selection = None;
        app.confirmer_suppression = false;
    }
}

// ---- Petits éléments de style -----------------------------------------------------------------

/// Une carte blanche aux coins arrondis, avec une ombre légère.
fn carte<R>(ui: &mut Ui, contenu: impl FnOnce(&mut Ui) -> R) -> R {
    Frame::new()
        .fill(couleur(SURFACE))
        .stroke(Stroke::new(1.0, couleur(FILET)))
        .corner_radius(CornerRadius::same(14))
        .inner_margin(Margin::same(18))
        .shadow(ombre())
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            contenu(ui)
        })
        .inner
}

/// Le titre d'une carte : un vrai titre pour les lecteurs d'écran, plus petit que celui de la page.
fn titre_de_carte(ui: &mut Ui, texte: &str) {
    ui.heading(RichText::new(texte).size(18.0));
}

/// Un texte secondaire, plus petit et plus gris (mais toujours >= 4,5:1).
fn discret(ui: &mut Ui, texte: &str) {
    ui.label(
        RichText::new(texte)
            .size(13.5)
            .color(couleur(TEXTE_DISCRET)),
    );
}

/// Une puce arrondie : l'état est toujours écrit dans la puce, la couleur n'est qu'un renfort.
fn puce(ui: &mut Ui, texte: &str, fond: [u8; 3], texte_couleur: [u8; 3]) {
    Frame::new()
        .fill(couleur(fond))
        .corner_radius(CornerRadius::same(12))
        .inner_margin(Margin::symmetric(10, 3))
        .show(ui, |ui| {
            ui.label(
                RichText::new(texte)
                    .size(13.5)
                    .color(couleur(texte_couleur)),
            );
        });
}

/// Un texte important sur fond rouge clair. La couleur n'est jamais seule : le texte dit tout.
fn bandeau_alerte(ui: &mut Ui, texte: &str) {
    Frame::new()
        .fill(couleur(ALERTE_FOND))
        .corner_radius(CornerRadius::same(8))
        .inner_margin(Margin::symmetric(10, 6))
        .show(ui, |ui| {
            ui.label(RichText::new(texte).color(couleur(ALERTE_TEXTE)));
        });
}

/// Un champ de saisie relié à son libellé, pour les lecteurs d'écran.
fn champ(ui: &mut Ui, libelle: &str, valeur: &mut String) -> Response {
    let etiquette = ui.label(
        RichText::new(libelle)
            .size(13.5)
            .color(couleur(TEXTE_DISCRET)),
    );
    ui.add(
        TextEdit::singleline(valeur)
            .desired_width(f32::INFINITY)
            .margin(Margin::symmetric(10, 8)),
    )
    .labelled_by(etiquette.id)
}

#[derive(Clone, Copy)]
enum Style {
    /// Le bouton à mettre en avant : fond d'accent, texte blanc.
    Principal,
    /// Un bouton ordinaire : fond blanc, bordure, texte sombre.
    Secondaire,
    /// Une action qui supprime : texte et bordure rouges sur fond blanc.
    Danger,
    /// La confirmation d'une suppression : fond rouge, texte blanc.
    Confirmation,
}

/// Un bouton qui ne sert que si `actif` : sans cela, un double-clic enregistrerait l'action
/// une fois, puis afficherait « Erreur : quantité manquante » par-dessus le succès, et la
/// personne croirait que ça a échoué et recommencerait.
fn bouton(ui: &mut Ui, texte: &str, style: Style, actif: bool) -> Response {
    let bouton = match style {
        Style::Principal => Button::new(gras(ui, texte, 15.5).color(couleur(ACCENT_TEXTE)))
            .fill(couleur(ACCENT))
            .stroke(Stroke::NONE),
        Style::Secondaire => Button::new(texte),
        Style::Danger => Button::new(RichText::new(texte).color(couleur(ALERTE_TEXTE)))
            .fill(couleur(SURFACE))
            .stroke(Stroke::new(1.0, couleur(ALERTE_TEXTE))),
        Style::Confirmation => Button::new(gras(ui, texte, 15.5).color(couleur(ACCENT_TEXTE)))
            .fill(couleur(DANGER))
            .stroke(Stroke::NONE),
    };
    ui.add_enabled(
        actif,
        bouton
            .min_size(Vec2::new(0.0, 38.0))
            .corner_radius(CornerRadius::same(10)),
    )
}

fn rempli(texte: &str) -> bool {
    !texte.trim().is_empty()
}

// ---- Les zones de la fenêtre ------------------------------------------------------------------

fn entete(ui: &mut Ui) {
    ui.horizontal(|ui| {
        logo(ui);
        ui.vertical(|ui| {
            ui.heading(TITRE_FENETRE);
            discret(ui, SOUS_TITRE);
        });
    });
}

/// Un texte en semi-gras : la graisse, plus que la taille, fait ressortir ce qui compte.
fn gras(ui: &Ui, texte: &str, taille: f32) -> RichText {
    RichText::new(texte).font(FontId::new(taille, famille_disponible(ui.ctx())))
}

/// Un petit logo décoratif : un carré d'accent avec trois barres de stock. Il n'a pas de nom
/// pour les lecteurs d'écran : il n'apporte aucune information.
fn logo(ui: &mut Ui) {
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(46.0), Sense::hover());
    let peintre = ui.painter();
    peintre.rect_filled(rect, CornerRadius::same(13), couleur(ACCENT));
    for (i, hauteur) in [0.34, 0.54, 0.74].into_iter().enumerate() {
        let x = rect.left() + 11.0 + i as f32 * 8.5;
        let h = rect.height() * hauteur;
        let barre = egui::Rect::from_min_max(
            egui::pos2(x, rect.bottom() - 10.0 - h),
            egui::pos2(x + 5.5, rect.bottom() - 10.0),
        );
        peintre.rect_filled(barre, CornerRadius::same(3), couleur(ACCENT_TEXTE));
    }
}

fn pied_de_page(ui: &mut Ui, app: &Application) {
    let Some(etat) = &app.etat else { return };
    match &etat.retour {
        Some(Retour::Succes(message)) => {
            ui.label(message.as_str());
        }
        Some(Retour::Erreur(message)) => bandeau_alerte(ui, &format!("Erreur : {message}")),
        None => {}
    }
    discret(ui, &pied_de_page_donnees(&chemin_complet(etat.chemin())));
}

fn contenu_liste(ui: &mut Ui, app: &mut Application) {
    if let Some(etat) = &app.etat {
        chiffres(ui, etat.resume());
    }
    carte(ui, |ui| {
        titre_de_carte(ui, SECTION_LISTE);
        champ(ui, RECHERCHE, &mut app.recherche);
        ui.scope(|ui| {
            // Par défaut egui arrondit la case autant que les boutons : elle devient ronde et
            // ressemble à un bouton radio. Une case à cocher doit rester presque carrée.
            let v = ui.visuals_mut();
            for w in [
                &mut v.widgets.inactive,
                &mut v.widgets.hovered,
                &mut v.widgets.active,
            ] {
                w.corner_radius = CornerRadius::same(4);
            }
            ui.checkbox(&mut app.alertes_seulement, ALERTES_SEULEMENT);
        });
        ui.add_space(4.0);
        liste_produits(ui, app);
    });
    carte(ui, |ui| historique(ui, app));
}

/// Les trois chiffres du haut.
fn chiffres(ui: &mut Ui, resume: Resume) {
    ui.columns(3, |colonnes| {
        chiffre(
            &mut colonnes[0],
            CHIFFRE_PRODUITS,
            resume.produits.to_string(),
            false,
        );
        chiffre(
            &mut colonnes[1],
            CHIFFRE_ALERTES,
            resume.en_alerte.to_string(),
            resume.en_alerte > 0,
        );
        chiffre(
            &mut colonnes[2],
            CHIFFRE_UNITES,
            resume.unites.to_string(),
            false,
        );
    });
}

fn chiffre(ui: &mut Ui, libelle: &str, valeur: String, alerte: bool) {
    carte(ui, |ui| {
        discret(ui, libelle);
        let couleur_valeur = if alerte { ALERTE_TEXTE } else { TEXTE };
        ui.label(
            RichText::new(valeur)
                .size(30.0)
                .family(famille_disponible(ui.ctx()))
                .color(couleur(couleur_valeur)),
        );
    });
}

fn liste_produits(ui: &mut Ui, app: &mut Application) {
    let Some(etat) = &app.etat else { return };
    let lignes = etat.lignes(&app.recherche, app.alertes_seulement);
    if lignes.is_empty() {
        discret(ui, AUCUN_PRODUIT);
        return;
    }
    let mut clic: Option<String> = None;
    for ligne in &lignes {
        let choisi = app.selection.as_deref() == Some(ligne.nom.as_str());
        if ligne_produit(ui, ligne, choisi) {
            clic = Some(ligne.nom.clone());
        }
    }
    if let Some(nom) = clic {
        app.selection = Some(nom);
        app.saisie_nouveau_nom.clear();
        app.confirmer_suppression = false;
    }
}

/// Une ligne de la liste : le nom (qui se clique) avec son état dessous, la quantité à droite.
/// Un nom très long passe à la ligne au lieu de repousser l'alerte hors de l'écran.
/// Renvoie vrai si on vient de cliquer sur le nom.
fn ligne_produit(ui: &mut Ui, ligne: &LigneProduit, choisi: bool) -> bool {
    let (fond, contour) = if choisi {
        (ACCENT_DOUX, Stroke::new(2.0, couleur(ACCENT)))
    } else {
        (SURFACE, Stroke::new(1.0, couleur(FILET)))
    };
    let mut clic = false;
    Frame::new()
        .fill(couleur(fond))
        .stroke(contour)
        .corner_radius(CornerRadius::same(10))
        .inner_margin(Margin::symmetric(14, 10))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            // Une ligne compacte : le nom, sans l'épaisseur d'un bouton autour, aligné sur la puce.
            ui.spacing_mut().item_spacing.y = 4.0;
            ui.spacing_mut().button_padding = Vec2::new(3.0, 1.0);
            // Le cadre de la ligne montre déjà la sélection : pas de deuxième contour autour du
            // nom (sa couleur d'accent, elle, reste). Le focus clavier, lui, garde son contour :
            // egui le dessine avec le style « actif », que l'on ne touche pas ici.
            ui.visuals_mut().selection.stroke.width = 0.0;
            ui.visuals_mut().widgets.inactive.bg_stroke = Stroke::NONE;
            ui.horizontal(|ui| {
                let largeur_quantite = 76.0;
                let largeur_nom =
                    (ui.available_width() - largeur_quantite - ui.spacing().item_spacing.x)
                        .max(120.0);
                ui.allocate_ui_with_layout(
                    Vec2::new(largeur_nom, 0.0),
                    Layout::top_down(Align::Min),
                    |ui| {
                        ui.set_width(largeur_nom);
                        let nom = gras(ui, ligne.nom.as_str(), 16.5);
                        if ui.selectable_label(choisi, nom).clicked() {
                            clic = true;
                        }
                        ui.horizontal_wrapped(|ui| {
                            ui.add_space(3.0); // aligne la puce sur le texte du nom
                            if ligne.en_alerte {
                                puce(ui, ETAT_ALERTE, ALERTE_FOND, ALERTE_TEXTE);
                            } else {
                                puce(ui, ETAT_CORRECT, SUCCES_FOND, SUCCES_TEXTE);
                            }
                            discret(ui, &seuil_court(ligne.seuil));
                        });
                    },
                );
                ui.with_layout(Layout::top_down(Align::Max), |ui| {
                    ui.label(gras(ui, &ligne.quantite.to_string(), 24.0));
                    discret(ui, STOCK_UNITE);
                });
            });
        });
    clic
}

fn historique(ui: &mut Ui, app: &Application) {
    let Some(etat) = &app.etat else { return };
    titre_de_carte(ui, SECTION_HISTORIQUE);
    let lignes = etat.historique(20);
    if lignes.is_empty() {
        discret(ui, AUCUN_MOUVEMENT);
    }
    for ligne in lignes {
        // « Entrée +3 Riz » : le premier mot (le sens du mouvement) prend une couleur.
        let (mot, reste) = ligne.split_once(' ').unwrap_or((ligne.as_str(), ""));
        let couleur_mot = if mot == "Entrée" {
            SUCCES_TEXTE
        } else {
            SORTIE_TEXTE
        };
        ui.horizontal_wrapped(|ui| {
            ui.label(gras(ui, mot, 15.5).color(couleur(couleur_mot)));
            ui.label(reste);
        });
    }
}

/// Le produit choisi passe en premier : c'est là que l'on retire du stock (une vente) ou que
/// l'on en ajoute (une livraison). Derrière le formulaire d'ajout et ses suggestions, ces
/// boutons se retrouvaient hors de l'écran à la taille de fenêtre par défaut.
fn contenu_commandes(ui: &mut Ui, app: &mut Application) {
    if app.selection.is_some() {
        carte(ui, |ui| commandes_du_produit(ui, app));
        carte(ui, |ui| formulaire_ajout(ui, app));
    } else {
        carte(ui, |ui| formulaire_ajout(ui, app));
        carte(ui, |ui| commandes_du_produit(ui, app));
    }
}

/// Des produits courants à choisir d'un clic au lieu de les taper. Ils se filtrent pendant la
/// frappe ; en choisir un remplit le nom, et le seuil conseillé si aucun seuil n'est déjà tapé.
fn suggestions_de_produits(ui: &mut Ui, app: &mut Application) {
    let Some(etat) = &app.etat else { return };
    let suggestions = etat.suggestions(&app.saisie_nom, 6);
    if suggestions.is_empty() {
        return;
    }
    discret(ui, TITRE_SUGGESTIONS);
    let mut choix = None;
    ui.horizontal_wrapped(|ui| {
        for suggestion in &suggestions {
            let bouton = Button::new(RichText::new(suggestion.nom).size(14.0))
                .min_size(Vec2::new(0.0, 30.0))
                .corner_radius(CornerRadius::same(15));
            if ui.add(bouton).clicked() {
                choix = Some(**suggestion);
            }
        }
    });
    if let Some(suggestion) = choix {
        app.saisie_nom = suggestion.nom.to_string();
        if !rempli(&app.saisie_seuil) {
            app.saisie_seuil = suggestion.seuil.to_string();
        }
    }
}

fn formulaire_ajout(ui: &mut Ui, app: &mut Application) {
    titre_de_carte(ui, SECTION_AJOUT);
    champ(ui, CHAMP_NOM, &mut app.saisie_nom);
    suggestions_de_produits(ui, app);
    champ(ui, CHAMP_SEUIL, &mut app.saisie_seuil);
    if bouton(
        ui,
        BOUTON_AJOUTER,
        Style::Principal,
        rempli(&app.saisie_nom),
    )
    .clicked()
        && let Some(etat) = app.etat.as_mut()
        && etat.ajouter(&app.saisie_nom, &app.saisie_seuil)
    {
        app.saisie_nom.clear();
        app.saisie_seuil.clear();
    }
}

fn commandes_du_produit(ui: &mut Ui, app: &mut Application) {
    titre_de_carte(ui, SECTION_ACTIONS);
    let ligne = app.selection.as_ref().and_then(|nom| {
        let etat = app.etat.as_ref()?;
        etat.lignes("", false).into_iter().find(|l| &l.nom == nom)
    });
    let Some(ligne) = ligne else {
        discret(ui, AUCUNE_SELECTION);
        return;
    };

    ui.label(produit_selectionne(&ligne.nom, ligne.quantite));
    champ(ui, CHAMP_QUANTITE, &mut app.saisie_quantite);
    ui.horizontal_wrapped(|ui| {
        if bouton(
            ui,
            BOUTON_ENTREE,
            Style::Principal,
            rempli(&app.saisie_quantite),
        )
        .clicked()
            && let Some(etat) = app.etat.as_mut()
            && etat.entree(&ligne.nom, &app.saisie_quantite)
        {
            app.saisie_quantite.clear();
        }
        if bouton(
            ui,
            BOUTON_SORTIE,
            Style::Secondaire,
            rempli(&app.saisie_quantite),
        )
        .clicked()
            && let Some(etat) = app.etat.as_mut()
            && etat.sortie(&ligne.nom, &app.saisie_quantite)
        {
            app.saisie_quantite.clear();
        }
    });

    ui.separator();
    champ(ui, CHAMP_NOUVEAU_NOM, &mut app.saisie_nouveau_nom);
    if bouton(
        ui,
        BOUTON_RENOMMER,
        Style::Secondaire,
        rempli(&app.saisie_nouveau_nom),
    )
    .clicked()
    {
        let nouveau = app.saisie_nouveau_nom.clone();
        if let Some(etat) = app.etat.as_mut()
            && etat.renommer(&ligne.nom, &nouveau)
        {
            app.selection = Some(nouveau.trim().to_string());
            app.saisie_nouveau_nom.clear();
        }
    }

    ui.separator();
    suppression(ui, app, &ligne);
}

/// Supprimer demande toujours une confirmation : le premier clic ne supprime rien.
fn suppression(ui: &mut Ui, app: &mut Application, ligne: &LigneProduit) {
    if app.confirmer_suppression {
        ui.label(confirmation_suppression(&ligne.nom, ligne.quantite));
        ui.horizontal_wrapped(|ui| {
            if bouton(ui, BOUTON_CONFIRMER_SUPPRESSION, Style::Confirmation, true).clicked() {
                if let Some(etat) = app.etat.as_mut()
                    && etat.supprimer(&ligne.nom)
                {
                    app.selection = None;
                }
                app.confirmer_suppression = false;
            }
            if bouton(ui, BOUTON_ANNULER, Style::Secondaire, true).clicked() {
                app.confirmer_suppression = false;
            }
        });
    } else if bouton(ui, BOUTON_SUPPRIMER, Style::Danger, true).clicked() {
        app.confirmer_suppression = true;
    }
}

fn ecran_erreur_ouverture(ui: &mut Ui, app: &Application) {
    CentralPanel::default()
        .frame(
            Frame::new()
                .fill(couleur(FOND))
                .inner_margin(Margin::symmetric(24, 24)),
        )
        .show(ui, |ui| {
            carte(ui, |ui| {
                ui.heading(ERREUR_OUVERTURE_TITRE);
                if let Some(message) = &app.erreur_ouverture {
                    bandeau_alerte(ui, message);
                }
                if let Some(chemin) = &app.chemin_erreur {
                    ui.label(pied_de_page_donnees(chemin));
                }
                ui.label(ERREUR_OUVERTURE_AIDE);
                if bouton(ui, BOUTON_QUITTER, Style::Secondaire, true).clicked() {
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                }
            });
        });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::etat::Etat;
    use crate::palette::{self, Rgb, rapport_contraste};
    use eframe::egui::accesskit::Role;
    use eframe::egui::{Color32, Context, Pos2, RawInput, Rect, Shape, Vec2};
    use egui_kittest::{Harness, kittest::Queryable};
    use std::path::{Path, PathBuf};

    fn chemin_temp(nom: &str) -> PathBuf {
        let chemin = std::env::temp_dir().join(format!(
            "gestion-stock-gui-ui-{}-{nom}.txt",
            std::process::id()
        ));
        let _ = std::fs::remove_file(&chemin);
        chemin
    }

    /// « Riz 5kg » (seuil 5, stock 3, en alerte) et « Sucre » (seuil 0, stock 10, correct).
    fn app_avec_produits(nom: &str) -> Application {
        let mut etat = Etat::ouvrir(chemin_temp(nom)).unwrap();
        assert!(etat.ajouter("Riz 5kg", "5"));
        assert!(etat.entree("Riz 5kg", "3"));
        assert!(etat.ajouter("Sucre", "0"));
        assert!(etat.entree("Sucre", "10"));
        Application::nouvelle(etat)
    }

    /// Une grande fenêtre : avec les cartes et l'espace du design, une fenêtre de 800 x 600 ne
    /// montrerait pas tous les boutons, et ce qui est hors de l'écran n'existe pas pour le test.
    fn harness(app: Application) -> Harness<'static, Application> {
        let mut h = Harness::builder()
            .with_size(Vec2::new(1100.0, 900.0))
            .build_ui_state(|ui, app: &mut Application| dessiner(ui, app), app);
        h.run();
        h
    }

    /// Tape dans un champ comme le ferait une personne : d'abord le focus, puis le texte.
    fn saisir(h: &mut Harness<'static, Application>, libelle: &str, texte: &str) {
        h.get_by_label(libelle).focus();
        h.run();
        h.get_by_label(libelle).type_text(texte);
        h.run();
    }

    #[test]
    fn les_commandes_sans_selection_ont_un_nom_accessible() {
        let h = harness(app_avec_produits("sans-selection"));
        // On interroge le RÔLE autant que le nom : un champ sans `labelled_by` ne serait
        // trouvé que comme simple libellé, jamais comme champ de saisie.
        for nom in [RECHERCHE, CHAMP_NOM, CHAMP_SEUIL] {
            h.get_by_role_and_label(Role::TextInput, nom);
        }
        h.get_by_role_and_label(Role::CheckBox, ALERTES_SEULEMENT);
        h.get_by_role_and_label(Role::Button, BOUTON_AJOUTER);
        assert!(h.query_by_label(BOUTON_ENTREE).is_none());
        h.get_by_label(AUCUNE_SELECTION);
    }

    #[test]
    fn choisir_un_produit_affiche_ses_commandes() {
        let mut h = harness(app_avec_produits("selection"));
        h.get_by_label("Riz 5kg").click();
        h.run();
        assert_eq!(h.state().selection.as_deref(), Some("Riz 5kg"));
        for nom in [CHAMP_QUANTITE, CHAMP_NOUVEAU_NOM] {
            h.get_by_role_and_label(Role::TextInput, nom);
        }
        for nom in [
            BOUTON_ENTREE,
            BOUTON_SORTIE,
            BOUTON_RENOMMER,
            BOUTON_SUPPRIMER,
        ] {
            h.get_by_role_and_label(Role::Button, nom);
        }
        h.get_by_label(&produit_selectionne("Riz 5kg", 3));
    }

    #[test]
    fn ajouter_un_produit_par_l_interface() {
        let mut h = harness(app_avec_produits("ajout"));
        saisir(&mut h, CHAMP_NOM, "Café 250g");
        saisir(&mut h, CHAMP_SEUIL, "2");
        h.get_by_label(BOUTON_AJOUTER).click();
        h.run();
        let lignes = h.state().etat.as_ref().unwrap().lignes("café", false);
        assert_eq!((lignes[0].nom.as_str(), lignes[0].seuil), ("Café 250g", 2));
        assert!(h.state().saisie_nom.is_empty() && h.state().saisie_seuil.is_empty());
    }

    #[test]
    fn une_sortie_trop_grande_affiche_une_erreur_lisible_et_garde_la_saisie() {
        let mut h = harness(app_avec_produits("sortie-erreur"));
        h.get_by_label("Riz 5kg").click();
        h.run();
        saisir(&mut h, CHAMP_QUANTITE, "5");
        h.get_by_label(BOUTON_SORTIE).click();
        h.run();
        h.get_by_label_contains(
            "Erreur : Stock insuffisant pour « Riz 5kg » : 3 disponible(s), 5 demandé(s).",
        );
        assert_eq!(h.state().saisie_quantite, "5");
    }

    #[test]
    fn l_alerte_est_ecrite_et_pas_seulement_coloree() {
        let h = harness(app_avec_produits("alerte"));
        assert_eq!(h.get_all_by_label(ETAT_ALERTE).count(), 1); // Riz 5kg
        assert_eq!(h.get_all_by_label(ETAT_CORRECT).count(), 1); // Sucre
    }

    #[test]
    fn la_suppression_demande_une_confirmation() {
        let mut h = harness(app_avec_produits("suppression"));
        h.get_by_label("Riz 5kg").click();
        h.run();
        h.get_by_label(BOUTON_SUPPRIMER).click();
        h.run();
        h.get_by_label_contains("Supprimer « Riz 5kg » ? Il reste 3 en stock.");
        assert_eq!(h.state().etat.as_ref().unwrap().lignes("", false).len(), 2);
        h.get_by_label(BOUTON_ANNULER).click();
        h.run();
        assert_eq!(h.state().etat.as_ref().unwrap().lignes("", false).len(), 2);
        h.get_by_label(BOUTON_SUPPRIMER).click();
        h.run();
        h.get_by_label(BOUTON_CONFIRMER_SUPPRESSION).click();
        h.run();
        assert_eq!(h.state().etat.as_ref().unwrap().lignes("", false).len(), 1);
        assert_eq!(h.state().selection, None);
    }

    #[test]
    fn une_selection_disparue_est_abandonnee_sans_planter() {
        let mut app = app_avec_produits("fantome");
        app.selection = Some("Fantôme".into());
        let h = harness(app);
        assert_eq!(h.state().selection, None);
        h.get_by_label(AUCUNE_SELECTION);
    }

    #[test]
    fn la_recherche_qui_masque_le_produit_selectionne_ne_plante_pas() {
        let mut h = harness(app_avec_produits("recherche"));
        h.get_by_label("Riz 5kg").click();
        h.run();
        saisir(&mut h, RECHERCHE, "sucre");
        assert!(h.query_by_label("Riz 5kg").is_none());
        h.get_by_label(BOUTON_ENTREE); // la sélection (masquée mais existante) reste utilisable
    }

    #[test]
    fn l_ecran_d_erreur_d_ouverture_est_lisible() {
        let h = harness(Application::en_erreur(
            "Fichier de données corrompu à la ligne 2.".into(),
            Path::new("/données/boutique/stock.txt"),
        ));
        h.get_by_label(ERREUR_OUVERTURE_TITRE);
        h.get_by_label_contains("ligne 2");
        // Après un double-clic, rien d'autre ne dit de quel fichier il s'agit.
        h.get_by_label(&pied_de_page_donnees("/données/boutique/stock.txt"));
        h.get_by_label(ERREUR_OUVERTURE_AIDE);
        h.get_by_label(BOUTON_QUITTER);
    }

    #[test]
    fn le_zoom_au_clavier_est_actif() {
        assert!(eframe::egui::Context::default().options(|o| o.zoom_with_keyboard));
    }

    #[test]
    fn un_double_clic_sur_sortie_n_enregistre_qu_une_sortie_et_garde_le_message_de_succes() {
        let mut h = harness(app_avec_produits("double-clic"));
        h.get_by_label("Sucre").click();
        h.run();
        saisir(&mut h, CHAMP_QUANTITE, "2");
        h.get_by_label(BOUTON_SORTIE).click();
        h.run();
        h.get_by_label(BOUTON_SORTIE).click(); // le champ vient d'être vidé
        h.run();
        let etat = h.state().etat.as_ref().unwrap();
        assert_eq!(etat.lignes("sucre", false)[0].quantite, 8);
        assert!(
            matches!(etat.retour, Some(Retour::Succes(_))),
            "{:?}",
            etat.retour
        );
    }

    // ---- Ce qu'egui peint VRAIMENT, avec la vraie palette ----------------------------------

    /// Un texte tel qu'egui le peint, avec la couleur du fond qui est derrière lui.
    struct TextePeint {
        texte: String,
        couleur: Rgb,
        fond: Rgb,
        rect: Rect,
    }

    struct Releve {
        fonds: Vec<(Rect, Color32)>,
        fond_par_defaut: Color32,
        textes: Vec<TextePeint>,
    }

    fn rgb(c: Color32) -> Rgb {
        [c.r(), c.g(), c.b()]
    }

    /// Parcourt les formes dans l'ordre où egui les peint : un fond opaque peint avant un
    /// texte, et qui le contient, est le fond de ce texte.
    fn parcourir(forme: &Shape, releve: &mut Releve) {
        match forme {
            Shape::Vec(formes) => formes.iter().for_each(|f| parcourir(f, releve)),
            Shape::Rect(r) if r.fill.a() == 255 => releve.fonds.push((r.rect, r.fill)),
            Shape::Text(t) => {
                let mut couleur = t
                    .override_text_color
                    .or(t.galley.job.sections.first().map(|s| s.format.color))
                    .unwrap_or(t.fallback_color);
                if couleur == Color32::PLACEHOLDER {
                    couleur = t.fallback_color;
                }
                let rect = t.galley.rect.translate(t.pos.to_vec2());
                let fond = releve
                    .fonds
                    .iter()
                    .rev()
                    .find(|(r, _)| r.contains(rect.center()))
                    .map_or(releve.fond_par_defaut, |(_, c)| *c);
                if !t.galley.job.text.trim().is_empty() {
                    releve.textes.push(TextePeint {
                        texte: t.galley.job.text.clone(),
                        couleur: rgb(couleur),
                        fond: rgb(fond),
                        rect,
                    });
                }
            }
            _ => {}
        }
    }

    /// Dessine l'application avec la VRAIE palette (`palette::appliquer`) puis relève chaque
    /// texte peint. `taille` : la fenêtre ; `zoom` : le texte agrandi (Cmd/Ctrl +), qui
    /// réduit d'autant la place disponible, en points.
    fn textes_peints(app: &mut Application, taille: Vec2, zoom: f32) -> Vec<TextePeint> {
        let ctx = Context::default();
        palette::appliquer(&ctx);
        let entree = || RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, taille / zoom)),
            ..Default::default()
        };
        // Comme dans une vraie fenêtre, la mise en page se stabilise en quelques images.
        for _ in 0..4 {
            let _ = ctx.run_ui(entree(), |ui| dessiner(ui, app));
        }
        let sortie = ctx.run_ui(entree(), |ui| dessiner(ui, app));
        let mut releve = Releve {
            fonds: Vec::new(),
            fond_par_defaut: ctx.global_style().visuals.panel_fill,
            textes: Vec::new(),
        };
        for forme in &sortie.shapes {
            parcourir(&forme.shape, &mut releve);
        }
        releve.textes
    }

    /// Toutes les saisies remplies : aucun bouton n'est grisé (un contrôle inactif est
    /// dispensé de contraste), et un produit est sélectionné.
    fn app_toute_remplie(nom: &str) -> Application {
        let mut app = app_avec_produits(nom);
        app.selection = Some("Riz 5kg".into());
        app.saisie_nom = "Thé vert".into();
        app.saisie_seuil = "3".into();
        app.saisie_quantite = "2".into();
        app.saisie_nouveau_nom = "Riz basmati".into();
        app
    }

    #[test]
    fn tous_les_textes_peints_atteignent_4_5_sur_leur_fond() {
        for confirmation in [false, true] {
            let mut app = app_toute_remplie("contraste");
            app.confirmer_suppression = confirmation;
            let textes = textes_peints(&mut app, Vec2::new(960.0, 640.0), 1.0);
            assert!(textes.len() > 20, "trop peu de textes : {}", textes.len());
            for t in &textes {
                let rapport = rapport_contraste(t.couleur, t.fond);
                assert!(
                    rapport >= 4.5,
                    "« {} » : {rapport:.2}:1 ({:?} sur {:?})",
                    t.texte,
                    t.couleur,
                    t.fond
                );
            }
        }
    }

    #[test]
    fn l_alerte_reste_visible_avec_un_texte_agrandi_ou_une_petite_fenetre() {
        let cas = [
            // Assez hautes pour que seul un débordement HORIZONTAL fasse échouer le test : en
            // hauteur, on peut toujours faire défiler.
            (
                "texte agrandi à 200 %",
                Vec2::new(960.0, 1400.0),
                2.0,
                "Riz 5kg",
            ),
            (
                "petite fenêtre, nom très long",
                Vec2::new(640.0, 1000.0),
                1.0,
                "Huile d'arachide Lesieur bouteille plastique 1 litre (carton de 12)",
            ),
        ];
        for (nom_du_cas, taille, zoom, produit) in cas {
            let mut etat = Etat::ouvrir(chemin_temp("visible")).unwrap();
            assert!(etat.ajouter(produit, "5")); // stock 0 <= seuil 5 : en alerte
            let mut app = Application::nouvelle(etat);
            let ecran = Rect::from_min_size(Pos2::ZERO, taille / zoom);
            let textes = textes_peints(&mut app, taille, zoom);
            let alerte = textes
                .iter()
                .find(|t| t.texte == ETAT_ALERTE)
                .unwrap_or_else(|| {
                    let relevés: Vec<&str> = textes.iter().map(|t| t.texte.as_str()).collect();
                    panic!("{nom_du_cas} : « {ETAT_ALERTE} » n'est pas peint ; textes relevés : {relevés:?}")
                });
            assert!(
                ecran.contains_rect(alerte.rect),
                "{nom_du_cas} : l'alerte déborde de l'écran : {:?} hors de {ecran:?}",
                alerte.rect
            );
        }
    }

    // ---- Suggestions de produits : choisir au lieu de taper ----------------------------------

    #[test]
    fn des_suggestions_de_produits_sont_proposees_a_l_ajout_et_pas_celles_deja_en_stock() {
        let mut app = app_avec_produits("suggestions");
        // Déjà en stock, avec une autre casse : ne doit pas être proposé.
        assert!(app.etat.as_mut().unwrap().ajouter("SEL FIN 1KG", "6"));
        let h = harness(app);
        h.get_by_role_and_label(Role::Button, "Huile de tournesol 1 L");
        assert!(
            h.query_by_role_and_label(Role::Button, "Sel fin 1kg")
                .is_none()
        );
    }

    #[test]
    fn choisir_une_suggestion_remplit_le_nom_et_le_seuil_sans_rien_taper() {
        let mut h = harness(app_avec_produits("choix-suggestion"));
        h.get_by_role_and_label(Role::Button, "Huile de tournesol 1 L")
            .click();
        h.run();
        assert_eq!(h.state().saisie_nom, "Huile de tournesol 1 L");
        assert_eq!(h.state().saisie_seuil, "8");
        h.get_by_label(BOUTON_AJOUTER).click();
        h.run();
        let lignes = h.state().etat.as_ref().unwrap().lignes("tournesol", false);
        assert_eq!(
            (lignes[0].nom.as_str(), lignes[0].seuil),
            ("Huile de tournesol 1 L", 8)
        );
        assert!(h.state().saisie_nom.is_empty() && h.state().saisie_seuil.is_empty());
    }

    #[test]
    fn les_suggestions_suivent_la_frappe() {
        let mut h = harness(app_avec_produits("frappe-suggestion"));
        saisir(&mut h, CHAMP_NOM, "huile");
        h.get_by_role_and_label(Role::Button, "Huile de tournesol 1 L");
        h.get_by_role_and_label(Role::Button, "Huile d'arachide 1 L");
        assert!(
            h.query_by_role_and_label(Role::Button, "Sel fin 1kg")
                .is_none()
        );
    }

    #[test]
    fn choisir_une_suggestion_garde_un_seuil_deja_tape() {
        let mut h = harness(app_avec_produits("seuil-garde"));
        saisir(&mut h, CHAMP_SEUIL, "9");
        h.get_by_role_and_label(Role::Button, "Huile de tournesol 1 L")
            .click();
        h.run();
        assert_eq!(h.state().saisie_nom, "Huile de tournesol 1 L");
        assert_eq!(h.state().saisie_seuil, "9");
    }

    #[test]
    fn les_commandes_de_sortie_sont_visibles_sans_defiler_a_la_taille_par_defaut() {
        // La taille de la fenêtre au lancement (voir `main.rs`) : c'est ce que voit la personne.
        let taille = Vec2::new(960.0, 640.0);
        let mut h = Harness::builder().with_size(taille).build_ui_state(
            |ui, app: &mut Application| {
                // Le VRAI style : sans lui (polices, espacements), tout tiendrait à l'écran
                // et le test ne verrait pas ce que voit la personne.
                palette::appliquer(ui.ctx());
                dessiner(ui, app);
            },
            app_avec_produits("sortie-visible"),
        );
        h.run();
        h.get_by_label("Riz 5kg").click(); // la ligne doit elle-même être à l'écran
        h.run();
        let ecran = Rect::from_min_size(Pos2::ZERO, taille);
        for nom in [CHAMP_QUANTITE, BOUTON_ENTREE, BOUTON_SORTIE] {
            let rect = h.get_by_label(nom).rect();
            assert!(
                ecran.contains_rect(rect),
                "« {nom} » est hors de l'écran : {rect:?} hors de {ecran:?}"
            );
        }
    }
}
