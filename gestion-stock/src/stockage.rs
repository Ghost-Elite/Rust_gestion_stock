//! Lecture et écriture du stock dans un fichier texte, une ligne par enregistrement
//! (format version 2) :
//!
//! ```text
//! F;2                           version du format
//! R;europe;EUR;2000             réglages : profil ; devise ; taux par défaut (centièmes de %)
//! P;Café 250g;7;2;350;550       produit : nom ; quantité ; seuil ; prix ; taux (vides s'ils manquent)
//! M;E;Café 250g;10              mouvement : E (entrée) ou S (sortie) ; nom ; quantité
//! S;1;V;1700000000;60;800;E;1000;200;;<précédente>;<empreinte>
//!                               pièce du journal : numéro ; genre (V vente, A annulation) ; heure UTC ;
//!                               décalage en minutes ; total ; moyen (E, C, X) ; reçu ; rendu ;
//!                               vente annulée ; empreinte précédente ; empreinte
//! L;1;Pain;3;120;550            ligne de la pièce 1 : nom ; quantité ; prix ; taux
//! ```
//!
//! Les fichiers de l'ancien format (sans ligne `F`, produits à quatre champs) se lisent
//! toujours, sans perte.

use crate::erreur::ErreurStock;
use crate::horodatage::Horodatage;
use crate::monnaie::{Devise, Montant};
use crate::reglages::{Profil, Reglages};
use crate::stock::{Genre, Mouvement, Stock};
use crate::vente::{GenrePiece, LigneVente, Moyen, Piece};
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
        ["F", version] => (*version == "2").then_some(()),
        ["R", profil, devise, taux] => {
            let reglages = Reglages {
                profil: Profil::depuis_code(profil)?,
                devise: Devise::depuis_code(devise)?,
                taux_defaut_bp: taux.parse().ok()?,
            };
            stock.definir_reglages(reglages).ok()
        }
        ["P", nom, quantite, seuil] => lire_produit(stock, nom, quantite, seuil, "", ""),
        ["P", nom, quantite, seuil, prix, taux] => {
            lire_produit(stock, nom, quantite, seuil, prix, taux)
        }
        [
            "S",
            numero,
            genre,
            utc,
            decalage,
            total,
            moyen,
            recu,
            rendu,
            annulee,
            precedente,
            hash,
        ] => {
            let genre = match (*genre, *annulee) {
                ("V", "") => GenrePiece::Vente,
                ("A", de) if !de.is_empty() => GenrePiece::Annulation {
                    de: de.parse().ok()?,
                },
                _ => return None,
            };
            if !est_empreinte(precedente) || !est_empreinte(hash) {
                return None;
            }
            stock.pieces.push(Piece {
                numero: numero.parse().ok()?,
                genre,
                horodatage: Horodatage {
                    utc: utc.parse().ok()?,
                    decalage_minutes: decalage.parse().ok()?,
                },
                lignes: Vec::new(),
                moyen: Moyen::depuis_code(moyen)?,
                total: Montant(total.parse().ok()?),
                recu: Montant(recu.parse().ok()?),
                rendu: Montant(rendu.parse().ok()?),
                hash_precedent: precedente.to_string(),
                hash: hash.to_string(),
            });
            Some(())
        }
        ["L", numero, nom, quantite, prix, taux] => {
            let numero: u64 = numero.parse().ok()?;
            // Une ligne appartient à la pièce S qui la précède immédiatement.
            let piece = stock.pieces.last_mut()?;
            if piece.numero != numero {
                return None;
            }
            piece.lignes.push(LigneVente {
                nom: nom.to_string(),
                quantite: quantite.parse().ok()?,
                prix: Montant(prix.parse().ok()?),
                taux_bp: taux.parse().ok()?,
            });
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

/// Une empreinte SHA-256 : 64 chiffres hexadécimaux minuscules.
fn est_empreinte(texte: &str) -> bool {
    texte.len() == 64
        && texte
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

/// Ajoute un produit lu dans le fichier. `prix` et `taux` vides veulent dire « pas défini ».
fn lire_produit(
    stock: &mut Stock,
    nom: &str,
    quantite: &str,
    seuil: &str,
    prix: &str,
    taux: &str,
) -> Option<()> {
    let quantite: u32 = quantite.parse().ok()?;
    let seuil: u32 = seuil.parse().ok()?;
    let prix = match prix {
        "" => None,
        texte => Some(Montant(texte.parse().ok()?)),
    };
    let taux = match taux {
        "" => None,
        texte => Some(texte.parse().ok()?),
    };
    stock.ajouter_produit(nom, seuil).ok()?;
    // Vérifie aussi les bornes : prix négatif ou énorme, taux au-delà de 100 %.
    stock.definir_prix(nom, prix, taux).ok()?;
    stock.produits.last_mut()?.quantite = quantite;
    Some(())
}

/// Écrit tout le stock. On écrit d'abord dans `<fichier>.tmp`, on force l'écriture sur le
/// disque, puis on renomme : si le programme ou l'ordinateur s'arrête en cours de route,
/// c'est l'ancien fichier complet ou le nouveau fichier complet qui reste, jamais un fichier vide.
pub fn sauvegarder(stock: &Stock, chemin: &Path) -> Result<(), ErreurStock> {
    let entete = {
        let r = stock.reglages;
        format!(
            "F;2\nR;{};{};{}\n",
            r.profil.code(),
            r.devise.code(),
            r.taux_defaut_bp
        )
    };
    let produits = stock.produits.iter().map(|p| {
        format!(
            "P;{};{};{};{};{}\n",
            p.nom,
            p.quantite,
            p.seuil,
            p.prix.map_or(String::new(), |m| m.0.to_string()),
            p.taux_bp.map_or(String::new(), |t| t.to_string())
        )
    });
    let mouvements = stock.historique.iter().map(|m| {
        let genre = match m.genre {
            Genre::Entree => "E",
            Genre::Sortie => "S",
        };
        format!("M;{genre};{};{}\n", m.nom, m.quantite)
    });
    let pieces = stock.pieces.iter().flat_map(|piece| {
        let (genre, annulee) = match piece.genre {
            GenrePiece::Vente => ("V", String::new()),
            GenrePiece::Annulation { de } => ("A", de.to_string()),
        };
        let entete_piece = format!(
            "S;{};{};{};{};{};{};{};{};{};{};{}\n",
            piece.numero,
            genre,
            piece.horodatage.utc,
            piece.horodatage.decalage_minutes,
            piece.total.0,
            piece.moyen.code(),
            piece.recu.0,
            piece.rendu.0,
            annulee,
            piece.hash_precedent,
            piece.hash
        );
        let numero = piece.numero;
        let lignes = piece.lignes.iter().map(move |l| {
            format!(
                "L;{numero};{};{};{};{}\n",
                l.nom, l.quantite, l.prix.0, l.taux_bp
            )
        });
        std::iter::once(entete_piece).chain(lignes)
    });
    let texte: String = std::iter::once(entete)
        .chain(produits)
        .chain(mouvements)
        .chain(pieces)
        .collect();

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

    #[test]
    fn aller_retour_v2_conserve_prix_taux_et_reglages() {
        use crate::monnaie::{Devise, Montant};
        use crate::reglages::{Profil, Reglages};
        let mut s = Stock::nouveau();
        s.definir_reglages(Reglages {
            profil: Profil::EtatsUnis,
            devise: Devise::Usd,
            taux_defaut_bp: 825,
        })
        .unwrap();
        s.ajouter_produit("Pain", 5).unwrap();
        s.ajouter_produit("Lait", 2).unwrap();
        s.ajouter_produit("Sel", 1).unwrap();
        s.definir_prix("Pain", Some(Montant(120)), Some(550))
            .unwrap();
        s.definir_prix("Lait", Some(Montant(95)), None).unwrap();
        s.enregistrer_entree("Pain", 7).unwrap();
        let chemin = chemin_temp("v2");
        sauvegarder(&s, &chemin).unwrap();
        let texte = std::fs::read_to_string(&chemin).unwrap();
        assert!(texte.starts_with("F;2\nR;etats-unis;USD;825\n"), "{texte}");
        let relu = charger(&chemin).unwrap();
        assert_eq!(relu.produits(), s.produits());
        assert_eq!(relu.reglages(), s.reglages());
        assert_eq!(relu.historique(), s.historique());
        std::fs::remove_file(&chemin).unwrap();
    }

    #[test]
    fn un_fichier_de_l_ancien_format_se_lit_sans_perte() {
        use crate::reglages::Reglages;
        let chemin = chemin_temp("ancien");
        std::fs::write(&chemin, "P;Riz;12;5\nM;E;Riz;12\n").unwrap();
        let s = charger(&chemin).unwrap();
        let p = &s.produits()[0];
        assert_eq!((p.nom.as_str(), p.quantite, p.seuil), ("Riz", 12, 5));
        assert_eq!((p.prix, p.taux_bp), (None, None));
        assert_eq!(s.historique().len(), 1);
        assert_eq!(s.reglages(), Reglages::default());
        std::fs::remove_file(&chemin).unwrap();
    }

    #[test]
    fn un_prix_ou_un_taux_illisible_donne_fichier_corrompu_avec_la_ligne() {
        for (contenu, ligne) in [
            ("F;2\nP;Riz;1;0;abc;\n", 2),
            ("F;2\nP;Riz;1;0;-5;\n", 2),
            ("F;2\nP;Riz;1;0;;abc\n", 2),
            ("F;2\nP;Riz;1;0;;10001\n", 2),
            ("F;2\nP;Riz;1;0;1000000001;\n", 2),
            ("F;2\nP;Riz;1;0;100\n", 2), // cinq champs : ni l'ancien format ni le nouveau
        ] {
            let chemin = chemin_temp("prix-illisible");
            std::fs::write(&chemin, contenu).unwrap();
            assert_eq!(
                charger(&chemin).unwrap_err(),
                ErreurStock::FichierCorrompu { ligne },
                "{contenu:?}"
            );
            std::fs::remove_file(&chemin).unwrap();
        }
    }

    #[test]
    fn une_ligne_r_ou_une_version_illisible_donne_fichier_corrompu() {
        for (contenu, ligne) in [
            ("F;2\nR;mars;EUR;2000\n", 2),
            ("F;2\nR;europe;JPY;2000\n", 2),
            ("F;2\nR;europe;EUR;abc\n", 2),
            ("F;2\nR;europe;EUR;10001\n", 2),
            ("F;3\n", 1),
            ("F;\n", 1),
        ] {
            let chemin = chemin_temp("reglages-illisibles");
            std::fs::write(&chemin, contenu).unwrap();
            assert_eq!(
                charger(&chemin).unwrap_err(),
                ErreurStock::FichierCorrompu { ligne },
                "{contenu:?}"
            );
            std::fs::remove_file(&chemin).unwrap();
        }
    }

    // ---- Journal : lignes S et L -----------------------------------------------------------

    use crate::horodatage::Horodatage;
    use crate::monnaie::Montant;
    use crate::vente::{GenrePiece, Moyen};

    const QUAND: Horodatage = Horodatage {
        utc: 1_700_000_000,
        decalage_minutes: 60,
    };

    fn stock_avec_ventes() -> Stock {
        let mut s = Stock::nouveau();
        for (nom, stock, prix, taux) in [("Pain", 20, 120, 550), ("Savon", 20, 250, 2000)] {
            s.ajouter_produit(nom, 0).unwrap();
            s.enregistrer_entree(nom, stock).unwrap();
            s.definir_prix(nom, Some(Montant(prix)), Some(taux))
                .unwrap();
        }
        let panier = |articles: &[(&str, u32)]| -> Vec<(String, u32)> {
            articles.iter().map(|(n, q)| (n.to_string(), *q)).collect()
        };
        s.encaisser(
            &panier(&[("Pain", 3), ("Savon", 1)]),
            Moyen::Especes,
            Montant(2000),
            QUAND,
        )
        .unwrap();
        s.encaisser(&panier(&[("Pain", 1)]), Moyen::Carte, Montant(0), QUAND)
            .unwrap();
        s.annuler(1, QUAND).unwrap();
        s
    }

    #[test]
    fn aller_retour_complet_avec_journal() {
        let s = stock_avec_ventes();
        let chemin = chemin_temp("journal");
        sauvegarder(&s, &chemin).unwrap();
        let relu = charger(&chemin).unwrap();
        assert_eq!(relu.pieces(), s.pieces());
        assert_eq!(relu.pieces().len(), 3);
        assert_eq!(relu.pieces()[2].genre, GenrePiece::Annulation { de: 1 });
        assert_eq!(relu.produits(), s.produits());
        assert_eq!(relu.historique(), s.historique());
        assert_eq!(relu.verifier_journal(), Ok(()));
        // Le fichier relu se réécrit à l'identique.
        let chemin2 = chemin_temp("journal-2");
        sauvegarder(&relu, &chemin2).unwrap();
        assert_eq!(
            std::fs::read_to_string(&chemin).unwrap(),
            std::fs::read_to_string(&chemin2).unwrap()
        );
        std::fs::remove_file(&chemin).unwrap();
        std::fs::remove_file(&chemin2).unwrap();
    }

    #[test]
    fn le_fichier_contient_les_lignes_s_et_l_documentees() {
        let s = stock_avec_ventes();
        let chemin = chemin_temp("format-journal");
        sauvegarder(&s, &chemin).unwrap();
        let texte = std::fs::read_to_string(&chemin).unwrap();
        let lignes: Vec<&str> = texte.lines().collect();
        let i = lignes.iter().position(|l| l.starts_with("S;1;")).unwrap();
        let champs: Vec<&str> = lignes[i].split(';').collect();
        assert_eq!(
            &champs[..10],
            [
                "S",
                "1",
                "V",
                "1700000000",
                "60",
                "610",
                "E",
                "2000",
                "1390",
                ""
            ]
        );
        assert_eq!(champs[10], "0".repeat(64));
        assert_eq!(champs[11].len(), 64);
        assert_eq!(lignes[i + 1], "L;1;Pain;3;120;550");
        assert_eq!(lignes[i + 2], "L;1;Savon;1;250;2000");
        std::fs::remove_file(&chemin).unwrap();
    }

    #[test]
    fn une_piece_modifiee_dans_le_fichier_se_charge_mais_ne_se_verifie_pas() {
        let s = stock_avec_ventes();
        let chemin = chemin_temp("piece-modifiee");
        sauvegarder(&s, &chemin).unwrap();
        let texte = std::fs::read_to_string(&chemin).unwrap();
        // On « corrige » à la main le total de la vente n° 2 (120 -> 1).
        let modifie: String = texte
            .lines()
            .map(|l| {
                if l.starts_with("S;2;") {
                    l.replacen(";120;", ";1;", 1)
                } else {
                    l.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
            + "\n";
        assert_ne!(modifie, texte);
        std::fs::write(&chemin, modifie).unwrap();
        let relu = charger(&chemin).unwrap();
        assert_eq!(
            relu.verifier_journal(),
            Err(ErreurStock::JournalAltere { numero: 2 })
        );
        std::fs::remove_file(&chemin).unwrap();
    }

    #[test]
    fn une_ligne_s_ou_l_illisible_donne_fichier_corrompu_avec_son_numero() {
        let h = "a".repeat(64);
        let z = "0".repeat(64);
        let cas = [
            // moyen inconnu
            format!("F;2\nS;1;V;1;0;100;Z;100;0;;{z};{h}\n"),
            // nombre illisible
            format!("F;2\nS;1;V;abc;0;100;C;100;0;;{z};{h}\n"),
            // empreinte absente ou trop courte
            "F;2\nS;1;V;1;0;100;C;100;0;;;\n".to_string(),
            format!("F;2\nS;1;V;1;0;100;C;100;0;;{z};abc\n"),
            // genre inconnu, annulation sans vente annulée, vente avec vente annulée
            format!("F;2\nS;1;Q;1;0;100;C;100;0;;{z};{h}\n"),
            format!("F;2\nS;1;A;1;0;-100;C;-100;0;;{z};{h}\n"),
            format!("F;2\nS;1;V;1;0;100;C;100;0;7;{z};{h}\n"),
            // ligne L sans pièce S, ou d'un autre numéro
            "F;2\nL;1;Pain;1;100;0\n".to_string(),
            format!("F;2\nS;1;V;1;0;100;C;100;0;;{z};{h}\nL;2;Pain;1;100;0\n"),
            // ligne L à quantité illisible
            format!("F;2\nS;1;V;1;0;100;C;100;0;;{z};{h}\nL;1;Pain;-1;100;0\n"),
        ];
        for (index, contenu) in cas.iter().enumerate() {
            let chemin = chemin_temp("journal-illisible");
            std::fs::write(&chemin, contenu).unwrap();
            let ligne_attendue = if index == 8 || index == 9 { 3 } else { 2 };
            assert_eq!(
                charger(&chemin).unwrap_err(),
                ErreurStock::FichierCorrompu {
                    ligne: ligne_attendue
                },
                "cas {index} : {contenu:?}"
            );
            std::fs::remove_file(&chemin).unwrap();
        }
    }
}
