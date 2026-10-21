//! Plusieurs postes en même temps : jamais de survente, jamais de trou dans la numérotation, pas
//! d'interblocage. Ces tests ont besoin d'un serveur MySQL :
//! `KORA_TEST_URL=mysql://root@127.0.0.1:3306 cargo test --test concurrence -- --ignored`.

mod commun;

use commun::base_jetable::BaseJetable;
use gestion_stock::base::{Configuration, DepotMysql};
use gestion_stock::depot::Depot;
use gestion_stock::erreur::ErreurStock;
use gestion_stock::horodatage::Horodatage;
use gestion_stock::monnaie::Montant;
use gestion_stock::vente::Moyen;
use std::sync::Barrier;
use std::time::{Duration, Instant};

const QUAND: Horodatage = Horodatage {
    utc: 1_700_000_000,
    decalage_minutes: 0,
};

fn panier(articles: &[(&str, u32)]) -> Vec<(String, u32)> {
    articles.iter().map(|(n, q)| (n.to_string(), *q)).collect()
}

fn produit(config: &Configuration, nom: &str, stock: u32) {
    let mut d = DepotMysql::connecter(config).unwrap();
    d.ajouter_produit(nom, 0).unwrap();
    d.enregistrer_entree(nom, stock).unwrap();
    d.definir_prix(nom, Some(Montant(100)), Some(0)).unwrap();
}

fn stock_de(config: &Configuration, nom: &str) -> u32 {
    let mut d = DepotMysql::connecter(config).unwrap();
    let stock = d.instantane().unwrap();
    stock
        .produits()
        .iter()
        .find(|p| p.nom == nom)
        .unwrap()
        .quantite
}

/// Les numéros des pièces de la caisse, dans l'ordre.
fn numeros(config: &Configuration) -> Vec<u64> {
    let mut d = DepotMysql::connecter(config).unwrap();
    d.instantane()
        .unwrap()
        .pieces()
        .iter()
        .map(|p| p.numero)
        .collect()
}

#[test]
#[ignore = "demande un serveur MySQL (KORA_TEST_URL)"]
fn huit_fils_se_disputent_le_dernier_exemplaire_un_seul_le_vend() {
    let base = BaseJetable::creer();
    let config = base.config();
    produit(&config, "Rare", 1);
    let depart = Barrier::new(8);
    let resultats: Vec<Result<u64, ErreurStock>> = std::thread::scope(|s| {
        let fils: Vec<_> = (0..8)
            .map(|_| {
                s.spawn(|| {
                    let mut d = DepotMysql::connecter(&config).unwrap();
                    depart.wait();
                    d.encaisser(&panier(&[("Rare", 1)]), Moyen::Carte, Montant(0), QUAND)
                        .map(|p| p.numero)
                })
            })
            .collect();
        fils.into_iter().map(|f| f.join().unwrap()).collect()
    });
    let reussies = resultats.iter().filter(|r| r.is_ok()).count();
    assert_eq!(reussies, 1, "{resultats:?}");
    for r in resultats.iter().filter(|r| r.is_err()) {
        assert!(
            matches!(r, Err(ErreurStock::StockInsuffisant { .. })),
            "{r:?}"
        );
    }
    assert_eq!(stock_de(&config, "Rare"), 0);
    assert_eq!(numeros(&config), [1]);
    DepotMysql::connecter(&config)
        .unwrap()
        .verifier_journal()
        .unwrap();
}

#[test]
#[ignore = "demande un serveur MySQL (KORA_TEST_URL)"]
fn deux_caisses_vendent_le_meme_produit_en_meme_temps() {
    let base = BaseJetable::creer();
    produit(&base.config(), "Riz", 100);
    let configs = [base.config_caisse("01"), base.config_caisse("04")];
    let depart = Barrier::new(2);
    std::thread::scope(|s| {
        for config in &configs {
            s.spawn(|| {
                let mut d = DepotMysql::connecter(config).unwrap();
                depart.wait();
                for _ in 0..50 {
                    d.encaisser(&panier(&[("Riz", 1)]), Moyen::Carte, Montant(0), QUAND)
                        .unwrap();
                }
            });
        }
    });
    assert_eq!(stock_de(&base.config(), "Riz"), 0);
    for config in &configs {
        assert_eq!(
            numeros(config),
            (1..=50).collect::<Vec<u64>>(),
            "caisse {}",
            config.caisse
        );
        DepotMysql::connecter(config)
            .unwrap()
            .verifier_journal()
            .unwrap();
    }
}

#[test]
#[ignore = "demande un serveur MySQL (KORA_TEST_URL)"]
fn des_paniers_aux_produits_dans_des_ordres_contraires_ne_s_interbloquent_pas() {
    let base = BaseJetable::creer();
    produit(&base.config(), "Alpha", 200);
    produit(&base.config(), "Zeta", 200);
    let configs = [base.config_caisse("01"), base.config_caisse("04")];
    let paniers = [
        panier(&[("Alpha", 1), ("Zeta", 1)]),
        panier(&[("Zeta", 1), ("Alpha", 1)]),
    ];
    let depart = Barrier::new(2);
    let debut = Instant::now();
    std::thread::scope(|s| {
        for (config, panier) in configs.iter().zip(&paniers) {
            let depart = &depart;
            s.spawn(move || {
                let mut d = DepotMysql::connecter(config).unwrap();
                depart.wait();
                for _ in 0..40 {
                    d.encaisser(panier, Moyen::Carte, Montant(0), QUAND)
                        .unwrap();
                }
            });
        }
    });
    assert!(
        debut.elapsed() < Duration::from_secs(10),
        "{:?}",
        debut.elapsed()
    );
    assert_eq!(stock_de(&base.config(), "Alpha"), 120);
    assert_eq!(stock_de(&base.config(), "Zeta"), 120);
}

#[test]
#[ignore = "demande un serveur MySQL (KORA_TEST_URL)"]
fn des_entrees_et_des_ventes_en_meme_temps_donnent_le_bon_stock() {
    let base = BaseJetable::creer();
    produit(&base.config(), "Lait", 100);
    let configs = [base.config_caisse("01"), base.config_caisse("04")];
    let depart = Barrier::new(3);
    std::thread::scope(|s| {
        for config in &configs {
            let depart = &depart;
            s.spawn(move || {
                let mut d = DepotMysql::connecter(config).unwrap();
                depart.wait();
                for _ in 0..20 {
                    d.encaisser(&panier(&[("Lait", 1)]), Moyen::Carte, Montant(0), QUAND)
                        .unwrap();
                }
            });
        }
        let config = base.config();
        let depart = &depart;
        s.spawn(move || {
            let mut d = DepotMysql::connecter(&config).unwrap();
            depart.wait();
            for _ in 0..50 {
                d.enregistrer_entree("Lait", 1).unwrap();
            }
        });
    });
    assert_eq!(stock_de(&base.config(), "Lait"), 100 + 50 - 40);
    for config in &configs {
        assert_eq!(numeros(config).len(), 20);
    }
}

#[test]
#[ignore = "demande un serveur MySQL (KORA_TEST_URL)"]
fn des_annulations_et_des_ventes_de_deux_caisses_en_meme_temps() {
    let base = BaseJetable::creer();
    produit(&base.config(), "Alpha", 200);
    produit(&base.config(), "Zeta", 200);
    let (une, quatre) = (base.config_caisse("01"), base.config_caisse("04"));
    let depart = Barrier::new(2);
    std::thread::scope(|s| {
        // La caisse 01 vend puis annule aussitôt, trente fois.
        s.spawn(|| {
            let mut d = DepotMysql::connecter(&une).unwrap();
            depart.wait();
            for _ in 0..30 {
                let vente = d
                    .encaisser(
                        &panier(&[("Alpha", 2), ("Zeta", 1)]),
                        Moyen::Carte,
                        Montant(0),
                        QUAND,
                    )
                    .unwrap();
                d.annuler(vente.numero, QUAND).unwrap();
            }
        });
        // La caisse 04 vend les mêmes produits dans l'autre ordre, trente fois.
        s.spawn(|| {
            let mut d = DepotMysql::connecter(&quatre).unwrap();
            depart.wait();
            for _ in 0..30 {
                d.encaisser(
                    &panier(&[("Zeta", 1), ("Alpha", 1)]),
                    Moyen::Carte,
                    Montant(0),
                    QUAND,
                )
                .unwrap();
            }
        });
    });
    assert_eq!(stock_de(&une, "Alpha"), 200 - 30);
    assert_eq!(stock_de(&une, "Zeta"), 200 - 30);
    assert_eq!(numeros(&une).len(), 60, "30 ventes et 30 annulations");
    assert_eq!(numeros(&quatre).len(), 30);
    for config in [&une, &quatre] {
        DepotMysql::connecter(config)
            .unwrap()
            .verifier_journal()
            .unwrap();
    }
}
