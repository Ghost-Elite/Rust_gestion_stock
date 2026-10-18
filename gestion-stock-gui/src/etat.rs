//! État de l'application : toute la logique, sans aucune fenêtre (donc testable).
//!
//! Règle commune à toutes les actions : on travaille sur une COPIE du stock ; si l'action
//! réussit on sauvegarde la copie, et seulement si la sauvegarde réussit la copie remplace
//! le stock. Ainsi la liste affichée ne diffère jamais du fichier, même si l'écriture échoue.

use gestion_stock::erreur::ErreurStock;
use gestion_stock::stock::{Genre, Stock};
use gestion_stock::stockage::{charger, sauvegarder};
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

/// Le message affiché en bas de la fenêtre après chaque action.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Retour {
    Succes(String),
    Erreur(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LigneProduit {
    pub nom: String,
    pub quantite: u32,
    pub seuil: u32,
    pub en_alerte: bool,
}

#[derive(Debug)]
pub struct Etat {
    stock: Stock,
    chemin: PathBuf,
    pub retour: Option<Retour>,
}

fn lire_quantite(texte: &str) -> Result<u32, ErreurStock> {
    let texte = texte.trim();
    if texte.is_empty() {
        return Err(ErreurStock::Utilisation(
            "Quantité manquante. Saisissez un nombre entier positif, par exemple 12.".to_string(),
        ));
    }
    texte.parse().map_err(|_| {
        ErreurStock::Utilisation(format!(
            "Quantité invalide : « {texte} ». Saisissez un nombre entier positif, par exemple 12."
        ))
    })
}

/// Un seuil vide veut dire 0.
fn lire_seuil(texte: &str) -> Result<u32, ErreurStock> {
    let texte = texte.trim();
    if texte.is_empty() {
        return Ok(0);
    }
    texte.parse().map_err(|_| {
        ErreurStock::Utilisation(format!(
            "Seuil d'alerte invalide : « {texte} ». Saisissez un nombre entier positif, par exemple 5, ou laissez vide pour 0."
        ))
    })
}

/// Le chemin tel qu'on l'affiche : toujours absolu, pour que la personne sache exactement
/// quel fichier est utilisé, même si `STOCK_FICHIER` est un chemin relatif.
pub fn chemin_complet(chemin: &Path) -> String {
    std::path::absolute(chemin)
        .unwrap_or_else(|_| chemin.to_path_buf())
        .display()
        .to_string()
}

const FICHIER_MODIFIE: &str = "Le fichier de données a été modifié par un autre programme ou une autre fenêtre : rien n'a été enregistré. Fermez puis rouvrez le programme pour voir le stock à jour.";

impl Etat {
    /// Crée le dossier du fichier s'il manque. Un fichier absent donne un stock vide,
    /// et on le dit ; un fichier abîmé est une erreur (il n'est jamais écrasé).
    pub fn ouvrir(chemin: PathBuf) -> Result<Etat, ErreurStock> {
        if let Some(dossier) = chemin.parent().filter(|d| !d.as_os_str().is_empty()) {
            std::fs::create_dir_all(dossier).map_err(|e| {
                let raison = match e.kind() {
                    ErrorKind::PermissionDenied => "accès refusé".to_string(),
                    _ => e.to_string(),
                };
                ErreurStock::Io(format!(
                    "impossible de créer le dossier « {} » : {raison}.",
                    dossier.display()
                ))
            })?;
        }
        let existait = chemin.exists();
        let stock = charger(&chemin)?;
        let retour = (!existait).then(|| {
            Retour::Succes(
                "Aucun fichier de données trouvé : un stock vide est utilisé. Le fichier sera créé au premier enregistrement."
                    .to_string(),
            )
        });
        Ok(Etat {
            stock,
            chemin,
            retour,
        })
    }

    pub fn chemin(&self) -> &Path {
        &self.chemin
    }

    /// Produits triés par nom ; `recherche` : texte contenu dans le nom, sans casse
    /// ni espaces autour.
    pub fn lignes(&self, recherche: &str, alertes_seulement: bool) -> Vec<LigneProduit> {
        let cherche = recherche.trim().to_lowercase();
        self.stock
            .produits()
            .into_iter()
            .filter(|p| p.nom.to_lowercase().contains(&cherche))
            .filter(|p| !alertes_seulement || p.en_alerte())
            .map(|p| LigneProduit {
                nom: p.nom.clone(),
                quantite: p.quantite,
                seuil: p.seuil,
                en_alerte: p.en_alerte(),
            })
            .collect()
    }

    /// Du plus récent au plus ancien.
    pub fn historique(&self, limite: usize) -> Vec<String> {
        self.stock
            .historique()
            .iter()
            .rev()
            .take(limite)
            .map(|m| match m.genre {
                Genre::Entree => format!("Entrée +{} {}", m.quantite, m.nom),
                Genre::Sortie => format!("Sortie -{} {}", m.quantite, m.nom),
            })
            .collect()
    }

    /// Refuse d'enregistrer si le fichier n'est plus tel que cet écran l'a chargé : sinon on
    /// l'écraserait avec un stock périmé et les mouvements faits ailleurs (autre fenêtre,
    /// programme en ligne de commande) seraient perdus sans un mot.
    fn verifier_fichier_inchange(&self) -> Result<(), ErreurStock> {
        let sur_disque = charger(&self.chemin)?;
        if sur_disque.produits() == self.stock.produits()
            && sur_disque.historique() == self.stock.historique()
        {
            Ok(())
        } else {
            Err(ErreurStock::Utilisation(FICHIER_MODIFIE.to_string()))
        }
    }

    /// Vérifie le fichier, applique l'action sur une copie, sauvegarde, puis adopte la copie
    /// (voir l'en-tête).
    fn appliquer(
        &mut self,
        action: impl FnOnce(&mut Stock) -> Result<String, ErreurStock>,
    ) -> bool {
        let resultat = self.verifier_fichier_inchange().and_then(|()| {
            let mut copie = self.stock.clone();
            let message = action(&mut copie)?;
            sauvegarder(&copie, &self.chemin)?;
            Ok((copie, message))
        });
        match resultat {
            Ok((copie, message)) => {
                self.stock = copie;
                self.retour = Some(Retour::Succes(message));
                true
            }
            Err(e) => {
                self.retour = Some(Retour::Erreur(e.to_string()));
                false
            }
        }
    }

    /// Un seuil vide veut dire 0.
    pub fn ajouter(&mut self, nom: &str, seuil: &str) -> bool {
        self.appliquer(|stock| {
            let seuil = lire_seuil(seuil)?;
            stock.ajouter_produit(nom, seuil)?;
            Ok(format!(
                "Produit « {} » ajouté (seuil d'alerte : {seuil}).",
                nom.trim()
            ))
        })
    }

    pub fn entree(&mut self, nom: &str, quantite: &str) -> bool {
        self.appliquer(|stock| {
            let quantite = lire_quantite(quantite)?;
            let produit = stock.enregistrer_entree(nom, quantite)?;
            Ok(format!(
                "Entrée enregistrée : +{quantite} {}, stock actuel : {}.",
                produit.nom, produit.quantite
            ))
        })
    }

    pub fn sortie(&mut self, nom: &str, quantite: &str) -> bool {
        self.appliquer(|stock| {
            let quantite = lire_quantite(quantite)?;
            let produit = stock.enregistrer_sortie(nom, quantite)?;
            let mut texte = format!(
                "Sortie enregistrée : -{quantite} {}, stock actuel : {}.",
                produit.nom, produit.quantite
            );
            if produit.en_alerte() {
                texte.push_str(&format!(
                    " ATTENTION : stock bas (seuil d'alerte : {}).",
                    produit.seuil
                ));
            }
            Ok(texte)
        })
    }

    pub fn renommer(&mut self, ancien: &str, nouveau: &str) -> bool {
        self.appliquer(|stock| {
            stock.renommer_produit(ancien, nouveau)?;
            Ok(format!(
                "Produit renommé : « {} » devient « {} ».",
                ancien.trim(),
                nouveau.trim()
            ))
        })
    }

    pub fn supprimer(&mut self, nom: &str) -> bool {
        self.appliquer(|stock| {
            stock.supprimer_produit(nom)?;
            Ok(format!(
                "Produit « {} » supprimé (l'historique est conservé).",
                nom.trim()
            ))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chemin_temp(nom: &str) -> PathBuf {
        let chemin = std::env::temp_dir().join(format!(
            "gestion-stock-gui-{}-{nom}.txt",
            std::process::id()
        ));
        let _ = std::fs::remove_file(&chemin);
        chemin
    }

    /// « Riz 5kg », seuil 5, stock 12.
    fn etat_avec_riz(nom: &str) -> Etat {
        let mut e = Etat::ouvrir(chemin_temp(nom)).unwrap();
        assert!(e.ajouter("Riz 5kg", "5"));
        assert!(e.entree("Riz 5kg", "12"));
        e
    }

    #[test]
    fn ouvrir_un_fichier_absent_donne_un_stock_vide_et_le_dit() {
        let e = Etat::ouvrir(chemin_temp("absent")).unwrap();
        assert!(e.lignes("", false).is_empty());
        assert_eq!(
            e.retour,
            Some(Retour::Succes(
                "Aucun fichier de données trouvé : un stock vide est utilisé. Le fichier sera créé au premier enregistrement."
                    .into()
            ))
        );
    }

    #[test]
    fn ouvrir_cree_le_dossier_des_donnees() {
        let dossier =
            std::env::temp_dir().join(format!("gestion-stock-gui-{}-dossier", std::process::id()));
        let _ = std::fs::remove_dir_all(&dossier);
        let mut e = Etat::ouvrir(dossier.join("stock.txt")).unwrap();
        assert!(e.ajouter("Riz", ""));
        assert!(dossier.join("stock.txt").exists());
        std::fs::remove_dir_all(&dossier).unwrap();
    }

    #[test]
    fn ouvrir_un_fichier_abime_renvoie_l_erreur_sans_toucher_au_fichier() {
        let c = chemin_temp("abime");
        std::fs::write(&c, "P;Riz;12;5\nn'importe quoi\n").unwrap();
        assert_eq!(
            Etat::ouvrir(c.clone()).unwrap_err(),
            ErreurStock::FichierCorrompu { ligne: 2 }
        );
        assert_eq!(
            std::fs::read_to_string(&c).unwrap(),
            "P;Riz;12;5\nn'importe quoi\n"
        );
    }

    #[test]
    fn ajouter_valide_sauvegarde_et_confirme() {
        let c = chemin_temp("ajout");
        let mut e = Etat::ouvrir(c.clone()).unwrap();
        assert!(e.ajouter("  Riz 5kg ", "5"));
        assert_eq!(
            e.retour,
            Some(Retour::Succes(
                "Produit « Riz 5kg » ajouté (seuil d'alerte : 5).".into()
            ))
        );
        assert_eq!(
            e.lignes("", false),
            [LigneProduit {
                nom: "Riz 5kg".into(),
                quantite: 0,
                seuil: 5,
                en_alerte: true
            }]
        );
        assert!(
            std::fs::read_to_string(&c)
                .unwrap()
                .contains("P;Riz 5kg;0;5")
        );
    }

    #[test]
    fn ajouter_sans_seuil_utilise_zero() {
        let mut e = Etat::ouvrir(chemin_temp("sans-seuil")).unwrap();
        assert!(e.ajouter("Sel", "  "));
        assert_eq!(e.lignes("", false)[0].seuil, 0);
    }

    #[test]
    fn saisies_invalides_de_l_ajout_ne_changent_rien() {
        for (nom, seuil) in [
            ("", "5"),
            ("a;b", "5"),
            ("Riz", "abc"),
            ("Riz", "-1"),
            ("Riz", "2,5"),
        ] {
            let mut e = Etat::ouvrir(chemin_temp("invalide")).unwrap();
            assert!(!e.ajouter(nom, seuil), "{nom:?} {seuil:?}");
            assert!(matches!(e.retour, Some(Retour::Erreur(_))));
            assert!(e.lignes("", false).is_empty());
        }
        let mut e = Etat::ouvrir(chemin_temp("seuil")).unwrap();
        e.ajouter("Riz", "abc");
        assert_eq!(
            e.retour,
            Some(Retour::Erreur(
                "Seuil d'alerte invalide : « abc ». Saisissez un nombre entier positif, par exemple 5, ou laissez vide pour 0."
                    .into()
            ))
        );
    }

    #[test]
    fn entree_et_sortie_confirment_et_signalent_le_stock_bas() {
        let mut e = etat_avec_riz("mouvements");
        assert!(e.entree("riz 5kg", "3"));
        assert_eq!(
            e.retour,
            Some(Retour::Succes(
                "Entrée enregistrée : +3 Riz 5kg, stock actuel : 15.".into()
            ))
        );
        assert!(e.sortie("Riz 5kg", "2"));
        assert_eq!(
            e.retour,
            Some(Retour::Succes(
                "Sortie enregistrée : -2 Riz 5kg, stock actuel : 13.".into()
            ))
        );
        assert!(e.sortie("Riz 5kg", "9"));
        assert_eq!(
            e.retour,
            Some(Retour::Succes(
                "Sortie enregistrée : -9 Riz 5kg, stock actuel : 4. ATTENTION : stock bas (seuil d'alerte : 5)."
                    .into()
            ))
        );
    }

    #[test]
    fn quantites_invalides_donnent_un_message_clair() {
        let mut e = etat_avec_riz("quantites");
        for q in ["", "  ", "abc", "-3", "2,5", "99999999999"] {
            assert!(!e.entree("Riz 5kg", q), "{q:?}");
            assert!(!e.sortie("Riz 5kg", q), "{q:?}");
            assert!(
                matches!(&e.retour, Some(Retour::Erreur(m)) if m.starts_with("Quantité ")),
                "{q:?}"
            );
        }
        e.entree("Riz 5kg", "");
        assert_eq!(
            e.retour,
            Some(Retour::Erreur(
                "Quantité manquante. Saisissez un nombre entier positif, par exemple 12.".into()
            ))
        );
        e.entree("Riz 5kg", "abc");
        assert_eq!(
            e.retour,
            Some(Retour::Erreur(
                "Quantité invalide : « abc ». Saisissez un nombre entier positif, par exemple 12."
                    .into()
            ))
        );
        assert!(!e.sortie("Riz 5kg", "0"));
        assert_eq!(
            e.retour,
            Some(Retour::Erreur(
                "La quantité doit être supérieure à zéro.".into()
            ))
        );
        assert_eq!(e.lignes("", false)[0].quantite, 12);
    }

    #[test]
    fn sortie_trop_grande_refusee_sans_toucher_au_fichier() {
        let mut e = etat_avec_riz("trop-grande");
        let avant = std::fs::read_to_string(e.chemin()).unwrap();
        assert!(!e.sortie("Riz 5kg", "99"));
        assert_eq!(
            e.retour,
            Some(Retour::Erreur(
                "Stock insuffisant pour « Riz 5kg » : 12 disponible(s), 99 demandé(s).".into()
            ))
        );
        assert_eq!(e.lignes("", false)[0].quantite, 12);
        assert_eq!(std::fs::read_to_string(e.chemin()).unwrap(), avant);
    }

    #[test]
    fn echec_de_sauvegarde_laisse_la_liste_affichee_inchangee() {
        let c = chemin_temp("sauvegarde-impossible");
        let mut e = Etat::ouvrir(c.clone()).unwrap();
        // Un DOSSIER à la place du fichier temporaire : l'écriture est impossible.
        let bloc = PathBuf::from(format!("{}.tmp", c.display()));
        std::fs::create_dir(&bloc).unwrap();
        assert!(!e.ajouter("Sel", "1"));
        assert!(
            matches!(&e.retour, Some(Retour::Erreur(m)) if m.contains(&c.display().to_string()))
        );
        assert!(e.lignes("", false).is_empty());
        std::fs::remove_dir(&bloc).unwrap();
    }

    #[test]
    fn renommer_met_a_jour_la_liste_et_l_historique() {
        let mut e = etat_avec_riz("renommer");
        assert!(e.renommer("riz 5kg", " Riz basmati "));
        assert_eq!(
            e.retour,
            Some(Retour::Succes(
                "Produit renommé : « riz 5kg » devient « Riz basmati ».".into()
            ))
        );
        assert_eq!(e.lignes("", false)[0].nom, "Riz basmati");
        assert_eq!(e.historique(10)[0], "Entrée +12 Riz basmati");
        assert!(!e.renommer("Sel", "Sel fin"));
        assert!(matches!(e.retour, Some(Retour::Erreur(_))));
    }

    #[test]
    fn supprimer_retire_le_produit_et_garde_l_historique() {
        let mut e = etat_avec_riz("supprimer");
        assert!(e.supprimer("RIZ 5KG"));
        assert_eq!(
            e.retour,
            Some(Retour::Succes(
                "Produit « RIZ 5KG » supprimé (l'historique est conservé).".into()
            ))
        );
        assert!(e.lignes("", false).is_empty());
        assert_eq!(e.historique(10).len(), 1);
        assert!(!e.supprimer("Riz 5kg"));
    }

    #[test]
    fn lignes_filtrent_par_recherche_et_par_alerte() {
        let mut e = etat_avec_riz("filtres"); // Riz 5kg : 12, seuil 5 (pas en alerte)
        e.ajouter("Riz basmati", "5"); // 0, seuil 5 (en alerte)
        e.ajouter("Sucre", "0");
        e.entree("Sucre", "4");
        assert_eq!(e.lignes("  RIZ ", false).len(), 2);
        assert_eq!(e.lignes("sucre", false).len(), 1);
        assert_eq!(e.lignes("zzz", false).len(), 0);
        let alertes: Vec<String> = e.lignes("", true).into_iter().map(|l| l.nom).collect();
        assert_eq!(alertes, ["Riz basmati"]);
    }

    #[test]
    fn historique_du_plus_recent_au_plus_ancien_avec_limite() {
        let mut e = etat_avec_riz("historique");
        e.sortie("Riz 5kg", "2");
        assert_eq!(
            e.historique(10),
            ["Sortie -2 Riz 5kg", "Entrée +12 Riz 5kg"]
        );
        assert_eq!(e.historique(1), ["Sortie -2 Riz 5kg"]);
    }

    #[test]
    fn deux_fenetres_sur_le_meme_fichier_ne_perdent_aucun_mouvement() {
        let mut a = etat_avec_riz("deux-fenetres"); // le fichier contient Riz 5kg : 12
        let mut b = Etat::ouvrir(a.chemin().to_path_buf()).unwrap();
        assert!(a.sortie("Riz 5kg", "5")); // fichier : 7
        // B montre encore 12 : si elle enregistrait, elle effacerait la sortie de A.
        assert!(!b.sortie("Riz 5kg", "2"));
        assert!(
            matches!(&b.retour, Some(Retour::Erreur(m)) if m.contains("modifié par un autre")),
            "{:?}",
            b.retour
        );
        let sur_disque = Etat::ouvrir(a.chemin().to_path_buf()).unwrap();
        assert_eq!(sur_disque.lignes("", false)[0].quantite, 7);
        assert_eq!(b.lignes("", false)[0].quantite, 12); // B n'a rien changé
    }

    #[test]
    fn le_chemin_affiche_est_toujours_absolu() {
        let affiche = chemin_complet(Path::new("stock.txt"));
        assert!(Path::new(&affiche).is_absolute(), "{affiche}");
        assert!(affiche.ends_with("stock.txt"));
    }
}
