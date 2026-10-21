//! L'écran de recherche : trouver un article par son nom (sans casse ni accents), voir s'il en
//! reste, et le mettre au ticket sans quitter l'écran. Dimensions et couleurs :
//! celles de `maquette/7-recherche.html`.

use super::composants::{LigneBouton, bouton_a_lignes, repere, zone};
use super::vente::ajouter_au_ticket;
use super::{Application, LARGEUR_CAISSE_DEUX_COLONNES};
use crate::etat::LigneProduit;
use crate::palette::{
    ACCENT, ACCENT_TEXTE, ALERTE_TEXTE, AVERTISSEMENT_TEXTE, FILET, FILET_LIGNE, FOND, GRISE_FOND,
    GRISE_TEXTE, Rgb, SUCCES_BOUTON, SUCCES_TEXTE, SURFACE, TEXTE, TEXTE_DISCRET, TEXTE_SECONDAIRE,
    couleur,
};
use crate::police::{Graisse, police_disponible};
use crate::textes::*;
use eframe::egui::{
    Align, CentralPanel, CornerRadius, FontId, Frame, Label, Layout, Margin, RichText, ScrollArea,
    Sense, Stroke, TextEdit, Ui, UiBuilder, Vec2,
};

/// Jusqu'à cette quantité (comprise), l'article est signalé « Plus que N en stock ».
const STOCK_BAS_MAX: u32 = 5;
const HAUTEUR_LIGNE: f32 = 84.0;

/// Ce que l'on écrit du stock d'un article, et la couleur du texte : sans prix il n'est pas
/// vendable ; à zéro c'est une rupture ; jusqu'à cinq on prévient ; au-delà tout va bien.
pub(crate) fn etat_du_stock(quantite: u32, a_un_prix: bool) -> (String, Rgb) {
    if !a_un_prix {
        (PRIX_MANQUANT.to_string(), ALERTE_TEXTE)
    } else if quantite == 0 {
        (RUPTURE.to_string(), ALERTE_TEXTE)
    } else if quantite <= STOCK_BAS_MAX {
        (plus_que(quantite), AVERTISSEMENT_TEXTE)
    } else {
        (EN_STOCK.to_string(), SUCCES_TEXTE)
    }
}

pub(super) fn ecran_recherche(ui: &mut Ui, app: &mut Application) {
    let cote = if ui.available_width() >= LARGEUR_CAISSE_DEUX_COLONNES {
        120
    } else {
        16
    };
    CentralPanel::default()
        .frame(Frame::new().fill(couleur(FOND)).inner_margin(Margin {
            left: cote,
            right: cote,
            top: 24,
            bottom: 24,
        }))
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 16.0;
            ui.heading(RichText::new(RECHERCHE_TITRE).size(24.0));
            champ_de_recherche(ui, app);
            let resultats = app
                .etat
                .as_ref()
                .map(|etat| etat.recherche_articles(&app.recherche_articles))
                .unwrap_or_default();
            ui.label(
                RichText::new(compteur_articles(resultats.len()))
                    .size(15.0)
                    .color(couleur(TEXTE_DISCRET)),
            );
            let choix = carte_des_resultats(ui, app, &resultats);
            if let Some(nom) = choix
                && ajouter_au_ticket(app, &nom, 1)
            {
                app.ajoute = Some(nom);
            }
        });
}

/// La loupe du champ de recherche (24 × 24, couleur d'accent), dessinée.
fn icone_loupe(ui: &mut Ui) {
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(24.0), Sense::hover());
    let trait_ = Stroke::new(2.0, couleur(ACCENT));
    let peintre = ui.painter();
    peintre.circle_stroke(rect.min + Vec2::new(11.0, 11.0), 8.0, trait_);
    peintre.line_segment(
        [
            rect.min + Vec2::new(17.0, 17.0),
            rect.min + Vec2::new(21.5, 21.5),
        ],
        trait_,
    );
}

fn champ_de_recherche(ui: &mut Ui, app: &mut Application) {
    Frame::new()
        .fill(couleur(SURFACE))
        .stroke(Stroke::new(2.0, couleur(ACCENT)))
        .corner_radius(CornerRadius::same(12))
        .inner_margin(Margin::symmetric(20, 0))
        .show(ui, |ui| {
            zone(
                ui,
                Vec2::new(ui.available_width(), 60.0),
                Layout::left_to_right(Align::Center),
                |ui| {
                    ui.spacing_mut().item_spacing.x = 12.0;
                    icone_loupe(ui);
                    let champ = TextEdit::singleline(&mut app.recherche_articles)
                        .hint_text(
                            RichText::new(CHAMP_RECHERCHE_ARTICLE)
                                .size(20.0)
                                .color(couleur(TEXTE_DISCRET)),
                        )
                        .font(FontId::proportional(20.0))
                        .frame(Frame::NONE)
                        .desired_width(f32::INFINITY);
                    let reponse = ui.add(champ);
                    ui.ctx().accesskit_node_builder(reponse.id, |noeud| {
                        noeud.set_label(CHAMP_RECHERCHE_ARTICLE);
                    });
                    if std::mem::take(&mut app.focus_recherche) {
                        reponse.request_focus();
                    }
                    if reponse.changed() {
                        app.ajoute = None;
                    }
                },
            );
        });
}

/// La liste des résultats ; renvoie le nom de l'article dont on vient de cliquer « Ajouter ».
fn carte_des_resultats(
    ui: &mut Ui,
    app: &Application,
    resultats: &[LigneProduit],
) -> Option<String> {
    let mut choix = None;
    Frame::new()
        .fill(couleur(SURFACE))
        .stroke(Stroke::new(1.0, couleur(FILET)))
        .corner_radius(CornerRadius::same(12))
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing = Vec2::ZERO;
            ui.set_width(ui.available_width());
            ui.set_min_height(ui.available_height());
            ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    if resultats.is_empty() {
                        ui.add_space(48.0);
                        ui.vertical_centered(|ui| {
                            ui.label(
                                RichText::new(AUCUN_ARTICLE)
                                    .size(17.0)
                                    .color(couleur(TEXTE_DISCRET)),
                            );
                        });
                        ui.add_space(48.0);
                    }
                    for ligne in resultats {
                        let ajoute = app.ajoute.as_deref() == Some(ligne.nom.as_str());
                        let prix = app
                            .etat
                            .as_ref()
                            .and_then(|etat| ligne.prix.map(|p| etat.formater(p)));
                        if ligne_d_article(ui, ligne, prix.as_deref(), ajoute) {
                            choix = Some(ligne.nom.clone());
                        }
                    }
                });
        });
    choix
}

/// Une ligne de résultat : repère, nom et état du stock, prix, bouton. Renvoie vrai si l'on vient
/// de cliquer « Ajouter ».
fn ligne_d_article(ui: &mut Ui, ligne: &LigneProduit, prix: Option<&str>, ajoute: bool) -> bool {
    let (rect, _) = ui.allocate_exact_size(
        Vec2::new(ui.available_width(), HAUTEUR_LIGNE),
        Sense::hover(),
    );
    ui.painter().hline(
        rect.x_range(),
        rect.bottom() - 0.5,
        Stroke::new(1.0, couleur(FILET_LIGNE)),
    );
    let interieur = rect.shrink2(Vec2::new(20.0, 10.0));
    let mut enfant = ui.new_child(
        UiBuilder::new()
            .max_rect(interieur)
            .layout(Layout::left_to_right(Align::Center)),
    );
    enfant.spacing_mut().item_spacing.x = 20.0;
    repere(&mut enfant, &ligne.nom, 64.0);
    let (etat_texte, etat_couleur) = etat_du_stock(ligne.quantite, ligne.prix.is_some());
    let vendable = ligne.prix.is_some() && ligne.quantite > 0;
    let largeur_nom = (interieur.width() - 64.0 - 150.0 - 150.0 - 3.0 * 20.0).max(80.0);
    zone(
        &mut enfant,
        Vec2::new(largeur_nom, 64.0),
        Layout::top_down(Align::Min),
        |ui| {
            ui.spacing_mut().item_spacing.y = 2.0;
            ui.add(
                Label::new(
                    RichText::new(&ligne.nom)
                        .font(police_disponible(ui.ctx(), Graisse::Moyenne, 18.0))
                        .color(couleur(TEXTE)),
                )
                .truncate()
                .selectable(false),
            );
            ui.add(
                Label::new(
                    RichText::new(etat_texte)
                        .size(14.0)
                        .color(couleur(etat_couleur)),
                )
                .selectable(false),
            );
        },
    );
    zone(
        &mut enfant,
        Vec2::new(150.0, 64.0),
        Layout::right_to_left(Align::Center),
        |ui| {
            if let Some(prix) = prix {
                ui.add(
                    Label::new(
                        RichText::new(prix)
                            .font(police_disponible(ui.ctx(), Graisse::SemiGrasse, 19.0))
                            .color(couleur(TEXTE_SECONDAIRE)),
                    )
                    .selectable(false),
                );
            }
        },
    );
    let (texte, fond, couleur_texte, nom) = if !vendable {
        let raison = if ligne.prix.is_none() {
            PRIX_MANQUANT
        } else {
            EPUISE
        };
        (
            BOUTON_INDISPONIBLE,
            GRISE_FOND,
            GRISE_TEXTE,
            produit_non_vendable(&ligne.nom, raison),
        )
    } else if ajoute {
        (
            BOUTON_AJOUTE,
            SUCCES_BOUTON,
            ACCENT_TEXTE,
            ajouter_article(&ligne.nom),
        )
    } else {
        (
            BOUTON_AJOUTER_ARTICLE,
            ACCENT,
            ACCENT_TEXTE,
            ajouter_article(&ligne.nom),
        )
    };
    bouton_a_lignes(
        &mut enfant,
        &nom,
        Vec2::new(150.0, 48.0),
        fond,
        Stroke::NONE,
        10,
        &[LigneBouton {
            texte,
            taille: 16.0,
            graisse: Graisse::SemiGrasse,
            couleur: couleur_texte,
        }],
        vendable,
    )
    .clicked()
}

#[cfg(test)]
mod tests {
    use super::super::essais::*;
    use super::super::*;
    use super::etat_du_stock;
    use crate::palette::{ALERTE_TEXTE, AVERTISSEMENT_TEXTE, SUCCES_TEXTE};
    use eframe::egui::accesskit::Role;
    use eframe::egui::{Pos2, Rect, Vec2};
    use egui_kittest::kittest::{NodeT, Queryable};

    /// Pain (10), Savon (10), Lait (10), Sel (épuisé), Thé (sans prix), et Café (3 en stock).
    fn app_de_recherche(nom: &str, recherche: &str) -> Application {
        let mut app = app_vendeuse_avec_blocages(nom);
        let etat = app.etat.as_mut().unwrap();
        assert!(etat.ajouter_avec_prix("Café", "0", "4", ""));
        assert!(etat.entree("Café", "3"));
        app.ecran = Ecran::Recherche;
        app.recherche_articles = recherche.to_string();
        app
    }

    #[test]
    fn l_etat_du_stock_s_ecrit_et_se_colore() {
        let (texte, couleur) = etat_du_stock(0, true);
        assert_eq!((texte.as_str(), couleur), (RUPTURE, ALERTE_TEXTE));
        for quantite in 1..=5 {
            let (texte, couleur) = etat_du_stock(quantite, true);
            assert_eq!(texte, plus_que(quantite));
            assert_eq!(couleur, AVERTISSEMENT_TEXTE);
        }
        let (texte, couleur) = etat_du_stock(6, true);
        assert_eq!((texte.as_str(), couleur), (EN_STOCK, SUCCES_TEXTE));
        // Sans prix, le produit n'est pas vendable, quel que soit son stock.
        let (texte, couleur) = etat_du_stock(40, false);
        assert_eq!((texte.as_str(), couleur), (PRIX_MANQUANT, ALERTE_TEXTE));
        assert_eq!(plus_que(3), "Plus que 3 en stock");
    }

    #[test]
    fn la_recherche_ignore_la_casse_et_les_accents_et_compte_les_resultats() {
        let mut h = harness_vente(app_de_recherche("recherche", ""), FENETRE);
        h.get_by_label(&compteur_articles(6));
        saisir(&mut h, CHAMP_RECHERCHE_ARTICLE, "THE");
        h.get_by_label(&compteur_articles(1));
        assert_eq!(h.state().recherche_articles, "THE");
        h.get_by_label("Thé");
        assert!(h.query_by_label("Pain").is_none());
        h.state_mut().recherche_articles = "zzz".to_string();
        h.run();
        h.get_by_label(AUCUN_ARTICLE);
        h.get_by_label(&compteur_articles(0));
    }

    #[test]
    fn ajouter_met_une_unite_au_ticket_sans_quitter_la_recherche() {
        let mut h = harness_vente(app_de_recherche("ajouter", "pain"), FENETRE);
        cliquer(&mut h, &ajouter_article("Pain"));
        assert_eq!(h.state().panier, [("Pain".to_string(), 1)]);
        assert_eq!(h.state().ecran, Ecran::Recherche);
        assert_eq!(h.state().ajoute.as_deref(), Some("Pain"));
        // Une deuxième unité est possible ; changer le texte retire « Ajouté ✓ ».
        cliquer(&mut h, &ajouter_article("Pain"));
        assert_eq!(h.state().panier, [("Pain".to_string(), 2)]);
        saisir(&mut h, CHAMP_RECHERCHE_ARTICLE, "s");
        assert!(h.state().ajoute.is_none());
    }

    #[test]
    fn un_produit_epuise_ou_sans_prix_a_un_bouton_grise() {
        let mut app = app_de_recherche("indisponibles", "");
        // Le texte du bouton est dessiné à la main : on lit ce qui est peint.
        let textes = textes_peints(&mut app, FENETRE, 1.0);
        let indisponibles = textes
            .iter()
            .filter(|t| t.texte == BOUTON_INDISPONIBLE)
            .count();
        assert_eq!(indisponibles, 2);
        let mut h = harness_vente(app, FENETRE);
        for (nom, raison) in [("Sel", EPUISE), ("Thé", PRIX_MANQUANT)] {
            let libelle = produit_non_vendable(nom, raison);
            assert!(est_grise(&h, &libelle), "{libelle} devrait être grisé");
            h.get_by_role_and_label(Role::Button, &libelle).click();
            h.run();
        }
        assert!(h.state().panier.is_empty());
        h.get_by_label(RUPTURE);
        h.get_by_label(PRIX_MANQUANT);
        h.get_by_label(&plus_que(3));
        assert!(!est_grise(&h, &ajouter_article("Café")));
    }

    #[test]
    fn on_ne_depasse_pas_le_stock_depuis_la_recherche() {
        let mut app = app_de_recherche("plein", "savon");
        app.panier = vec![("Savon".to_string(), 10)];
        let mut h = harness_vente(app, FENETRE);
        cliquer(&mut h, &ajouter_article("Savon"));
        assert_eq!(h.state().panier, [("Savon".to_string(), 10)]);
        assert_eq!(
            h.state().toast.as_ref().unwrap().texte,
            stock_limite("Savon", 10)
        );
        assert!(h.state().ajoute.is_none());
    }

    #[test]
    fn un_nom_tres_long_est_coupe_sans_pousser_le_prix() {
        let nom = "Huile d'arachide Lesieur bouteille plastique 1 litre (carton de douze) édition spéciale fêtes";
        let mut app = app_de_recherche("long", "huile");
        let etat = app.etat.as_mut().unwrap();
        assert!(etat.ajouter_avec_prix(nom, "0", "1,50", ""));
        assert!(etat.entree(nom, "20"));
        let h = harness_vente(app, FENETRE);
        let rect_nom = h.get_by_label(nom).rect();
        let prix = h
            .get_all_by_label("1,50\u{a0}€")
            .next()
            .expect("le prix")
            .rect();
        assert!(rect_nom.right() <= prix.left(), "{rect_nom:?} / {prix:?}");
        let bouton = h
            .get_by_role_and_label(Role::Button, &ajouter_article(nom))
            .rect();
        let ecran = Rect::from_min_size(Pos2::ZERO, FENETRE);
        assert!(ecran.contains_rect(bouton), "{bouton:?}");
    }

    #[test]
    fn le_texte_tape_au_scan_est_repris_et_donne_les_correspondances() {
        let mut app = app_vendeuse("scan-vers-recherche");
        let etat = app.etat.as_mut().unwrap();
        assert!(etat.ajouter_avec_prix("Laitue", "0", "1", ""));
        assert!(etat.entree("Laitue", "4"));
        let mut h = harness_vente(app, FENETRE);
        scanner(&mut h, "lai");
        assert_eq!(h.state().ecran, Ecran::Recherche);
        h.get_by_label(&compteur_articles(2)); // Lait et Laitue
        assert!(
            h.get_by_label(CHAMP_RECHERCHE_ARTICLE)
                .accesskit_node()
                .is_focused()
        );
    }

    #[test]
    fn tous_les_textes_peints_de_la_recherche_atteignent_4_5() {
        let mut app = app_de_recherche("contraste-recherche", "");
        let textes = sans_les_textes_grises(textes_peints(&mut app, FENETRE, 1.0));
        for attendu in [
            RECHERCHE_TITRE,
            CHAMP_RECHERCHE_ARTICLE,
            BOUTON_AJOUTER_ARTICLE,
        ] {
            assert!(
                textes.iter().any(|t| t.texte == attendu),
                "« {attendu} » n'est pas peint"
            );
        }
        verifier_contrastes(&textes, "recherche");
        // Après un ajout, le bouton « Ajouté ✓ » (blanc sur vert) et le message.
        app.panier = vec![("Pain".to_string(), 1)];
        app.ajoute = Some("Pain".to_string());
        app.recherche_articles = "pain".to_string();
        let textes = sans_les_textes_grises(textes_peints(&mut app, FENETRE, 1.0));
        assert!(textes.iter().any(|t| t.texte == BOUTON_AJOUTE));
        verifier_contrastes(&textes, "recherche, article ajouté");
        // Aucun résultat.
        app.recherche_articles = "zzz".to_string();
        let textes = textes_peints(&mut app, FENETRE, 1.0);
        assert!(textes.iter().any(|t| t.texte == AUCUN_ARTICLE));
        verifier_contrastes(&textes, "recherche vide");
    }

    #[test]
    fn en_fenetre_etroite_la_recherche_reste_utilisable() {
        let taille = Vec2::new(640.0, 700.0);
        let h = harness_vente(app_de_recherche("etroite", ""), taille);
        let ecran = Rect::from_min_size(Pos2::ZERO, taille);
        for (role, nom) in [
            (Role::TextInput, CHAMP_RECHERCHE_ARTICLE.to_string()),
            (Role::Button, ajouter_article("Pain")),
        ] {
            let rect = h.get_by_role_and_label(role, &nom).rect();
            assert!(ecran.contains_rect(rect), "« {nom} » : {rect:?}");
        }
    }
}
