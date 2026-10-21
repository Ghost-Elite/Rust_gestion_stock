//! Importer un `stock.txt` dans la base. Ces tests ont besoin d'un serveur MySQL :
//! `KORA_TEST_URL=mysql://root@127.0.0.1:3306 cargo test --test import_mysql -- --ignored`.

mod commun;

use commun::base_jetable::BaseJetable;
use gestion_stock::base::{DepotMysql, importer};
use gestion_stock::depot::{Depot, DepotFichier};
use gestion_stock::erreur::ErreurStock;
use gestion_stock::horodatage::Horodatage;
use gestion_stock::monnaie::{Devise, Montant};
use gestion_stock::reglages::{Profil, Reglages};
use gestion_stock::stock::Stock;
use gestion_stock::stockage::sauvegarder;
use gestion_stock::vente::Moyen;
use mysql::prelude::Queryable;
use std::path::PathBuf;
use std::time::{Duration, Instant};

const QUAND: Horodatage = Horodatage {
    utc: 1_700_000_000,
    decalage_minutes: 0,
};

fn fichier(nom: &str) -> PathBuf {
    let chemin = std::env::temp_dir().join(format!(
        "gestion-stock-import-{}-{nom}.txt",
        std::process::id()
    ));
    let _ = std::fs::remove_file(&chemin);
    chemin
}

fn panier(articles: &[(&str, u32)]) -> Vec<(String, u32)> {
    articles.iter().map(|(n, q)| (n.to_string(), *q)).collect()
}

fn compte(base: &BaseJetable, table: &str) -> u64 {
    base.connexion()
        .query_first(format!("SELECT COUNT(*) FROM {table}"))
        .unwrap()
        .unwrap()
}

/// Un `stock.txt` réaliste : des réglages en FCFA, des produits (dont un en emoji), des entrées,
/// des ventes de plusieurs moyens et une annulation.
fn stock_txt(nom: &str) -> PathBuf {
    let chemin = fichier(nom);
    let mut d = DepotFichier::ouvrir(&chemin).unwrap();
    d.definir_reglages(Reglages {
        profil: Profil::Europe,
        devise: Devise::Xof,
        taux_defaut_bp: 1800,
    })
    .unwrap();
    for (produit, prix) in [
        ("Riz parfumé 5 kg", 4750),
        ("Pain 🥖", 175),
        ("Thé", 900),
        ("the", 800),
    ] {
        d.ajouter_produit(produit, 3).unwrap();
        d.enregistrer_entree(produit, 50).unwrap();
        d.definir_prix(produit, Some(Montant(prix)), None).unwrap();
    }
    d.enregistrer_sortie("the", 2).unwrap();
    d.encaisser(
        &panier(&[("Riz parfumé 5 kg", 2), ("Pain 🥖", 3)]),
        Moyen::Especes,
        Montant(20_000),
        QUAND,
    )
    .unwrap();
    d.encaisser(&panier(&[("Thé", 1)]), Moyen::Carte, Montant(0), QUAND)
        .unwrap();
    d.encaisser(&panier(&[("the", 4)]), Moyen::Autre, Montant(0), QUAND)
        .unwrap();
    d.annuler(2, QUAND).unwrap();
    chemin
}

#[test]
#[ignore = "demande un serveur MySQL (KORA_TEST_URL)"]
fn un_stock_txt_s_importe_et_se_relit_a_l_identique() {
    let base = BaseJetable::creer();
    let chemin = stock_txt("complet");
    let avant = std::fs::read_to_string(&chemin).unwrap();
    let en_memoire = gestion_stock::stockage::charger(&chemin).unwrap();

    let resume = importer(&base.config(), &chemin).unwrap();
    assert_eq!(
        (resume.produits, resume.mouvements, resume.pieces),
        (4, en_memoire.historique().len(), 4)
    );

    let mut depot = DepotMysql::connecter(&base.config()).unwrap();
    let en_base = depot.instantane().unwrap();
    assert_eq!(en_base.reglages(), en_memoire.reglages());
    assert_eq!(en_base.produits(), en_memoire.produits());
    assert_eq!(en_base.historique(), en_memoire.historique());
    assert_eq!(
        en_base.pieces(),
        en_memoire.pieces(),
        "mêmes numéros, mêmes empreintes"
    );
    depot.verifier_journal().unwrap();
    // La suite : la base continue la numérotation et la chaîne du fichier.
    let suite = depot
        .encaisser(&panier(&[("Thé", 1)]), Moyen::Carte, Montant(0), QUAND)
        .unwrap();
    assert_eq!(suite.numero, 5);
    assert_eq!(suite.hash_precedent, en_memoire.pieces()[3].hash);
    assert_eq!(
        std::fs::read_to_string(&chemin).unwrap(),
        avant,
        "le fichier n'est jamais modifié"
    );
    std::fs::remove_file(chemin).unwrap();
}

#[test]
#[ignore = "demande un serveur MySQL (KORA_TEST_URL)"]
fn un_deuxieme_import_ou_une_base_deja_utilisee_est_refuse() {
    let base = BaseJetable::creer();
    let chemin = stock_txt("deux-fois");
    importer(&base.config(), &chemin).unwrap();
    let avant = (
        compte(&base, "produit"),
        compte(&base, "mouvement"),
        compte(&base, "piece"),
    );
    let e = importer(&base.config(), &chemin).unwrap_err().to_string();
    assert!(e.contains("n'est pas vide"), "{e}");
    assert_eq!(
        (
            compte(&base, "produit"),
            compte(&base, "mouvement"),
            compte(&base, "piece")
        ),
        avant
    );

    let autre = BaseJetable::creer();
    DepotMysql::connecter(&autre.config())
        .unwrap()
        .ajouter_produit("Sel", 0)
        .unwrap();
    let e = importer(&autre.config(), &chemin).unwrap_err().to_string();
    assert!(e.contains("n'est pas vide"), "{e}");
    std::fs::remove_file(chemin).unwrap();
}

#[test]
#[ignore = "demande un serveur MySQL (KORA_TEST_URL)"]
fn un_fichier_a_la_chaine_alteree_est_refuse_avant_toute_ecriture() {
    let base = BaseJetable::creer();
    let chemin = stock_txt("altere");
    let texte = std::fs::read_to_string(&chemin).unwrap();
    // On change la quantité de la première ligne de la pièce 3.
    let abime: String = texte
        .lines()
        .map(|l| {
            if l.starts_with("L;3;") {
                l.replacen(";4;", ";5;", 1)
            } else {
                l.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";
    assert_ne!(abime, texte);
    std::fs::write(&chemin, abime).unwrap();
    let e = importer(&base.config(), &chemin).unwrap_err();
    assert_eq!(e, ErreurStock::JournalAltere { numero: 3 });
    for table in ["produit", "mouvement", "piece", "piece_ligne", "reglage"] {
        assert_eq!(compte(&base, table), 0, "{table} doit rester vide");
    }
    std::fs::remove_file(chemin).unwrap();
}

#[test]
#[ignore = "demande un serveur MySQL (KORA_TEST_URL)"]
fn un_fichier_absent_ou_une_base_pas_prete_est_refuse() {
    let base = BaseJetable::creer();
    let e = importer(&base.config(), &fichier("n-existe-pas"))
        .unwrap_err()
        .to_string();
    assert!(e.contains("n'existe pas"), "{e}");
    let vierge = BaseJetable::vierge();
    let chemin = stock_txt("base-vierge");
    let e = importer(&vierge.config(), &chemin).unwrap_err().to_string();
    assert!(e.contains("stock migrer"), "{e}");
    std::fs::remove_file(chemin).unwrap();
}

#[test]
#[ignore = "demande un serveur MySQL (KORA_TEST_URL)"]
fn soixante_mille_pieces_s_importent_sans_depasser_le_paquet_du_serveur() {
    let base = BaseJetable::creer();
    let chemin = fichier("gros");
    let mut stock = Stock::nouveau();
    stock.ajouter_produit("Riz", 0).unwrap();
    stock.enregistrer_entree("Riz", 100_000).unwrap();
    stock
        .definir_prix("Riz", Some(Montant(1000)), Some(1800))
        .unwrap();
    for i in 0..60_000 {
        let quand = Horodatage {
            utc: 1_700_000_000 + i,
            decalage_minutes: 0,
        };
        stock
            .encaisser(&panier(&[("Riz", 1)]), Moyen::Carte, Montant(0), quand)
            .unwrap();
    }
    stock.verifier_journal().unwrap();
    sauvegarder(&stock, &chemin).unwrap();

    let debut = Instant::now();
    let resume = importer(&base.config(), &chemin).unwrap();
    let duree = debut.elapsed();
    assert_eq!(resume.pieces, 60_000);
    assert!(duree < Duration::from_secs(60), "import en {duree:?}");
    assert_eq!(compte(&base, "piece"), 60_000);
    assert_eq!(compte(&base, "piece_ligne"), 60_000);
    let debut = Instant::now();
    DepotMysql::connecter(&base.config())
        .unwrap()
        .verifier_journal()
        .unwrap();
    assert!(
        debut.elapsed() < Duration::from_secs(60),
        "vérification en {:?}",
        debut.elapsed()
    );
    std::fs::remove_file(chemin).unwrap();
}
