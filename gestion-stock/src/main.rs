use gestion_stock::commandes::{AIDE, Commande, analyser, executer};
use gestion_stock::erreur::ErreurStock;
use gestion_stock::stockage::{charger, sauvegarder};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

/// Le chemin tel que le programme l'utilise, rendu absolu pour que le commerçant sache
/// exactement quel fichier est lu, même si le programme est lancé depuis un autre dossier.
fn chemin_complet(chemin: &Path) -> String {
    std::path::absolute(chemin)
        .unwrap_or_else(|_| chemin.to_path_buf())
        .display()
        .to_string()
}

fn lancer() -> Result<(), ErreurStock> {
    // `args_os` + conversion tolérante : un caractère illisible ne fait pas paniquer le programme.
    let args: Vec<String> = std::env::args_os()
        .skip(1)
        .map(|a| a.to_string_lossy().into_owned())
        .collect();
    let commande = analyser(&args)?;

    // L'aide ne touche pas au fichier de données.
    if commande == Commande::Aide {
        println!("{AIDE}");
        return Ok(());
    }

    let chemin = std::env::var_os("STOCK_FICHIER")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("stock.txt"));
    // Un fichier introuvable donne un stock vide : on le dit, pour que lancer le programme
    // depuis le mauvais dossier ne passe pas pour un stock réellement vide.
    let existait = chemin.exists();
    if !existait {
        eprintln!(
            "Note : fichier de données introuvable (« {} »). On part d'un stock vide.",
            chemin_complet(&chemin)
        );
    }
    let mut stock = charger(&chemin)?;
    let texte = executer(&commande, &mut stock)?;
    // On ne sauvegarde que si tout s'est bien passé et que le stock a changé.
    if commande.modifie_le_stock() {
        sauvegarder(&stock, &chemin)?;
        if !existait {
            eprintln!(
                "Nouveau fichier de données créé : « {} ».",
                chemin_complet(&chemin)
            );
        }
    }
    println!("{texte}");
    Ok(())
}

fn main() -> ExitCode {
    match lancer() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Erreur : {e}");
            ExitCode::FAILURE
        }
    }
}
