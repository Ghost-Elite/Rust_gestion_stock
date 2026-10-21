use gestion_stock::base::{Configuration, DepotMysql, importer, migrer};
use gestion_stock::commandes::{AIDE, Commande, analyser, executer};
use gestion_stock::depot::Depot;
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

/// La configuration de la base ; une erreur claire si le fichier `base.conf` n'existe pas.
fn configuration_de_la_base() -> Result<Configuration, ErreurStock> {
    let chemin = Configuration::chemin_par_defaut().ok_or_else(|| {
        ErreurStock::Base(
            "Impossible de trouver le dossier personnel : indiquez le fichier de configuration avec la variable KORA_BASE_CONF."
                .to_string(),
        )
    })?;
    if Configuration::lisible_par_d_autres(&chemin) {
        eprintln!(
            "Attention : « {} » contient un mot de passe et peut être lu par d'autres utilisateurs. Tapez : chmod 600 \"{}\"",
            chemin.display(),
            chemin.display()
        );
    }
    Configuration::charger(&chemin)?.ok_or_else(|| {
        ErreurStock::Base(format!(
            "Aucune base configurée : créez le fichier « {} » (voir le README, section « La base MySQL (XAMPP) »).",
            chemin.display()
        ))
    })
}

/// Les commandes qui parlent à la base MySQL. Elles ne lisent pas le fichier de données (sauf
/// `importer`, qui en copie un dans la base).
fn lancer_la_base(commande: &Commande) -> Result<(), ErreurStock> {
    let config = configuration_de_la_base()?;
    match commande {
        Commande::Migrer => {
            let version = migrer(&config)?;
            println!("Base « {} » à jour : version {version}.", config.base);
        }
        Commande::Importer { fichier } => {
            let resume = importer(&config, Path::new(fichier))?;
            println!(
                "Import terminé : {} produits, {} mouvements, {} pièces. Le fichier « {fichier} » n'a pas été modifié.",
                resume.produits, resume.mouvements, resume.pieces
            );
        }
        Commande::VerifierBase => {
            DepotMysql::connecter(&config)?.verifier_journal()?;
            println!(
                "Le journal de la caisse {} est cohérent : aucune pièce n'a été modifiée.",
                config.caisse
            );
        }
        _ => unreachable!("seules les commandes de la base arrivent ici"),
    }
    Ok(())
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

    if commande.concerne_la_base() {
        return lancer_la_base(&commande);
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
