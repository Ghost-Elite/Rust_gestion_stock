//! Tests de bout en bout : on lance le vrai programme `stock`, comme le ferait le commerçant.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// Un fichier de données propre à chaque test, supprimé s'il existe déjà.
fn fichier(nom: &str) -> PathBuf {
    let chemin = std::env::temp_dir().join(format!(
        "gestion-stock-cli-{}-{nom}.txt",
        std::process::id()
    ));
    let _ = std::fs::remove_file(&chemin);
    chemin
}

fn lancer(f: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_stock"))
        .args(args)
        .env("STOCK_FICHIER", f)
        .output()
        .unwrap()
}

fn sortie(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}

fn erreur(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).into_owned()
}

#[test]
fn scenario_complet_persiste_entre_deux_lancements() {
    let f = fichier("scenario");
    assert!(
        lancer(&f, &["ajouter", "Riz 5kg", "--seuil", "5"])
            .status
            .success()
    );
    assert!(lancer(&f, &["entree", "Riz 5kg", "12"]).status.success());
    let o = lancer(&f, &["sortie", "Riz 5kg", "8"]);
    assert!(o.status.success());
    assert!(sortie(&o).contains("ATTENTION : stock bas pour « Riz 5kg »"));
    let liste = sortie(&lancer(&f, &["lister"]));
    assert!(liste.contains("Riz 5kg") && liste.contains("ALERTE"));
    let histo = sortie(&lancer(&f, &["historique"]));
    assert_eq!(
        histo.lines().collect::<Vec<_>>(),
        ["Sortie -8 Riz 5kg", "Entrée +12 Riz 5kg"]
    );
    let _ = std::fs::remove_file(&f);
}

#[test]
fn sortie_impossible_echoue_sans_toucher_au_fichier() {
    let f = fichier("echec");
    lancer(&f, &["ajouter", "Riz", "--seuil", "1"]);
    lancer(&f, &["entree", "Riz", "3"]);
    let avant = std::fs::read_to_string(&f).unwrap();
    let o = lancer(&f, &["sortie", "Riz", "5"]);
    assert_eq!(o.status.code(), Some(1));
    assert!(erreur(&o).contains("Stock insuffisant"));
    assert_eq!(std::fs::read_to_string(&f).unwrap(), avant);
    let _ = std::fs::remove_file(&f);
}

#[test]
fn sans_argument_affiche_l_aide_et_ne_cree_pas_de_fichier() {
    let f = fichier("aide");
    let o = lancer(&f, &[]);
    assert!(o.status.success());
    assert!(sortie(&o).contains("historique"));
    assert!(!f.exists());
}

#[test]
fn commande_inconnue_echoue_avec_le_code_1() {
    let f = fichier("inconnue");
    let o = lancer(&f, &["voler"]);
    assert_eq!(o.status.code(), Some(1));
    assert!(erreur(&o).starts_with("Erreur : "));
    assert!(!f.exists());
}

#[test]
fn fichier_abime_signale_la_ligne_et_reste_intact() {
    let f = fichier("abime");
    std::fs::write(&f, "P;Riz;12;5\nn'importe quoi\n").unwrap();
    let o = lancer(&f, &["entree", "Riz", "1"]);
    assert_eq!(o.status.code(), Some(1));
    assert!(erreur(&o).contains("ligne 2"));
    assert_eq!(
        std::fs::read_to_string(&f).unwrap(),
        "P;Riz;12;5\nn'importe quoi\n"
    );
    let _ = std::fs::remove_file(&f);
}
