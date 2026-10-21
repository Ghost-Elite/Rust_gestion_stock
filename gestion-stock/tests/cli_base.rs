//! Les commandes `migrer`, `importer` et `verifier`, lancées comme le ferait le commerçant.
//! Les tests sans serveur tournent toujours ; les autres :
//! `KORA_TEST_URL=mysql://root@127.0.0.1:3306 cargo test --test cli_base -- --ignored`.

mod commun;

use commun::base_jetable::BaseJetable;
use gestion_stock::base::Configuration;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn temp(nom: &str) -> PathBuf {
    let chemin = std::env::temp_dir().join(format!(
        "gestion-stock-clibase-{}-{nom}",
        std::process::id()
    ));
    let _ = std::fs::remove_file(&chemin);
    chemin
}

fn lancer(conf: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_stock"))
        .args(args)
        .env("KORA_BASE_CONF", conf)
        .env("STOCK_FICHIER", temp("jamais-lu.txt"))
        .output()
        .unwrap()
}

fn sortie(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}

fn erreur(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).into_owned()
}

fn ecrire_conf(nom: &str, c: &Configuration) -> PathBuf {
    let chemin = temp(nom);
    std::fs::write(
        &chemin,
        format!(
            "hote={}\nport={}\nbase={}\nutilisateur={}\nmot_de_passe={}\ncaisse={}\n",
            c.hote, c.port, c.base, c.utilisateur, "x", c.caisse
        ),
    )
    .unwrap();
    chemin
}

#[test]
fn sans_fichier_de_configuration_les_commandes_de_la_base_le_disent() {
    let absent = temp("absent.conf");
    for commande in [&["migrer"][..], &["verifier"], &["importer", "stock.txt"]] {
        let o = lancer(&absent, commande);
        assert_eq!(o.status.code(), Some(1), "{commande:?}");
        let e = erreur(&o);
        assert!(
            e.contains("Aucune base configurée") && e.contains("absent.conf"),
            "{e}"
        );
    }
}

#[test]
fn importer_sans_fichier_donne_le_message_d_utilisation() {
    let o = lancer(&temp("peu-importe.conf"), &["importer"]);
    assert_eq!(o.status.code(), Some(1));
    assert!(erreur(&o).contains("importer"), "{}", erreur(&o));
}

#[test]
fn l_aide_cite_les_commandes_de_la_base() {
    let o = lancer(&temp("peu-importe.conf"), &["aide"]);
    for mot in ["migrer", "importer", "verifier"] {
        assert!(sortie(&o).contains(mot), "{}", sortie(&o));
    }
}

#[test]
#[ignore = "demande un serveur MySQL (KORA_TEST_URL)"]
fn migrer_puis_importer_puis_verifier_en_ligne_de_commande() {
    use gestion_stock::depot::{Depot, DepotFichier};
    use gestion_stock::horodatage::Horodatage;
    use gestion_stock::monnaie::Montant;
    use gestion_stock::vente::Moyen;

    let base = BaseJetable::vierge();
    // Le compte de test (root) n'a pas de mot de passe : on écrit le fichier à la main.
    let conf = ecrire_conf("base.conf", &base.config());
    let o = lancer(&conf, &["migrer"]);
    // Le mot de passe « x » est refusé par root : la commande échoue en le disant, sans l'écrire.
    assert_eq!(o.status.code(), Some(1), "{}", erreur(&o));
    assert!(erreur(&o).contains("Accès refusé"), "{}", erreur(&o));
    assert!(!erreur(&o).contains("mot de passe x"), "{}", erreur(&o));
    std::fs::remove_file(&conf).unwrap();

    // Avec un vrai compte : on en crée un pour la base jetable.
    let mut admin = base.connexion();
    use mysql::prelude::Queryable;
    admin
        .query_drop(format!(
            "CREATE USER IF NOT EXISTS 'kora_test'@'127.0.0.1' IDENTIFIED BY 'x'; \
             GRANT ALL PRIVILEGES ON `{}`.* TO 'kora_test'@'127.0.0.1'",
            base.config().base
        ))
        .unwrap_or_else(|_| {
            admin
                .query_drop("CREATE USER IF NOT EXISTS 'kora_test'@'127.0.0.1' IDENTIFIED BY 'x'")
                .unwrap();
            admin
                .query_drop(format!(
                    "GRANT ALL PRIVILEGES ON `{}`.* TO 'kora_test'@'127.0.0.1'",
                    base.config().base
                ))
                .unwrap();
        });
    let config = Configuration {
        utilisateur: "kora_test".to_string(),
        mot_de_passe: "x".to_string(),
        ..base.config()
    };
    let conf = ecrire_conf("base-ok.conf", &config);
    let o = lancer(&conf, &["migrer"]);
    assert!(o.status.success(), "{}", erreur(&o));
    assert!(sortie(&o).contains("version 1"), "{}", sortie(&o));

    let stock = temp("a-importer.txt");
    let mut d = DepotFichier::ouvrir(&stock).unwrap();
    d.ajouter_produit("Pain", 0).unwrap();
    d.enregistrer_entree("Pain", 10).unwrap();
    d.definir_prix("Pain", Some(Montant(120)), Some(550))
        .unwrap();
    d.encaisser(
        &[("Pain".to_string(), 2)],
        Moyen::Carte,
        Montant(0),
        Horodatage {
            utc: 1_700_000_000,
            decalage_minutes: 0,
        },
    )
    .unwrap();
    let o = lancer(&conf, &["importer", stock.to_str().unwrap()]);
    assert!(o.status.success(), "{}", erreur(&o));
    assert!(
        sortie(&o).contains("1 produits") && sortie(&o).contains("1 pièces"),
        "{}",
        sortie(&o)
    );
    let o = lancer(&conf, &["verifier"]);
    assert!(o.status.success(), "{}", erreur(&o));
    assert!(sortie(&o).contains("cohérent"), "{}", sortie(&o));
    admin
        .query_drop("DROP USER IF EXISTS 'kora_test'@'127.0.0.1'")
        .unwrap();
    for f in [conf, stock] {
        std::fs::remove_file(f).unwrap();
    }
}
