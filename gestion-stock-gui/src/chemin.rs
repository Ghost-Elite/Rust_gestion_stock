//! Où se trouve le fichier de données.
//!
//! Jamais le dossier courant par défaut : un double-clic lance le programme depuis un
//! dossier imprévisible, et le commerçant ne retrouverait pas son stock.

use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Systeme {
    MacOs,
    Windows,
    Autre,
}

pub fn systeme_courant() -> Systeme {
    if cfg!(target_os = "macos") {
        Systeme::MacOs
    } else if cfg!(windows) {
        Systeme::Windows
    } else {
        Systeme::Autre
    }
}

/// Le fichier dans le dossier de données de l'utilisateur. `var` lit une variable
/// d'environnement ; une variable vide compte comme absente.
pub fn chemin_donnees(systeme: Systeme, var: impl Fn(&str) -> Option<String>) -> Option<PathBuf> {
    let lire = |cle: &str| var(cle).filter(|valeur| !valeur.is_empty());
    match systeme {
        Systeme::MacOs => Some(
            PathBuf::from(lire("HOME")?)
                .join("Library")
                .join("Application Support")
                .join("GestionStock")
                .join("stock.txt"),
        ),
        Systeme::Windows => Some(
            PathBuf::from(lire("APPDATA")?)
                .join("GestionStock")
                .join("stock.txt"),
        ),
        Systeme::Autre => {
            let base = match lire("XDG_DATA_HOME") {
                Some(xdg) => PathBuf::from(xdg),
                None => PathBuf::from(lire("HOME")?).join(".local").join("share"),
            };
            Some(base.join("gestion-stock").join("stock.txt"))
        }
    }
}

/// Priorité : `STOCK_FICHIER` non vide, puis le dossier de données, puis « stock.txt ».
pub fn choisir(stock_fichier: Option<String>, donnees: Option<PathBuf>) -> PathBuf {
    stock_fichier
        .filter(|valeur| !valeur.is_empty())
        .map(PathBuf::from)
        .or(donnees)
        .unwrap_or_else(|| PathBuf::from("stock.txt"))
}

pub fn chemin_par_defaut() -> PathBuf {
    choisir(
        std::env::var("STOCK_FICHIER").ok(),
        chemin_donnees(systeme_courant(), |cle| std::env::var(cle).ok()),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env<'a>(paires: &'a [(&'a str, &'a str)]) -> impl Fn(&str) -> Option<String> + 'a {
        move |cle| {
            paires
                .iter()
                .find(|(k, _)| *k == cle)
                .map(|(_, v)| v.to_string())
        }
    }

    #[test]
    fn macos_utilise_application_support() {
        assert_eq!(
            chemin_donnees(Systeme::MacOs, env(&[("HOME", "/Users/awa")])),
            Some(PathBuf::from(
                "/Users/awa/Library/Application Support/GestionStock/stock.txt"
            ))
        );
    }

    #[test]
    fn windows_utilise_appdata() {
        assert_eq!(
            chemin_donnees(
                Systeme::Windows,
                env(&[("APPDATA", "C:\\Users\\Awa\\AppData\\Roaming")])
            ),
            Some(
                PathBuf::from("C:\\Users\\Awa\\AppData\\Roaming")
                    .join("GestionStock")
                    .join("stock.txt")
            )
        );
    }

    #[test]
    fn linux_prefere_xdg_data_home() {
        assert_eq!(
            chemin_donnees(
                Systeme::Autre,
                env(&[("XDG_DATA_HOME", "/home/awa/.data"), ("HOME", "/home/awa")])
            ),
            Some(PathBuf::from("/home/awa/.data/gestion-stock/stock.txt"))
        );
    }

    #[test]
    fn linux_sans_xdg_utilise_local_share() {
        assert_eq!(
            chemin_donnees(Systeme::Autre, env(&[("HOME", "/home/awa")])),
            Some(PathBuf::from(
                "/home/awa/.local/share/gestion-stock/stock.txt"
            ))
        );
    }

    #[test]
    fn une_variable_vide_est_ignoree() {
        assert_eq!(
            chemin_donnees(
                Systeme::Autre,
                env(&[("XDG_DATA_HOME", ""), ("HOME", "/home/awa")])
            ),
            Some(PathBuf::from(
                "/home/awa/.local/share/gestion-stock/stock.txt"
            ))
        );
    }

    #[test]
    fn sans_variable_il_n_y_a_pas_de_dossier_de_donnees() {
        assert_eq!(chemin_donnees(Systeme::MacOs, env(&[])), None);
        assert_eq!(chemin_donnees(Systeme::Windows, env(&[])), None);
        assert_eq!(chemin_donnees(Systeme::Autre, env(&[])), None);
    }

    #[test]
    fn choisir_respecte_la_priorite() {
        let donnees = Some(PathBuf::from("/d/stock.txt"));
        assert_eq!(
            choisir(Some("/x/mon.txt".into()), donnees.clone()),
            PathBuf::from("/x/mon.txt")
        );
        assert_eq!(
            choisir(Some("".into()), donnees.clone()),
            PathBuf::from("/d/stock.txt")
        );
        assert_eq!(choisir(None, donnees), PathBuf::from("/d/stock.txt"));
        assert_eq!(choisir(None, None), PathBuf::from("stock.txt"));
    }
}
