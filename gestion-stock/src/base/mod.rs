//! La base de données MySQL (le MariaDB de XAMPP) : configuration, schéma, dépôt.

pub mod configuration;
pub mod depot_mysql;
pub mod migrations;

pub use configuration::Configuration;
pub use depot_mysql::DepotMysql;
pub use migrations::{VERSION_COURANTE, migrer, version};

use crate::erreur::ErreurStock;

/// Traduit une erreur de la bibliothèque `mysql` en message français pour la personne à la caisse.
/// Le mot de passe n'y figure jamais : même si le serveur le citait, il serait remplacé par `***`.
pub fn erreur_de_base(e: &mysql::Error, config: &Configuration) -> ErreurStock {
    use mysql::{DriverError, Error};
    let adresse = format!("{}:{}", config.hote, config.port);
    let message = match e {
        Error::IoError(_)
        | Error::CodecError(_)
        | Error::DriverError(DriverError::ConnectTimeout | DriverError::CouldNotConnect(_)) => {
            format!(
                "Le serveur de base de données ne répond pas ({adresse}). Vérifiez qu'il est démarré (XAMPP) et que le réseau fonctionne."
            )
        }
        Error::MySqlError(serveur) => match serveur.code {
            1045 => format!(
                "Accès refusé : la base n'accepte pas l'utilisateur « {} » avec ce mot de passe.",
                config.utilisateur
            ),
            1044 => format!(
                "L'utilisateur « {} » n'a pas le droit d'utiliser la base « {} ».",
                config.utilisateur, config.base
            ),
            1049 => format!(
                "La base « {} » n'existe pas : créez-la avec le fichier creer_base.sql.",
                config.base
            ),
            code => format!(
                "Erreur de la base de données (code {code}) : {}",
                serveur.message
            ),
        },
        autre => format!("Erreur de la base de données : {autre}"),
    };
    ErreurStock::Base(masquer(&message, &config.mot_de_passe))
}

/// Remplace le mot de passe, s'il apparaît quelque part dans `texte`, par `***`.
fn masquer(texte: &str, mot_de_passe: &str) -> String {
    if mot_de_passe.is_empty() {
        texte.to_string()
    } else {
        texte.replace(mot_de_passe, "***")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mysql::{DriverError, Error, MySqlError};
    use std::io;

    fn config() -> Configuration {
        Configuration::depuis_texte(
            "hote=192.168.1.12\nport=3307\nbase=boutique\nutilisateur=kora\nmot_de_passe=secret\ncaisse=01\n",
        )
        .unwrap()
    }

    fn serveur(code: u16, message: &str) -> Error {
        Error::MySqlError(MySqlError {
            state: "HY000".to_string(),
            message: message.to_string(),
            code,
        })
    }

    #[test]
    fn un_serveur_injoignable_nomme_l_hote_et_le_port() {
        for e in [
            Error::IoError(io::Error::new(io::ErrorKind::ConnectionRefused, "refusé")),
            Error::DriverError(DriverError::ConnectTimeout),
        ] {
            let texte = erreur_de_base(&e, &config()).to_string();
            assert!(texte.contains("192.168.1.12:3307"), "{texte}");
            assert!(texte.contains("ne répond pas"), "{texte}");
        }
    }

    #[test]
    fn un_acces_refuse_nomme_l_utilisateur_sans_le_mot_de_passe() {
        let e = serveur(
            1045,
            "Access denied for user 'kora'@'localhost' (using password: YES)",
        );
        let texte = erreur_de_base(&e, &config()).to_string();
        assert!(texte.contains("kora"), "{texte}");
        assert!(!texte.contains("secret"), "{texte}");
    }

    #[test]
    fn une_base_inconnue_dit_comment_la_creer() {
        let texte =
            erreur_de_base(&serveur(1049, "Unknown database 'boutique'"), &config()).to_string();
        assert!(
            texte.contains("boutique") && texte.contains("creer_base.sql"),
            "{texte}"
        );
    }

    #[test]
    fn un_message_du_serveur_qui_contiendrait_le_mot_de_passe_est_masque() {
        let texte =
            erreur_de_base(&serveur(1064, "syntax error near 'secret'"), &config()).to_string();
        assert!(!texte.contains("secret"), "{texte}");
        assert!(texte.contains("1064"), "{texte}");
    }
}
