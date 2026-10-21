//! Le schéma de la base : tables, versions, unicité. Ces tests ont besoin d'un serveur MySQL :
//! `KORA_TEST_URL=mysql://root@127.0.0.1:3306 cargo test --test migrations -- --ignored`.

mod commun;

use commun::base_jetable::BaseJetable;
use gestion_stock::base::{VERSION_COURANTE, migrer, version};
use mysql::prelude::Queryable;

const TABLES: [&str; 7] = [
    "schema_version",
    "reglage",
    "caisse",
    "produit",
    "mouvement",
    "piece",
    "piece_ligne",
];

#[test]
#[ignore = "demande un serveur MySQL (KORA_TEST_URL)"]
fn une_base_vierge_n_a_pas_de_version_puis_migrer_cree_les_sept_tables() {
    let base = BaseJetable::vierge();
    assert_eq!(version(&base.config()).unwrap(), None);
    assert_eq!(migrer(&base.config()).unwrap(), VERSION_COURANTE);
    assert_eq!(version(&base.config()).unwrap(), Some(VERSION_COURANTE));
    let mut c = base.connexion();
    let mut presentes: Vec<String> = c
        .query(
            "SELECT table_name FROM information_schema.tables \
             WHERE table_schema = DATABASE() ORDER BY table_name",
        )
        .unwrap();
    presentes.sort();
    let mut attendues: Vec<String> = TABLES.iter().map(|t| t.to_string()).collect();
    attendues.sort();
    assert_eq!(presentes, attendues);
}

#[test]
#[ignore = "demande un serveur MySQL (KORA_TEST_URL)"]
fn migrer_deux_fois_ne_change_rien() {
    let base = BaseJetable::creer();
    assert_eq!(migrer(&base.config()).unwrap(), VERSION_COURANTE);
    let mut c = base.connexion();
    let lignes: u64 = c
        .query_first("SELECT COUNT(*) FROM schema_version")
        .unwrap()
        .unwrap();
    assert_eq!(lignes, 1);
}

#[test]
#[ignore = "demande un serveur MySQL (KORA_TEST_URL)"]
fn une_base_plus_recente_que_le_programme_est_refusee() {
    let base = BaseJetable::creer();
    base.connexion()
        .query_drop("INSERT INTO schema_version (version) VALUES (99)")
        .unwrap();
    let e = migrer(&base.config()).unwrap_err().to_string();
    assert!(e.contains("plus récente") && e.contains("99"), "{e}");
}

#[test]
#[ignore = "demande un serveur MySQL (KORA_TEST_URL)"]
fn les_noms_de_produit_sont_uniques_casse_ignoree_accents_comptes() {
    let base = BaseJetable::creer();
    let mut c = base.connexion();
    let ajouter = |c: &mut mysql::Conn, nom: &str| {
        c.exec_drop(
            "INSERT INTO produit (nom, nom_cle) VALUES (?, ?)",
            (nom, nom.trim().to_lowercase()),
        )
    };
    ajouter(&mut c, "Thé").unwrap();
    ajouter(&mut c, "the").unwrap(); // un autre produit : les accents comptent
    ajouter(&mut c, "Riz").unwrap();
    assert!(
        ajouter(&mut c, " riz ").is_err(),
        "Riz et riz sont le même produit"
    );
    ajouter(&mut c, "Pain 🥖").unwrap();
    let nom: String = c
        .exec_first("SELECT nom FROM produit WHERE nom_cle = ?", ("pain 🥖",))
        .unwrap()
        .unwrap();
    assert_eq!(nom, "Pain 🥖");
}

#[test]
#[ignore = "demande un serveur MySQL (KORA_TEST_URL)"]
fn une_piece_et_son_annulation_sont_uniques_par_caisse() {
    let base = BaseJetable::creer();
    let mut c = base.connexion();
    let zeros = "0".repeat(64);
    c.query_drop("INSERT INTO caisse (code) VALUES ('01'), ('04')")
        .unwrap();
    let mut inserer = |caisse: &str, numero: u64, genre: &str, annule: Option<u64>| {
        c.exec_drop(
            "INSERT INTO piece (caisse, numero, genre, annule_numero, utc, decalage_minutes, \
             total, moyen, recu, rendu, hash_precedent, hash) \
             VALUES (?, ?, ?, ?, 1700000000, 0, 100, 'C', 100, 0, ?, ?)",
            (caisse, numero, genre, annule, &zeros, &zeros),
        )
    };
    inserer("01", 1, "V", None).unwrap();
    assert!(
        inserer("01", 1, "V", None).is_err(),
        "(caisse, numéro) est unique"
    );
    inserer("04", 1, "V", None).unwrap(); // une autre caisse : même numéro permis
    inserer("01", 2, "A", Some(1)).unwrap();
    assert!(
        inserer("01", 3, "A", Some(1)).is_err(),
        "une vente ne s'annule qu'une fois"
    );
    inserer("04", 2, "A", Some(1)).unwrap(); // l'annulation de la vente 1 de la caisse 04
    assert!(
        inserer("01", 4, "A", None).is_err(),
        "une annulation nomme sa vente"
    );
    assert!(
        inserer("01", 5, "V", Some(1)).is_err(),
        "une vente n'annule rien"
    );
}
