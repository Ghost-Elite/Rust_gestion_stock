//! Le dessin de la fenêtre avec egui. Aucune logique métier ici : tout passe par `Etat`.
//!
//! Règles d'accessibilité suivies partout : chaque champ est relié à son libellé
//! (`labelled_by`), chaque bouton porte son texte complet, et une alerte est toujours
//! écrite en toutes lettres (la couleur ne vient qu'en plus).
//!
//! Le style : un fond gris très clair, des cartes blanches aux coins arrondis, une seule
//! couleur d'accent, des puces pour les états, de l'espace. Les couleurs viennent toutes de
//! `palette`, où leur contraste est vérifié par des tests.

use crate::etat::{Etat, LignePiece, LigneProduit, Resume, Retour, chemin_complet};
use crate::palette::{
    ACCENT, ACCENT_DOUX, ALERTE_FOND, ALERTE_TEXTE, AVERTISSEMENT_TEXTE, ENTETE_FOND, ENTETE_TEXTE,
    FILET, FOND, SUCCES_FOND, SUCCES_TEXTE, SURFACE, TEXTE, couleur,
};
use crate::police::{Graisse, famille_disponible};
use crate::textes::*;
use eframe::egui::{
    self, Align, Align2, Button, CentralPanel, CornerRadius, Frame, Id, Layout, Margin, Panel,
    RichText, ScrollArea, Stroke, Ui, Vec2, WidgetInfo, WidgetType,
};
use gestion_stock::monnaie::Devise;
use gestion_stock::reglages::Profil;
use gestion_stock::vente::Moyen;
use std::path::Path;
use std::time::Duration;

mod composants;
mod entete;
#[cfg(test)]
mod essais;
mod paiement;
mod ticket;
mod vente;

use composants::*;

/// La taille de la fenêtre au lancement et sa taille minimale, en points. Les tests les utilisent
/// aussi : ils vérifient que l'essentiel est à l'écran à la taille que voit vraiment la personne.
pub const TAILLE_FENETRE: [f32; 2] = [1366.0, 768.0];
pub const TAILLE_MINIMALE: [f32; 2] = [640.0, 420.0];

/// En dessous de cette largeur (en points, donc aussi quand le texte est agrandi), on passe
/// en une seule colonne : les commandes se placent sous la liste au lieu d'être à côté.
const LARGEUR_DEUX_COLONNES: f32 = 800.0;

/// En dessous de cette largeur, l'écran de vente passe en une seule colonne qui défile.
const LARGEUR_CAISSE_DEUX_COLONNES: f32 = 1100.0;

/// Les écrans de l'application. On passe de l'un à l'autre par le Menu et par l'en-tête : il n'y
/// a plus d'onglets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Ecran {
    /// L'écran de caisse : c'est celui que l'on ouvre en premier, la vente est le geste courant.
    #[default]
    Vente,
    /// La recherche d'un article à mettre au ticket.
    Recherche,
    /// Les tickets, où l'on rembourse une vente.
    Historique,
    Stock,
    Reglages,
}

/// Ce que l'on demande de confirmer avant de l'avoir fait.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Confirmation {
    AnnulerVente,
    SupprimerArticle,
}

/// Une fenêtre modale ouverte par-dessus l'écran (les autres variantes arrivent avec leurs
/// écrans).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Modale {
    Menu,
    Confirmer(Confirmation),
    /// Le texte tapé ne correspond à aucun article.
    Introuvable(String),
    /// Le paiement en espèces.
    Especes,
    /// Le paiement par carte ou autre moyen, confirmé à la main (aucun terminal n'est relié).
    Terminal {
        moyen: Moyen,
        refuse: bool,
    },
    /// La vente qui vient d'être enregistrée, avec le rendu de monnaie et (au choix) son ticket.
    VenteTerminee {
        ticket_visible: bool,
    },
}

/// Un message de quelques secondes, en bas de l'écran. `depuis` est l'heure d'egui du premier
/// affichage (posée à ce moment-là).
#[derive(Debug, Clone, PartialEq)]
pub struct Toast {
    pub texte: String,
    pub depuis: Option<f64>,
}

/// Combien de secondes un message éphémère reste affiché.
const DUREE_MESSAGE: f64 = 3.0;

fn toast_visible(depuis: f64, maintenant: f64) -> bool {
    maintenant - depuis < DUREE_MESSAGE
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
    pub ecran: Ecran,
    /// Ce qui est tapé dans le champ de scan de l'écran de vente.
    pub scan: String,
    /// La ligne du ticket sélectionnée (par son nom de produit).
    pub ligne_choisie: Option<String>,
    /// Vrai quand le champ de scan doit reprendre le focus à l'image suivante.
    pub focus_scan: bool,
    /// Ce qui est tapé dans la recherche d'articles.
    pub recherche_articles: String,
    /// La fenêtre modale ouverte, s'il y en a une.
    pub modale: Option<Modale>,
    /// Le message éphémère affiché en bas de l'écran.
    pub toast: Option<Toast>,
    /// Le panier en cours : (nom du produit, quantité).
    pub panier: Vec<(String, u32)>,
    /// Vrai quand le champ « Montant reçu » doit prendre le focus à l'image suivante.
    pub focus_recu: bool,
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
            focus_scan: true,
            ..Application::default()
        }
    }

    /// `quantite` unités de plus de ce produit dans le panier.
    fn ajouter_au_panier(&mut self, nom: &str, quantite: u32) {
        match self.panier.iter_mut().find(|(n, _)| n == nom) {
            Some((_, deja)) => *deja = deja.saturating_add(quantite),
            None => self.panier.push((nom.to_string(), quantite)),
        }
    }

    /// Une unité de moins ; la ligne disparaît quand il n'en reste plus.
    fn retirer_du_panier_une_unite(&mut self, nom: &str) {
        if let Some((_, quantite)) = self.panier.iter_mut().find(|(n, _)| n == nom) {
            *quantite = quantite.saturating_sub(1);
        }
        self.panier.retain(|(_, quantite)| *quantite > 0);
    }

    /// Affiche un message de quelques secondes en bas de l'écran.
    pub fn message(&mut self, texte: impl Into<String>) {
        self.toast = Some(Toast {
            texte: texte.into(),
            depuis: None,
        });
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
    succes_vers_message(app);
    Panel::top(Id::new("entete"))
        .frame(
            Frame::new()
                .fill(couleur(ENTETE_FOND))
                .inner_margin(Margin::symmetric(20, 0)),
        )
        .exact_size(56.0)
        .show_separator_line(false)
        .show(ui, |ui| entete::entete(ui, app));
    // Une erreur reste affichée sous l'en-tête jusqu'à la prochaine action réussie.
    if let Some(Retour::Erreur(message)) = app.etat.as_ref().and_then(|etat| etat.retour.clone()) {
        Panel::top(Id::new("bandeau_erreur"))
            .frame(
                Frame::new()
                    .fill(couleur(FOND))
                    .inner_margin(Margin::symmetric(16, 8)),
            )
            .show_separator_line(false)
            .show(ui, |ui| {
                bandeau_alerte(ui, &format!("Erreur : {message}"));
            });
    }
    match app.ecran {
        Ecran::Vente => vente::ecran_vente(ui, app),
        Ecran::Recherche => page(ui, 900.0, |ui| {
            carte(ui, |ui| titre_de_carte(ui, RECHERCHE_TITRE));
        }),
        Ecran::Historique => page(ui, 900.0, |ui| ecran_ventes(ui, app)),
        Ecran::Stock => ecran_stock(ui, app),
        Ecran::Reglages => page(ui, 640.0, |ui| ecran_reglages(ui, app)),
    }
    match app.modale.clone() {
        Some(Modale::Menu) => entete::fenetre_menu(ui.ctx(), app),
        Some(Modale::Confirmer(question)) => vente::fenetre_confirmation(ui.ctx(), app, question),
        Some(Modale::Introuvable(saisie)) => vente::fenetre_introuvable(ui.ctx(), app, &saisie),
        Some(Modale::Especes) => paiement::fenetre_especes(ui.ctx(), app),
        Some(Modale::Terminal { moyen, refuse }) => {
            paiement::fenetre_terminal(ui.ctx(), app, moyen, refuse);
        }
        Some(Modale::VenteTerminee { ticket_visible }) => {
            paiement::fenetre_vente_terminee(ui.ctx(), app, ticket_visible);
        }
        None => {}
    }
    afficher_message(ui.ctx(), app);
}

/// Un message de succès de l'état devient un message éphémère : il n'y a plus de pied de page.
fn succes_vers_message(app: &mut Application) {
    let Some(etat) = app.etat.as_mut() else {
        return;
    };
    if let Some(Retour::Succes(texte)) = etat.retour.take_if(|r| matches!(r, Retour::Succes(_))) {
        app.message(texte);
    }
}

/// La pastille sombre du message éphémère, centrée au-dessus de la barre de fonctions.
fn afficher_message(ctx: &egui::Context, app: &mut Application) {
    let maintenant = ctx.input(|i| i.time);
    let Some(toast) = app.toast.as_mut() else {
        return;
    };
    let depuis = *toast.depuis.get_or_insert(maintenant);
    if !toast_visible(depuis, maintenant) {
        app.toast = None;
        return;
    }
    ctx.request_repaint_after(Duration::from_secs_f64(
        DUREE_MESSAGE - (maintenant - depuis),
    ));
    let texte = toast.texte.clone();
    egui::Area::new(Id::new("message_ephemere"))
        .anchor(Align2::CENTER_BOTTOM, Vec2::new(0.0, -104.0))
        .order(egui::Order::Tooltip)
        .fade_in(false)
        .interactable(false)
        .show(ctx, |ui| {
            Frame::new()
                .fill(couleur(TEXTE))
                .corner_radius(CornerRadius::same(10))
                .inner_margin(Margin::symmetric(18, 12))
                .show(ui, |ui| {
                    ui.add(
                        egui::Label::new(
                            RichText::new(texte).size(15.0).color(couleur(ENTETE_TEXTE)),
                        )
                        .wrap_mode(egui::TextWrapMode::Extend),
                    );
                });
        });
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
                .family(famille_disponible(ui.ctx(), Graisse::SemiGrasse))
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
            AVERTISSEMENT_TEXTE
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
    use super::essais::*;
    use super::*;
    use crate::etat::Etat;
    use crate::palette::{self, rapport_contraste};
    use eframe::egui::accesskit::Role;
    use eframe::egui::{Pos2, Rect, Vec2};
    use egui_kittest::{Harness, kittest::Queryable};
    use gestion_stock::monnaie::Devise;
    use gestion_stock::monnaie::Montant;
    use gestion_stock::reglages::Profil;
    use std::path::Path;

    /// « Riz 5kg » (seuil 5, stock 3, en alerte) et « Sucre » (seuil 0, stock 10, correct).
    fn app_avec_produits(nom: &str) -> Application {
        let mut etat = Etat::ouvrir(chemin_temp(nom)).unwrap();
        assert!(etat.ajouter("Riz 5kg", "5"));
        assert!(etat.entree("Riz 5kg", "3"));
        assert!(etat.ajouter("Sucre", "0"));
        assert!(etat.entree("Sucre", "10"));
        let mut app = Application::nouvelle(etat);
        app.ecran = Ecran::Stock; // l'écran testé ici est celui du stock
        app
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
        // Le message de succès (devenu un message éphémère) n'est pas recouvert par une erreur.
        assert!(
            h.state().toast.is_some(),
            "le message de succès est affiché"
        );
        assert!(
            !matches!(etat.retour, Some(Retour::Erreur(_))),
            "{:?}",
            etat.retour
        );
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
            // hauteur, on peut toujours faire défiler (les cartes plates et les cibles de 48
            // points de la maquette sont un peu plus hautes que l'ancien style).
            (
                "texte agrandi à 200 %",
                Vec2::new(960.0, 2400.0),
                2.0,
                "Riz 5kg",
            ),
            (
                "petite fenêtre, nom très long",
                Vec2::new(640.0, 1800.0),
                1.0,
                "Huile d'arachide Lesieur bouteille plastique 1 litre (carton de 12)",
            ),
        ];
        for (nom_du_cas, taille, zoom, produit) in cas {
            let mut etat = Etat::ouvrir(chemin_temp("visible")).unwrap();
            assert!(etat.ajouter(produit, "5")); // stock 0 <= seuil 5 : en alerte
            let mut app = Application::nouvelle(etat);
            app.ecran = Ecran::Stock;
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

    // ---- Le cadre : en-tête, Menu, messages --------------------------------------------------

    #[test]
    fn l_en_tete_montre_le_magasin_la_caisse_et_l_heure() {
        let h = harness_vente(app_vendeuse("entete"), FENETRE);
        h.get_by_label(NOM_MAGASIN);
        h.get_by_label(&libelle_caisse());
        h.get_by_label("23:13"); // horloge fixe : 14/11/2023 23:13 heure locale
    }

    #[test]
    fn le_menu_mene_aux_ecrans_et_le_retour_ramene_a_la_vente() {
        let mut h = harness_vente(app_vendeuse("menu"), FENETRE);
        cliquer(&mut h, BOUTON_MENU); // bouton TEMPORAIRE de l'en-tête de vente (tâche 6)
        assert_eq!(h.state().modale, Some(Modale::Menu));
        for nom in [ECRAN_HISTORIQUE, ECRAN_STOCK, ECRAN_REGLAGES, BOUTON_FERMER] {
            h.get_by_role_and_label(Role::Button, nom);
        }
        cliquer(&mut h, ECRAN_STOCK);
        assert_eq!(h.state().ecran, Ecran::Stock);
        assert!(h.state().modale.is_none());
        cliquer(&mut h, BOUTON_RETOUR_VENTE);
        assert_eq!(h.state().ecran, Ecran::Vente);
        // « Fermer » referme le menu sans changer d'écran.
        cliquer(&mut h, BOUTON_MENU);
        cliquer(&mut h, BOUTON_FERMER);
        assert!(h.state().modale.is_none());
        assert_eq!(h.state().ecran, Ecran::Vente);
    }

    #[test]
    fn chaque_ecran_secondaire_a_un_retour_a_la_vente() {
        for ecran in [
            Ecran::Stock,
            Ecran::Reglages,
            Ecran::Historique,
            Ecran::Recherche,
        ] {
            let mut app = app_vendeuse("retour");
            app.ecran = ecran;
            let mut h = harness_vente(app, FENETRE);
            cliquer(&mut h, BOUTON_RETOUR_VENTE);
            assert_eq!(h.state().ecran, Ecran::Vente, "depuis {ecran:?}");
        }
    }

    #[test]
    fn la_fenetre_menu_est_un_dialogue_nomme_et_montre_le_fichier_de_donnees() {
        let mut app = app_vendeuse("dialogue");
        app.modale = Some(Modale::Menu);
        let h = harness_vente(app, FENETRE);
        h.get_by_role_and_label(Role::Dialog, TITRE_MENU);
        h.get_by_label_contains("Données : ");
    }

    #[test]
    fn un_succes_devient_un_message_ephemere_et_une_erreur_un_bandeau() {
        let mut h = harness_vente(app_avec_produits("messages"), FENETRE);
        h.get_by_label("Riz 5kg").click();
        h.run();
        saisir(&mut h, CHAMP_QUANTITE, "2");
        cliquer(&mut h, BOUTON_ENTREE);
        let texte = h
            .state()
            .toast
            .as_ref()
            .expect("un message éphémère")
            .texte
            .clone();
        h.get_by_label(&texte);
        assert!(h.state().etat.as_ref().unwrap().retour.is_none());

        // Une erreur reste affichée, en bandeau, image après image.
        saisir(&mut h, CHAMP_QUANTITE, "50");
        cliquer(&mut h, BOUTON_SORTIE);
        h.get_by_label_contains("Erreur :");
        h.run();
        h.get_by_label_contains("Erreur :");
        // Une action réussie la remplace par un message de succès.
        saisir(&mut h, CHAMP_QUANTITE, "1");
        cliquer(&mut h, BOUTON_ENTREE);
        assert!(h.query_by_label_contains("Erreur :").is_none());
    }

    #[test]
    fn un_message_ephemere_s_efface_apres_trois_secondes() {
        assert!(toast_visible(10.0, 10.0));
        assert!(toast_visible(10.0, 12.9));
        assert!(!toast_visible(10.0, 13.0));
    }

    #[test]
    fn tous_les_textes_peints_de_l_en_tete_et_du_menu_atteignent_4_5() {
        let mut app = app_vendeuse("contraste-entete");
        let textes = sans_les_textes_grises(textes_peints(&mut app, FENETRE, 1.0));
        for attendu in [NOM_MAGASIN, "Caisse 01", "23:13", FONCTION_MENU] {
            assert!(
                textes.iter().any(|t| t.texte == attendu),
                "« {attendu} » n'est pas peint"
            );
        }
        verifier_contrastes(&textes, "en-tête");
        app.modale = Some(Modale::Menu);
        let textes = sans_les_textes_grises(textes_peints(&mut app, FENETRE, 1.0));
        assert!(textes.iter().any(|t| t.texte == TITRE_MENU));
        verifier_contrastes(&textes, "menu");
        // Un message éphémère : texte blanc sur fond sombre.
        app.modale = None;
        app.message("Vente annulée");
        let textes = sans_les_textes_grises(textes_peints(&mut app, FENETRE, 1.0));
        assert!(textes.iter().any(|t| t.texte == "Vente annulée"));
        verifier_contrastes(&textes, "message");
    }

    #[test]
    fn l_application_s_ouvre_sur_la_vente() {
        let app = app_vendeuse("ouverture");
        assert_eq!(app.ecran, Ecran::Vente);
    }

    // ---- Ventes, réglages, prix ---------------------------------------------------------------

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
        cliquer(&mut h, BOUTON_RETOUR_VENTE);
        assert_eq!(h.state().ecran, Ecran::Vente);
        h.get_by_label(TICKET_VIDE_TITRE);
    }

    #[test]
    fn sans_vente_la_liste_le_dit() {
        let mut app = app_vendeuse("aucune-vente");
        app.ecran = Ecran::Historique;
        let h = harness_vente(app, FENETRE);
        h.get_by_label(AUCUNE_VENTE);
    }

    #[test]
    fn un_journal_altere_affiche_un_bandeau_et_grise_encaisser_et_annuler() {
        let mut app = app_journal_altere("altere-ui");
        app.ecran = Ecran::Historique;
        let mut h = harness_vente(app, FENETRE);
        h.get_by_label_contains("Le journal de caisse a été modifié");
        h.get_by_label_contains("Les ventes et les annulations sont bloquées");
        assert!(est_grise(&h, &annuler_la_vente(1)));

        // Même constat sur l'écran de vente : on peut remplir un ticket, pas payer.
        cliquer(&mut h, BOUTON_RETOUR_VENTE);
        scanner(&mut h, "savon");
        h.get_by_label_contains("Le journal de caisse a été modifié");
        for nom in [BOUTON_ESPECES, BOUTON_CARTE, BOUTON_AUTRE] {
            assert!(est_grise(&h, nom), "{nom} devrait être grisé");
            h.get_by_role_and_label(Role::Button, nom).click();
            h.run();
        }
        assert!(h.state().modale.is_none());
        assert_eq!(nombre_de_pieces(&h), 1);
    }

    #[test]
    fn les_reglages_se_changent_et_se_conservent() {
        let chemin = chemin_temp("reglages-ui");
        let etat = Etat::ouvrir(chemin.clone()).unwrap().avec_horloge(|| FIXE);
        let mut app = Application::nouvelle(etat);
        app.ecran = Ecran::Reglages;
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
        cliquer(&mut h, BOUTON_RETOUR_VENTE);
        scanner(&mut h, "pain");
        for ligne in [LIBELLE_SOUS_TOTAL, "Taxe 8.25\u{a0}%", "$0.83", "10.83"] {
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
        app.ecran = Ecran::Reglages;
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
        app.ecran = Ecran::Reglages;
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
        // Le riz est maintenant vendable : on peut le mettre au ticket ; le sucre, sans prix, non.
        cliquer(&mut h, BOUTON_RETOUR_VENTE);
        scanner(&mut h, "riz");
        assert_eq!(h.state().panier, [("Riz 5kg".to_string(), 1)]);
        scanner(&mut h, "sucre");
        assert_eq!(h.state().panier.len(), 1);
        assert_eq!(
            h.state().toast.as_ref().unwrap().texte,
            produit_non_vendable("Sucre", PRIX_MANQUANT)
        );
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
        app.ecran = Ecran::Stock;
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
        app.ecran = Ecran::Stock;
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
        app.ecran = Ecran::Reglages;
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
        app.ecran = Ecran::Stock;
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
        app.ecran = Ecran::Reglages;
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
        app.ecran = Ecran::Historique;
        let textes = textes_peints(&mut app, FENETRE, 1.0);
        assert!(
            textes
                .iter()
                .any(|t| t.texte.contains("Les ventes et les annulations"))
        );
        verifier_contrastes(&sauf(textes, &[BOUTON_ANNULER_VENTE]), "journal altéré");

        let mut app = app_vendeuse("contraste-reglages");
        app.ecran = Ecran::Reglages;
        let textes = textes_peints(&mut app, FENETRE, 1.0);
        assert!(
            textes
                .iter()
                .any(|t| t.texte == BOUTON_ENREGISTRER_REGLAGES)
        );
        verifier_contrastes(&textes, "réglages");

        let mut app = app_avec_ventes("contraste-reglages-figes");
        app.ecran = Ecran::Reglages;
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
        app.ecran = Ecran::Stock;
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
