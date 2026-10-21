//! Les migrations du schéma : des fichiers SQL numérotés, appliqués dans l'ordre, une seule fois.

use super::{Configuration, erreur_de_base};
use crate::erreur::ErreurStock;
use mysql::Conn;
use mysql::prelude::Queryable;

/// La version du schéma que ce programme sait utiliser.
pub const VERSION_COURANTE: u32 = 1;

/// Les migrations, par numéro de version croissant.
const MIGRATIONS: [(u32, &str); 1] = [(1, include_str!("../../migrations/001_schema.sql"))];

/// Coupe un fichier SQL en instructions : les lignes de commentaire `--` sont retirées, puis le
/// texte est coupé aux `;`. (Les migrations n'ont pas de `;` dans leurs textes.)
pub(super) fn instructions(sql: &str) -> Vec<String> {
    let sans_commentaires: String = sql
        .lines()
        .filter(|ligne| !ligne.trim_start().starts_with("--"))
        .collect::<Vec<_>>()
        .join("\n");
    sans_commentaires
        .split(';')
        .map(str::trim)
        .filter(|instruction| !instruction.is_empty())
        .map(str::to_string)
        .collect()
}

pub(super) fn connecter(config: &Configuration) -> Result<Conn, ErreurStock> {
    Conn::new(config.options()).map_err(|e| erreur_de_base(&e, config))
}

/// La version du schéma de la base : `None` si elle n'a pas encore de table `schema_version`
/// (ou si celle-ci est vide).
pub fn version(config: &Configuration) -> Result<Option<u32>, ErreurStock> {
    let mut connexion = connecter(config)?;
    version_de(&mut connexion, config)
}

pub(super) fn version_de(
    connexion: &mut Conn,
    config: &Configuration,
) -> Result<Option<u32>, ErreurStock> {
    let erreur = |e: mysql::Error| erreur_de_base(&e, config);
    let tables: u64 = connexion
        .query_first(
            "SELECT COUNT(*) FROM information_schema.tables \
             WHERE table_schema = DATABASE() AND table_name = 'schema_version'",
        )
        .map_err(erreur)?
        .unwrap_or(0);
    if tables == 0 {
        return Ok(None);
    }
    let version: Option<Option<u32>> = connexion
        .query_first("SELECT MAX(version) FROM schema_version")
        .map_err(erreur)?;
    Ok(version.flatten())
}

/// Amène la base à la version courante et renvoie la version atteinte. Sans effet si elle y est
/// déjà ; refusée si la base est plus récente que ce programme.
pub fn migrer(config: &Configuration) -> Result<u32, ErreurStock> {
    let mut connexion = connecter(config)?;
    let erreur = |e: mysql::Error| erreur_de_base(&e, config);
    let actuelle = version_de(&mut connexion, config)?;
    if let Some(v) = actuelle
        && v > VERSION_COURANTE
    {
        return Err(ErreurStock::Base(format!(
            "La base est plus récente que ce programme (version {v}, ce programme connaît la version {VERSION_COURANTE}) : mettez le programme à jour."
        )));
    }
    for (numero, sql) in MIGRATIONS {
        if actuelle.is_some_and(|v| v >= numero) {
            continue;
        }
        for instruction in instructions(sql) {
            connexion.query_drop(&instruction).map_err(erreur)?;
        }
        connexion
            .exec_drop("INSERT INTO schema_version (version) VALUES (?)", (numero,))
            .map_err(erreur)?;
    }
    Ok(VERSION_COURANTE)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn un_fichier_sql_se_coupe_en_instructions_sans_commentaires() {
        let sql =
            "-- un titre\nCREATE TABLE a (x INT); -- fin\n\n-- encore\nINSERT INTO a VALUES (1);\n";
        assert_eq!(
            instructions(sql),
            vec![
                "CREATE TABLE a (x INT)".to_string(),
                "-- fin\n\nINSERT INTO a VALUES (1)".to_string()
            ]
        );
    }

    #[test]
    fn la_migration_1_contient_les_sept_tables() {
        let instructions = instructions(MIGRATIONS[0].1);
        assert_eq!(instructions.len(), 7);
        assert!(
            instructions
                .iter()
                .all(|i| i.starts_with("CREATE TABLE IF NOT EXISTS "))
        );
    }
}
