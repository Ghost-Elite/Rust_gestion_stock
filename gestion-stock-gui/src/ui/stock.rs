//! L'écran Stock : la liste des produits (avec recherche et alertes), leurs mouvements, l'ajout
//! d'un produit avec son prix, et les actions sur le produit choisi (entrée, sortie, prix, nom,
//! suppression). Il garde les cartes et les champs de l'application d'origine, dans le cadre de
//! la caisse (en-tête sombre, couleurs et police de la maquette).

use super::composants::*;
use super::{Application, LARGEUR_DEUX_COLONNES};
use crate::etat::{LigneProduit, Resume};
use crate::palette::{
    ACCENT, ACCENT_DOUX, ALERTE_FOND, ALERTE_TEXTE, AVERTISSEMENT_TEXTE, FILET, FOND, SUCCES_FOND,
    SUCCES_TEXTE, SURFACE, TEXTE, couleur,
};
use crate::police::{Graisse, famille_disponible};
use crate::textes::*;
use eframe::egui::{
    Align, Button, CentralPanel, CornerRadius, Frame, Id, Layout, Margin, Panel, RichText,
    ScrollArea, Stroke, Ui, Vec2,
};

pub(super) fn ecran_stock(ui: &mut Ui, app: &mut Application) {
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
pub(super) fn abandonner_la_selection_perimee(app: &mut Application) {
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

#[cfg(test)]
mod tests {
    use super::super::essais::*;
    use super::super::*;
    use crate::palette::{self, rapport_contraste};
    use eframe::egui::accesskit::Role;
    use eframe::egui::{Pos2, Rect, Vec2};
    use egui_kittest::{Harness, kittest::Queryable};
    use gestion_stock::monnaie::Montant;

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
    fn les_noms_de_champs_et_boutons_des_prix_ont_un_role_et_un_nom() {
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
    fn tous_les_textes_peints_du_stock_avec_prix_atteignent_4_5() {
        // Les contrôles grisés sont dispensés de contraste : on les écarte par leur nom.
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
        let textes: Vec<TextePeint> = textes
            .into_iter()
            .filter(|t| !grises.contains(&t.texte.as_str()))
            .collect();
        verifier_contrastes(&textes, "stock avec prix");
    }
}
