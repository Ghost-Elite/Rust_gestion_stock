//! Le dessin de la fenêtre avec egui. Aucune logique métier ici : tout passe par `Etat`.
//!
//! Règles d'accessibilité suivies partout : chaque champ est relié à son libellé
//! (`labelled_by`), chaque bouton porte son texte complet, et une alerte est toujours
//! écrite en toutes lettres (la couleur ne vient qu'en plus).
//!
//! Le style : un fond gris très clair, des cartes blanches aux coins arrondis, une seule
//! couleur d'accent, des puces pour les états, de l'espace. Les couleurs viennent toutes de
//! `palette`, où leur contraste est vérifié par des tests.

use crate::etat::{Etat, LignePiece, LigneProduit, Reglement, Resume, Retour, chemin_complet};
use crate::palette::{
    ACCENT, ACCENT_DOUX, ACCENT_TEXTE, ALERTE_FOND, ALERTE_TEXTE, DANGER, FILET, FOCUS, FOND,
    SORTIE_TEXTE, SUCCES_FOND, SUCCES_TEXTE, SURFACE, TEXTE, TEXTE_DISCRET, couleur, ombre,
};
use crate::police::famille_disponible;
use crate::textes::*;
use eframe::egui::{
    self, Align, Button, CentralPanel, CornerRadius, CursorIcon, FontId, Frame, Id, Layout, Margin,
    Panel, Response, RichText, ScrollArea, Sense, Stroke, StrokeKind, TextEdit, Ui, Vec2,
    WidgetInfo, WidgetType,
};
use gestion_stock::erreur::ErreurStock;
use gestion_stock::monnaie::{Devise, Montant};
use gestion_stock::reglages::Profil;
use gestion_stock::vente::Moyen;
use gestion_stock::vente::{Devis, GenrePiece, GroupeTaux};
use std::path::Path;

/// La taille de la fenêtre au lancement et sa taille minimale, en points. Les tests les utilisent
/// aussi : ils vérifient que l'essentiel est à l'écran à la taille que voit vraiment la personne.
pub const TAILLE_FENETRE: [f32; 2] = [1100.0, 720.0];
pub const TAILLE_MINIMALE: [f32; 2] = [640.0, 420.0];

/// En dessous de cette largeur (en points, donc aussi quand le texte est agrandi), on passe
/// en une seule colonne : les commandes se placent sous la liste au lieu d'être à côté.
const LARGEUR_DEUX_COLONNES: f32 = 800.0;

/// Les sections de l'application.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Onglet {
    /// L'écran de caisse : c'est celui que l'on ouvre en premier, la vente est le geste courant.
    #[default]
    Vente,
    Stock,
    /// Le journal des ventes, où l'on annule une vente.
    Ventes,
    Reglages,
}

/// Les réglages en cours de modification : ils ne comptent qu'une fois enregistrés.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrouillonReglages {
    pub profil: Profil,
    pub devise: Devise,
    pub taux: String,
}

#[derive(Default)]
pub struct Application {
    pub onglet: Onglet,
    /// Le panier en cours : (nom du produit, quantité).
    pub panier: Vec<(String, u32)>,
    pub moyen: Moyen,
    /// Ce que le client a donné, tel que tapé (pour un paiement en espèces).
    pub saisie_recu: String,
    pub recherche_vente: String,
    /// La vente que l'on s'apprête à annuler (en attente de confirmation).
    pub annulation_en_attente: Option<u64>,
    /// `None` : on montre les réglages enregistrés.
    pub reglages_saisis: Option<BrouillonReglages>,
    /// Le prix et le taux du formulaire « Ajouter un produit ».
    pub saisie_prix: String,
    pub saisie_taux: String,
    /// Le prix et le taux du produit sélectionné, pré-remplis à la sélection.
    pub saisie_prix_produit: String,
    pub saisie_taux_produit: String,
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

    /// Une unité de plus de ce produit dans le panier.
    fn ajouter_au_panier(&mut self, nom: &str) {
        match self.panier.iter_mut().find(|(n, _)| n == nom) {
            Some((_, quantite)) => *quantite += 1,
            None => self.panier.push((nom.to_string(), 1)),
        }
    }

    /// Une unité de moins ; la ligne disparaît quand il n'en reste plus.
    fn retirer_du_panier_une_unite(&mut self, nom: &str) {
        if let Some((_, quantite)) = self.panier.iter_mut().find(|(n, _)| n == nom) {
            *quantite = quantite.saturating_sub(1);
        }
        self.panier.retain(|(_, quantite)| *quantite > 0);
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
                .inner_margin(Margin::symmetric(24, 12)),
        )
        .show(ui, |ui| entete(ui, app));
    Panel::bottom(Id::new("pied_de_page"))
        .frame(
            Frame::new()
                .fill(couleur(SURFACE))
                .inner_margin(Margin::symmetric(24, 10)),
        )
        .show(ui, |ui| pied_de_page(ui, app));
    match app.onglet {
        Onglet::Vente => ecran_vente(ui, app),
        Onglet::Stock => ecran_stock(ui, app),
        Onglet::Ventes => page(ui, 900.0, |ui| ecran_ventes(ui, app)),
        Onglet::Reglages => page(ui, 640.0, |ui| ecran_reglages(ui, app)),
    }
}

/// La page d'un écran « de lecture » (Ventes, Réglages) : une colonne centrée, qui défile.
fn page(ui: &mut Ui, largeur_max: f32, contenu: impl FnOnce(&mut Ui)) {
    let marge = if ui.available_width() >= LARGEUR_DEUX_COLONNES {
        Margin::symmetric(24, 16)
    } else {
        Margin::symmetric(16, 14)
    };
    CentralPanel::default()
        .frame(Frame::new().fill(couleur(FOND)).inner_margin(marge))
        .show(ui, |ui| {
            ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
                ui.set_max_width(largeur_max);
                contenu(ui);
            });
        });
}

fn ecran_stock(ui: &mut Ui, app: &mut Application) {
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
    carte_marge(ui, 18, contenu).inner
}

/// La même carte avec une marge au choix : le paiement, qui doit tenir à l'écran, en prend moins.
fn carte_marge<R>(
    ui: &mut Ui,
    marge: i8,
    contenu: impl FnOnce(&mut Ui) -> R,
) -> egui::InnerResponse<R> {
    Frame::new()
        .fill(couleur(SURFACE))
        .stroke(Stroke::new(1.0, couleur(FILET)))
        .corner_radius(CornerRadius::same(14))
        .inner_margin(Margin::same(marge))
        .shadow(ombre())
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            contenu(ui)
        })
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

/// Un champ de saisie sur la même ligne que son libellé, pour gagner de la hauteur. On peut
/// y ajouter quelque chose à droite : `reserve` est la largeur à lui laisser.
fn champ_en_ligne(
    ui: &mut Ui,
    libelle: &str,
    valeur: &mut String,
    reserve: f32,
    a_droite: impl FnOnce(&mut Ui),
) -> Response {
    ui.horizontal(|ui| {
        let etiquette = ui.label(
            RichText::new(libelle)
                .size(13.5)
                .color(couleur(TEXTE_DISCRET)),
        );
        let largeur = (ui.available_width() - reserve - ui.spacing().item_spacing.x).max(60.0);
        let champ = ui
            .add(
                TextEdit::singleline(valeur)
                    .desired_width(largeur)
                    .margin(Margin::symmetric(10, 8)),
            )
            .labelled_by(etiquette.id);
        a_droite(ui);
        champ
    })
    .inner
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

/// La largeur d'un onglet : la même pour tous, pour les aligner à droite sans surprise.
const LARGEUR_ONGLET: f32 = 104.0;

/// Largeur à partir de laquelle les onglets tiennent à droite du titre, sur la même ligne.
const LARGEUR_ENTETE_EN_LIGNE: f32 = 980.0;

fn entete(ui: &mut Ui, app: &mut Application) {
    let large = ui.available_width() >= LARGEUR_ENTETE_EN_LIGNE;
    ui.horizontal(|ui| {
        logo(ui);
        ui.vertical(|ui| {
            ui.heading(TITRE_FENETRE);
            discret(ui, SOUS_TITRE);
        });
        if large {
            // Les onglets à droite, sur la ligne du titre : de la hauteur gagnée pour la caisse.
            let place = ONGLETS.len() as f32 * (LARGEUR_ONGLET + ui.spacing().item_spacing.x);
            ui.add_space((ui.available_width() - place).max(0.0));
            onglets(ui, app);
        }
    });
    if !large {
        ui.add_space(6.0);
        ui.horizontal_wrapped(|ui| onglets(ui, app));
    }
}

const ONGLETS: [(Onglet, &str); 4] = [
    (Onglet::Vente, ONGLET_VENTE),
    (Onglet::Stock, ONGLET_STOCK),
    (Onglet::Ventes, ONGLET_VENTES),
    (Onglet::Reglages, ONGLET_REGLAGES),
];

/// Les sections de l'application. Chaque onglet est un bouton dont l'état « sélectionné » est
/// annoncé par les lecteurs d'écran ; la couleur n'est qu'un renfort.
fn onglets(ui: &mut Ui, app: &mut Application) {
    for (onglet, texte) in ONGLETS {
        if bouton_choix(ui, texte, app.onglet == onglet, LARGEUR_ONGLET).clicked() {
            app.onglet = onglet;
        }
    }
}

/// Un bouton « à bascule » : celui qui est choisi est plein et en accent, les autres sont
/// des boutons ordinaires.
fn bouton_choix(ui: &mut Ui, texte: &str, choisi: bool, largeur: f32) -> Response {
    let bouton = if choisi {
        Button::new(gras(ui, texte, 15.5).color(couleur(ACCENT_TEXTE)))
            .fill(couleur(ACCENT))
            .stroke(Stroke::NONE)
    } else {
        Button::new(texte)
    };
    ui.add(
        bouton
            .selected(choisi)
            .min_size(Vec2::new(largeur, 38.0))
            .corner_radius(CornerRadius::same(10)),
    )
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
        let prix = ligne
            .prix
            .map_or_else(|| SANS_PRIX.to_string(), |p| prix_court(&etat.formater(p)));
        if ligne_produit(ui, ligne, &prix, choisi) {
            clic = Some(ligne.nom.clone());
        }
    }
    if let Some(nom) = clic {
        // Le formulaire de prix part de ce que le produit a déjà : enregistrer sans rien
        // changer ne change rien, et changer le prix ne perd pas le taux.
        (app.saisie_prix_produit, app.saisie_taux_produit) = etat.saisies_du_prix(&nom);
        app.selection = Some(nom);
        app.saisie_nouveau_nom.clear();
        app.confirmer_suppression = false;
    }
}

/// Une ligne de la liste : le nom (qui se clique) avec son état dessous, la quantité à droite.
/// Un nom très long passe à la ligne au lieu de repousser l'alerte hors de l'écran.
/// Renvoie vrai si on vient de cliquer sur le nom.
fn ligne_produit(ui: &mut Ui, ligne: &LigneProduit, prix: &str, choisi: bool) -> bool {
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
                            discret(ui, prix);
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
    champ(ui, CHAMP_PRIX_AJOUT, &mut app.saisie_prix);
    champ(ui, CHAMP_TAUX_AJOUT, &mut app.saisie_taux);
    if bouton(
        ui,
        BOUTON_AJOUTER,
        Style::Principal,
        rempli(&app.saisie_nom),
    )
    .clicked()
        && let Some(etat) = app.etat.as_mut()
        && etat.ajouter_avec_prix(
            &app.saisie_nom,
            &app.saisie_seuil,
            &app.saisie_prix,
            &app.saisie_taux,
        )
    {
        app.saisie_nom.clear();
        app.saisie_seuil.clear();
        app.saisie_prix.clear();
        app.saisie_taux.clear();
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
    champ(ui, CHAMP_PRIX, &mut app.saisie_prix_produit);
    champ(ui, CHAMP_TAUX, &mut app.saisie_taux_produit);
    if !rempli(&app.saisie_prix_produit) {
        discret(ui, PRIX_VIDE_RETIRE);
    }
    if bouton(ui, BOUTON_ENREGISTRER_PRIX, Style::Secondaire, true).clicked()
        && let Some(etat) = app.etat.as_mut()
    {
        etat.definir_prix(
            &ligne.nom,
            &app.saisie_prix_produit,
            &app.saisie_taux_produit,
        );
    }

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

// ---- Le journal des ventes ----------------------------------------------------------------------

/// Combien de pièces on montre : les plus récentes. Le journal, lui, garde tout.
const PIECES_AFFICHEES: usize = 50;
/// La place réservée, à droite d'une ligne du journal, au total et au bouton d'annulation.
const LARGEUR_ACTIONS_VENTE: f32 = 280.0;

enum ActionVente {
    Demander(u64),
    Confirmer(u64),
    Garder,
}

fn ecran_ventes(ui: &mut Ui, app: &mut Application) {
    let Some(etat) = &app.etat else { return };
    if let Err(message) = etat.integrite() {
        bandeau_alerte(ui, &journal_altere(&message));
        ui.add_space(8.0);
    }
    carte(ui, |ui| liste_des_ventes(ui, app));
}

fn liste_des_ventes(ui: &mut Ui, app: &mut Application) {
    titre_de_carte(ui, SECTION_VENTES);
    let Some(etat) = &app.etat else { return };
    let pieces = etat.pieces_recentes(PIECES_AFFICHEES);
    if pieces.is_empty() {
        discret(ui, AUCUNE_VENTE);
        return;
    }
    let integre = etat.integrite().is_ok();
    let mut action = None;
    for (i, piece) in pieces.iter().enumerate() {
        if i > 0 {
            ui.separator();
        }
        let confirmation = app.annulation_en_attente == Some(piece.numero);
        if let Some(a) = ligne_de_piece(ui, piece, confirmation, integre) {
            action = Some(a);
        }
    }
    match action {
        Some(ActionVente::Demander(numero)) => app.annulation_en_attente = Some(numero),
        Some(ActionVente::Garder) => app.annulation_en_attente = None,
        Some(ActionVente::Confirmer(numero)) => {
            if let Some(etat) = app.etat.as_mut() {
                etat.annuler(numero);
            }
            app.annulation_en_attente = None;
        }
        None => {}
    }
}

/// Une pièce du journal. Annuler demande toujours une confirmation, comme supprimer un produit :
/// le premier clic ne change rien.
fn ligne_de_piece(
    ui: &mut Ui,
    piece: &LignePiece,
    confirmation: bool,
    integre: bool,
) -> Option<ActionVente> {
    let mut action = None;
    ui.horizontal(|ui| {
        let largeur =
            (ui.available_width() - LARGEUR_ACTIONS_VENTE - ui.spacing().item_spacing.x).max(120.0);
        ui.allocate_ui_with_layout(
            Vec2::new(largeur, 0.0),
            Layout::top_down(Align::Min),
            |ui| {
                ui.set_width(largeur);
                ui.label(gras(
                    ui,
                    &titre_de_piece(piece.numero, &piece.libelle),
                    16.0,
                ));
                discret(ui, &detail_de_piece(&piece.quand, &piece.moyen));
            },
        );
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if piece.annulable && !confirmation {
                let annuler = bouton(ui, BOUTON_ANNULER_VENTE, Style::Secondaire, integre);
                annuler.widget_info(|| {
                    WidgetInfo::labeled(WidgetType::Button, integre, annuler_la_vente(piece.numero))
                });
                if annuler.clicked() {
                    action = Some(ActionVente::Demander(piece.numero));
                }
            }
            ui.label(gras(ui, &piece.total, 17.0));
        });
    });
    if confirmation && piece.annulable {
        ui.label(confirmation_annulation(piece.numero, &piece.total));
        ui.horizontal_wrapped(|ui| {
            if bouton(
                ui,
                BOUTON_CONFIRMER_ANNULATION,
                Style::Confirmation,
                integre,
            )
            .clicked()
            {
                action = Some(ActionVente::Confirmer(piece.numero));
            }
            if bouton(ui, BOUTON_GARDER_VENTE, Style::Secondaire, true).clicked() {
                action = Some(ActionVente::Garder);
            }
        });
    }
    action
}

// ---- Les réglages --------------------------------------------------------------------------------

fn nom_du_profil(profil: Profil) -> &'static str {
    match profil {
        Profil::Europe => PROFIL_EUROPE,
        Profil::EtatsUnis => PROFIL_ETATS_UNIS,
    }
}

fn nom_de_la_devise(devise: Devise) -> &'static str {
    match devise {
        Devise::Eur => DEVISE_EUR,
        Devise::Usd => DEVISE_USD,
        Devise::Gbp => DEVISE_GBP,
        Devise::Xof => DEVISE_XOF,
    }
}

fn ecran_reglages(ui: &mut Ui, app: &mut Application) {
    carte(ui, |ui| formulaire_reglages(ui, app));
}

/// Les choix se font sur une copie (le brouillon) : rien ne compte avant « Enregistrer ». Une
/// fois des ventes enregistrées, la devise et le mode de prix sont grisés (voir `Etat`).
fn formulaire_reglages(ui: &mut Ui, app: &mut Application) {
    titre_de_carte(ui, SECTION_REGLAGES);
    ui.label(RAPPEL_REGLAGES);
    let Some(etat) = &app.etat else { return };
    let enregistres = etat.reglages();
    let verrouilles = etat.reglages_verrouilles();
    let depart = app
        .reglages_saisis
        .clone()
        .unwrap_or_else(|| BrouillonReglages {
            profil: enregistres.profil,
            devise: enregistres.devise,
            taux: enregistres
                .devise
                .formater_taux_brut(enregistres.taux_defaut_bp),
        });
    let mut brouillon = depart.clone();

    ui.add_space(4.0);
    discret(ui, LIBELLE_PROFIL);
    ui.add_enabled_ui(!verrouilles, |ui| {
        ui.horizontal_wrapped(|ui| {
            for profil in [Profil::Europe, Profil::EtatsUnis] {
                let choisi = brouillon.profil == profil;
                if bouton_choix(ui, nom_du_profil(profil), choisi, 200.0).clicked() {
                    brouillon.profil = profil;
                }
            }
        });
    });
    ui.label(match brouillon.profil {
        Profil::Europe => EXPLICATION_EUROPE,
        Profil::EtatsUnis => EXPLICATION_ETATS_UNIS,
    });

    ui.add_space(4.0);
    discret(ui, LIBELLE_DEVISE);
    ui.add_enabled_ui(!verrouilles, |ui| {
        ui.horizontal_wrapped(|ui| {
            for devise in [Devise::Eur, Devise::Usd, Devise::Gbp, Devise::Xof] {
                let choisie = brouillon.devise == devise;
                if bouton_choix(ui, nom_de_la_devise(devise), choisie, 120.0).clicked() {
                    brouillon.devise = devise;
                }
            }
        });
    });

    ui.add_space(4.0);
    champ(ui, CHAMP_TAUX_DEFAUT, &mut brouillon.taux);
    if verrouilles {
        discret(ui, REGLAGES_VERROUILLES);
    }
    ui.add_space(4.0);
    let enregistrer = bouton(ui, BOUTON_ENREGISTRER_REGLAGES, Style::Principal, true).clicked();
    discret(ui, MENTION_CAISSE_NON_CERTIFIEE);

    if enregistrer
        && let Some(etat) = app.etat.as_mut()
        && etat.definir_reglages(brouillon.profil, brouillon.devise, &brouillon.taux)
    {
        app.reglages_saisis = None;
        return;
    }
    // Un refus garde les choix faits : on ne retape pas tout pour corriger un taux.
    if brouillon != depart {
        app.reglages_saisis = Some(brouillon);
    }
}

// ---- L'écran de vente -------------------------------------------------------------------------

const LARGEUR_TUILE: f32 = 158.0;
const HAUTEUR_TUILE: f32 = 62.0;
const LARGEUR_PANIER: f32 = 440.0;
/// La place réservée aux boutons « − », quantité, « + » et « Retirer » d'une ligne de panier.
const LARGEUR_COMMANDES_LIGNE: f32 = 208.0;

/// Le devis du panier : `None` si le panier est vide.
type DevisPanier = Option<Result<Devis, ErreurStock>>;

fn nom_du_moyen(moyen: Moyen) -> &'static str {
    match moyen {
        Moyen::Especes => MOYEN_ESPECES,
        Moyen::Carte => MOYEN_CARTE,
        Moyen::Autre => MOYEN_AUTRE,
    }
}

fn devis_du_panier(app: &Application) -> DevisPanier {
    if app.panier.is_empty() {
        return None;
    }
    app.etat.as_ref().map(|etat| etat.devis(&app.panier))
}

/// Vrai juste après une vente : le ticket prend la place du panier jusqu'à « Nouvelle vente ».
fn ticket_affiche(app: &Application) -> bool {
    app.etat
        .as_ref()
        .and_then(|etat| etat.derniere_piece())
        .is_some_and(|piece| piece.genre == GenrePiece::Vente)
}

fn ecran_vente(ui: &mut Ui, app: &mut Application) {
    let devis = devis_du_panier(app);
    if ui.available_width() >= LARGEUR_DEUX_COLONNES {
        Panel::right(Id::new("panier"))
            .frame(Frame::new().fill(couleur(FOND)).inner_margin(Margin {
                left: 4,
                right: 24,
                top: 16,
                bottom: 16,
            }))
            .show_separator_line(false)
            .default_size(LARGEUR_PANIER)
            .min_size(330.0)
            .show(ui, |ui| colonne_panier(ui, app, &devis));
        CentralPanel::default()
            .frame(Frame::new().fill(couleur(FOND)).inner_margin(Margin {
                left: 24,
                right: 4,
                top: 16,
                bottom: 16,
            }))
            .show(ui, |ui| {
                ScrollArea::vertical()
                    .auto_shrink(false)
                    .show(ui, |ui| carte_produits(ui, app));
            });
    } else {
        // Une seule colonne : le panier (ou le ticket) passe avant les produits dès qu'il y a
        // quelque chose dedans, pour que « Encaisser » ne soit pas tout en bas d'une longue liste.
        CentralPanel::default()
            .frame(
                Frame::new()
                    .fill(couleur(FOND))
                    .inner_margin(Margin::symmetric(16, 14)),
            )
            .show(ui, |ui| {
                ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
                    if ticket_affiche(app) || !app.panier.is_empty() {
                        panier_empile(ui, app, &devis);
                        carte_produits(ui, app);
                    } else {
                        carte_produits(ui, app);
                        panier_empile(ui, app, &devis);
                    }
                });
            });
    }
}

/// La hauteur de la carte de paiement, mesurée à l'image précédente (en mémoire d'egui).
fn hauteur_du_paiement(ui: &Ui) -> f32 {
    ui.data(|donnees| donnees.get_temp::<f32>(Id::new("hauteur_du_paiement")))
        .unwrap_or(280.0)
}

/// Le bouton « Montant juste » se place à droite du champ si la carte est au moins aussi large.
const LARGEUR_MONTANT_JUSTE_A_COTE: f32 = 360.0;
const LARGEUR_BOUTON_JUSTE: f32 = 140.0;

/// La place minimale gardée aux lignes du panier pour épingler le paiement dessous.
const HAUTEUR_MIN_LIGNES: f32 = 130.0;

/// La colonne de droite. Quand il y a assez de hauteur, les lignes du panier défilent et le
/// paiement reste dessous, à l'écran, même avec un grand panier. Sinon (fenêtre basse), tout
/// défile ensemble : rien n'est jamais rogné. On épingle en réservant la hauteur du paiement
/// plutôt qu'en imbriquant un panneau egui : un panneau dans un panneau avale les clics.
fn colonne_panier(ui: &mut Ui, app: &mut Application, devis: &DevisPanier) {
    if ticket_affiche(app) {
        ScrollArea::vertical().show(ui, |ui| carte(ui, |ui| ticket(ui, app)));
        return;
    }
    let ecart = ui.spacing().item_spacing.y;
    let reserve = hauteur_du_paiement(ui) + ecart;
    let epingle = !app.panier.is_empty() && ui.available_height() >= reserve + HAUTEUR_MIN_LIGNES;
    if !epingle {
        ScrollArea::vertical().show(ui, |ui| panier_empile(ui, app, devis));
        return;
    }
    carte(ui, |ui| carte_panier(ui, app, devis, Some(reserve)));
    carte_paiement(ui, app, devis);
}

/// Le titre « Panier » avec « Vider le panier » à sa droite, puis les lignes. `reserve` : la
/// hauteur à laisser libre sous la carte (le paiement épinglé) ; les lignes défilent dans le
/// reste. `None` : les lignes prennent toute leur place, c'est la page qui défile.
fn carte_panier(ui: &mut Ui, app: &mut Application, devis: &DevisPanier, reserve: Option<f32>) {
    let mut vider = false;
    ui.horizontal(|ui| {
        titre_de_carte(ui, SECTION_PANIER);
        if !app.panier.is_empty() {
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                vider = bouton(ui, BOUTON_VIDER, Style::Secondaire, true).clicked();
            });
        }
    });
    if vider {
        app.panier.clear();
    }
    if app.panier.is_empty() {
        discret(ui, PANIER_VIDE);
        return;
    }
    match reserve {
        Some(reserve) => {
            // 18 : la marge basse de la carte, qui doit tenir elle aussi dans la hauteur.
            let hauteur = (ui.available_height() - reserve - 18.0).max(60.0);
            ScrollArea::vertical()
                .auto_shrink([false, true])
                .max_height(hauteur)
                .show(ui, |ui| lignes_du_panier(ui, app, devis));
        }
        None => lignes_du_panier(ui, app, devis),
    }
}

/// La carte de paiement, dont on mesure la hauteur pour la réserver à l'image suivante.
fn carte_paiement(ui: &mut Ui, app: &mut Application, devis: &DevisPanier) {
    let hauteur = carte_marge(ui, 14, |ui| paiement(ui, app, devis))
        .response
        .rect
        .height();
    if (hauteur - hauteur_du_paiement(ui)).abs() > 0.5 {
        ui.data_mut(|donnees| donnees.insert_temp(Id::new("hauteur_du_paiement"), hauteur));
        ui.ctx().request_repaint(); // la mise en page se corrige à l'image suivante
    }
}

fn panier_empile(ui: &mut Ui, app: &mut Application, devis: &DevisPanier) {
    if ticket_affiche(app) {
        carte(ui, |ui| ticket(ui, app));
        return;
    }
    carte(ui, |ui| carte_panier(ui, app, devis, None));
    if !app.panier.is_empty() {
        carte_paiement(ui, app, devis);
    }
}

// ---- Les produits à vendre ----------------------------------------------------------------------

fn carte_produits(ui: &mut Ui, app: &mut Application) {
    carte(ui, |ui| {
        titre_de_carte(ui, SECTION_PRODUITS_VENTE);
        champ(ui, RECHERCHE_VENTE, &mut app.recherche_vente);
        ui.add_space(6.0);
        tuiles(ui, app);
    });
}

/// Les produits en tuiles de même largeur, rangées selon la place disponible : un clic sur une
/// tuile met une unité au panier.
fn tuiles(ui: &mut Ui, app: &mut Application) {
    let Some(etat) = &app.etat else { return };
    let lignes = etat.lignes(&app.recherche_vente, false);
    if lignes.is_empty() {
        discret(ui, AUCUN_PRODUIT_VENDABLE);
        return;
    }
    let ecart = ui.spacing().item_spacing.x;
    let par_rangee = (((ui.available_width() + ecart) / (LARGEUR_TUILE + ecart)) as usize).max(1);
    let mut choix: Option<String> = None;
    for rangee in lignes.chunks(par_rangee) {
        ui.horizontal_top(|ui| {
            for ligne in rangee {
                let au_panier = app
                    .panier
                    .iter()
                    .find(|(nom, _)| *nom == ligne.nom)
                    .map_or(0, |(_, quantite)| *quantite);
                if tuile(ui, etat, ligne, au_panier) {
                    choix = Some(ligne.nom.clone());
                }
            }
        });
    }
    if let Some(nom) = choix {
        // Choisir un produit commence une nouvelle vente : le ticket précédent se ferme.
        if let Some(etat) = app.etat.as_mut() {
            etat.effacer_ticket();
        }
        app.ajouter_au_panier(&nom);
    }
}

/// Le dessin d'une tuile : un cadre arrondi, le nom du produit en semi-gras (à la ligne s'il est
/// long) et son prix dessous, alignés à gauche. On la dessine nous-mêmes : un bouton egui
/// centre mal un texte sur plusieurs lignes. Elle reste un vrai contrôle : focus clavier avec
/// son contour, survol, état grisé.
fn bouton_tuile(ui: &mut Ui, actif: bool, nom: &str, prix: &str) -> Response {
    let marge = Vec2::new(12.0, 9.0);
    let famille = famille_disponible(ui.ctx());
    let couleur_texte = couleur(if actif { TEXTE } else { TEXTE_DISCRET });
    let galerie_nom = ui.painter().layout(
        nom.to_string(),
        FontId::new(15.5, famille),
        couleur_texte,
        LARGEUR_TUILE - 2.0 * marge.x,
    );
    let galerie_prix = (!prix.is_empty()).then(|| {
        ui.painter()
            .layout_no_wrap(prix.to_string(), FontId::proportional(14.0), couleur_texte)
    });
    let hauteur_texte =
        galerie_nom.size().y + galerie_prix.as_ref().map_or(0.0, |g| g.size().y + 2.0);
    let taille = Vec2::new(
        LARGEUR_TUILE,
        (hauteur_texte + 2.0 * marge.y).max(HAUTEUR_TUILE),
    );
    let sense = if actif {
        Sense::click()
    } else {
        Sense::hover()
    };
    let (rect, reponse) = ui.allocate_exact_size(taille, sense);
    if ui.is_rect_visible(rect) {
        let visuels = ui.style().interact(&reponse);
        let (fond, contour) = if actif {
            (visuels.weak_bg_fill, visuels.bg_stroke)
        } else {
            (couleur(FOND), Stroke::new(1.0, couleur(FILET)))
        };
        let peintre = ui.painter();
        peintre.rect(
            rect,
            CornerRadius::same(12),
            fond,
            contour,
            StrokeKind::Inside,
        );
        if reponse.has_focus() {
            peintre.rect_stroke(
                rect,
                CornerRadius::same(12),
                Stroke::new(2.0, couleur(FOCUS)),
                StrokeKind::Inside,
            );
        }
        let mut position = rect.min + marge;
        let hauteur_nom = galerie_nom.size().y;
        peintre.galley(position, galerie_nom, couleur_texte);
        if let Some(galerie_prix) = galerie_prix {
            position.y += hauteur_nom + 2.0;
            peintre.galley(position, galerie_prix, couleur_texte);
        }
    }
    if actif {
        reponse.on_hover_cursor(CursorIcon::PointingHand)
    } else {
        reponse
    }
}

/// Une tuile de produit. Quand on ne peut pas vendre, la tuile est grisée ET la raison est
/// écrite dessous, hors du bouton : un bouton grisé est peu lisible, et la raison ne doit pas
/// dépendre de la couleur. Renvoie vrai si on vient de cliquer dessus.
fn tuile(ui: &mut Ui, etat: &Etat, ligne: &LigneProduit, au_panier: u32) -> bool {
    let raison = if ligne.prix.is_none() {
        Some(PRIX_MANQUANT)
    } else if ligne.quantite == 0 {
        Some(EPUISE)
    } else if ligne.quantite <= au_panier {
        Some(TOUT_AU_PANIER)
    } else {
        None
    };
    let prix = ligne.prix.map(|p| etat.formater(p)).unwrap_or_default();
    let mut clic = false;
    ui.vertical(|ui| {
        ui.set_min_width(LARGEUR_TUILE);
        ui.set_max_width(LARGEUR_TUILE);
        ui.spacing_mut().item_spacing.y = 4.0;
        let actif = raison.is_none();
        let reponse = bouton_tuile(ui, actif, &ligne.nom, &prix);
        // Le nom que l'on entend : l'action (ou la raison), le produit et le prix.
        let nom_accessible = match raison {
            None => ajouter_au_panier(&ligne.nom, &prix),
            Some(raison) => produit_non_vendable(&ligne.nom, raison),
        };
        reponse
            .widget_info(|| WidgetInfo::labeled(WidgetType::Button, actif, nom_accessible.clone()));
        clic = reponse.clicked();
        match raison {
            Some(TOUT_AU_PANIER) => puce(ui, TOUT_AU_PANIER, ACCENT_DOUX, ACCENT),
            Some(raison) => puce(ui, raison, ALERTE_FOND, ALERTE_TEXTE),
            None if ligne.en_alerte => {
                ui.label(
                    RichText::new(stock_bas(ligne.quantite))
                        .size(13.5)
                        .color(couleur(ALERTE_TEXTE)),
                );
            }
            None => discret(ui, &stock_restant(ligne.quantite)),
        }
    });
    clic
}

// ---- Le panier -------------------------------------------------------------------------------------

enum ActionPanier {
    Plus(String),
    Moins(String),
    Retirer(String),
}

fn lignes_du_panier(ui: &mut Ui, app: &mut Application, devis: &DevisPanier) {
    let Some(etat) = &app.etat else { return };
    if let Some(Err(erreur)) = devis {
        bandeau_alerte(ui, PANIER_INVALIDE);
        discret(ui, &erreur.to_string());
    }
    let produits = etat.lignes("", false);
    let mut action = None;
    ui.spacing_mut().item_spacing.y = 4.0; // des lignes serrées : on veut en voir plusieurs
    for (i, (nom, quantite)) in app.panier.iter().enumerate() {
        if i > 0 {
            ui.separator();
        }
        let stock = produits.iter().find(|p| &p.nom == nom).map(|p| p.quantite);
        let detail = match devis {
            Some(Ok(devis)) => devis.lignes.iter().find(|l| &l.nom == nom).and_then(|l| {
                let total = l.prix.multiplier(*quantite).ok()?;
                Some(ligne_de_panier(
                    &etat.formater(l.prix),
                    *quantite,
                    &etat.formater(total),
                ))
            }),
            _ => None,
        };
        if let Some(a) = ligne_du_panier(ui, nom, *quantite, stock, detail.as_deref()) {
            action = Some(a);
        }
    }
    match action {
        Some(ActionPanier::Plus(nom)) => app.ajouter_au_panier(&nom),
        Some(ActionPanier::Moins(nom)) => app.retirer_du_panier_une_unite(&nom),
        Some(ActionPanier::Retirer(nom)) => app.panier.retain(|(n, _)| *n != nom),
        None => {}
    }
}

/// Un petit bouton carré (« − », « + »). Son texte n'est qu'un symbole : on lui donne un nom
/// complet pour les lecteurs d'écran (« Retirer une unité de Pain »).
fn bouton_symbole(ui: &mut Ui, symbole: &str, nom_accessible: &str, actif: bool) -> Response {
    let reponse = ui.add_enabled(
        actif,
        Button::new(RichText::new(symbole).size(18.0))
            .min_size(Vec2::splat(38.0))
            .corner_radius(CornerRadius::same(10)),
    );
    reponse.widget_info(|| WidgetInfo::labeled(WidgetType::Button, actif, nom_accessible));
    reponse
}

fn ligne_du_panier(
    ui: &mut Ui,
    nom: &str,
    quantite: u32,
    stock: Option<u32>,
    detail: Option<&str>,
) -> Option<ActionPanier> {
    let mut action = None;
    ui.horizontal(|ui| {
        let largeur_nom =
            (ui.available_width() - LARGEUR_COMMANDES_LIGNE - ui.spacing().item_spacing.x)
                .max(90.0);
        ui.allocate_ui_with_layout(
            Vec2::new(largeur_nom, 0.0),
            Layout::top_down(Align::Min),
            |ui| {
                ui.set_width(largeur_nom);
                ui.label(gras(ui, nom, 16.0));
                if let Some(detail) = detail {
                    discret(ui, detail);
                }
            },
        );
        // De droite à gauche : « Retirer », « + », la quantité, « − ».
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let retirer = bouton(ui, BOUTON_RETIRER, Style::Secondaire, true);
            retirer.widget_info(|| {
                WidgetInfo::labeled(WidgetType::Button, true, retirer_du_panier(nom))
            });
            if retirer.clicked() {
                action = Some(ActionPanier::Retirer(nom.to_string()));
            }
            let place = stock.is_some_and(|stock| quantite < stock);
            if bouton_symbole(ui, "+", &ajouter_une_unite(nom), place).clicked() {
                action = Some(ActionPanier::Plus(nom.to_string()));
            }
            ui.label(gras(ui, &quantite.to_string(), 16.5));
            if bouton_symbole(ui, "\u{2212}", &retirer_une_unite(nom), true).clicked() {
                action = Some(ActionPanier::Moins(nom.to_string()));
            }
        });
    });
    action
}

/// Les lignes de totaux, avec « vrai » pour la ligne qui compte (le total à payer).
/// Europe : les prix sont TTC, on montre le total puis la TVA qu'il contient. États-Unis : les
/// prix sont hors taxe, on montre le sous-total, la taxe par taux, puis le total.
fn lignes_totaux(etat: &Etat, groupes: &[GroupeTaux], total: Montant) -> Vec<(String, bool)> {
    let reglages = etat.reglages();
    let (devise, profil) = (reglages.devise, reglages.profil);
    let ligne_total = (
        format!("{} : {}", libelle_total(profil), devise.formater(total)),
        true,
    );
    let taxes = groupes.iter().map(|groupe| {
        let libelle = libelle_taxe(profil, &devise.formater_taux(groupe.taux_bp));
        (
            format!("{libelle} : {}", devise.formater(groupe.taxe)),
            false,
        )
    });
    match profil {
        Profil::Europe => std::iter::once(ligne_total).chain(taxes).collect(),
        Profil::EtatsUnis => {
            let sous_total = Montant(groupes.iter().map(|g| g.base.0).sum());
            let ligne_sous_total = (
                format!("{LIBELLE_SOUS_TOTAL} : {}", devise.formater(sous_total)),
                false,
            );
            std::iter::once(ligne_sous_total)
                .chain(taxes)
                .chain(std::iter::once(ligne_total))
                .collect()
        }
    }
}

fn afficher_totaux(ui: &mut Ui, lignes: Vec<(String, bool)>) {
    for (texte, fort) in lignes {
        if fort {
            ui.label(gras(ui, &texte, 19.0));
        } else {
            ui.label(texte);
        }
    }
}

// ---- Le paiement ------------------------------------------------------------------------------------

fn paiement(ui: &mut Ui, app: &mut Application, devis: &DevisPanier) {
    ui.spacing_mut().item_spacing.y = 5.0;
    let Some(etat) = &app.etat else { return };
    let journal = etat.integrite();
    if let Err(message) = &journal {
        bandeau_alerte(ui, &journal_altere(message));
    }
    let valide = matches!(devis, Some(Ok(_))) && journal.is_ok();
    let total = match devis {
        Some(Ok(devis)) => devis.total,
        _ => Montant::ZERO,
    };
    if let Some(Ok(devis)) = devis {
        afficher_totaux(ui, lignes_totaux(etat, &devis.groupes, devis.total));
    }
    ui.horizontal_wrapped(|ui| {
        for moyen in [Moyen::Especes, Moyen::Carte, Moyen::Autre] {
            if bouton_choix(ui, nom_du_moyen(moyen), app.moyen == moyen, 90.0).clicked() {
                app.moyen = moyen;
            }
        }
    });
    let mut encaissable = valide;
    if app.moyen == Moyen::Especes {
        // « Montant juste » à droite du champ s'il y a la place, sinon dessous.
        let a_cote = ui.available_width() >= LARGEUR_MONTANT_JUSTE_A_COTE;
        let mut juste = false;
        champ_en_ligne(
            ui,
            CHAMP_RECU,
            &mut app.saisie_recu,
            if a_cote { LARGEUR_BOUTON_JUSTE } else { 0.0 },
            |ui| {
                if a_cote {
                    juste = bouton(ui, BOUTON_MONTANT_JUSTE, Style::Secondaire, valide).clicked();
                }
            },
        );
        if !a_cote {
            juste = bouton(ui, BOUTON_MONTANT_JUSTE, Style::Secondaire, valide).clicked();
        }
        if juste {
            app.saisie_recu = etat.reglages().devise.formater_brut(total);
        }
        let reglement = etat.reglement_especes(total, &app.saisie_recu);
        match reglement {
            Reglement::Rendu(rendu) => {
                ui.label(gras(ui, &rendu_de_monnaie(&etat.formater(rendu)), 17.0));
            }
            Reglement::Manque(manque) => bandeau_alerte(ui, &il_manque(&etat.formater(manque))),
            Reglement::Illisible => bandeau_alerte(ui, RECU_ILLISIBLE),
            Reglement::Attente => {}
        }
        encaissable = valide && matches!(reglement, Reglement::Rendu(_));
    }
    let encaisser = ui
        .with_layout(Layout::top_down_justified(Align::Center), |ui| {
            bouton(ui, BOUTON_ENCAISSER, Style::Principal, encaissable)
        })
        .inner;
    if encaisser.clicked()
        && let Some(etat) = app.etat.as_mut()
        && etat.encaisser(&app.panier, app.moyen, &app.saisie_recu)
    {
        app.panier.clear();
        app.saisie_recu.clear();
    }
}

// ---- Le ticket --------------------------------------------------------------------------------------

/// Le ticket de la vente qui vient d'être faite. Il garde les prix et les taux du moment de la
/// vente (ce sont ceux de la pièce du journal), pas ceux d'aujourd'hui.
fn ticket(ui: &mut Ui, app: &mut Application) {
    let Some(etat) = &app.etat else { return };
    let Some(piece) = etat.derniere_piece() else {
        return;
    };
    let devise = etat.reglages().devise;
    titre_de_carte(ui, &titre_ticket(piece.numero));
    discret(ui, &piece.horodatage.local().en_francais());
    ui.add_space(4.0);
    for ligne in &piece.lignes {
        let total = ligne
            .prix
            .multiplier(ligne.quantite)
            .map(|m| devise.formater(m))
            .unwrap_or_default();
        ui.label(ligne_de_ticket(ligne.quantite, &ligne.nom, &total));
    }
    ui.separator();
    afficher_totaux(
        ui,
        lignes_totaux(etat, &etat.detail_taxes(piece), piece.total),
    );
    ui.label(moyen_du_ticket(nom_du_moyen(piece.moyen)));
    if piece.moyen == Moyen::Especes {
        ui.label(recu_du_ticket(&devise.formater(piece.recu)));
        ui.label(gras(
            ui,
            &rendu_de_monnaie(&devise.formater(piece.rendu)),
            17.0,
        ));
    }
    ui.add_space(4.0);
    discret(ui, MENTION_NON_CERTIFIE);
    let nouvelle = bouton(ui, BOUTON_NOUVELLE_VENTE, Style::Principal, true).clicked();
    if nouvelle && let Some(etat) = app.etat.as_mut() {
        etat.effacer_ticket();
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
    use egui_kittest::{
        Harness,
        kittest::{NodeT, Queryable},
    };
    use gestion_stock::horodatage::Horodatage;
    use gestion_stock::monnaie::Devise;
    use gestion_stock::reglages::Profil;
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
        let mut app = Application::nouvelle(etat);
        app.onglet = Onglet::Stock; // l'écran testé ici est celui du stock
        app
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
            app.onglet = Onglet::Stock;
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
        // La taille de la fenêtre au lancement : c'est ce que voit la personne.
        let taille = Vec2::new(TAILLE_FENETRE[0], TAILLE_FENETRE[1]);
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

    // ---- L'écran de vente ----------------------------------------------------------------------

    const FIXE: Horodatage = Horodatage {
        utc: 1_700_000_000,
        decalage_minutes: 60,
    };

    /// Pain : 10 en stock à 1,20 € (TVA 5,5 %) ; Savon : 10 à 2,50 € et Lait : 10 à 0,90 € (TVA par
    /// défaut, 20 %). L'horloge est fixe : 14/11/2023 à 23 h 13 heure locale.
    fn etat_vendeur(nom: &str) -> Etat {
        let mut e = Etat::ouvrir(chemin_temp(nom))
            .unwrap()
            .avec_horloge(|| FIXE);
        assert!(e.ajouter_avec_prix("Pain", "0", "1,20", "5,5"));
        assert!(e.entree("Pain", "10"));
        assert!(e.ajouter_avec_prix("Savon", "0", "2,50", ""));
        assert!(e.entree("Savon", "10"));
        assert!(e.ajouter_avec_prix("Lait", "0", "0,90", ""));
        assert!(e.entree("Lait", "10"));
        e
    }

    /// La boutique d'exemple, sans produit bloqué : tous les boutons peuvent être actifs.
    fn app_vendeuse(nom: &str) -> Application {
        Application::nouvelle(etat_vendeur(nom))
    }

    /// La même, avec un produit épuisé (« Sel ») et un produit sans prix (« Thé »).
    fn app_vendeuse_avec_blocages(nom: &str) -> Application {
        let mut etat = etat_vendeur(nom);
        assert!(etat.ajouter_avec_prix("Sel", "0", "0,80", "")); // stock 0 : épuisé
        assert!(etat.ajouter("Thé", "0"));
        assert!(etat.entree("Thé", "5")); // pas de prix
        Application::nouvelle(etat)
    }

    /// Avec le VRAI style (polices, espacements) : ce que voit la personne.
    fn harness_vente(app: Application, taille: Vec2) -> Harness<'static, Application> {
        let mut h = Harness::builder().with_size(taille).build_ui_state(
            |ui, app: &mut Application| {
                palette::appliquer(ui.ctx());
                dessiner(ui, app);
            },
            app,
        );
        h.run();
        h
    }

    const FENETRE: Vec2 = Vec2::new(TAILLE_FENETRE[0], TAILLE_FENETRE[1]);

    fn tuile(nom: &str, prix: &str) -> String {
        ajouter_au_panier(nom, prix)
    }

    fn cliquer(h: &mut Harness<'static, Application>, libelle: &str) {
        h.get_by_role_and_label(Role::Button, libelle).click();
        h.run();
    }

    fn est_grise(h: &Harness<'static, Application>, libelle: &str) -> bool {
        h.get_by_role_and_label(Role::Button, libelle)
            .accesskit_node()
            .is_disabled()
    }

    fn nombre_de_pieces(h: &Harness<'static, Application>) -> usize {
        h.state().etat.as_ref().unwrap().pieces_recentes(100).len()
    }

    #[test]
    fn l_application_s_ouvre_sur_la_vente() {
        let app = app_vendeuse("ouverture");
        assert_eq!(app.onglet, Onglet::Vente);
    }

    #[test]
    fn les_onglets_sont_annonces_avec_leur_nom_et_leur_role() {
        let mut h = harness_vente(app_vendeuse("onglets"), FENETRE);
        let selectionne = |h: &Harness<'static, Application>, nom: &str| {
            h.get_by_role_and_label(Role::Button, nom)
                .accesskit_node()
                .toggled()
        };
        // Le choix est dit par l'état « sélectionné » et pas seulement par la couleur.
        assert_eq!(
            selectionne(&h, ONGLET_VENTE),
            Some(eframe::egui::accesskit::Toggled::True)
        );
        assert_eq!(
            selectionne(&h, ONGLET_STOCK),
            Some(eframe::egui::accesskit::Toggled::False)
        );
        cliquer(&mut h, ONGLET_STOCK);
        assert_eq!(h.state().onglet, Onglet::Stock);
        h.get_by_role_and_label(Role::TextInput, RECHERCHE);
        assert!(h.query_by_label(SECTION_PANIER).is_none());
        cliquer(&mut h, ONGLET_VENTE);
        assert_eq!(h.state().onglet, Onglet::Vente);
        h.get_by_label(SECTION_PANIER);
    }

    #[test]
    fn cliquer_une_tuile_remplit_le_panier_et_les_totaux() {
        let mut h = harness_vente(app_vendeuse("tuile"), FENETRE);
        h.get_by_label(PANIER_VIDE);
        let pain = tuile("Pain", "1,20\u{a0}€");
        cliquer(&mut h, &pain);
        cliquer(&mut h, &pain);
        assert_eq!(h.state().panier, [("Pain".to_string(), 2)]);
        h.get_by_label("Pain");
        h.get_by_label(&ligne_de_panier("1,20\u{a0}€", 2, "2,40\u{a0}€"));
        h.get_by_label("Total TTC : 2,40\u{a0}€");
        assert!(h.query_by_label(PANIER_VIDE).is_none());
        // Rien n'est vendu tant qu'on n'a pas encaissé.
        assert_eq!(nombre_de_pieces(&h), 0);
    }

    #[test]
    fn un_produit_epuise_ou_sans_prix_ne_s_ajoute_pas() {
        let mut h = harness_vente(app_vendeuse_avec_blocages("bloques"), FENETRE);
        h.get_by_label(EPUISE);
        h.get_by_label(PRIX_MANQUANT);
        for (nom, raison) in [("Sel", EPUISE), ("Thé", PRIX_MANQUANT)] {
            let libelle = produit_non_vendable(nom, raison);
            assert!(est_grise(&h, &libelle), "{libelle} devrait être grisée");
            h.get_by_role_and_label(Role::Button, &libelle).click();
            h.run();
        }
        assert!(h.state().panier.is_empty());
        assert!(!est_grise(&h, &tuile("Pain", "1,20\u{a0}€")));
    }

    #[test]
    fn on_ne_peut_pas_mettre_au_panier_plus_que_le_stock() {
        let mut app = app_vendeuse("limite");
        app.panier = vec![("Savon".to_string(), 10)]; // tout le stock
        let mut h = harness_vente(app, FENETRE);
        h.get_by_label(TOUT_AU_PANIER);
        assert!(est_grise(
            &h,
            &produit_non_vendable("Savon", TOUT_AU_PANIER)
        ));
        assert!(est_grise(&h, &ajouter_une_unite("Savon")));
        assert!(!est_grise(&h, &retirer_une_unite("Savon")));
        cliquer(&mut h, &retirer_une_unite("Savon"));
        assert_eq!(h.state().panier, [("Savon".to_string(), 9)]);
        assert!(!est_grise(&h, &ajouter_une_unite("Savon")));
    }

    #[test]
    fn les_boutons_du_panier_ont_un_nom_complet_et_agissent() {
        let mut h = harness_vente(app_vendeuse("boutons-panier"), FENETRE);
        cliquer(&mut h, &tuile("Pain", "1,20\u{a0}€"));
        cliquer(&mut h, &tuile("Savon", "2,50\u{a0}€"));
        cliquer(&mut h, &ajouter_une_unite("Pain"));
        assert_eq!(
            h.state().panier,
            [("Pain".to_string(), 2), ("Savon".to_string(), 1)]
        );
        cliquer(&mut h, &retirer_une_unite("Pain"));
        cliquer(&mut h, &retirer_une_unite("Pain")); // la ligne tombe à 0 : elle disparaît
        assert_eq!(h.state().panier, [("Savon".to_string(), 1)]);
        cliquer(&mut h, &retirer_du_panier("Savon"));
        assert!(h.state().panier.is_empty());
        cliquer(&mut h, &tuile("Pain", "1,20\u{a0}€"));
        cliquer(&mut h, BOUTON_VIDER);
        assert!(h.state().panier.is_empty());
        h.get_by_label(PANIER_VIDE);
    }

    #[test]
    fn la_recherche_filtre_les_tuiles() {
        let mut h = harness_vente(app_vendeuse("recherche-vente"), FENETRE);
        saisir(&mut h, RECHERCHE_VENTE, "sav");
        h.get_by_role_and_label(Role::Button, &tuile("Savon", "2,50\u{a0}€"));
        assert!(h.query_by_label_contains("Ajouter Pain").is_none());
    }

    #[test]
    fn encaisser_en_especes_montre_le_rendu_puis_le_ticket() {
        let mut h = harness_vente(app_vendeuse("especes"), Vec2::new(1100.0, 900.0));
        let pain = tuile("Pain", "1,20\u{a0}€");
        for _ in 0..3 {
            cliquer(&mut h, &pain);
        }
        h.get_by_label("Total TTC : 3,60\u{a0}€");
        saisir(&mut h, CHAMP_RECU, "5");
        h.get_by_label(&rendu_de_monnaie("1,40\u{a0}€"));
        cliquer(&mut h, BOUTON_ENCAISSER);

        let etat = h.state().etat.as_ref().unwrap();
        assert_eq!(etat.lignes("pain", false)[0].quantite, 7);
        let pieces = etat.pieces_recentes(10);
        assert_eq!(pieces.len(), 1);
        assert_eq!(pieces[0].numero, 1);
        assert!(h.state().panier.is_empty());
        assert!(h.state().saisie_recu.is_empty());
        h.get_by_label(&titre_ticket(1));
        h.get_by_label_contains("14/11/2023 23:13");
        h.get_by_label(&ligne_de_ticket(3, "Pain", "3,60\u{a0}€"));
        h.get_by_label(&recu_du_ticket("5,00\u{a0}€"));
        h.get_by_label(&rendu_de_monnaie("1,40\u{a0}€"));
        h.get_by_label_contains("Vente n° 1 enregistrée");
        // Le bouton a disparu avec le panier : on ne peut pas encaisser deux fois.
        assert!(h.query_by_label(BOUTON_ENCAISSER).is_none());

        cliquer(&mut h, BOUTON_NOUVELLE_VENTE);
        assert!(h.query_by_label(&titre_ticket(1)).is_none());
        h.get_by_label(PANIER_VIDE);
        assert_eq!(nombre_de_pieces(&h), 1); // le ticket fermé ne supprime pas la vente
    }

    #[test]
    fn le_bouton_encaisser_est_grise_si_le_recu_ne_couvre_pas_le_total() {
        let mut h = harness_vente(app_vendeuse("recu-court"), Vec2::new(1100.0, 900.0));
        let pain = tuile("Pain", "1,20\u{a0}€");
        for _ in 0..3 {
            cliquer(&mut h, &pain);
        }
        // Aucun montant : rien n'est dit, mais on ne peut pas encaisser.
        assert!(est_grise(&h, BOUTON_ENCAISSER));
        saisir(&mut h, CHAMP_RECU, "2");
        h.get_by_label(&il_manque("1,60\u{a0}€"));
        assert!(est_grise(&h, BOUTON_ENCAISSER));
        h.get_by_role_and_label(Role::Button, BOUTON_ENCAISSER)
            .click();
        h.run();
        assert_eq!(nombre_de_pieces(&h), 0);

        saisir(&mut h, CHAMP_RECU, "x");
        h.get_by_label_contains("Montant illisible");
        assert!(est_grise(&h, BOUTON_ENCAISSER));
    }

    #[test]
    fn un_double_clic_sur_encaisser_n_enregistre_qu_une_vente() {
        let mut h = harness_vente(app_vendeuse("double-vente"), Vec2::new(1100.0, 900.0));
        cliquer(&mut h, &tuile("Pain", "1,20\u{a0}€"));
        saisir(&mut h, CHAMP_RECU, "2");
        cliquer(&mut h, BOUTON_ENCAISSER);
        // Le second clic tombe sur un bouton qui n'existe plus : rien ne se passe.
        if let Some(bouton) = h.query_by_role_and_label(Role::Button, BOUTON_ENCAISSER) {
            bouton.click();
            h.run();
        }
        assert_eq!(nombre_de_pieces(&h), 1);
        let etat = h.state().etat.as_ref().unwrap();
        assert_eq!(etat.lignes("pain", false)[0].quantite, 9);
    }

    #[test]
    fn par_carte_le_montant_recu_n_est_pas_demande() {
        let mut h = harness_vente(app_vendeuse("carte"), Vec2::new(1100.0, 900.0));
        cliquer(&mut h, &tuile("Savon", "2,50\u{a0}€"));
        cliquer(&mut h, MOYEN_CARTE);
        assert_eq!(h.state().moyen, Moyen::Carte);
        assert!(h.query_by_label(CHAMP_RECU).is_none());
        assert!(!est_grise(&h, BOUTON_ENCAISSER));
        cliquer(&mut h, BOUTON_ENCAISSER);
        assert_eq!(nombre_de_pieces(&h), 1);
        h.get_by_label(&moyen_du_ticket("Carte"));
    }

    #[test]
    fn montant_juste_remplit_le_recu_avec_le_total() {
        let mut h = harness_vente(app_vendeuse("juste"), Vec2::new(1100.0, 900.0));
        cliquer(&mut h, &tuile("Savon", "2,50\u{a0}€"));
        cliquer(&mut h, BOUTON_MONTANT_JUSTE);
        assert_eq!(h.state().saisie_recu, "2,50");
        h.get_by_label(&rendu_de_monnaie("0,00\u{a0}€"));
        assert!(!est_grise(&h, BOUTON_ENCAISSER));
    }

    #[test]
    fn un_panier_devenu_invalide_est_signale_et_ne_s_encaisse_pas() {
        let mut app = app_vendeuse("invalide");
        app.panier = vec![("Fantôme".to_string(), 1)];
        let h = harness_vente(app, Vec2::new(1100.0, 900.0));
        h.get_by_label(PANIER_INVALIDE);
        assert!(est_grise(&h, BOUTON_ENCAISSER));
    }

    #[test]
    fn choisir_une_tuile_apres_une_vente_ferme_le_ticket_et_commence_la_suivante() {
        let mut h = harness_vente(app_vendeuse("suivante"), Vec2::new(1100.0, 900.0));
        cliquer(&mut h, &tuile("Savon", "2,50\u{a0}€"));
        cliquer(&mut h, MOYEN_CARTE);
        cliquer(&mut h, BOUTON_ENCAISSER);
        h.get_by_label(&titre_ticket(1));
        cliquer(&mut h, &tuile("Pain", "1,20\u{a0}€"));
        assert!(h.query_by_label(&titre_ticket(1)).is_none());
        assert_eq!(h.state().panier, [("Pain".to_string(), 1)]);
    }

    #[test]
    fn le_ticket_detaille_la_tva_par_taux_en_europe() {
        let mut h = harness_vente(app_vendeuse("tva"), Vec2::new(1100.0, 900.0));
        for _ in 0..3 {
            cliquer(&mut h, &tuile("Pain", "1,20\u{a0}€"));
        }
        cliquer(&mut h, &tuile("Savon", "2,50\u{a0}€"));
        let lignes = [
            "Total TTC : 6,10\u{a0}€",
            "dont TVA 5,5\u{a0}% : 0,19\u{a0}€",
            "dont TVA 20\u{a0}% : 0,42\u{a0}€",
        ];
        for l in lignes {
            h.get_by_label(l);
        }
        cliquer(&mut h, MOYEN_CARTE);
        cliquer(&mut h, BOUTON_ENCAISSER);
        // Le ticket garde le détail, calculé depuis les lignes vendues.
        for l in lignes {
            h.get_by_label(l);
        }
    }

    #[test]
    fn le_ticket_detaille_la_taxe_aux_etats_unis() {
        let mut etat = Etat::ouvrir(chemin_temp("taxe-us"))
            .unwrap()
            .avec_horloge(|| FIXE);
        assert!(etat.definir_reglages(Profil::EtatsUnis, Devise::Usd, "8"));
        assert!(etat.ajouter_avec_prix("Pain", "0", "1.20", ""));
        assert!(etat.entree("Pain", "10"));
        let mut h = harness_vente(Application::nouvelle(etat), Vec2::new(1100.0, 900.0));
        cliquer(&mut h, &tuile("Pain", "$1.20"));
        cliquer(&mut h, &tuile("Pain", "$1.20"));
        // 2,40 $ hors taxe + 8 % (19,2 cents arrondis à 19) = 2,59 $.
        let lignes = [
            "Sous-total : $2.40",
            "Taxe 8\u{a0}% : $0.19",
            "Total : $2.59",
        ];
        for l in lignes {
            h.get_by_label(l);
        }
        cliquer(&mut h, MOYEN_CARTE);
        cliquer(&mut h, BOUTON_ENCAISSER);
        for l in lignes {
            h.get_by_label(l);
        }
    }

    #[test]
    fn le_ticket_affiche_la_mention_non_certifie() {
        let mut h = harness_vente(app_vendeuse("mention"), Vec2::new(1100.0, 900.0));
        cliquer(&mut h, &tuile("Savon", "2,50\u{a0}€"));
        cliquer(&mut h, MOYEN_CARTE);
        cliquer(&mut h, BOUTON_ENCAISSER);
        h.get_by_label_contains("n'est pas une caisse certifiée");
    }

    #[test]
    fn les_actions_principales_de_la_vente_sont_a_l_ecran_a_la_taille_par_defaut() {
        let mut app = app_vendeuse("ecran-defaut");
        // Trois lignes de panier : « Encaisser » doit rester visible malgré elles.
        app.panier = vec![
            ("Pain".to_string(), 2),
            ("Savon".to_string(), 1),
            ("Lait".to_string(), 3),
        ];
        let mut h = harness_vente(app, FENETRE);
        let ecran = Rect::from_min_size(Pos2::ZERO, FENETRE);
        for (role, nom) in [
            (Role::Button, BOUTON_ENCAISSER.to_string()),
            (Role::TextInput, CHAMP_RECU.to_string()),
            (Role::Button, tuile("Pain", "1,20\u{a0}€")),
        ] {
            let rect = h.get_by_role_and_label(role, &nom).rect();
            assert!(
                ecran.contains_rect(rect),
                "« {nom} » est hors de l'écran : {rect:?} hors de {ecran:?}"
            );
        }
        saisir(&mut h, CHAMP_RECU, "10");
        let rect = h.get_by_label_contains(LIBELLE_RENDU).rect();
        assert!(
            ecran.contains_rect(rect),
            "le rendu est hors de l'écran : {rect:?}"
        );
    }

    #[test]
    fn dans_une_fenetre_etroite_le_panier_et_encaisser_restent_accessibles() {
        let mut app = app_vendeuse("etroit");
        app.panier = vec![("Pain".to_string(), 2)];
        let h = harness_vente(app, Vec2::new(520.0, 1400.0));
        let ecran = Rect::from_min_size(Pos2::ZERO, Vec2::new(520.0, 1400.0));
        for nom in [BOUTON_ENCAISSER, MOYEN_CARTE] {
            let rect = h.get_by_role_and_label(Role::Button, nom).rect();
            assert!(ecran.contains_rect(rect), "« {nom} » : {rect:?}");
        }
        // Un panier non vide passe avant la liste des produits dans l'ordre de lecture.
        let panier = h.get_by_label(SECTION_PANIER).rect();
        let produits = h.get_by_label(SECTION_PRODUITS_VENTE).rect();
        assert!(panier.top() < produits.top());
    }

    fn verifier_contrastes(textes: &[TextePeint], nom_du_cas: &str) {
        assert!(
            textes.len() >= 10,
            "{nom_du_cas} : trop peu de textes : {}",
            textes.len()
        );
        for t in textes {
            let rapport = rapport_contraste(t.couleur, t.fond);
            assert!(
                rapport >= 4.5,
                "{nom_du_cas} : « {} » : {rapport:.2}:1 ({:?} sur {:?})",
                t.texte,
                t.couleur,
                t.fond
            );
        }
    }

    #[test]
    fn tous_les_textes_peints_de_l_ecran_vente_atteignent_4_5() {
        // Panier rempli, espèces, reçu suffisant : tous les boutons sont actifs.
        let mut app = app_vendeuse("contraste-vente");
        app.panier = vec![("Pain".to_string(), 2), ("Savon".to_string(), 1)];
        app.saisie_recu = "10".into();
        let textes = textes_peints(&mut app, FENETRE, 1.0);
        for attendu in [BOUTON_ENCAISSER, "Total TTC : 4,90\u{a0}€"] {
            assert!(
                textes.iter().any(|t| t.texte == attendu),
                "« {attendu} » n'est pas peint"
            );
        }
        verifier_contrastes(&textes, "panier");

        // Un montant qui manque s'écrit sur fond d'alerte : même exigence.
        app.saisie_recu = "1".into();
        let textes = textes_peints(&mut app, FENETRE, 1.0);
        assert!(textes.iter().any(|t| t.texte.starts_with("Il manque")));
        // Le bouton « Encaisser » est grisé ici : un contrôle inactif est dispensé de contraste.
        let actifs: Vec<TextePeint> = textes
            .into_iter()
            .filter(|t| t.texte != BOUTON_ENCAISSER)
            .collect();
        verifier_contrastes(&actifs, "reçu insuffisant");

        // Le ticket, juste après l'encaissement.
        app.saisie_recu = "10".into();
        let etat = app.etat.as_mut().unwrap();
        assert!(etat.encaisser(&app.panier, Moyen::Especes, "10"));
        app.panier.clear();
        let textes = textes_peints(&mut app, FENETRE, 1.0);
        assert!(textes.iter().any(|t| t.texte == MENTION_NON_CERTIFIE));
        verifier_contrastes(&textes, "ticket");
    }

    #[test]
    fn les_tuiles_bloquees_gardent_une_legende_lisible() {
        let mut app = app_vendeuse_avec_blocages("legende");
        let textes = textes_peints(&mut app, FENETRE, 1.0);
        for legende in [EPUISE, PRIX_MANQUANT] {
            let t = textes
                .iter()
                .find(|t| t.texte == legende)
                .unwrap_or_else(|| panic!("« {legende} » n'est pas peint"));
            let rapport = rapport_contraste(t.couleur, t.fond);
            // La légende est hors du bouton grisé : elle reste aussi lisible qu'un texte normal.
            assert!(rapport >= 4.5, "« {legende} » : {rapport:.2}:1");
        }
    }

    #[test]
    fn l_alerte_de_stock_reste_visible_a_200_pour_cent_sur_l_ecran_vente() {
        let taille = Vec2::new(960.0, 1400.0);
        let mut app = app_vendeuse_avec_blocages("zoom-vente");
        let ecran = Rect::from_min_size(Pos2::ZERO, taille / 2.0);
        let textes = textes_peints(&mut app, taille, 2.0);
        for legende in [EPUISE, PRIX_MANQUANT] {
            let t = textes
                .iter()
                .find(|t| t.texte == legende)
                .unwrap_or_else(|| panic!("« {legende} » n'est pas peint à 200 %"));
            assert!(
                ecran.contains_rect(t.rect),
                "« {legende} » déborde de l'écran : {:?} hors de {ecran:?}",
                t.rect
            );
        }
    }

    // ---- Ventes, réglages, prix ---------------------------------------------------------------

    /// Une vente par carte (Pain × 3 = 3,60 €) puis une en espèces (Savon × 1 = 2,50 €),
    /// affichées sur l'onglet Ventes.
    fn app_avec_ventes(nom: &str) -> Application {
        let mut etat = etat_vendeur(nom);
        assert!(etat.encaisser(&[("Pain".to_string(), 3)], Moyen::Carte, ""));
        assert!(etat.encaisser(&[("Savon".to_string(), 1)], Moyen::Especes, "5"));
        etat.effacer_ticket();
        let mut app = Application::nouvelle(etat);
        app.onglet = Onglet::Ventes;
        app
    }

    /// Un journal modifié à la main après une vente : le programme l'ouvre mais refuse de vendre.
    fn app_journal_altere(nom: &str) -> Application {
        let chemin = chemin_temp(nom);
        {
            let mut e = etat_vendeur(&format!("{nom}-source"));
            assert!(e.encaisser(&[("Pain".to_string(), 1)], Moyen::Carte, ""));
            std::fs::copy(e.chemin(), &chemin).unwrap();
        }
        let contenu = std::fs::read_to_string(&chemin).unwrap();
        // Une modification à la main, dans le journal : le pain vendu passe de 1,20 à 0,10.
        let altere = contenu.replacen("L;1;Pain;1;120;", "L;1;Pain;1;10;", 1);
        assert_ne!(
            contenu, altere,
            "le fichier n'a pas le format attendu :\n{contenu}"
        );
        std::fs::write(&chemin, altere).unwrap();
        Application::nouvelle(Etat::ouvrir(chemin).unwrap().avec_horloge(|| FIXE))
    }

    fn est_selectionne(h: &Harness<'static, Application>, libelle: &str) -> Option<bool> {
        h.get_by_role_and_label(Role::Button, libelle)
            .accesskit_node()
            .toggled()
            .map(|t| t == eframe::egui::accesskit::Toggled::True)
    }

    fn quantite_en_stock(h: &Harness<'static, Application>, nom: &str) -> u32 {
        let etat = h.state().etat.as_ref().unwrap();
        etat.lignes(nom, false)[0].quantite
    }

    #[test]
    fn les_quatre_onglets_s_affichent_et_se_selectionnent() {
        let mut h = harness_vente(app_vendeuse("quatre-onglets"), FENETRE);
        for nom in [ONGLET_VENTE, ONGLET_STOCK, ONGLET_VENTES, ONGLET_REGLAGES] {
            h.get_by_role_and_label(Role::Button, nom);
        }
        cliquer(&mut h, ONGLET_VENTES);
        assert_eq!(h.state().onglet, Onglet::Ventes);
        assert_eq!(est_selectionne(&h, ONGLET_VENTES), Some(true));
        assert_eq!(est_selectionne(&h, ONGLET_VENTE), Some(false));
        h.get_by_label(SECTION_VENTES);
        cliquer(&mut h, ONGLET_REGLAGES);
        assert_eq!(h.state().onglet, Onglet::Reglages);
        h.get_by_label(SECTION_REGLAGES);
    }

    #[test]
    fn les_quatre_onglets_tiennent_dans_une_fenetre_etroite() {
        let taille = Vec2::new(TAILLE_MINIMALE[0], 600.0);
        let h = harness_vente(app_vendeuse("onglets-etroits"), taille);
        let ecran = Rect::from_min_size(Pos2::ZERO, taille);
        for nom in [ONGLET_VENTE, ONGLET_STOCK, ONGLET_VENTES, ONGLET_REGLAGES] {
            let rect = h.get_by_role_and_label(Role::Button, nom).rect();
            assert!(
                ecran.contains_rect(rect),
                "« {nom} » : {rect:?} hors de {ecran:?}"
            );
        }
    }

    #[test]
    fn la_liste_des_ventes_montre_les_pieces_et_permet_d_annuler_avec_confirmation() {
        let mut h = harness_vente(app_avec_ventes("liste-ventes"), FENETRE);
        let (vente1, vente2) = (titre_de_piece(1, "Vente"), titre_de_piece(2, "Vente"));
        // La plus récente d'abord.
        assert!(h.get_by_label(&vente2).rect().top() < h.get_by_label(&vente1).rect().top());
        h.get_by_label(&detail_de_piece("14/11/2023 23:13", "Carte"));
        h.get_by_label(&detail_de_piece("14/11/2023 23:13", "Espèces"));
        h.get_by_label("3,60\u{a0}€");
        h.get_by_label("2,50\u{a0}€");

        // Première demande : on demande confirmation, rien n'est annulé.
        cliquer(&mut h, &annuler_la_vente(1));
        h.get_by_label(&confirmation_annulation(1, "3,60\u{a0}€"));
        assert_eq!(nombre_de_pieces(&h), 2);
        cliquer(&mut h, BOUTON_GARDER_VENTE);
        assert!(
            h.query_by_label_contains("Annuler la vente n° 1 (")
                .is_none()
        );
        assert_eq!(nombre_de_pieces(&h), 2);
        assert_eq!(quantite_en_stock(&h, "Pain"), 7);

        cliquer(&mut h, &annuler_la_vente(1));
        cliquer(&mut h, BOUTON_CONFIRMER_ANNULATION);
        // Une écriture inverse s'ajoute, le stock est remis, la vente reste visible.
        assert_eq!(nombre_de_pieces(&h), 3);
        h.get_by_label(&titre_de_piece(3, "Annulation de la vente n° 1"));
        h.get_by_label(&vente1);
        assert_eq!(quantite_en_stock(&h, "Pain"), 10);
        assert!(h.state().annulation_en_attente.is_none());
        h.get_by_label_contains("Vente n° 1 annulée : stock remis.");
        // Plus de bouton pour la vente déjà annulée ni pour l'annulation ; l'autre reste annulable.
        for numero in [1, 3] {
            let libelle = annuler_la_vente(numero);
            assert!(h.query_by_role_and_label(Role::Button, &libelle).is_none());
        }
        h.get_by_role_and_label(Role::Button, &annuler_la_vente(2));
        // L'annulation ne laisse pas un ticket de vente sur l'écran de vente.
        cliquer(&mut h, ONGLET_VENTE);
        h.get_by_label(PANIER_VIDE);
    }

    #[test]
    fn sans_vente_la_liste_le_dit() {
        let mut app = app_vendeuse("aucune-vente");
        app.onglet = Onglet::Ventes;
        let h = harness_vente(app, FENETRE);
        h.get_by_label(AUCUNE_VENTE);
    }

    #[test]
    fn un_journal_altere_affiche_un_bandeau_et_grise_encaisser_et_annuler() {
        let mut app = app_journal_altere("altere-ui");
        app.onglet = Onglet::Ventes;
        let mut h = harness_vente(app, FENETRE);
        h.get_by_label_contains("Le journal de caisse a été modifié");
        h.get_by_label_contains("Les ventes et les annulations sont bloquées");
        assert!(est_grise(&h, &annuler_la_vente(1)));

        // Même constat sur l'écran de vente : on peut remplir un panier, pas encaisser.
        cliquer(&mut h, ONGLET_VENTE);
        cliquer(&mut h, &tuile("Savon", "2,50\u{a0}€"));
        cliquer(&mut h, MOYEN_CARTE);
        h.get_by_label_contains("Le journal de caisse a été modifié");
        assert!(est_grise(&h, BOUTON_ENCAISSER));
        h.get_by_role_and_label(Role::Button, BOUTON_ENCAISSER)
            .click();
        h.run();
        assert_eq!(nombre_de_pieces(&h), 1);
    }

    #[test]
    fn les_reglages_se_changent_et_se_conservent() {
        let chemin = chemin_temp("reglages-ui");
        let etat = Etat::ouvrir(chemin.clone()).unwrap().avec_horloge(|| FIXE);
        let mut app = Application::nouvelle(etat);
        app.onglet = Onglet::Reglages;
        let mut h = harness_vente(app, FENETRE);
        // Les réglages enregistrés sont montrés d'abord (Europe, euro, 20 %).
        assert_eq!(est_selectionne(&h, PROFIL_EUROPE), Some(true));
        assert_eq!(est_selectionne(&h, DEVISE_EUR), Some(true));
        h.get_by_label_contains(RAPPEL_REGLAGES);
        h.get_by_label_contains("n'est pas une caisse certifiée");

        cliquer(&mut h, PROFIL_ETATS_UNIS);
        cliquer(&mut h, DEVISE_USD);
        assert_eq!(est_selectionne(&h, PROFIL_ETATS_UNIS), Some(true));
        assert_eq!(est_selectionne(&h, PROFIL_EUROPE), Some(false));
        h.get_by_label(EXPLICATION_ETATS_UNIS);
        // Rien n'est enregistré tant qu'on n'a pas validé.
        assert_eq!(
            h.state().etat.as_ref().unwrap().reglages().devise,
            Devise::Eur
        );
        h.state_mut().reglages_saisis.as_mut().unwrap().taux.clear();
        h.run();
        saisir(&mut h, CHAMP_TAUX_DEFAUT, "8,25");
        cliquer(&mut h, BOUTON_ENREGISTRER_REGLAGES);
        let reglages = h.state().etat.as_ref().unwrap().reglages();
        assert_eq!(
            (reglages.profil, reglages.devise, reglages.taux_defaut_bp),
            (Profil::EtatsUnis, Devise::Usd, 825)
        );
        h.get_by_label_contains("Réglages enregistrés");

        // L'écran de vente passe en dollars et la taxe s'ajoute au prix hors taxe.
        {
            let etat = h.state_mut().etat.as_mut().unwrap();
            assert!(etat.ajouter_avec_prix("Pain", "0", "10", ""));
            assert!(etat.entree("Pain", "5"));
        }
        cliquer(&mut h, ONGLET_VENTE);
        cliquer(&mut h, &tuile("Pain", "$10.00"));
        for ligne in [
            "Sous-total : $10.00",
            "Taxe 8.25\u{a0}% : $0.83",
            "Total : $10.83",
        ] {
            h.get_by_label(ligne);
        }

        // Les réglages se conservent : on rouvre le fichier.
        let rouvert = Etat::ouvrir(chemin).unwrap().reglages();
        assert_eq!(
            (rouvert.profil, rouvert.devise, rouvert.taux_defaut_bp),
            (Profil::EtatsUnis, Devise::Usd, 825)
        );
    }

    #[test]
    fn un_taux_illisible_est_refuse_et_garde_la_saisie() {
        let mut app = app_vendeuse("taux-illisible");
        app.onglet = Onglet::Reglages;
        let mut h = harness_vente(app, FENETRE);
        cliquer(&mut h, DEVISE_GBP);
        h.state_mut().reglages_saisis.as_mut().unwrap().taux = "beaucoup".into();
        h.run();
        cliquer(&mut h, BOUTON_ENREGISTRER_REGLAGES);
        h.get_by_label_contains("Erreur : ");
        let etat = h.state().etat.as_ref().unwrap();
        assert_eq!(etat.reglages().devise, Devise::Eur);
        // Les choix faits ne sont pas perdus.
        let brouillon = h.state().reglages_saisis.as_ref().unwrap();
        assert_eq!(
            (brouillon.devise, brouillon.taux.as_str()),
            (Devise::Gbp, "beaucoup")
        );
    }

    #[test]
    fn apres_une_vente_la_devise_et_le_profil_sont_figes_mais_pas_le_taux() {
        let mut app = app_avec_ventes("reglages-figes");
        app.onglet = Onglet::Reglages;
        let mut h = harness_vente(app, FENETRE);
        h.get_by_label(REGLAGES_VERROUILLES);
        for nom in [
            PROFIL_EUROPE,
            PROFIL_ETATS_UNIS,
            DEVISE_EUR,
            DEVISE_USD,
            DEVISE_GBP,
            DEVISE_XOF,
        ] {
            assert!(est_grise(&h, nom), "« {nom} » devrait être grisé");
        }
        // Le choix actuel reste annoncé comme sélectionné même grisé.
        assert_eq!(est_selectionne(&h, DEVISE_EUR), Some(true));
        // Le taux par défaut, lui, change encore.
        cliquer(&mut h, BOUTON_ENREGISTRER_REGLAGES); // sans rien modifier : réenregistre à l'identique
        h.get_by_label_contains("Réglages enregistrés");
        h.state_mut().reglages_saisis = Some(BrouillonReglages {
            profil: Profil::Europe,
            devise: Devise::Eur,
            taux: "10".into(),
        });
        h.run();
        cliquer(&mut h, BOUTON_ENREGISTRER_REGLAGES);
        assert_eq!(
            h.state().etat.as_ref().unwrap().reglages().taux_defaut_bp,
            1000
        );
    }

    #[test]
    fn le_prix_se_saisit_a_l_ajout_et_dans_les_actions() {
        let mut h = harness(app_avec_produits("prix-ajout"));
        saisir(&mut h, CHAMP_NOM, "Thé vert");
        saisir(&mut h, CHAMP_SEUIL, "2");
        saisir(&mut h, CHAMP_PRIX_AJOUT, "3,10");
        saisir(&mut h, CHAMP_TAUX_AJOUT, "5,5");
        h.get_by_label(BOUTON_AJOUTER).click();
        h.run();
        let ligne = h
            .state()
            .etat
            .as_ref()
            .unwrap()
            .lignes("thé", false)
            .remove(0);
        assert_eq!((ligne.prix, ligne.taux_bp), (Some(Montant(310)), Some(550)));
        for saisie in [
            &h.state().saisie_nom,
            &h.state().saisie_prix,
            &h.state().saisie_taux,
        ] {
            assert!(saisie.is_empty());
        }
        // La liste montre le prix, ou dit qu'il manque.
        h.get_by_label(&prix_court("3,10\u{a0}€"));
        assert_eq!(h.get_all_by_label(SANS_PRIX).count(), 2); // Riz et Sucre

        // Dans les actions : le prix du Riz.
        h.get_by_label("Riz 5kg").click();
        h.run();
        saisir(&mut h, CHAMP_PRIX, "2,50");
        h.get_by_label(BOUTON_ENREGISTRER_PRIX).click();
        h.run();
        h.get_by_label_contains("Prix de « Riz 5kg » : 2,50\u{a0}€");
        // Le riz est maintenant vendable : sa tuile n'est plus grisée.
        cliquer(&mut h, ONGLET_VENTE);
        assert!(!est_grise(&h, &tuile("Riz 5kg", "2,50\u{a0}€")));
        assert!(est_grise(&h, &produit_non_vendable("Sucre", PRIX_MANQUANT)));
    }

    #[test]
    fn un_prix_illisible_a_l_ajout_n_ajoute_rien_et_garde_la_saisie() {
        let mut h = harness(app_avec_produits("prix-illisible"));
        saisir(&mut h, CHAMP_NOM, "Thé vert");
        saisir(&mut h, CHAMP_PRIX_AJOUT, "abc");
        h.get_by_label(BOUTON_AJOUTER).click();
        h.run();
        h.get_by_label_contains("Erreur : ");
        assert!(
            h.state()
                .etat
                .as_ref()
                .unwrap()
                .lignes("thé", false)
                .is_empty()
        );
        assert_eq!(h.state().saisie_nom, "Thé vert");
        assert_eq!(h.state().saisie_prix, "abc");
    }

    #[test]
    fn choisir_un_produit_preremplit_son_prix_et_son_taux() {
        let mut app = app_vendeuse("preremplir");
        app.onglet = Onglet::Stock;
        let mut h = harness(app);
        h.get_by_label("Pain").click();
        h.run();
        let valeur = |h: &Harness<'static, Application>, champ: &str| {
            h.get_by_role_and_label(Role::TextInput, champ).value()
        };
        assert_eq!(valeur(&h, CHAMP_PRIX).as_deref(), Some("1,20"));
        assert_eq!(valeur(&h, CHAMP_TAUX).as_deref(), Some("5,5"));
        h.get_by_label("Savon").click();
        h.run();
        assert_eq!(valeur(&h, CHAMP_PRIX).as_deref(), Some("2,50"));
        assert_eq!(valeur(&h, CHAMP_TAUX).as_deref().unwrap_or(""), "");
        // Réenregistrer sans rien changer ne change rien.
        h.get_by_label(BOUTON_ENREGISTRER_PRIX).click();
        h.run();
        let ligne = h
            .state()
            .etat
            .as_ref()
            .unwrap()
            .lignes("savon", false)
            .remove(0);
        assert_eq!((ligne.prix, ligne.taux_bp), (Some(Montant(250)), None));
    }

    #[test]
    fn renommer_ou_supprimer_un_produit_vendu_affiche_le_message_de_la_bibliotheque() {
        let mut app = app_avec_ventes("produit-vendu");
        app.onglet = Onglet::Stock;
        let mut h = harness(app);
        h.get_by_label("Pain").click();
        h.run();
        saisir(&mut h, CHAMP_NOUVEAU_NOM, "Pain complet");
        h.get_by_label(BOUTON_RENOMMER).click();
        h.run();
        let message = "figure dans des ventes : il ne peut plus être renommé ni supprimé";
        h.get_by_label_contains(message);
        h.get_by_label(BOUTON_SUPPRIMER).click();
        h.run();
        h.get_by_label(BOUTON_CONFIRMER_SUPPRESSION).click();
        h.run();
        h.get_by_label_contains(message);
        assert_eq!(
            h.state().etat.as_ref().unwrap().lignes("pain", false).len(),
            1
        );
    }

    #[test]
    fn les_noms_de_champs_et_boutons_des_nouveaux_ecrans_ont_un_role_et_un_nom() {
        let mut app = app_vendeuse("roles-reglages");
        app.onglet = Onglet::Reglages;
        let h = harness_vente(app, FENETRE);
        h.get_by_role_and_label(Role::TextInput, CHAMP_TAUX_DEFAUT);
        for nom in [
            PROFIL_EUROPE,
            PROFIL_ETATS_UNIS,
            DEVISE_EUR,
            DEVISE_USD,
            DEVISE_GBP,
            DEVISE_XOF,
            BOUTON_ENREGISTRER_REGLAGES,
        ] {
            h.get_by_role_and_label(Role::Button, nom);
        }

        let mut app = app_vendeuse("roles-stock");
        app.onglet = Onglet::Stock;
        app.selection = Some("Pain".into());
        let h = harness_vente(app, Vec2::new(1100.0, 1400.0));
        for nom in [CHAMP_PRIX_AJOUT, CHAMP_TAUX_AJOUT, CHAMP_PRIX, CHAMP_TAUX] {
            h.get_by_role_and_label(Role::TextInput, nom);
        }
        h.get_by_role_and_label(Role::Button, BOUTON_ENREGISTRER_PRIX);
    }

    #[test]
    fn le_bouton_enregistrer_les_reglages_est_a_l_ecran_a_la_taille_par_defaut() {
        let mut app = app_vendeuse("reglages-ecran");
        app.onglet = Onglet::Reglages;
        let h = harness_vente(app, FENETRE);
        let ecran = Rect::from_min_size(Pos2::ZERO, FENETRE);
        for nom in [BOUTON_ENREGISTRER_REGLAGES, DEVISE_XOF, PROFIL_ETATS_UNIS] {
            let rect = h.get_by_role_and_label(Role::Button, nom).rect();
            assert!(
                ecran.contains_rect(rect),
                "« {nom} » : {rect:?} hors de {ecran:?}"
            );
        }
    }

    #[test]
    fn tous_les_textes_peints_des_ecrans_ventes_et_reglages_atteignent_4_5() {
        // Les contrôles grisés sont dispensés de contraste : on les écarte par leur nom.
        let sauf = |textes: Vec<TextePeint>, grises: &[&str]| -> Vec<TextePeint> {
            textes
                .into_iter()
                .filter(|t| !grises.contains(&t.texte.as_str()))
                .collect()
        };

        let mut app = app_avec_ventes("contraste-ventes");
        let textes = textes_peints(&mut app, FENETRE, 1.0);
        assert!(textes.iter().any(|t| t.texte == BOUTON_ANNULER_VENTE));
        verifier_contrastes(&textes, "ventes");
        app.annulation_en_attente = Some(1);
        let textes = textes_peints(&mut app, FENETRE, 1.0);
        assert!(
            textes
                .iter()
                .any(|t| t.texte == BOUTON_CONFIRMER_ANNULATION)
        );
        verifier_contrastes(&textes, "ventes, confirmation");

        let mut app = app_journal_altere("contraste-altere");
        app.onglet = Onglet::Ventes;
        let textes = textes_peints(&mut app, FENETRE, 1.0);
        assert!(
            textes
                .iter()
                .any(|t| t.texte.contains("Les ventes et les annulations"))
        );
        verifier_contrastes(&sauf(textes, &[BOUTON_ANNULER_VENTE]), "journal altéré");

        let mut app = app_vendeuse("contraste-reglages");
        app.onglet = Onglet::Reglages;
        let textes = textes_peints(&mut app, FENETRE, 1.0);
        assert!(
            textes
                .iter()
                .any(|t| t.texte == BOUTON_ENREGISTRER_REGLAGES)
        );
        verifier_contrastes(&textes, "réglages");

        let mut app = app_avec_ventes("contraste-reglages-figes");
        app.onglet = Onglet::Reglages;
        let textes = textes_peints(&mut app, FENETRE, 1.0);
        assert!(textes.iter().any(|t| t.texte == REGLAGES_VERROUILLES));
        let grises = [
            PROFIL_EUROPE,
            PROFIL_ETATS_UNIS,
            DEVISE_EUR,
            DEVISE_USD,
            DEVISE_GBP,
            DEVISE_XOF,
        ];
        verifier_contrastes(&sauf(textes, &grises), "réglages figés");

        // L'onglet Stock avec ses nouveaux champs de prix, produit sélectionné.
        let mut app = app_vendeuse("contraste-stock-prix");
        app.onglet = Onglet::Stock;
        app.selection = Some("Pain".into());
        app.saisie_prix_produit = "1,20".into();
        app.saisie_taux_produit = "5,5".into();
        app.saisie_nom = "Thé".into();
        app.saisie_prix = "3".into();
        let textes = textes_peints(&mut app, FENETRE, 1.0);
        assert!(textes.iter().any(|t| t.texte == BOUTON_ENREGISTRER_PRIX));
        // Les boutons « Renommer » et « Entrée/Sortie » sont grisés tant que leur champ est vide.
        let grises = [BOUTON_RENOMMER, BOUTON_ENTREE, BOUTON_SORTIE];
        verifier_contrastes(&sauf(textes, &grises), "stock avec prix");
    }
}
