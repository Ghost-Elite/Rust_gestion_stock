//! Le dépôt MySQL. Ces tests ont besoin d'un serveur MySQL :
//! `KORA_TEST_URL=mysql://root@127.0.0.1:3306 cargo test --test depot_mysql -- --ignored`.

mod commun;

use commun::base_jetable::BaseJetable;
use gestion_stock::base::DepotMysql;
use gestion_stock::depot::Depot;
use gestion_stock::erreur::ErreurStock;
use gestion_stock::monnaie::{Devise, Montant};
use gestion_stock::reglages::{Profil, Reglages};
use mysql::prelude::Queryable;

fn depot(base: &BaseJetable) -> DepotMysql {
    DepotMysql::connecter(&base.config()).unwrap()
}

fn compte(base: &BaseJetable, table: &str) -> u64 {
    base.connexion()
        .query_first(format!("SELECT COUNT(*) FROM {table}"))
        .unwrap()
        .unwrap()
}

#[test]
#[ignore = "demande un serveur MySQL (KORA_TEST_URL)"]
fn une_base_sans_tables_est_dite_pas_encore_creee() {
    let base = BaseJetable::vierge();
    let e = DepotMysql::connecter(&base.config())
        .map(|_| ())
        .unwrap_err()
        .to_string();
    assert!(
        e.contains("pas encore créée") && e.contains("stock migrer"),
        "{e}"
    );
}

#[test]
#[ignore = "demande un serveur MySQL (KORA_TEST_URL)"]
fn une_base_d_une_ancienne_version_est_dite_pas_a_jour() {
    let base = BaseJetable::creer();
    base.connexion()
        .query_drop("UPDATE schema_version SET version = 0")
        .unwrap();
    let e = DepotMysql::connecter(&base.config())
        .map(|_| ())
        .unwrap_err()
        .to_string();
    assert!(
        e.contains("pas à jour") && e.contains("stock migrer"),
        "{e}"
    );
}

#[test]
#[ignore = "demande un serveur MySQL (KORA_TEST_URL)"]
fn se_connecter_cree_la_ligne_de_la_caisse_du_poste() {
    let base = BaseJetable::creer();
    drop(depot(&base));
    drop(DepotMysql::connecter(&base.config_caisse("04")).unwrap());
    let mut c = base.connexion();
    let lignes: Vec<(String, u64, String)> = c
        .query("SELECT code, dernier_numero, dernier_hash FROM caisse ORDER BY code")
        .unwrap();
    let zeros = "0".repeat(64);
    assert_eq!(
        lignes,
        [
            ("01".to_string(), 0, zeros.clone()),
            ("04".to_string(), 0, zeros)
        ]
    );
}

#[test]
#[ignore = "demande un serveur MySQL (KORA_TEST_URL)"]
fn les_erreurs_de_connexion_ne_montrent_pas_le_mot_de_passe() {
    let base = BaseJetable::creer();
    let mauvais = gestion_stock::base::Configuration {
        mot_de_passe: "pas-le-bon-secret-xyz".to_string(),
        ..base.config()
    };
    let e = DepotMysql::connecter(&mauvais)
        .map(|_| ())
        .unwrap_err()
        .to_string();
    assert!(e.contains("Accès refusé"), "{e}");
    assert!(!e.contains("secret-xyz"), "{e}");
    let injoignable = gestion_stock::base::Configuration { port: 1, ..mauvais };
    let e = DepotMysql::connecter(&injoignable)
        .map(|_| ())
        .unwrap_err()
        .to_string();
    assert!(
        e.contains("ne répond pas") && e.contains("127.0.0.1:1"),
        "{e}"
    );
    assert!(!e.contains("secret-xyz"), "{e}");
}

#[test]
#[ignore = "demande un serveur MySQL (KORA_TEST_URL)"]
fn un_produit_ajoute_se_relit() {
    let base = BaseJetable::creer();
    let mut d = depot(&base);
    d.ajouter_produit("Riz 5kg", 5).unwrap();
    let stock = d.instantane().unwrap();
    let p = &stock.produits()[0];
    assert_eq!(
        (p.nom.as_str(), p.quantite, p.seuil, p.prix, p.taux_bp),
        ("Riz 5kg", 0, 5, None, None)
    );
    // Un autre dépôt, une autre connexion : la base est partagée.
    let autre = depot(&base).instantane().unwrap();
    assert_eq!(autre.produits().len(), 1);
}

#[test]
#[ignore = "demande un serveur MySQL (KORA_TEST_URL)"]
fn la_casse_et_les_espaces_ne_font_pas_un_autre_produit_mais_les_accents_oui() {
    let base = BaseJetable::creer();
    let mut d = depot(&base);
    d.ajouter_produit("Thé", 0).unwrap();
    d.ajouter_produit("the", 0).unwrap();
    d.ajouter_produit("Riz", 0).unwrap();
    assert_eq!(
        d.ajouter_produit(" riz ", 0).unwrap_err(),
        ErreurStock::ProduitExistant("Riz".to_string())
    );
    assert_eq!(compte(&base, "produit"), 3);
}

#[test]
#[ignore = "demande un serveur MySQL (KORA_TEST_URL)"]
fn un_nom_en_emoji_se_relit_a_l_identique() {
    let base = BaseJetable::creer();
    let mut d = depot(&base);
    d.ajouter_produit("Pain 🥖 à l'ancienne", 0).unwrap();
    assert_eq!(
        d.instantane().unwrap().produits()[0].nom,
        "Pain 🥖 à l'ancienne"
    );
}

#[test]
#[ignore = "demande un serveur MySQL (KORA_TEST_URL)"]
fn un_nom_trop_long_pour_la_base_est_refuse_et_190_caracteres_passent() {
    let base = BaseJetable::creer();
    let mut d = depot(&base);
    let limite = "é".repeat(190);
    d.ajouter_produit(&limite, 0).unwrap();
    assert_eq!(d.instantane().unwrap().produits()[0].nom, limite);
    let e = d
        .ajouter_produit(&"é".repeat(191), 0)
        .unwrap_err()
        .to_string();
    assert!(e.contains("190"), "{e}");
    assert_eq!(compte(&base, "produit"), 1);
}

#[test]
#[ignore = "demande un serveur MySQL (KORA_TEST_URL)"]
fn le_prix_et_le_taux_se_relisent_et_se_retirent() {
    let base = BaseJetable::creer();
    let mut d = depot(&base);
    d.ajouter_produit("Pain", 0).unwrap();
    d.definir_prix("pain", Some(Montant(120)), Some(550))
        .unwrap();
    let p = d.instantane().unwrap().produits()[0].clone();
    assert_eq!((p.prix, p.taux_bp), (Some(Montant(120)), Some(550)));
    d.definir_prix("Pain", None, None).unwrap();
    let p = d.instantane().unwrap().produits()[0].clone();
    assert_eq!((p.prix, p.taux_bp), (None, None));
    assert_eq!(
        d.definir_prix("Inconnu", Some(Montant(1)), None)
            .unwrap_err(),
        ErreurStock::ProduitInconnu("Inconnu".to_string())
    );
}

#[test]
#[ignore = "demande un serveur MySQL (KORA_TEST_URL)"]
fn les_reglages_se_relisent_et_se_verrouillent_des_qu_une_piece_existe() {
    let base = BaseJetable::creer();
    let mut d = depot(&base);
    assert_eq!(
        d.instantane().unwrap().reglages(),
        Reglages::default(),
        "valeurs d'origine"
    );
    let xof = Reglages {
        profil: Profil::Europe,
        devise: Devise::Xof,
        taux_defaut_bp: 1800,
    };
    d.definir_reglages(xof).unwrap();
    assert_eq!(d.instantane().unwrap().reglages(), xof);
    assert_eq!(depot(&base).instantane().unwrap().reglages(), xof);

    // Une pièce quelconque (insérée à la main : la vente n'est pas encore réalisée) fige la devise.
    let zeros = "0".repeat(64);
    base.connexion()
        .exec_drop(
            "INSERT INTO piece (caisse, numero, genre, annule_numero, utc, decalage_minutes, \
             total, moyen, recu, rendu, hash_precedent, hash) \
             VALUES ('01', 1, 'V', NULL, 1700000000, 0, 100, 'C', 100, 0, ?, ?)",
            (&zeros, &zeros),
        )
        .unwrap();
    let dollars = Reglages {
        profil: Profil::EtatsUnis,
        devise: Devise::Usd,
        taux_defaut_bp: 825,
    };
    assert_eq!(
        d.definir_reglages(dollars).unwrap_err(),
        ErreurStock::ReglagesVerrouilles
    );
    d.definir_reglages(Reglages {
        taux_defaut_bp: 2000,
        ..xof
    })
    .unwrap();
    assert_eq!(d.instantane().unwrap().reglages().taux_defaut_bp, 2000);
}

#[test]
#[ignore = "demande un serveur MySQL (KORA_TEST_URL)"]
fn une_regle_refusee_ne_laisse_aucune_trace() {
    let base = BaseJetable::creer();
    let mut d = depot(&base);
    d.ajouter_produit("Riz", 0).unwrap();
    for mauvais in ["", "a;b", "a\nb"] {
        assert!(matches!(
            d.ajouter_produit(mauvais, 0),
            Err(ErreurStock::NomInvalide(_))
        ));
    }
    assert!(d.definir_prix("Riz", Some(Montant(-1)), None).is_err());
    assert!(
        d.definir_prix("Riz", Some(Montant(5)), Some(10_001))
            .is_err()
    );
    assert!(d.ajouter_produit("Riz", 0).is_err());
    assert_eq!(compte(&base, "produit"), 1);
    let p = d.instantane().unwrap().produits()[0].clone();
    assert_eq!((p.prix, p.taux_bp), (None, None));
}
