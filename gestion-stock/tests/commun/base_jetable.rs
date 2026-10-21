//! Une base MySQL jetable pour un test : créée à la demande, supprimée à la fin.
//!
//! `KORA_TEST_URL` donne un compte qui a le droit de créer des bases, sans nom de base :
//! `mysql://root@127.0.0.1:3306`. Chaque test crée `kora_marche_test_<processus>_<compteur>` et
//! ne touche à aucune autre base.

use gestion_stock::base::{Configuration, migrer};
use mysql::prelude::Queryable;
use mysql::{Conn, Opts};
use std::sync::atomic::{AtomicU32, Ordering};

static COMPTEUR: AtomicU32 = AtomicU32::new(0);

const PREFIXE: &str = "kora_marche_test_";

pub struct BaseJetable {
    nom: String,
    config: Configuration,
    admin: Opts,
}

fn options_admin() -> Opts {
    let url = std::env::var("KORA_TEST_URL").unwrap_or_else(|_| {
        panic!(
            "KORA_TEST_URL manque : lancez par exemple \
             `KORA_TEST_URL=mysql://root@127.0.0.1:3306 cargo test -- --ignored` \
             (MySQL doit être démarré dans XAMPP)."
        )
    });
    Opts::from_url(&url).expect("KORA_TEST_URL n'est pas une URL mysql:// valide")
}

impl BaseJetable {
    /// Une base vide : aucune table.
    pub fn vierge() -> BaseJetable {
        let admin = options_admin();
        let nom = format!(
            "{PREFIXE}{}_{}",
            std::process::id(),
            COMPTEUR.fetch_add(1, Ordering::SeqCst)
        );
        let mut connexion = Conn::new(admin.clone()).expect("connexion administrateur");
        connexion
            .query_drop(format!("CREATE DATABASE `{nom}` CHARACTER SET utf8mb4"))
            .expect("création de la base jetable");
        let config = Configuration {
            hote: admin.get_ip_or_hostname().to_string(),
            port: admin.get_tcp_port(),
            base: nom.clone(),
            utilisateur: admin.get_user().unwrap_or("root").to_string(),
            mot_de_passe: admin.get_pass().unwrap_or("").to_string(),
            caisse: "01".to_string(),
        };
        BaseJetable { nom, config, admin }
    }

    /// Une base avec le schéma de la version courante.
    pub fn creer() -> BaseJetable {
        let base = BaseJetable::vierge();
        migrer(&base.config).expect("migration de la base jetable");
        base
    }

    pub fn config(&self) -> Configuration {
        self.config.clone()
    }

    /// La même base, vue depuis une autre caisse.
    pub fn config_caisse(&self, caisse: &str) -> Configuration {
        Configuration {
            caisse: caisse.to_string(),
            ..self.config.clone()
        }
    }

    /// Une connexion d'administration partageable avec un crochet de test (pour tuer une
    /// connexion du dépôt).
    pub fn connexion_admin(&self) -> std::sync::Arc<std::sync::Mutex<Conn>> {
        std::sync::Arc::new(std::sync::Mutex::new(
            Conn::new(self.admin.clone()).expect("connexion administrateur"),
        ))
    }

    /// Une connexion directe, pour inspecter ou abîmer les tables.
    pub fn connexion(&self) -> Conn {
        Conn::new(self.config.options()).expect("connexion à la base jetable")
    }
}

impl Drop for BaseJetable {
    fn drop(&mut self) {
        // Garde-fou : on ne supprime jamais qu'une base de test.
        assert!(self.nom.starts_with(PREFIXE));
        if let Ok(mut connexion) = Conn::new(self.admin.clone()) {
            let _ = connexion.query_drop(format!("DROP DATABASE IF EXISTS `{}`", self.nom));
        }
    }
}
