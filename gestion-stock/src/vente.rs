//! La vente : calculer ce que coûte un panier (ici), puis l'encaisser.
//!
//! Toutes les sommes sont des entiers en centimes. La TVA ou la taxe se calcule PAR TAUX sur la
//! somme des lignes de ce taux (jamais ligne par ligne), arrondie à la moitié supérieure : c'est
//! ce que fait un ticket de caisse, et le total reste exactement la somme des lignes.

use crate::erreur::ErreurStock;
use crate::monnaie::Montant;
use crate::reglages::Profil;
use crate::stock::Stock;
use std::collections::BTreeMap;

/// Une ligne d'une vente : le prix et le taux sont ceux du moment de la vente, ils ne
/// changeront plus même si le produit change de prix ensuite.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LigneVente {
    pub nom: String,
    pub quantite: u32,
    pub prix: Montant,
    pub taux_bp: u32,
}

/// Le détail d'un taux sur un ticket : `base + taxe = ttc`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GroupeTaux {
    pub taux_bp: u32,
    pub base: Montant,
    pub taxe: Montant,
    pub ttc: Montant,
}

/// Ce que coûte un panier : les lignes, le détail par taux (du plus petit au plus grand
/// taux) et le total à payer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Devis {
    pub lignes: Vec<LigneVente>,
    pub groupes: Vec<GroupeTaux>,
    pub total: Montant,
}

/// `n / d` arrondi à la moitié supérieure, pour `n >= 0` et `d > 0` : 0,5 monte à 1.
fn arrondi_demi_haut(n: i64, d: i64) -> i64 {
    (2 * n + d) / (2 * d)
}

impl Stock {
    /// Calcule le devis d'un panier de (nom, quantité). Un même produit (casse et espaces
    /// ignorés) saisi deux fois est cumulé. Le devis ne regarde PAS le stock : c'est
    /// l'encaissement qui vérifie qu'il y en a assez.
    pub fn devis(&self, demande: &[(String, u32)]) -> Result<Devis, ErreurStock> {
        if demande.is_empty() {
            return Err(ErreurStock::PanierVide);
        }
        // Un produit saisi plusieurs fois devient une seule ligne, dans l'ordre de la saisie.
        let mut cumul: Vec<(usize, u32)> = Vec::new();
        for (nom, quantite) in demande {
            if *quantite == 0 {
                return Err(ErreurStock::QuantiteNulle);
            }
            let i = self.chercher(nom)?;
            match cumul.iter_mut().find(|(j, _)| *j == i) {
                Some((_, total)) => {
                    *total = total
                        .checked_add(*quantite)
                        .ok_or(ErreurStock::QuantiteTropGrande)?;
                }
                None => cumul.push((i, *quantite)),
            }
        }

        let mut lignes = Vec::new();
        let mut par_taux: BTreeMap<u32, Montant> = BTreeMap::new();
        for (i, quantite) in cumul {
            let produit = &self.produits[i];
            let prix = produit
                .prix
                .ok_or_else(|| ErreurStock::PrixManquant(produit.nom.clone()))?;
            let taux_bp = produit.taux_bp.unwrap_or(self.reglages.taux_defaut_bp);
            let montant = prix.multiplier(quantite)?;
            let somme = par_taux.entry(taux_bp).or_insert(Montant::ZERO);
            *somme = somme.ajouter(montant)?;
            lignes.push(LigneVente {
                nom: produit.nom.clone(),
                quantite,
                prix,
                taux_bp,
            });
        }

        let mut groupes = Vec::new();
        let mut total = Montant::ZERO;
        for (taux_bp, somme) in par_taux {
            let taux = i64::from(taux_bp);
            let groupe = match self.reglages.profil {
                // Le prix contient déjà la TVA : on la retrouve.
                Profil::Europe => {
                    let ttc = somme;
                    let taxe = Montant(arrondi_demi_haut(ttc.0 * taux, 10_000 + taux));
                    GroupeTaux {
                        taux_bp,
                        base: Montant(ttc.0 - taxe.0),
                        taxe,
                        ttc,
                    }
                }
                // Le prix est hors taxe : on ajoute la taxe.
                Profil::EtatsUnis => {
                    let base = somme;
                    let taxe = Montant(arrondi_demi_haut(base.0 * taux, 10_000));
                    GroupeTaux {
                        taux_bp,
                        base,
                        taxe,
                        ttc: base.ajouter(taxe)?,
                    }
                }
            };
            total = total.ajouter(groupe.ttc)?;
            groupes.push(groupe);
        }
        Ok(Devis {
            lignes,
            groupes,
            total,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::erreur::ErreurStock;
    use crate::monnaie::{Devise, MONTANT_MAX, Montant};
    use crate::reglages::{Profil, Reglages};
    use crate::stock::Stock;

    /// Crée un produit avec son stock, son prix (en centimes) et son taux (centièmes de %).
    fn produit(s: &mut Stock, nom: &str, stock: u32, prix: Option<i64>, taux: Option<u32>) {
        s.ajouter_produit(nom, 0).unwrap();
        if stock > 0 {
            s.enregistrer_entree(nom, stock).unwrap();
        }
        s.definir_prix(nom, prix.map(Montant), taux).unwrap();
    }

    /// Europe, euro, 20 % par défaut. Pain 1,20 € et Lait 0,95 € à 5,5 %, Savon 2,50 € à 20 %.
    fn stock_europe() -> Stock {
        let mut s = Stock::nouveau();
        produit(&mut s, "Pain", 10, Some(120), Some(550));
        produit(&mut s, "Lait", 10, Some(95), Some(550));
        produit(&mut s, "Savon", 10, Some(250), Some(2000));
        produit(&mut s, "Riz", 10, None, None);
        s
    }

    /// États-Unis, dollar. Pain et Lait exonérés (0 %), Savon à 8 %.
    fn stock_us() -> Stock {
        let mut s = Stock::nouveau();
        s.definir_reglages(Reglages {
            profil: Profil::EtatsUnis,
            devise: Devise::Usd,
            taux_defaut_bp: 825,
        })
        .unwrap();
        produit(&mut s, "Pain", 10, Some(120), Some(0));
        produit(&mut s, "Lait", 10, Some(95), Some(0));
        produit(&mut s, "Savon", 10, Some(250), Some(800));
        s
    }

    fn demande(articles: &[(&str, u32)]) -> Vec<(String, u32)> {
        articles.iter().map(|(n, q)| (n.to_string(), *q)).collect()
    }

    #[test]
    fn europe_ttc_et_tva_par_taux() {
        // Pain x3 = 360, Lait x2 = 190, Savon x1 = 250
        let d = stock_europe()
            .devis(&demande(&[("Pain", 3), ("Lait", 2), ("Savon", 1)]))
            .unwrap();
        assert_eq!(
            d.groupes,
            [
                GroupeTaux {
                    taux_bp: 550,
                    ttc: Montant(550),
                    taxe: Montant(29), // 550 * 550 / 10550 = 28,67 -> 29
                    base: Montant(521),
                },
                GroupeTaux {
                    taux_bp: 2000,
                    ttc: Montant(250),
                    taxe: Montant(42), // 250 * 2000 / 12000 = 41,67 -> 42
                    base: Montant(208),
                },
            ]
        );
        assert_eq!(d.total, Montant(800));
        assert_eq!(
            d.lignes,
            [
                LigneVente {
                    nom: "Pain".into(),
                    quantite: 3,
                    prix: Montant(120),
                    taux_bp: 550
                },
                LigneVente {
                    nom: "Lait".into(),
                    quantite: 2,
                    prix: Montant(95),
                    taux_bp: 550
                },
                LigneVente {
                    nom: "Savon".into(),
                    quantite: 1,
                    prix: Montant(250),
                    taux_bp: 2000
                },
            ]
        );
    }

    #[test]
    fn etats_unis_taxe_ajoutee_au_prix_hors_taxe() {
        let d = stock_us()
            .devis(&demande(&[("Pain", 3), ("Lait", 2), ("Savon", 1)]))
            .unwrap();
        assert_eq!(
            d.groupes,
            [
                GroupeTaux {
                    taux_bp: 0,
                    base: Montant(550),
                    taxe: Montant(0),
                    ttc: Montant(550)
                },
                GroupeTaux {
                    taux_bp: 800,
                    base: Montant(250),
                    taxe: Montant(20),
                    ttc: Montant(270)
                },
            ]
        );
        assert_eq!(d.total, Montant(820));
    }

    #[test]
    fn arrondi_a_la_moitie_superieure_aux_etats_unis() {
        // (base, taux) -> taxe attendue
        for (base, taux, attendu) in [
            (100, 825, 8), // 8,25 -> 8
            (10, 825, 1),  // 0,825 -> 1
            (6, 825, 0),   // 0,495 -> 0
            (1, 5000, 1),  // 0,5 -> 1 : la moitié monte
            (3, 5000, 2),  // 1,5 -> 2
        ] {
            let mut s = Stock::nouveau();
            s.definir_reglages(Reglages {
                profil: Profil::EtatsUnis,
                devise: Devise::Usd,
                taux_defaut_bp: 0,
            })
            .unwrap();
            produit(&mut s, "X", 1, Some(base), Some(taux));
            let d = s.devis(&demande(&[("X", 1)])).unwrap();
            assert_eq!(d.groupes[0].taxe, Montant(attendu), "{base} à {taux}");
            assert_eq!(d.total, Montant(base + attendu));
        }
    }

    #[test]
    fn arrondi_a_la_moitie_superieure_en_europe() {
        // 3 c TTC à 20 % : TVA = 3 * 2000 / 12000 = 0,5 -> 1 ; base = 2
        let mut s = Stock::nouveau();
        produit(&mut s, "X", 1, Some(3), Some(2000));
        let d = s.devis(&demande(&[("X", 1)])).unwrap();
        assert_eq!(d.groupes[0].taxe, Montant(1));
        assert_eq!(d.groupes[0].base, Montant(2));
        assert_eq!(d.total, Montant(3));
    }

    #[test]
    fn aucun_centime_perdu_ni_invente() {
        // Trois produits à 0,33 € TTC (20 %) : 99 c au total ; TVA = 99 * 2000 / 12000 = 16,5 -> 17
        let mut s = Stock::nouveau();
        for nom in ["A", "B", "C"] {
            produit(&mut s, nom, 1, Some(33), Some(2000));
        }
        let d = s.devis(&demande(&[("A", 1), ("B", 1), ("C", 1)])).unwrap();
        assert_eq!(d.total, Montant(99));
        assert_eq!(d.groupes.len(), 1);
        let g = &d.groupes[0];
        assert_eq!((g.taxe, g.base), (Montant(17), Montant(82)));
        assert_eq!(g.base.0 + g.taxe.0, g.ttc.0);
        assert_eq!(g.ttc, d.total);
    }

    #[test]
    fn le_total_est_toujours_la_somme_des_lignes_en_europe() {
        let s = stock_europe();
        for quantites in [(1, 1, 1), (7, 3, 2), (10, 10, 10), (4, 9, 1)] {
            let d = s
                .devis(&demande(&[
                    ("Pain", quantites.0),
                    ("Lait", quantites.1),
                    ("Savon", quantites.2),
                ]))
                .unwrap();
            let somme: i64 = d
                .lignes
                .iter()
                .map(|l| l.prix.0 * i64::from(l.quantite))
                .sum();
            assert_eq!(d.total.0, somme, "{quantites:?}");
            for g in &d.groupes {
                assert_eq!(g.base.0 + g.taxe.0, g.ttc.0);
            }
        }
    }

    #[test]
    fn le_taux_par_defaut_s_applique_sans_taux_produit() {
        let mut s = Stock::nouveau();
        produit(&mut s, "Sel", 5, Some(100), None);
        let d = s.devis(&demande(&[("Sel", 1)])).unwrap();
        assert_eq!(d.lignes[0].taux_bp, 2000);
        assert_eq!(d.groupes[0].taux_bp, 2000);
    }

    #[test]
    fn un_meme_produit_saisi_deux_fois_est_cumule() {
        let d = stock_europe()
            .devis(&demande(&[("Pain", 2), ("  pain ", 1)]))
            .unwrap();
        assert_eq!(d.lignes.len(), 1);
        assert_eq!(
            (d.lignes[0].nom.as_str(), d.lignes[0].quantite),
            ("Pain", 3)
        );
        assert_eq!(d.total, Montant(360));
    }

    #[test]
    fn refus_clairs() {
        let s = stock_europe();
        assert_eq!(s.devis(&[]), Err(ErreurStock::PanierVide));
        assert_eq!(
            s.devis(&demande(&[("Riz", 1)])),
            Err(ErreurStock::PrixManquant("Riz".into()))
        );
        assert_eq!(
            s.devis(&demande(&[("Sucre", 1)])),
            Err(ErreurStock::ProduitInconnu("Sucre".into()))
        );
        assert_eq!(
            s.devis(&demande(&[("Pain", 0)])),
            Err(ErreurStock::QuantiteNulle)
        );
        // Un produit sans prix dans le panier bloque tout le devis.
        assert_eq!(
            s.devis(&demande(&[("Pain", 1), ("Riz", 1)])),
            Err(ErreurStock::PrixManquant("Riz".into()))
        );
    }

    #[test]
    fn un_montant_qui_deborde_est_refuse() {
        let mut s = Stock::nouveau();
        produit(&mut s, "Cher", 1, Some(MONTANT_MAX.0), Some(0));
        produit(&mut s, "Autre", 1, Some(MONTANT_MAX.0), Some(0));
        produit(&mut s, "Vrac", 1, Some(100), Some(0));
        // Quantité énorme : 100 c x 4 milliards dépasse la limite.
        assert_eq!(
            s.devis(&demande(&[("Vrac", u32::MAX)])),
            Err(ErreurStock::MontantTropGrand)
        );
        // Chaque ligne passe, mais leur somme dépasse la limite.
        assert_eq!(
            s.devis(&demande(&[("Cher", 1), ("Autre", 1)])),
            Err(ErreurStock::MontantTropGrand)
        );
        // La même quantité saisie deux fois et cumulée au-delà d'un u32.
        assert_eq!(
            s.devis(&demande(&[("Vrac", u32::MAX), ("Vrac", 1)])),
            Err(ErreurStock::QuantiteTropGrande)
        );
    }

    #[test]
    fn les_groupes_sont_tries_par_taux_croissant() {
        let d = stock_europe()
            .devis(&demande(&[("Savon", 1), ("Pain", 1)]))
            .unwrap();
        let taux: Vec<u32> = d.groupes.iter().map(|g| g.taux_bp).collect();
        assert_eq!(taux, [550, 2000]);
    }
}
