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
use gestion_stock_gui::ui::{Application, Confirmation, Ecran, Modale};

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
    etat.retour = None; // pas de message éphémère sur les images
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
    // L'application est dessinée comme dans la vraie fenêtre (son `App::ui`, sans marge autour),
    // avec la palette et les polices installées avant la première image, comme `main.rs`.
    let mut h = Harness::builder()
        .wgpu()
        .with_size(taille)
        .with_pixels_per_point(1.0)
        // Le banc de test impose le thème sombre APRÈS la création : on force le clair, comme le fait
        // `palette::appliquer` dans la vraie fenêtre (même sur un Mac en mode sombre).
        .with_theme(eframe::egui::Theme::Light)
        .build_eframe(|cc| {
            palette::appliquer(&cc.egui_ctx);
            app
        });
    h.run();
    h.snapshot_options(nom, &SnapshotOptions::new().output_path(dossier));
}

/// Les produits de la maquette, avec leur prix en FCFA et leur stock.
const PRODUITS_KORA: [(&str, &str, &str); 15] = [
    ("Riz parfumé 5 kg", "4750", "61"),
    ("Huile d'arachide 1 L", "1600", "40"),
    ("Lait en poudre 400 g", "2350", "34"),
    ("Lait en poudre 900 g", "4900", "12"),
    ("Lait concentré sucré 397 g", "850", "3"),
    ("Lait UHT 1 L", "1100", "48"),
    ("Lait caillé 500 g", "900", "0"),
    ("Sucre en morceaux 1 kg", "800", "75"),
    ("Eau minérale 1,5 L", "400", "230"),
    ("Baguette", "175", "140"),
    ("Spaghetti 500 g", "650", "30"),
    ("Concentré de tomate 400 g", "700", "30"),
    ("Yaourt nature × 4", "1500", "22"),
    ("Poulet entier", "3400", "9"),
    ("Café moulu 250 g", "1900", "5"),
];

/// La boutique de la maquette : produits en FCFA, TVA 18 % incluse, à 14 h 32. Sert à comparer
/// l'écran, élément par élément, aux captures de `maquette/`.
fn kora_exemple(nom: &str) -> Application {
    use gestion_stock::monnaie::Devise;
    use gestion_stock::reglages::Profil;
    let chemin = std::env::temp_dir().join(format!(
        "gestion-stock-gui-kora-{}-{nom}.txt",
        std::process::id()
    ));
    let _ = std::fs::remove_file(&chemin);
    let mut etat = Etat::ouvrir(chemin).unwrap().avec_horloge(|| Horodatage {
        utc: 1_699_972_320, // 14/11/2023 14:32 (heure locale = UTC)
        decalage_minutes: 0,
    });
    assert!(etat.definir_reglages(Profil::Europe, Devise::Xof, "18"));
    for (produit, prix, stock) in PRODUITS_KORA {
        assert!(etat.ajouter_avec_prix(produit, "0", prix, ""));
        if stock != "0" {
            assert!(etat.entree(produit, stock));
        }
    }
    etat.retour = None;
    let mut app = Application::nouvelle(etat);
    app.ecran = Ecran::Vente;
    app.panier = vec![
        ("Riz parfumé 5 kg".to_string(), 1),
        ("Huile d'arachide 1 L".to_string(), 2),
        ("Eau minérale 1,5 L".to_string(), 6),
        ("Baguette".to_string(), 3),
        ("Yaourt nature × 4".to_string(), 1),
        ("Poulet entier".to_string(), 1),
        ("Concentré de tomate 400 g".to_string(), 2),
    ];
    app.ligne_choisie = Some("Concentré de tomate 400 g".to_string());
    app
}

/// Les ventes du jour de la maquette (écran « Historique ») : dix tickets, un remboursé, le plus
/// récent choisi. L'heure avance de sept minutes en sept minutes.
fn kora_historique() -> Application {
    use gestion_stock::monnaie::Devise;
    use gestion_stock::reglages::Profil;
    use std::{cell::Cell, rc::Rc};
    let chemin = std::env::temp_dir().join(format!(
        "gestion-stock-gui-kora-{}-historique.txt",
        std::process::id()
    ));
    let _ = std::fs::remove_file(&chemin);
    let heure = Rc::new(Cell::new(1_699_968_120_i64)); // 13 h 22
    let horloge = heure.clone();
    let mut etat = Etat::ouvrir(chemin)
        .unwrap()
        .avec_horloge(move || Horodatage {
            utc: horloge.get(),
            decalage_minutes: 0,
        });
    assert!(etat.definir_reglages(Profil::Europe, Devise::Xof, "18"));
    for (produit, prix, stock) in PRODUITS_KORA {
        assert!(etat.ajouter_avec_prix(produit, "0", prix, ""));
        if stock != "0" {
            assert!(etat.entree(produit, "300"));
        }
    }
    let ticket = |lignes: &[(&str, u32)]| -> Vec<(String, u32)> {
        lignes.iter().map(|(n, q)| (n.to_string(), *q)).collect()
    };
    let ventes: [(Vec<(String, u32)>, Moyen); 10] = [
        (
            ticket(&[("Riz parfumé 5 kg", 1), ("Baguette", 2)]),
            Moyen::Carte,
        ),
        (
            ticket(&[
                ("Poulet entier", 3),
                ("Spaghetti 500 g", 3),
                ("Baguette", 3),
            ]),
            Moyen::Especes,
        ),
        (
            ticket(&[("Eau minérale 1,5 L", 5), ("Lait UHT 1 L", 1)]),
            Moyen::Autre,
        ),
        (
            ticket(&[
                ("Riz parfumé 5 kg", 2),
                ("Huile d'arachide 1 L", 1),
                ("Café moulu 250 g", 1),
            ]),
            Moyen::Especes,
        ),
        (
            ticket(&[
                ("Lait en poudre 900 g", 1),
                ("Poulet entier", 1),
                ("Baguette", 4),
            ]),
            Moyen::Carte,
        ),
        (
            ticket(&[
                ("Poulet entier", 1),
                ("Yaourt nature × 4", 1),
                ("Baguette", 3),
            ]),
            Moyen::Especes,
        ),
        (
            ticket(&[("Café moulu 250 g", 1), ("Eau minérale 1,5 L", 1)]),
            Moyen::Especes,
        ),
        (
            ticket(&[
                ("Riz parfumé 5 kg", 1),
                ("Eau minérale 1,5 L", 1),
                ("Sucre en morceaux 1 kg", 1),
            ]),
            Moyen::Autre,
        ),
        (
            ticket(&[("Riz parfumé 5 kg", 4), ("Poulet entier", 1)]),
            Moyen::Especes,
        ),
        (
            ticket(&[
                ("Lait en poudre 400 g", 1),
                ("Sucre en morceaux 1 kg", 2),
                ("Baguette", 2),
                ("Café moulu 250 g", 1),
                ("Yaourt nature × 4", 1),
                ("Poulet entier", 1),
                ("Eau minérale 1,5 L", 1),
            ]),
            Moyen::Carte,
        ),
    ];
    for (i, (panier, moyen)) in ventes.iter().enumerate() {
        assert!(etat.encaisser(panier, *moyen, "100000"));
        if i == 4 {
            assert!(etat.annuler(5)); // le 5e ticket est remboursé
        }
        heure.set(heure.get() + 7 * 60);
    }
    etat.effacer_ticket();
    etat.retour = None;
    let mut app = Application::nouvelle(etat);
    app.ecran = Ecran::Historique;
    app
}

// Un test par capture : le moteur de test veut un seul rendu par test. Toutes à la taille de la
// maquette (1366 × 768), à comparer aux images de
// `docs/superpowers/specs/2026-10-21-kora-marche-pos/maquette/`.

const ECRAN: Vec2 = Vec2::new(1366.0, 768.0);

#[test]
#[ignore = "outil de design : lancer à la main"]
fn apercu_vente_vide() {
    let mut app = application_exemple("vente-vide");
    app.ecran = Ecran::Vente;
    capturer("vente_vide", ECRAN, app);
}

#[test]
#[ignore = "outil de design : lancer à la main"]
fn apercu_vente_panier() {
    capturer("vente_panier", ECRAN, vente_en_cours("vente-panier"));
}

#[test]
#[ignore = "outil de design : lancer à la main"]
fn apercu_vente_confirmation() {
    let mut app = vente_en_cours("vente-confirmation");
    app.modale = Some(Modale::Confirmer(Confirmation::AnnulerVente));
    capturer("vente_confirmation", ECRAN, app);
}

#[test]
#[ignore = "outil de design : lancer à la main"]
fn apercu_paiement_especes() {
    let mut app = vente_en_cours("paiement-especes");
    app.saisie_recu = "50".into();
    app.modale = Some(Modale::Especes);
    capturer("paiement_especes", ECRAN, app);
}

#[test]
#[ignore = "outil de design : lancer à la main"]
fn apercu_paiement_carte() {
    let mut app = vente_en_cours("paiement-carte");
    app.modale = Some(Modale::Terminal {
        moyen: Moyen::Carte,
        refuse: false,
    });
    capturer("paiement_carte", ECRAN, app);
}

#[test]
#[ignore = "outil de design : lancer à la main"]
fn apercu_vente_terminee() {
    let mut app = vente_en_cours("vente-terminee");
    let etat = app.etat.as_mut().unwrap();
    assert!(etat.encaisser(&app.panier, Moyen::Especes, "50"));
    etat.retour = None;
    app.panier.clear();
    app.modale = Some(Modale::VenteTerminee {
        ticket_visible: true,
    });
    capturer("vente_terminee", ECRAN, app);
}

#[test]
#[ignore = "outil de design : lancer à la main"]
fn apercu_recherche() {
    let mut app = application_exemple("recherche");
    app.ouvrir_la_recherche("a".into());
    app.panier = vec![("Riz parfumé 5kg".to_string(), 1)];
    app.ajoute = Some("Riz parfumé 5kg".to_string());
    capturer("recherche", ECRAN, app);
}

#[test]
#[ignore = "outil de design : lancer à la main"]
fn apercu_historique() {
    let mut app = vente_en_cours("historique");
    let etat = app.etat.as_mut().unwrap();
    assert!(etat.encaisser(&app.panier, Moyen::Especes, "50"));
    assert!(etat.encaisser(&[("Lait concentré".to_string(), 3)], Moyen::Carte, ""));
    assert!(etat.encaisser(&[("Riz parfumé 5kg".to_string(), 1)], Moyen::Autre, ""));
    assert!(etat.annuler(3));
    etat.effacer_ticket();
    etat.retour = None;
    app.panier.clear();
    app.ecran = Ecran::Historique;
    app.historique.choisi = Some(1);
    capturer("historique", ECRAN, app);
}

#[test]
#[ignore = "outil de design : lancer à la main"]
fn apercu_stock() {
    let mut app = application_exemple("stock");
    app.selection = Some("Farine de blé 25kg".into());
    app.saisie_quantite = "5".into();
    capturer("stock", ECRAN, app);
}

#[test]
#[ignore = "outil de design : lancer à la main"]
fn apercu_reglages() {
    let mut app = application_exemple("reglages");
    app.ecran = Ecran::Reglages;
    capturer("reglages", ECRAN, app);
}

#[test]
#[ignore = "outil de design : lancer à la main"]
fn apercu_menu() {
    let mut app = vente_en_cours("menu");
    app.modale = Some(Modale::Menu);
    capturer("menu", ECRAN, app);
}

#[test]
#[ignore = "outil de design : lancer à la main"]
fn apercu_vente_etroite() {
    capturer(
        "vente_etroite",
        Vec2::new(640.0, 900.0),
        vente_en_cours("vente-etroite"),
    );
}

#[test]
#[ignore = "outil de design : lancer à la main"]
fn apercu_kora_vente() {
    capturer("kora_vente", ECRAN, kora_exemple("vente"));
}

#[test]
#[ignore = "outil de design : lancer à la main"]
fn apercu_kora_especes() {
    let mut app = kora_exemple("especes");
    app.saisie_recu = "20000".into();
    app.modale = Some(Modale::Especes);
    capturer("kora_especes", ECRAN, app);
}

#[test]
#[ignore = "outil de design : lancer à la main"]
fn apercu_kora_carte() {
    let mut app = kora_exemple("carte");
    app.modale = Some(Modale::Terminal {
        moyen: Moyen::Carte,
        refuse: false,
    });
    capturer("kora_carte", ECRAN, app);
}

#[test]
#[ignore = "outil de design : lancer à la main"]
fn apercu_kora_recherche() {
    let mut app = kora_exemple("recherche");
    app.ouvrir_la_recherche("lait".into());
    capturer("kora_recherche", ECRAN, app);
}

#[test]
#[ignore = "outil de design : lancer à la main"]
fn apercu_kora_historique() {
    capturer("kora_historique", ECRAN, kora_historique());
}

#[test]
#[ignore = "outil de design : lancer à la main"]
fn apercu_kora_sans_article() {
    // Le cas d'un fichier tout juste créé : des produits, mais ni prix ni stock.
    let chemin = std::env::temp_dir().join(format!(
        "gestion-stock-gui-kora-{}-sans-article.txt",
        std::process::id()
    ));
    let _ = std::fs::remove_file(&chemin);
    let mut etat = Etat::ouvrir(chemin).unwrap();
    for produit in ["Lait", "Riz parfumé 5kg", "Café moulu 250g"] {
        assert!(etat.ajouter(produit, "2"));
    }
    etat.retour = None;
    capturer("kora_sans_article", ECRAN, Application::nouvelle(etat));
}
