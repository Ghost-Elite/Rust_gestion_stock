//! Lecture et écriture du stock dans un fichier texte, une ligne par enregistrement :
//!
//! ```text
//! P;Café 250g;7;2     produit : nom ; quantité ; seuil
//! M;E;Café 250g;10    mouvement : E (entrée) ou S (sortie) ; nom ; quantité
//! ```

use crate::erreur::ErreurStock;
use crate::stock::{Genre, Mouvement, Stock};
use std::fs;
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};

/// Traduit une erreur du système en message français qui nomme le fichier concerné.
fn erreur_io(chemin: &Path, e: std::io::Error) -> ErreurStock {
    let raison = match e.kind() {
        ErrorKind::NotFound => "dossier ou fichier introuvable".to_string(),
        ErrorKind::PermissionDenied => "accès refusé".to_string(),
        ErrorKind::IsADirectory => "c'est un dossier, pas un fichier de données".to_string(),
        ErrorKind::InvalidData => "le fichier n'est pas en UTF-8 : réenregistrez-le en UTF-8 \
                                   (ou supprimez les caractères illisibles)"
            .to_string(),
        _ => e.to_string(),
    };
    ErreurStock::Io(format!(
        "impossible d'utiliser « {} » : {raison}.",
        chemin.display()
    ))
}

/// `stock.txt` -> `stock.txt.tmp` : on ajoute `.tmp` au nom complet, pour ne jamais
/// écraser un autre fichier de l'utilisateur (comme un `stock.tmp` à lui).
fn chemin_temporaire(chemin: &Path) -> PathBuf {
    let mut nom = chemin.as_os_str().to_owned();
    nom.push(".tmp");
    PathBuf::from(nom)
}

/// Lit le fichier de données. Un fichier absent (premier lancement) donne un stock vide.
pub fn charger(chemin: &Path) -> Result<Stock, ErreurStock> {
    let contenu = match fs::read_to_string(chemin) {
        Ok(texte) => texte,
        Err(e) if e.kind() == ErrorKind::NotFound => return Ok(Stock::nouveau()),
        Err(e) => return Err(erreur_io(chemin, e)),
    };
    let mut stock = Stock::nouveau();
    for (index, ligne) in contenu.lines().enumerate() {
        if ligne.trim().is_empty() {
            continue;
        }
        lire_ligne(&mut stock, ligne).ok_or(ErreurStock::FichierCorrompu { ligne: index + 1 })?;
    }
    Ok(stock)
}

/// Ajoute au stock l'enregistrement décrit par la ligne. `None` si la ligne est illisible.
fn lire_ligne(stock: &mut Stock, ligne: &str) -> Option<()> {
    let champs: Vec<&str> = ligne.split(';').collect();
    match champs.as_slice() {
        ["P", nom, quantite, seuil] => {
            let quantite: u32 = quantite.parse().ok()?;
            let seuil: u32 = seuil.parse().ok()?;
            stock.ajouter_produit(nom, seuil).ok()?;
            stock.produits.last_mut()?.quantite = quantite;
            Some(())
        }
        ["M", genre, nom, quantite] => {
            let genre = match *genre {
                "E" => Genre::Entree,
                "S" => Genre::Sortie,
                _ => return None,
            };
            stock.historique.push(Mouvement {
                genre,
                nom: nom.to_string(),
                quantite: quantite.parse().ok()?,
            });
            Some(())
        }
        _ => None,
    }
}

/// Écrit tout le stock. On écrit d'abord dans `<fichier>.tmp`, on force l'écriture sur le
/// disque, puis on renomme : si le programme ou l'ordinateur s'arrête en cours de route,
/// c'est l'ancien fichier complet ou le nouveau fichier complet qui reste, jamais un fichier vide.
pub fn sauvegarder(stock: &Stock, chemin: &Path) -> Result<(), ErreurStock> {
    let produits = stock
        .produits
        .iter()
        .map(|p| format!("P;{};{};{}\n", p.nom, p.quantite, p.seuil));
    let mouvements = stock.historique.iter().map(|m| {
        let genre = match m.genre {
            Genre::Entree => "E",
            Genre::Sortie => "S",
        };
        format!("M;{genre};{};{}\n", m.nom, m.quantite)
    });
    let texte: String = produits.chain(mouvements).collect();

    let io = |e: std::io::Error| erreur_io(chemin, e);
    let temporaire = chemin_temporaire(chemin);
    let mut fichier = fs::File::create(&temporaire).map_err(io)?;
    fichier.write_all(texte.as_bytes()).map_err(io)?;
    fichier.sync_all().map_err(io)?;
    drop(fichier);
    fs::rename(&temporaire, chemin).map_err(io)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn chemin_temp(nom: &str) -> PathBuf {
        std::env::temp_dir().join(format!("gestion-stock-{}-{nom}.txt", std::process::id()))
    }

    #[test]
    fn aller_retour_conserve_produits_et_historique() {
        let mut s = Stock::nouveau();
        s.ajouter_produit("Café 250g", 2).unwrap();
        s.enregistrer_entree("café 250g", 10).unwrap();
        s.enregistrer_sortie("Café 250g", 3).unwrap();
        let chemin = chemin_temp("aller-retour");
        sauvegarder(&s, &chemin).unwrap();
        let relu = charger(&chemin).unwrap();
        assert_eq!(relu.produits(), s.produits());
        assert_eq!(relu.historique(), s.historique());
        std::fs::remove_file(&chemin).unwrap();
    }

    #[test]
    fn fichier_absent_donne_un_stock_vide() {
        let s = charger(&chemin_temp("absent")).unwrap();
        assert!(s.produits().is_empty() && s.historique().is_empty());
    }

    #[test]
    fn ligne_illisible_signalee_avec_son_numero() {
        let chemin = chemin_temp("illisible");
        std::fs::write(&chemin, "P;Riz;12;5\nn'importe quoi\n").unwrap();
        assert_eq!(
            charger(&chemin).unwrap_err(),
            ErreurStock::FichierCorrompu { ligne: 2 }
        );
        std::fs::remove_file(&chemin).unwrap();
    }

    #[test]
    fn nombre_non_numerique_signale() {
        let chemin = chemin_temp("non-numerique");
        std::fs::write(&chemin, "P;Riz;beaucoup;5\n").unwrap();
        assert_eq!(
            charger(&chemin).unwrap_err(),
            ErreurStock::FichierCorrompu { ligne: 1 }
        );
        std::fs::remove_file(&chemin).unwrap();
    }

    #[test]
    fn produit_en_double_dans_le_fichier_signale() {
        let chemin = chemin_temp("double");
        std::fs::write(&chemin, "P;Riz;1;0\nP;riz;2;0\n").unwrap();
        assert_eq!(
            charger(&chemin).unwrap_err(),
            ErreurStock::FichierCorrompu { ligne: 2 }
        );
        std::fs::remove_file(&chemin).unwrap();
    }

    #[test]
    fn lignes_vides_ignorees() {
        let chemin = chemin_temp("vides");
        std::fs::write(&chemin, "P;Riz;1;0\n\n").unwrap();
        assert_eq!(charger(&chemin).unwrap().produits().len(), 1);
        std::fs::remove_file(&chemin).unwrap();
    }

    #[test]
    fn la_sauvegarde_ne_laisse_pas_de_fichier_temporaire() {
        let chemin = chemin_temp("tmp");
        sauvegarder(&Stock::nouveau(), &chemin).unwrap();
        assert!(!chemin_temporaire(&chemin).exists());
        std::fs::remove_file(&chemin).unwrap();
    }

    #[test]
    fn erreur_d_ecriture_devient_erreur_io() {
        let chemin = std::env::temp_dir()
            .join("dossier-qui-n-existe-pas-gestion-stock")
            .join("stock.txt");
        assert!(matches!(
            sauvegarder(&Stock::nouveau(), &chemin),
            Err(ErreurStock::Io(_))
        ));
    }

    #[test]
    fn dossier_a_la_place_d_un_fichier_donne_un_message_en_francais() {
        let dossier = std::env::temp_dir();
        let message = charger(&dossier).unwrap_err().to_string();
        assert!(
            message.contains(&dossier.display().to_string()),
            "{message}"
        );
        assert!(message.contains("dossier"), "{message}");
        assert!(!message.contains("os error"), "{message}");
    }

    #[test]
    fn fichier_non_utf8_donne_un_message_en_francais() {
        let chemin = chemin_temp("non-utf8");
        std::fs::write(&chemin, [0xFF, 0xFE, 0x00]).unwrap();
        let message = charger(&chemin).unwrap_err().to_string();
        assert!(message.contains(&chemin.display().to_string()), "{message}");
        assert!(
            message.contains("UTF-8") && message.contains("réenregistrez"),
            "{message}"
        );
        std::fs::remove_file(&chemin).unwrap();
    }

    #[test]
    fn ecriture_dans_un_dossier_inexistant_nomme_le_fichier_en_francais() {
        let chemin = std::env::temp_dir()
            .join("dossier-qui-n-existe-pas-gestion-stock")
            .join("stock.txt");
        let message = sauvegarder(&Stock::nouveau(), &chemin)
            .unwrap_err()
            .to_string();
        assert!(message.contains(&chemin.display().to_string()), "{message}");
        assert!(message.contains("introuvable"), "{message}");
        assert!(!message.contains("os error"), "{message}");
    }

    #[test]
    fn la_sauvegarde_ne_touche_pas_a_un_fichier_voisin_en_tmp() {
        let chemin = chemin_temp("voisin");
        let voisin = chemin.with_extension("tmp");
        std::fs::write(&voisin, "précieux").unwrap();
        sauvegarder(&Stock::nouveau(), &chemin).unwrap();
        assert_eq!(std::fs::read_to_string(&voisin).unwrap(), "précieux");
        std::fs::remove_file(&chemin).unwrap();
        std::fs::remove_file(&voisin).unwrap();
    }
}
