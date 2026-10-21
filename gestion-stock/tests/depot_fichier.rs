//! Le dépôt fichier obéit aux mêmes règles que le dépôt MySQL (même scénario).

mod commun;

use gestion_stock::depot::{Depot, DepotFichier};
use gestion_stock::horodatage::Horodatage;
use gestion_stock::monnaie::Montant;
use gestion_stock::vente::Moyen;
use std::path::PathBuf;

fn fichier(nom: &str) -> PathBuf {
    let chemin = std::env::temp_dir().join(format!(
        "gestion-stock-depot-{}-{nom}.txt",
        std::process::id()
    ));
    let _ = std::fs::remove_file(&chemin);
    chemin
}

#[test]
fn le_depot_fichier_obeit_aux_regles() {
    let chemin = fichier("conformite");
    let mut depot = DepotFichier::ouvrir(&chemin).unwrap();
    commun::conformite::parcourir_les_regles(&mut depot);
    // Tout est dans le fichier : un dépôt rouvert voit la même chose.
    let rouvert = DepotFichier::ouvrir(&chemin).unwrap().instantane().unwrap();
    assert_eq!(rouvert.pieces().len(), 4);
    rouvert.verifier_journal().unwrap();
    std::fs::remove_file(&chemin).unwrap();
}

#[test]
fn une_operation_refusee_ne_change_pas_le_fichier() {
    let chemin = fichier("refus");
    let mut depot = DepotFichier::ouvrir(&chemin).unwrap();
    depot.ajouter_produit("Pain", 0).unwrap();
    let avant = std::fs::read_to_string(&chemin).unwrap();
    assert!(depot.enregistrer_sortie("Pain", 1).is_err());
    assert!(
        depot
            .encaisser(
                &[("Pain".to_string(), 1)],
                Moyen::Carte,
                Montant(0),
                Horodatage {
                    utc: 1,
                    decalage_minutes: 0
                }
            )
            .is_err()
    );
    assert_eq!(std::fs::read_to_string(&chemin).unwrap(), avant);
    std::fs::remove_file(&chemin).unwrap();
}

#[test]
fn un_fichier_absent_donne_un_depot_vide() {
    let chemin = fichier("absent");
    let mut depot = DepotFichier::ouvrir(&chemin).unwrap();
    assert!(depot.instantane().unwrap().produits().is_empty());
    assert!(
        !chemin.exists(),
        "ouvrir ne crée rien : seule une opération écrit"
    );
}
