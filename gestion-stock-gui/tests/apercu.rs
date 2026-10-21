//! Aperçus visuels de l'interface (captures d'écran) : un OUTIL pour regarder le design,
//! pas un test de non-régression (le rendu varie d'un ordinateur à l'autre).
//!
//! ```bash
//! APERCU_DOSSIER=/chemin/de/sortie UPDATE_SNAPSHOTS=true \
//!   cargo test --test apercu -- --ignored
//! ```

use eframe::egui::Vec2;
use egui_kittest::{Harness, SnapshotOptions};
use gestion_stock::horodatage::Horodatage;
use gestion_stock::vente::Moyen;
use gestion_stock_gui::etat::Etat;
use gestion_stock_gui::palette;
use gestion_stock_gui::ui::{Application, Ecran, Modale, dessiner};

/// Une boutique d'exemple : des produits en alerte, d'autres corrects, un peu d'historique.
fn application_exemple(nom: &str) -> Application {
    let chemin = std::env::temp_dir().join(format!(
        "gestion-stock-gui-apercu-{}-{nom}.txt",
        std::process::id()
    ));
    let _ = std::fs::remove_file(&chemin);
    let mut etat = Etat::ouvrir(chemin).unwrap().avec_horloge(|| Horodatage {
        utc: 1_700_000_000,
        decalage_minutes: 60,
    });
    for (produit, seuil, stock) in [
        ("Riz parfumé 5kg", "5", "8"),
        ("Sucre en poudre 1kg", "10", "42"),
        ("Huile d'arachide 1 L", "6", "24"),
        ("Farine de blé 25kg", "4", "2"),
        ("Café moulu 250g", "5", "18"),
        ("Savon de Marseille", "3", "0"),
        ("Lait concentré", "8", "31"),
    ] {
        assert!(etat.ajouter(produit, seuil));
        if stock != "0" {
            assert!(etat.entree(produit, stock));
        }
    }
    assert!(etat.sortie("Riz parfumé 5kg", "5")); // il en reste 3, sous le seuil de 5
    assert!(etat.sortie("Café moulu 250g", "2"));
    // Des prix, sauf pour le savon (« Prix manquant ») ; le taux réduit pour les denrées.
    for (produit, prix, taux) in [
        ("Riz parfumé 5kg", "8,90", "5,5"),
        ("Sucre en poudre 1kg", "1,45", "5,5"),
        ("Huile d'arachide 1 L", "3,20", "5,5"),
        ("Farine de blé 25kg", "21,00", "5,5"),
        ("Café moulu 250g", "4,75", "5,5"),
        ("Lait concentré", "1,10", "5,5"),
    ] {
        assert!(etat.definir_prix(produit, prix, taux));
    }
    let mut app = Application::nouvelle(etat);
    app.ecran = Ecran::Stock;
    app
}

/// La vente en cours : sept lignes comme dans la maquette, la dernière choisie.
fn vente_en_cours(nom: &str) -> Application {
    let mut app = application_exemple(nom);
    app.ecran = Ecran::Vente;
    app.panier = vec![
        ("Riz parfumé 5kg".to_string(), 1),
        ("Huile d'arachide 1 L".to_string(), 2),
        ("Café moulu 250g".to_string(), 1),
        ("Lait concentré".to_string(), 3),
        ("Sucre en poudre 1kg".to_string(), 2),
        ("Farine de blé 25kg".to_string(), 1),
    ];
    app.ligne_choisie = Some("Sucre en poudre 1kg".to_string());
    app
}

fn capturer(nom: &str, taille: Vec2, app: Application) {
    let dossier = std::env::var("APERCU_DOSSIER").unwrap_or_else(|_| "tests/snapshots".into());
    let mut h = Harness::builder()
        .wgpu()
        .with_size(taille)
        .with_pixels_per_point(1.0)
        .build_ui_state(|ui, app: &mut Application| dessiner(ui, app), app);
    // Comme dans la vraie fenêtre : la palette et les polices avant la première image.
    palette::appliquer(&h.ctx);
    h.run();
    h.snapshot_options(nom, &SnapshotOptions::new().output_path(dossier));
}

// Un test par capture : le moteur de test veut un seul rendu par test.

#[test]
#[ignore = "outil de design : lancer à la main"]
fn apercu_liste_et_commandes() {
    let mut app = application_exemple("selection");
    app.selection = Some("Farine de blé 25kg".into());
    app.saisie_quantite = "5".into();
    capturer("liste_et_commandes", Vec2::new(1366.0, 768.0), app);
}

#[test]
#[ignore = "outil de design : lancer à la main"]
fn apercu_liste_sans_selection() {
    capturer(
        "liste_sans_selection",
        Vec2::new(1366.0, 768.0),
        application_exemple("sans-selection"),
    );
}

#[test]
#[ignore = "outil de design : lancer à la main"]
fn apercu_fenetre_etroite() {
    let mut app = application_exemple("etroit");
    app.selection = Some("Savon de Marseille".into());
    capturer("fenetre_etroite", Vec2::new(520.0, 760.0), app);
}

#[test]
#[ignore = "outil de design : lancer à la main"]
fn apercu_vente_vide() {
    let mut app = application_exemple("vente-vide");
    app.ecran = Ecran::Vente;
    capturer("vente_vide", Vec2::new(1366.0, 768.0), app);
}

#[test]
#[ignore = "outil de design : lancer à la main"]
fn apercu_vente_panier() {
    capturer(
        "vente_panier",
        Vec2::new(1366.0, 768.0),
        vente_en_cours("vente-panier"),
    );
}

#[test]
#[ignore = "outil de design : lancer à la main"]
fn apercu_paiement_especes() {
    let mut app = vente_en_cours("paiement-especes");
    app.saisie_recu = "50".into();
    app.modale = Some(Modale::Especes);
    capturer("paiement_especes", Vec2::new(1366.0, 768.0), app);
}

#[test]
#[ignore = "outil de design : lancer à la main"]
fn apercu_paiement_carte() {
    let mut app = vente_en_cours("paiement-carte");
    app.modale = Some(Modale::Terminal {
        moyen: Moyen::Carte,
        refuse: false,
    });
    capturer("paiement_carte", Vec2::new(1366.0, 768.0), app);
}

#[test]
#[ignore = "outil de design : lancer à la main"]
fn apercu_vente_terminee() {
    let mut app = vente_en_cours("vente-terminee");
    let etat = app.etat.as_mut().unwrap();
    assert!(etat.encaisser(&app.panier, Moyen::Especes, "50"));
    app.panier.clear();
    app.modale = Some(Modale::VenteTerminee {
        ticket_visible: true,
    });
    capturer("vente_terminee", Vec2::new(1366.0, 768.0), app);
}

#[test]
#[ignore = "outil de design : lancer à la main"]
fn apercu_vente_etroite() {
    capturer(
        "vente_etroite",
        Vec2::new(520.0, 900.0),
        vente_en_cours("vente-etroite"),
    );
}

#[test]
#[ignore = "outil de design : lancer à la main"]
fn apercu_ventes() {
    let mut app = vente_en_cours("ventes");
    let etat = app.etat.as_mut().unwrap();
    assert!(etat.encaisser(&app.panier, Moyen::Especes, "20"));
    assert!(etat.encaisser(&[("Lait concentré".to_string(), 3)], Moyen::Carte, ""));
    app.panier.clear();
    app.ecran = Ecran::Historique;
    app.annulation_en_attente = Some(1);
    capturer("ventes", Vec2::new(1100.0, 720.0), app);
}

#[test]
#[ignore = "outil de design : lancer à la main"]
fn apercu_reglages() {
    let mut app = application_exemple("reglages");
    app.ecran = Ecran::Reglages;
    capturer("reglages", Vec2::new(1100.0, 720.0), app);
}

#[test]
#[ignore = "outil de design : lancer à la main"]
fn apercu_stock_avec_prix() {
    let mut app = application_exemple("stock-prix");
    app.selection = Some("Riz parfumé 5kg".into());
    app.saisie_prix_produit = "8,90".into();
    app.saisie_taux_produit = "5,5".into();
    capturer("stock_avec_prix", Vec2::new(1100.0, 900.0), app);
}
