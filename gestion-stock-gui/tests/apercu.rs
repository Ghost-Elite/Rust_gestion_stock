//! Aperçus visuels de l'interface (captures d'écran) : un OUTIL pour regarder le design,
//! pas un test de non-régression (le rendu varie d'un ordinateur à l'autre).
//!
//! ```bash
//! APERCU_DOSSIER=/chemin/de/sortie UPDATE_SNAPSHOTS=true \
//!   cargo test --test apercu -- --ignored
//! ```

use eframe::egui::Vec2;
use egui_kittest::{Harness, SnapshotOptions};
use gestion_stock_gui::etat::Etat;
use gestion_stock_gui::palette;
use gestion_stock_gui::ui::{Application, dessiner};

/// Une boutique d'exemple : des produits en alerte, d'autres corrects, un peu d'historique.
fn application_exemple(nom: &str) -> Application {
    let chemin = std::env::temp_dir().join(format!(
        "gestion-stock-gui-apercu-{}-{nom}.txt",
        std::process::id()
    ));
    let _ = std::fs::remove_file(&chemin);
    let mut etat = Etat::ouvrir(chemin).unwrap();
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
    Application::nouvelle(etat)
}

fn capturer(nom: &str, taille: Vec2, app: Application) {
    let dossier = std::env::var("APERCU_DOSSIER").unwrap_or_else(|_| "tests/snapshots".into());
    let mut h = Harness::builder()
        .wgpu()
        .with_size(taille)
        .with_pixels_per_point(1.5)
        .build_ui_state(
            |ui, app: &mut Application| {
                palette::appliquer(ui.ctx());
                dessiner(ui, app);
            },
            app,
        );
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
    capturer("liste_et_commandes", Vec2::new(1040.0, 700.0), app);
}

#[test]
#[ignore = "outil de design : lancer à la main"]
fn apercu_liste_sans_selection() {
    capturer(
        "liste_sans_selection",
        Vec2::new(1040.0, 700.0),
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
