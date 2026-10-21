//! Le fichier `base.conf` : où est la base, avec quel compte, et quelle est la caisse de ce poste.
//!
//! Format : une ligne `cle=valeur` par réglage, `#` en début de ligne pour un commentaire.
//! Clés : `hote` (défaut `127.0.0.1`), `port` (défaut 3306), `base` (défaut `kora_marche`),
//! `utilisateur`, `mot_de_passe` et `caisse` (deux chiffres) sont obligatoires. Le mot de passe
//! peut contenir `=` ou `#`, mais ni commencer ni finir par une espace.

use crate::erreur::ErreurStock;
use mysql::{Opts, OptsBuilder};
use std::ffi::OsString;
use std::fmt;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Où se trouve la base et qui est ce poste.
#[derive(Clone, PartialEq, Eq)]
pub struct Configuration {
    pub hote: String,
    pub port: u16,
    pub base: String,
    pub utilisateur: String,
    pub mot_de_passe: String,
    /// Le numéro de ce poste, sur deux chiffres : `01`, `04`…
    pub caisse: String,
}

impl fmt::Debug for Configuration {
    /// Le mot de passe n'apparaît jamais, même dans un message de débogage.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Configuration")
            .field("hote", &self.hote)
            .field("port", &self.port)
            .field("base", &self.base)
            .field("utilisateur", &self.utilisateur)
            .field("mot_de_passe", &"***")
            .field("caisse", &self.caisse)
            .finish()
    }
}

fn erreur_ligne(numero: usize, message: &str) -> ErreurStock {
    ErreurStock::Base(format!(
        "Fichier de configuration de la base, ligne {numero} : {message}"
    ))
}

fn manque(cle: &str) -> ErreurStock {
    ErreurStock::Base(format!(
        "Fichier de configuration de la base : il manque « {cle} » (ou sa valeur est vide)."
    ))
}

/// Le chemin du fichier : la variable `KORA_BASE_CONF` si elle est renseignée, sinon
/// `<dossier personnel>/.config/kora-marche/base.conf`.
fn chemin_depuis(variable: Option<OsString>, personnel: Option<OsString>) -> Option<PathBuf> {
    match variable {
        Some(chemin) if !chemin.is_empty() => Some(PathBuf::from(chemin)),
        _ => personnel.filter(|d| !d.is_empty()).map(|dossier| {
            PathBuf::from(dossier)
                .join(".config")
                .join("kora-marche")
                .join("base.conf")
        }),
    }
}

impl Configuration {
    pub fn depuis_texte(texte: &str) -> Result<Configuration, ErreurStock> {
        let mut hote = None;
        let mut port = None;
        let mut base = None;
        let mut utilisateur = None;
        let mut mot_de_passe = None;
        let mut caisse = None;
        for (i, ligne) in texte.lines().enumerate() {
            let numero = i + 1;
            let ligne = ligne.trim();
            if ligne.is_empty() || ligne.starts_with('#') {
                continue;
            }
            let (cle, valeur) = ligne
                .split_once('=')
                .ok_or_else(|| erreur_ligne(numero, "« cle=valeur » est attendu."))?;
            let (cle, valeur) = (cle.trim(), valeur.trim());
            let place = match cle {
                "hote" => &mut hote,
                "base" => &mut base,
                "utilisateur" => &mut utilisateur,
                "mot_de_passe" => &mut mot_de_passe,
                "caisse" => &mut caisse,
                "port" => {
                    if port.is_some() {
                        return Err(erreur_ligne(numero, "« port » est écrit deux fois."));
                    }
                    port = Some(valeur.parse::<u16>().map_err(|_| {
                        erreur_ligne(numero, "« port » doit être un nombre entre 1 et 65535.")
                    })?);
                    continue;
                }
                inconnue => {
                    return Err(erreur_ligne(
                        numero,
                        &format!("clé inconnue : « {inconnue} »."),
                    ));
                }
            };
            if place.is_some() {
                return Err(erreur_ligne(
                    numero,
                    &format!("« {cle} » est écrit deux fois."),
                ));
            }
            *place = Some(valeur.to_string());
        }
        let vide_si_absent = |valeur: Option<String>, cle: &str| match valeur {
            Some(v) if !v.is_empty() => Ok(v),
            _ => Err(manque(cle)),
        };
        let caisse = vide_si_absent(caisse, "caisse")?;
        if caisse.len() != 2 || !caisse.chars().all(|c| c.is_ascii_digit()) {
            return Err(ErreurStock::Base(format!(
                "Fichier de configuration de la base : « caisse » doit avoir exactement deux chiffres (reçu : « {caisse} »)."
            )));
        }
        Ok(Configuration {
            hote: hote
                .filter(|h| !h.is_empty())
                .unwrap_or_else(|| "127.0.0.1".to_string()),
            port: port.unwrap_or(3306),
            base: base
                .filter(|b| !b.is_empty())
                .unwrap_or_else(|| "kora_marche".to_string()),
            utilisateur: vide_si_absent(utilisateur, "utilisateur")?,
            mot_de_passe: vide_si_absent(mot_de_passe, "mot_de_passe")?,
            caisse,
        })
    }

    /// `KORA_BASE_CONF`, sinon `~/.config/kora-marche/base.conf`.
    pub fn chemin_par_defaut() -> Option<PathBuf> {
        chemin_depuis(std::env::var_os("KORA_BASE_CONF"), std::env::var_os("HOME"))
    }

    /// Lit le fichier ; `None` s'il n'existe pas (le programme reste alors en mode fichier).
    pub fn charger(chemin: &Path) -> Result<Option<Configuration>, ErreurStock> {
        let texte = match std::fs::read_to_string(chemin) {
            Ok(texte) => texte,
            Err(e) if e.kind() == ErrorKind::NotFound => return Ok(None),
            Err(e) => {
                return Err(ErreurStock::Base(format!(
                    "Impossible de lire le fichier de configuration de la base « {} » : {e}.",
                    chemin.display()
                )));
            }
        };
        Configuration::depuis_texte(&texte).map(Some)
    }

    /// Vrai si d'autres utilisateurs de l'ordinateur peuvent lire le fichier (il contient un mot
    /// de passe) : sous Unix, des droits au-delà de `600`.
    pub fn lisible_par_d_autres(chemin: &Path) -> bool {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::metadata(chemin).is_ok_and(|m| m.permissions().mode() & 0o077 != 0)
        }
        #[cfg(not(unix))]
        {
            let _ = chemin;
            false
        }
    }

    /// Les options de connexion : 3 secondes pour joindre le serveur, 5 pour lire ou écrire.
    pub fn options(&self) -> Opts {
        OptsBuilder::new()
            .ip_or_hostname(Some(self.hote.clone()))
            .tcp_port(self.port)
            .db_name(Some(self.base.clone()))
            .user(Some(self.utilisateur.clone()))
            .pass(Some(self.mot_de_passe.clone()))
            .tcp_connect_timeout(Some(Duration::from_secs(3)))
            .read_timeout(Some(Duration::from_secs(5)))
            .write_timeout(Some(Duration::from_secs(5)))
            .into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    const COMPLET: &str = "\
# la base de la boutique
hote=192.168.1.12
port=3307
base=boutique
utilisateur=kora
mot_de_passe=s3cret=ok
caisse=04
";

    #[test]
    fn un_fichier_complet_se_lit() {
        let c = Configuration::depuis_texte(COMPLET).unwrap();
        assert_eq!(c.hote, "192.168.1.12");
        assert_eq!(c.port, 3307);
        assert_eq!(c.base, "boutique");
        assert_eq!(c.utilisateur, "kora");
        assert_eq!(
            c.mot_de_passe, "s3cret=ok",
            "le mot de passe peut contenir ="
        );
        assert_eq!(c.caisse, "04");
    }

    #[test]
    fn les_valeurs_par_defaut_s_appliquent() {
        let c =
            Configuration::depuis_texte("utilisateur=kora\nmot_de_passe=x\ncaisse=01\n").unwrap();
        assert_eq!(
            (c.hote.as_str(), c.port, c.base.as_str()),
            ("127.0.0.1", 3306, "kora_marche")
        );
    }

    #[test]
    fn commentaires_et_lignes_vides_sont_ignores() {
        let texte = "\n   \n# un commentaire\nutilisateur = kora \n\nmot_de_passe=x\ncaisse=01\r\n";
        let c = Configuration::depuis_texte(texte).unwrap();
        assert_eq!((c.utilisateur.as_str(), c.caisse.as_str()), ("kora", "01"));
    }

    #[test]
    fn la_caisse_a_deux_chiffres() {
        for mauvaise in ["1", "ab", "001", "", "0 "] {
            let texte = format!("utilisateur=kora\nmot_de_passe=x\ncaisse={mauvaise}\n");
            assert!(
                Configuration::depuis_texte(&texte).is_err(),
                "« {mauvaise} »"
            );
        }
    }

    #[test]
    fn ce_qui_manque_ou_est_faux_est_refuse_avec_son_numero_de_ligne() {
        let e = |texte: &str| Configuration::depuis_texte(texte).unwrap_err().to_string();
        assert!(
            e("utilisateur=k\nmot_de_passe=x\n").contains("caisse"),
            "caisse manquante"
        );
        assert!(e("mot_de_passe=x\ncaisse=01\n").contains("utilisateur"));
        assert!(e("utilisateur=k\ncaisse=01\n").contains("mot_de_passe"));
        assert!(e("utilisateur=k\nmot_de_passe=\ncaisse=01\n").contains("mot_de_passe"));
        let inconnue = e("utilisateur=k\nmot_de_passe=x\ncaisse=01\nserveur=a\n");
        assert!(
            inconnue.contains("ligne 4") && inconnue.contains("serveur"),
            "{inconnue}"
        );
        let repetee = e("utilisateur=k\nutilisateur=l\nmot_de_passe=x\ncaisse=01\n");
        assert!(
            repetee.contains("ligne 2") && repetee.contains("utilisateur"),
            "{repetee}"
        );
        let sans_egal = e("utilisateur=k\nbonjour\nmot_de_passe=x\ncaisse=01\n");
        assert!(sans_egal.contains("ligne 2"), "{sans_egal}");
        let port = e("port=abc\nutilisateur=k\nmot_de_passe=x\ncaisse=01\n");
        assert!(port.contains("ligne 1") && port.contains("port"), "{port}");
    }

    #[test]
    fn le_mot_de_passe_ne_sort_pas_dans_debug() {
        let c = Configuration::depuis_texte(COMPLET).unwrap();
        let texte = format!("{c:?}");
        assert!(texte.contains("***"), "{texte}");
        assert!(!texte.contains("s3cret"), "{texte}");
    }

    #[test]
    fn le_chemin_par_defaut_suit_la_variable_puis_le_dossier_personnel() {
        use std::ffi::OsString;
        use std::path::PathBuf;
        let var = Some(OsString::from("/tmp/ma-base.conf"));
        let home = Some(OsString::from("/Users/moi"));
        assert_eq!(
            chemin_depuis(var.clone(), home.clone()),
            Some(PathBuf::from("/tmp/ma-base.conf"))
        );
        assert_eq!(
            chemin_depuis(None, home.clone()),
            Some(PathBuf::from("/Users/moi/.config/kora-marche/base.conf"))
        );
        assert_eq!(
            chemin_depuis(Some(OsString::new()), home),
            Some(PathBuf::from("/Users/moi/.config/kora-marche/base.conf"))
        );
        assert_eq!(chemin_depuis(None, None), None);
    }

    fn fichier_temp(nom: &str, contenu: &str) -> std::path::PathBuf {
        let chemin =
            std::env::temp_dir().join(format!("kora-base-{}-{nom}.conf", std::process::id()));
        std::fs::write(&chemin, contenu).unwrap();
        chemin
    }

    #[test]
    fn charger_un_chemin_absent_donne_none() {
        let absent = Path::new("/chemin/qui/n/existe/pas/base.conf");
        assert_eq!(
            Configuration::charger(absent).unwrap().map(|c| c.caisse),
            None
        );
    }

    #[test]
    fn charger_lit_le_fichier() {
        let chemin = fichier_temp("lire", COMPLET);
        let c = Configuration::charger(&chemin).unwrap().unwrap();
        assert_eq!(c.caisse, "04");
        std::fs::remove_file(chemin).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn les_droits_trop_larges_sont_signales() {
        use std::os::unix::fs::PermissionsExt;
        let chemin = fichier_temp("droits", COMPLET);
        std::fs::set_permissions(&chemin, std::fs::Permissions::from_mode(0o600)).unwrap();
        assert!(!Configuration::lisible_par_d_autres(&chemin));
        std::fs::set_permissions(&chemin, std::fs::Permissions::from_mode(0o644)).unwrap();
        assert!(Configuration::lisible_par_d_autres(&chemin));
        std::fs::remove_file(chemin).unwrap();
    }

    #[test]
    fn les_options_ont_des_delais() {
        let c = Configuration::depuis_texte(COMPLET).unwrap();
        let options = c.options();
        assert_eq!(options.get_ip_or_hostname(), "192.168.1.12");
        assert_eq!(options.get_tcp_port(), 3307);
        assert_eq!(options.get_db_name(), Some("boutique"));
        assert_eq!(options.get_user(), Some("kora"));
        assert!(options.get_tcp_connect_timeout().is_some());
        assert!(options.get_read_timeout().is_some());
        assert!(options.get_write_timeout().is_some());
    }
}
