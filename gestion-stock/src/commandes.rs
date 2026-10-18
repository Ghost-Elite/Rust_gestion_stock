//! Analyse des arguments tapés par le commerçant et production du texte à afficher.
//! Ce module ne lit ni n'écrit aucun fichier : il ne fait que parler au `Stock`.

use crate::erreur::ErreurStock;
use crate::stock::{Genre, Produit, Stock};
use std::str::FromStr;

pub const AIDE: &str = r#"Gestion de stock - commandes disponibles :

  stock ajouter "Riz 5kg" [--seuil 5]   Ajoute un produit (seuil d'alerte : 0 par défaut)
  stock entree  "Riz 5kg" 12            Enregistre une entrée de marchandise
  stock sortie  "Riz 5kg" 2             Enregistre une sortie (vente, perte...)
  stock lister                          Affiche tout le stock
  stock alertes                         Affiche les produits bientôt épuisés
  stock historique [10]                 Affiche les N derniers mouvements (10 par défaut)
  stock aide                            Affiche ce message

Le nom d'un produit avec des espaces se met entre guillemets.
Les données sont dans le fichier « stock.txt » (ou celui de la variable STOCK_FICHIER)."#;

#[derive(Debug, PartialEq, Eq)]
pub enum Commande {
    Ajouter { nom: String, seuil: u32 },
    Entree { nom: String, quantite: u32 },
    Sortie { nom: String, quantite: u32 },
    Lister,
    Alertes,
    Historique { limite: usize },
    Aide,
}

impl Commande {
    /// Vrai si la commande change le stock : seulement alors il faut sauvegarder.
    pub fn modifie_le_stock(&self) -> bool {
        matches!(
            self,
            Commande::Ajouter { .. } | Commande::Entree { .. } | Commande::Sortie { .. }
        )
    }
}

fn utilisation(message: &str) -> ErreurStock {
    ErreurStock::Utilisation(format!(
        "{message} Tapez « stock aide » pour voir les commandes."
    ))
}

fn lire_nombre<T: FromStr>(texte: &str, quoi: &str) -> Result<T, ErreurStock> {
    texte.parse().map_err(|_| {
        utilisation(&format!(
            "{quoi} invalide : « {texte} » (un nombre entier positif est attendu)."
        ))
    })
}

/// `args` ne contient pas le nom du programme.
pub fn analyser(args: &[String]) -> Result<Commande, ErreurStock> {
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    match args.as_slice() {
        [] | ["aide"] => Ok(Commande::Aide),
        ["lister"] => Ok(Commande::Lister),
        ["alertes"] => Ok(Commande::Alertes),
        ["historique"] => Ok(Commande::Historique { limite: 10 }),
        ["historique", n] => Ok(Commande::Historique {
            limite: lire_nombre(n, "Nombre de mouvements")?,
        }),
        ["ajouter", nom] => Ok(Commande::Ajouter {
            nom: nom.to_string(),
            seuil: 0,
        }),
        ["ajouter", nom, "--seuil", seuil] => Ok(Commande::Ajouter {
            nom: nom.to_string(),
            seuil: lire_nombre(seuil, "Seuil")?,
        }),
        ["entree", nom, quantite] => Ok(Commande::Entree {
            nom: nom.to_string(),
            quantite: lire_nombre(quantite, "Quantité")?,
        }),
        ["sortie", nom, quantite] => Ok(Commande::Sortie {
            nom: nom.to_string(),
            quantite: lire_nombre(quantite, "Quantité")?,
        }),
        [
            commande @ ("aide" | "lister" | "alertes" | "historique" | "ajouter" | "entree"
            | "sortie"),
            ..,
        ] => Err(utilisation(&format!(
            "Arguments incorrects pour « {commande} »."
        ))),
        [commande, ..] => Err(utilisation(&format!("Commande inconnue : « {commande} »."))),
    }
}

fn tableau(produits: &[&Produit], si_vide: &str) -> String {
    if produits.is_empty() {
        return si_vide.to_string();
    }
    produits
        .iter()
        .map(|p| {
            let marque = if p.en_alerte() { "  ALERTE" } else { "" };
            format!(
                "{:<20} {:>6}  seuil {:>4}{}",
                p.nom, p.quantite, p.seuil, marque
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Exécute la commande et renvoie le texte à afficher.
pub fn executer(commande: &Commande, stock: &mut Stock) -> Result<String, ErreurStock> {
    match commande {
        Commande::Ajouter { nom, seuil } => {
            stock.ajouter_produit(nom, *seuil)?;
            Ok(format!(
                "Produit « {} » ajouté (seuil d'alerte : {seuil}).",
                nom.trim()
            ))
        }
        Commande::Entree { nom, quantite } => {
            let produit = stock.enregistrer_entree(nom, *quantite)?;
            Ok(format!(
                "Entrée enregistrée : +{quantite} {}, stock actuel : {}.",
                produit.nom, produit.quantite
            ))
        }
        Commande::Sortie { nom, quantite } => {
            let produit = stock.enregistrer_sortie(nom, *quantite)?;
            let mut texte = format!(
                "Sortie enregistrée : -{quantite} {}, stock actuel : {}.",
                produit.nom, produit.quantite
            );
            if produit.en_alerte() {
                texte.push_str(&format!(
                    "\nATTENTION : stock bas pour « {} » (seuil d'alerte : {}).",
                    produit.nom, produit.seuil
                ));
            }
            Ok(texte)
        }
        Commande::Lister => Ok(tableau(&stock.produits(), "Aucun produit enregistré.")),
        Commande::Alertes => Ok(tableau(
            &stock.produits_en_alerte(),
            "Aucun produit en alerte.",
        )),
        Commande::Historique { limite } => {
            let lignes: Vec<String> = stock
                .historique()
                .iter()
                .rev()
                .take(*limite)
                .map(|m| match m.genre {
                    Genre::Entree => format!("Entrée +{} {}", m.quantite, m.nom),
                    Genre::Sortie => format!("Sortie -{} {}", m.quantite, m.nom),
                })
                .collect();
            if lignes.is_empty() {
                Ok("Aucun mouvement enregistré.".to_string())
            } else {
                Ok(lignes.join("\n"))
            }
        }
        Commande::Aide => Ok(AIDE.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    fn stock_avec_riz() -> Stock {
        let mut s = Stock::nouveau();
        s.ajouter_produit("Riz 5kg", 5).unwrap();
        s
    }

    #[test]
    fn analyser_entree() {
        assert_eq!(
            analyser(&args(&["entree", "Riz 5kg", "12"])),
            Ok(Commande::Entree {
                nom: "Riz 5kg".into(),
                quantite: 12
            })
        );
    }

    #[test]
    fn analyser_sortie() {
        assert_eq!(
            analyser(&args(&["sortie", "Riz 5kg", "2"])),
            Ok(Commande::Sortie {
                nom: "Riz 5kg".into(),
                quantite: 2
            })
        );
    }

    #[test]
    fn analyser_ajouter_avec_et_sans_seuil() {
        assert_eq!(
            analyser(&args(&["ajouter", "Riz", "--seuil", "5"])),
            Ok(Commande::Ajouter {
                nom: "Riz".into(),
                seuil: 5
            })
        );
        assert_eq!(
            analyser(&args(&["ajouter", "Riz"])),
            Ok(Commande::Ajouter {
                nom: "Riz".into(),
                seuil: 0
            })
        );
    }

    #[test]
    fn analyser_refuse_les_quantites_invalides() {
        for q in ["abc", "-3", "2,5", ""] {
            assert!(
                matches!(
                    analyser(&args(&["entree", "Riz", q])),
                    Err(ErreurStock::Utilisation(_))
                ),
                "{q:?}"
            );
        }
    }

    #[test]
    fn analyser_refuse_les_arguments_manquants_ou_en_trop() {
        let cas: [&[&str]; 6] = [
            &["entree", "Riz"],
            &["sortie"],
            &["ajouter"],
            &["ajouter", "Riz", "--seuil"],
            &["entree", "Riz", "1", "2"],
            &["lister", "x"],
        ];
        for a in cas {
            assert!(
                matches!(analyser(&args(a)), Err(ErreurStock::Utilisation(_))),
                "{a:?}"
            );
        }
    }

    #[test]
    fn analyser_commandes_simples_et_aide() {
        assert_eq!(analyser(&args(&[])), Ok(Commande::Aide));
        assert_eq!(analyser(&args(&["aide"])), Ok(Commande::Aide));
        assert_eq!(analyser(&args(&["lister"])), Ok(Commande::Lister));
        assert_eq!(analyser(&args(&["alertes"])), Ok(Commande::Alertes));
        assert_eq!(
            analyser(&args(&["historique"])),
            Ok(Commande::Historique { limite: 10 })
        );
        assert_eq!(
            analyser(&args(&["historique", "3"])),
            Ok(Commande::Historique { limite: 3 })
        );
    }

    #[test]
    fn analyser_refuse_une_commande_inconnue() {
        assert!(matches!(
            analyser(&args(&["voler"])),
            Err(ErreurStock::Utilisation(_))
        ));
    }

    #[test]
    fn modifie_le_stock_seulement_pour_ajouter_entree_sortie() {
        assert!(
            Commande::Entree {
                nom: "x".into(),
                quantite: 1
            }
            .modifie_le_stock()
        );
        assert!(!Commande::Lister.modifie_le_stock());
        assert!(!Commande::Aide.modifie_le_stock());
    }

    #[test]
    fn executer_ajouter_entree_sortie_avec_alerte() {
        let mut s = Stock::nouveau();
        let c = |v: &[&str]| analyser(&args(v)).unwrap();
        assert_eq!(
            executer(&c(&["ajouter", "Riz 5kg", "--seuil", "5"]), &mut s).unwrap(),
            "Produit « Riz 5kg » ajouté (seuil d'alerte : 5)."
        );
        assert_eq!(
            executer(&c(&["entree", "Riz 5kg", "12"]), &mut s).unwrap(),
            "Entrée enregistrée : +12 Riz 5kg, stock actuel : 12."
        );
        assert_eq!(
            executer(&c(&["sortie", "Riz 5kg", "2"]), &mut s).unwrap(),
            "Sortie enregistrée : -2 Riz 5kg, stock actuel : 10."
        );
        assert_eq!(
            executer(&c(&["sortie", "Riz 5kg", "6"]), &mut s).unwrap(),
            "Sortie enregistrée : -6 Riz 5kg, stock actuel : 4.\nATTENTION : stock bas pour « Riz 5kg » (seuil d'alerte : 5)."
        );
    }

    #[test]
    fn executer_lister_et_alertes() {
        let mut s = stock_avec_riz();
        s.ajouter_produit("Sucre", 0).unwrap();
        s.enregistrer_entree("Riz 5kg", 4).unwrap(); // en alerte (4 <= 5)
        s.enregistrer_entree("Sucre", 10).unwrap();
        let tout = executer(&Commande::Lister, &mut s).unwrap();
        let lignes: Vec<&str> = tout.lines().collect();
        assert_eq!(lignes.len(), 2);
        assert!(lignes[0].starts_with("Riz 5kg ") && lignes[0].ends_with("seuil    5  ALERTE"));
        assert!(lignes[1].starts_with("Sucre ") && !lignes[1].contains("ALERTE"));
        let alertes = executer(&Commande::Alertes, &mut s).unwrap();
        assert_eq!(alertes.lines().count(), 1);
        assert!(alertes.starts_with("Riz 5kg"));
    }

    #[test]
    fn executer_listes_vides() {
        let mut s = Stock::nouveau();
        assert_eq!(
            executer(&Commande::Lister, &mut s).unwrap(),
            "Aucun produit enregistré."
        );
        assert_eq!(
            executer(&Commande::Alertes, &mut s).unwrap(),
            "Aucun produit en alerte."
        );
        assert_eq!(
            executer(&Commande::Historique { limite: 10 }, &mut s).unwrap(),
            "Aucun mouvement enregistré."
        );
    }

    #[test]
    fn executer_historique_du_plus_recent_au_plus_ancien_avec_limite() {
        let mut s = stock_avec_riz();
        s.enregistrer_entree("Riz 5kg", 12).unwrap();
        s.enregistrer_sortie("Riz 5kg", 2).unwrap();
        assert_eq!(
            executer(&Commande::Historique { limite: 10 }, &mut s).unwrap(),
            "Sortie -2 Riz 5kg\nEntrée +12 Riz 5kg"
        );
        assert_eq!(
            executer(&Commande::Historique { limite: 1 }, &mut s).unwrap(),
            "Sortie -2 Riz 5kg"
        );
    }

    #[test]
    fn executer_remonte_les_erreurs_du_stock() {
        let mut s = stock_avec_riz();
        assert_eq!(
            executer(
                &Commande::Sortie {
                    nom: "Riz 5kg".into(),
                    quantite: 1
                },
                &mut s
            )
            .unwrap_err(),
            ErreurStock::StockInsuffisant {
                nom: "Riz 5kg".into(),
                disponible: 0,
                demandee: 1
            }
        );
    }

    #[test]
    fn aide_cite_toutes_les_commandes() {
        for mot in [
            "ajouter",
            "entree",
            "sortie",
            "lister",
            "alertes",
            "historique",
            "aide",
        ] {
            assert!(AIDE.contains(mot), "{mot}");
        }
    }
}
