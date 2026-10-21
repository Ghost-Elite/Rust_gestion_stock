//! La vente : calculer ce que coûte un panier (ici), puis l'encaisser.
//!
//! Toutes les sommes sont des entiers en centimes. La TVA ou la taxe se calcule PAR TAUX sur la
//! somme des lignes de ce taux (jamais ligne par ligne), arrondie à la moitié supérieure : c'est
//! ce que fait un ticket de caisse, et le total reste exactement la somme des lignes.

use crate::erreur::ErreurStock;
use crate::horodatage::Horodatage;
use crate::journal::{HASH_INITIAL, empreinte};
use crate::monnaie::{MONTANT_MAX, Montant};
use crate::reglages::Profil;
use crate::stock::{Genre as GenreStock, Mouvement, Stock};
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
        for (i, quantite) in cumul {
            let produit = &self.produits[i];
            let prix = produit
                .prix
                .ok_or_else(|| ErreurStock::PrixManquant(produit.nom.clone()))?;
            lignes.push(LigneVente {
                nom: produit.nom.clone(),
                quantite,
                prix,
                taux_bp: produit.taux_bp.unwrap_or(self.reglages.taux_defaut_bp),
            });
        }
        let (groupes, total) = detail_taxes(&lignes, self.reglages.profil)?;
        Ok(Devis {
            lignes,
            groupes,
            total,
        })
    }
}

/// Le détail par taux (du plus petit au plus grand) et le total d'une liste de lignes. Sert au
/// devis, et aussi à réafficher le ticket d'une pièce déjà enregistrée : ses lignes contiennent
/// leur propre prix et leur propre taux, donc le résultat ne dépend pas des prix d'aujourd'hui.
pub fn detail_taxes(
    lignes: &[LigneVente],
    profil: Profil,
) -> Result<(Vec<GroupeTaux>, Montant), ErreurStock> {
    let mut par_taux: BTreeMap<u32, Montant> = BTreeMap::new();
    for ligne in lignes {
        let montant = ligne.prix.multiplier(ligne.quantite)?;
        let somme = par_taux.entry(ligne.taux_bp).or_insert(Montant::ZERO);
        *somme = somme.ajouter(montant)?;
    }

    let mut groupes = Vec::new();
    let mut total = Montant::ZERO;
    for (taux_bp, somme) in par_taux {
        let taux = i64::from(taux_bp);
        let groupe = match profil {
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
    Ok((groupes, total))
}

/// Comment le client a payé.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Moyen {
    #[default]
    Especes,
    Carte,
    /// Chèque, virement, mobile money… : on note seulement que c'est payé.
    Autre,
}

impl Moyen {
    /// Le code utilisé dans le fichier de données.
    pub fn code(self) -> &'static str {
        match self {
            Moyen::Especes => "E",
            Moyen::Carte => "C",
            Moyen::Autre => "X",
        }
    }

    pub fn depuis_code(code: &str) -> Option<Moyen> {
        match code {
            "E" => Some(Moyen::Especes),
            "C" => Some(Moyen::Carte),
            "X" => Some(Moyen::Autre),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GenrePiece {
    Vente,
    /// Une annulation TOTALE : écriture inverse de la vente numéro `de`.
    Annulation {
        de: u64,
    },
}

/// Une écriture du journal. Une fois enregistrée, elle ne change plus jamais : l'empreinte
/// `hash` couvre son contenu ET l'empreinte de la pièce précédente.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Piece {
    /// Numérotation continue 1, 2, 3… sans trou, ventes et annulations confondues.
    pub numero: u64,
    pub genre: GenrePiece,
    pub horodatage: Horodatage,
    /// Toujours en quantités positives ; c'est le genre qui dit si on retire ou remet du stock.
    pub lignes: Vec<LigneVente>,
    pub moyen: Moyen,
    /// Signé : négatif pour une annulation.
    pub total: Montant,
    pub recu: Montant,
    pub rendu: Montant,
    pub hash_precedent: String,
    pub hash: String,
}

impl Stock {
    pub fn pieces(&self) -> &[Piece] {
        &self.pieces
    }

    fn dernier_hash(&self) -> String {
        self.pieces
            .last()
            .map_or_else(|| HASH_INITIAL.to_string(), |p| p.hash.clone())
    }

    /// Encaisse un panier : le stock baisse, une pièce numérotée entre dans le journal.
    ///
    /// - Espèces : `recu` doit couvrir le total, et le rendu est la différence.
    /// - Carte, autre : le client a payé exactement le total, `recu` est ignoré.
    ///
    /// Tout est vérifié AVANT de modifier quoi que ce soit : si un seul produit manque de
    /// stock, ou si le paiement est insuffisant, rien ne change, pas même le numéro de ticket.
    pub fn encaisser(
        &mut self,
        demande: &[(String, u32)],
        moyen: Moyen,
        recu: Montant,
        quand: Horodatage,
    ) -> Result<&Piece, ErreurStock> {
        let devis = self.devis(demande)?;
        let devise = self.reglages.devise;
        let (recu, rendu) = match moyen {
            Moyen::Especes => {
                if recu > MONTANT_MAX {
                    return Err(ErreurStock::MontantTropGrand);
                }
                if recu < devis.total {
                    return Err(ErreurStock::MontantInsuffisant {
                        du: devise.formater(devis.total),
                        recu: devise.formater(recu),
                    });
                }
                (recu, Montant(recu.0 - devis.total.0))
            }
            Moyen::Carte | Moyen::Autre => (devis.total, Montant::ZERO),
        };

        // Le stock de CHAQUE ligne (déjà cumulée par produit) est vérifié avant tout changement.
        let mut positions = Vec::with_capacity(devis.lignes.len());
        for ligne in &devis.lignes {
            let i = self.chercher(&ligne.nom)?;
            let disponible = self.produits[i].quantite;
            if disponible < ligne.quantite {
                return Err(ErreurStock::StockInsuffisant {
                    nom: self.produits[i].nom.clone(),
                    disponible,
                    demandee: ligne.quantite,
                });
            }
            positions.push(i);
        }
        let numero = match self.pieces.last() {
            Some(derniere) => derniere
                .numero
                .checked_add(1)
                .ok_or_else(|| ErreurStock::Utilisation("Le journal est plein.".to_string()))?,
            None => 1,
        };

        // À partir d'ici plus rien ne peut échouer : on applique.
        for (ligne, i) in devis.lignes.iter().zip(positions) {
            let produit = &mut self.produits[i];
            produit.quantite -= ligne.quantite;
            self.historique.push(Mouvement {
                genre: GenreStock::Sortie,
                nom: produit.nom.clone(),
                quantite: ligne.quantite,
            });
        }
        let hash_precedent = self.dernier_hash();
        let mut piece = Piece {
            numero,
            genre: GenrePiece::Vente,
            horodatage: quand,
            lignes: devis.lignes,
            moyen,
            total: devis.total,
            recu,
            rendu,
            hash_precedent,
            hash: String::new(),
        };
        piece.hash = empreinte(&piece.hash_precedent, &piece);
        let i = self.pieces.len();
        self.pieces.push(piece);
        Ok(&self.pieces[i])
    }

    /// Annule une vente EN ENTIER, par une écriture inverse : une nouvelle pièce de total négatif
    /// entre dans le journal et le stock est remis. La vente d'origine n'est pas touchée.
    /// Une vente ne s'annule qu'une fois, et une annulation ne s'annule pas.
    pub fn annuler(&mut self, numero: u64, quand: Horodatage) -> Result<&Piece, ErreurStock> {
        let origine = self
            .pieces
            .iter()
            .find(|p| p.numero == numero)
            .ok_or(ErreurStock::PieceInconnue(numero))?;
        if matches!(origine.genre, GenrePiece::Annulation { .. }) {
            return Err(ErreurStock::AnnulationNonPermise(numero));
        }
        if self
            .pieces
            .iter()
            .any(|p| p.genre == GenrePiece::Annulation { de: numero })
        {
            return Err(ErreurStock::DejaAnnulee(numero));
        }
        let origine = origine.clone();

        // Tout est vérifié avant de remettre le moindre article en stock.
        let mut positions = Vec::with_capacity(origine.lignes.len());
        let mut remises: BTreeMap<usize, u32> = BTreeMap::new();
        for ligne in &origine.lignes {
            let i = self.chercher(&ligne.nom)?;
            let cumul = remises.entry(i).or_insert(0);
            *cumul = cumul
                .checked_add(ligne.quantite)
                .ok_or(ErreurStock::QuantiteTropGrande)?;
            positions.push(i);
        }
        for (i, remise) in &remises {
            self.produits[*i]
                .quantite
                .checked_add(*remise)
                .ok_or(ErreurStock::QuantiteTropGrande)?;
        }
        let total = origine
            .total
            .0
            .checked_neg()
            .map(Montant)
            .ok_or(ErreurStock::MontantTropGrand)?;
        let nouveau_numero = match self.pieces.last() {
            Some(derniere) => derniere
                .numero
                .checked_add(1)
                .ok_or_else(|| ErreurStock::Utilisation("Le journal est plein.".to_string()))?,
            None => 1,
        };

        for (ligne, i) in origine.lignes.iter().zip(positions) {
            let produit = &mut self.produits[i];
            produit.quantite = produit.quantite.saturating_add(ligne.quantite);
            self.historique.push(Mouvement {
                genre: GenreStock::Entree,
                nom: produit.nom.clone(),
                quantite: ligne.quantite,
            });
        }
        let hash_precedent = self.dernier_hash();
        let mut piece = Piece {
            numero: nouveau_numero,
            genre: GenrePiece::Annulation { de: numero },
            horodatage: quand,
            lignes: origine.lignes,
            moyen: origine.moyen,
            total,
            recu: total,
            rendu: Montant::ZERO,
            hash_precedent,
            hash: String::new(),
        };
        piece.hash = empreinte(&piece.hash_precedent, &piece);
        let i = self.pieces.len();
        self.pieces.push(piece);
        Ok(&self.pieces[i])
    }

    /// Recalcule toute la chaîne. Une pièce modifiée, retirée, ajoutée ou déplacée donne
    /// `JournalAltere` avec le numéro de la première pièce qui ne correspond plus.
    ///
    /// Cela détecte une modification maladroite ou accidentelle. Ce n'est PAS une protection
    /// contre quelqu'un qui réécrirait tout le fichier en recalculant les empreintes.
    pub fn verifier_journal(&self) -> Result<(), ErreurStock> {
        let mut attendu = HASH_INITIAL.to_string();
        for (index, piece) in self.pieces.iter().enumerate() {
            let numero_attendu = index as u64 + 1;
            if piece.numero != numero_attendu
                || piece.hash_precedent != attendu
                || piece.hash != empreinte(&piece.hash_precedent, piece)
            {
                return Err(ErreurStock::JournalAltere {
                    numero: piece.numero,
                });
            }
            attendu = piece.hash.clone();
        }
        Ok(())
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

    // ---- Le cœur sur un Stock partiel (ce que charge une transaction de la base) --------------

    /// Un stock complet avec trois ventes : 1 = Pain x2, 2 = Lait x1, 3 = Savon x1.
    fn stock_avec_trois_ventes() -> Stock {
        let mut s = stock_europe();
        s.encaisser(&demande(&[("Pain", 2)]), Moyen::Carte, Montant(0), QUAND)
            .unwrap();
        s.encaisser(&demande(&[("Lait", 1)]), Moyen::Carte, Montant(0), QUAND)
            .unwrap();
        s.encaisser(&demande(&[("Savon", 1)]), Moyen::Carte, Montant(0), QUAND)
            .unwrap();
        s
    }

    /// Ce qu'une transaction chargerait : seulement ces produits et ces pièces.
    fn partiel_de(complet: &Stock, produits: &[&str], pieces: &[u64]) -> Stock {
        Stock::partiel(
            complet
                .produits()
                .into_iter()
                .filter(|p| produits.contains(&p.nom.as_str()))
                .cloned()
                .collect(),
            complet.reglages(),
            complet
                .pieces()
                .iter()
                .filter(|p| pieces.contains(&p.numero))
                .cloned()
                .collect(),
        )
    }

    #[test]
    fn un_stock_partiel_n_a_pas_d_historique_de_mouvements() {
        let complet = stock_avec_trois_ventes();
        let partiel = partiel_de(&complet, &["Pain"], &[3]);
        assert!(partiel.historique().is_empty());
        assert_eq!(partiel.produits().len(), 1);
        assert_eq!(partiel.pieces().len(), 1);
        assert_eq!(partiel.reglages(), complet.reglages());
    }

    #[test]
    fn encaisser_sur_un_stock_partiel_continue_la_numerotation_et_la_chaine() {
        let mut complet = stock_avec_trois_ventes();
        let mut partiel = partiel_de(&complet, &["Pain"], &[3]); // seulement la DERNIÈRE pièce
        let panier = demande(&[("Pain", 1)]);
        let attendue = complet
            .encaisser(&panier, Moyen::Especes, Montant(500), QUAND)
            .unwrap()
            .clone();
        let obtenue = partiel
            .encaisser(&panier, Moyen::Especes, Montant(500), QUAND)
            .unwrap()
            .clone();
        assert_eq!(obtenue.numero, 4);
        assert_eq!(
            obtenue, attendue,
            "même numéro, mêmes empreintes que sur le stock complet"
        );
        assert_eq!(partiel.produits()[0].quantite, 7, "10 - 2 (vente 1) - 1");
    }

    #[test]
    fn annuler_sur_un_stock_partiel_donne_la_meme_piece_qu_un_stock_complet() {
        let mut complet = stock_avec_trois_ventes();
        // La pièce à annuler (1) et la dernière (3), avec le produit qu'elle remet en stock.
        let mut partiel = partiel_de(&complet, &["Pain"], &[1, 3]);
        let attendue = complet.annuler(1, QUAND).unwrap().clone();
        let obtenue = partiel.annuler(1, QUAND).unwrap().clone();
        assert_eq!(obtenue, attendue);
        assert_eq!(attendue.numero, 4);
        assert_eq!(
            partiel.produits()[0].quantite,
            complet
                .produits()
                .iter()
                .find(|p| p.nom == "Pain")
                .unwrap()
                .quantite
        );
        // Une seconde annulation : la pièce 4 (l'annulation) est la dernière et nomme la vente 1.
        let mut deuxieme = partiel_de(&complet, &["Pain"], &[1, 4]);
        assert_eq!(
            deuxieme.annuler(1, QUAND).unwrap_err(),
            ErreurStock::DejaAnnulee(1)
        );
        // Une annulation ne s'annule pas.
        let mut sur_annulation = partiel_de(&complet, &["Pain"], &[4]);
        assert_eq!(
            sur_annulation.annuler(4, QUAND).unwrap_err(),
            ErreurStock::AnnulationNonPermise(4)
        );
        // Une pièce qui n'est pas chargée est inconnue.
        let mut sans_la_piece = partiel_de(&complet, &["Pain"], &[3]);
        assert_eq!(
            sans_la_piece.annuler(1, QUAND).unwrap_err(),
            ErreurStock::PieceInconnue(1)
        );
    }

    #[test]
    fn renommer_ou_supprimer_un_produit_vendu_est_refuse_sur_un_stock_partiel() {
        let complet = stock_avec_trois_ventes();
        // Une SEULE pièce témoin suffit : la vente 1 porte une ligne « Pain ».
        let mut avec_temoin = partiel_de(&complet, &["Pain"], &[1]);
        assert_eq!(
            avec_temoin
                .renommer_produit("pain", "Baguette")
                .unwrap_err(),
            ErreurStock::ProduitVendu("Pain".to_string())
        );
        assert_eq!(
            avec_temoin.supprimer_produit("PAIN").unwrap_err(),
            ErreurStock::ProduitVendu("Pain".to_string())
        );
        // Sans pièce qui le nomme (ici la vente du Lait), le même produit se renomme.
        let mut sans_temoin = partiel_de(&complet, &["Pain"], &[2]);
        sans_temoin.renommer_produit("Pain", "Baguette").unwrap();
        assert_eq!(sans_temoin.produits()[0].nom, "Baguette");
    }

    #[test]
    fn les_reglages_sont_verrouilles_des_qu_une_piece_existe_dans_un_stock_partiel() {
        let complet = stock_avec_trois_ventes();
        let en_dollars = Reglages {
            profil: Profil::EtatsUnis,
            devise: Devise::Usd,
            taux_defaut_bp: 825,
        };
        // Une pièce quelconque suffit à verrouiller la devise et le profil.
        let mut avec_une_piece = partiel_de(&complet, &[], &[2]);
        assert_eq!(
            avec_une_piece.definir_reglages(en_dollars).unwrap_err(),
            ErreurStock::ReglagesVerrouilles
        );
        // Le taux par défaut reste modifiable.
        let nouveau_taux = Reglages {
            taux_defaut_bp: 1000,
            ..complet.reglages()
        };
        avec_une_piece.definir_reglages(nouveau_taux).unwrap();
        // Sans aucune pièce, tout se change.
        let mut sans_piece = partiel_de(&complet, &[], &[]);
        sans_piece.definir_reglages(en_dollars).unwrap();
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

    // ---- Encaisser -------------------------------------------------------------------------

    use crate::horodatage::Horodatage;
    use crate::stock::{Genre as GenreStock, Mouvement, Produit};

    const QUAND: Horodatage = Horodatage {
        utc: 1_700_000_000,
        decalage_minutes: 60,
    };

    fn zeros() -> String {
        "0".repeat(64)
    }

    /// Ce qui compte pour « rien n'a changé » : produits, historique de stock et journal.
    fn etat_de(s: &Stock) -> (Vec<Produit>, Vec<Mouvement>, Vec<Piece>) {
        (
            s.produits().into_iter().cloned().collect(),
            s.historique().to_vec(),
            s.pieces().to_vec(),
        )
    }

    fn quantite_de(s: &Stock, nom: &str) -> u32 {
        s.produits()
            .into_iter()
            .find(|p| p.nom == nom)
            .map(|p| p.quantite)
            .unwrap()
    }

    #[test]
    fn encaisser_retire_le_stock_numerote_et_chaine() {
        let mut s = stock_europe();
        let p1 = s
            .encaisser(&demande(&[("Pain", 3)]), Moyen::Carte, Montant(0), QUAND)
            .unwrap()
            .clone();
        assert_eq!(p1.numero, 1);
        assert_eq!(p1.genre, GenrePiece::Vente);
        assert_eq!(p1.total, Montant(360));
        assert_eq!(p1.moyen, Moyen::Carte);
        assert_eq!((p1.recu, p1.rendu), (Montant(360), Montant(0)));
        assert_eq!(p1.horodatage, QUAND);
        assert_eq!(p1.hash_precedent, zeros());
        assert_eq!(p1.hash.len(), 64);
        assert!(
            p1.hash
                .bytes()
                .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
        );
        assert_eq!(
            p1.lignes,
            [LigneVente {
                nom: "Pain".into(),
                quantite: 3,
                prix: Montant(120),
                taux_bp: 550
            }]
        );
        assert_eq!(quantite_de(&s, "Pain"), 7);
        assert_eq!(
            s.historique().last(),
            Some(&Mouvement {
                genre: GenreStock::Sortie,
                nom: "Pain".into(),
                quantite: 3
            })
        );

        let p2 = s
            .encaisser(&demande(&[("Lait", 1)]), Moyen::Autre, Montant(0), QUAND)
            .unwrap()
            .clone();
        assert_eq!(p2.numero, 2);
        assert_eq!(p2.hash_precedent, p1.hash);
        assert_ne!(p2.hash, p1.hash);
        assert_eq!(s.pieces().len(), 2);
        assert_eq!(s.verifier_journal(), Ok(()));
    }

    #[test]
    fn especes_rendu_et_refus() {
        let panier = demande(&[("Pain", 3), ("Lait", 2), ("Savon", 1)]); // 8,00 €
        let mut s = stock_europe();
        let p = s
            .encaisser(&panier, Moyen::Especes, Montant(1000), QUAND)
            .unwrap();
        assert_eq!(
            (p.total, p.recu, p.rendu),
            (Montant(800), Montant(1000), Montant(200))
        );
        let p = s
            .encaisser(&panier, Moyen::Especes, Montant(800), QUAND)
            .unwrap();
        assert_eq!((p.recu, p.rendu), (Montant(800), Montant(0)));

        let avant = etat_de(&s);
        assert_eq!(
            s.encaisser(&panier, Moyen::Especes, Montant(700), QUAND)
                .unwrap_err(),
            ErreurStock::MontantInsuffisant {
                du: "8,00\u{a0}€".into(),
                recu: "7,00\u{a0}€".into()
            }
        );
        assert!(
            s.encaisser(&panier, Moyen::Especes, Montant(-5), QUAND)
                .unwrap_err()
                .to_string()
                .contains("8,00")
        );
        assert_eq!(etat_de(&s), avant);
    }

    #[test]
    fn carte_et_autre_ignorent_le_montant_recu() {
        let mut s = stock_europe();
        for (moyen, recu) in [(Moyen::Carte, 0), (Moyen::Autre, 99_999)] {
            let p = s
                .encaisser(&demande(&[("Pain", 1)]), moyen, Montant(recu), QUAND)
                .unwrap();
            assert_eq!((p.recu, p.rendu), (p.total, Montant(0)), "{moyen:?}");
        }
    }

    #[test]
    fn un_produit_en_manque_empeche_toute_la_vente() {
        let mut s = Stock::nouveau();
        produit(&mut s, "Pain", 10, Some(120), Some(550));
        produit(&mut s, "Lait", 2, Some(95), Some(550));
        let avant = etat_de(&s);
        assert_eq!(
            s.encaisser(
                &demande(&[("Pain", 3), ("Lait", 5)]),
                Moyen::Carte,
                Montant(0),
                QUAND
            )
            .unwrap_err(),
            ErreurStock::StockInsuffisant {
                nom: "Lait".into(),
                disponible: 2,
                demandee: 5
            }
        );
        assert_eq!(etat_de(&s), avant);
        assert_eq!(quantite_de(&s, "Pain"), 10);
    }

    #[test]
    fn un_echec_ne_consomme_pas_de_numero() {
        let mut s = stock_europe();
        assert_eq!(
            s.encaisser(&demande(&[("Riz", 1)]), Moyen::Carte, Montant(0), QUAND)
                .unwrap_err(),
            ErreurStock::PrixManquant("Riz".into())
        );
        assert!(
            s.encaisser(&demande(&[("Pain", 99)]), Moyen::Carte, Montant(0), QUAND)
                .is_err()
        );
        let p = s
            .encaisser(&demande(&[("Pain", 1)]), Moyen::Carte, Montant(0), QUAND)
            .unwrap();
        assert_eq!(p.numero, 1);
        assert_eq!(p.hash_precedent, zeros());
    }

    #[test]
    fn un_produit_saisi_deux_fois_est_cumule_pour_le_stock() {
        let mut s = stock_europe(); // Pain : 10
        let avant = etat_de(&s);
        assert_eq!(
            s.encaisser(
                &demande(&[("Pain", 6), ("pain", 5)]),
                Moyen::Carte,
                Montant(0),
                QUAND
            )
            .unwrap_err(),
            ErreurStock::StockInsuffisant {
                nom: "Pain".into(),
                disponible: 10,
                demandee: 11
            }
        );
        assert_eq!(etat_de(&s), avant);
        // Et 6 + 4 = 10 passe, en une seule ligne de stock.
        s.encaisser(
            &demande(&[("Pain", 6), ("pain", 4)]),
            Moyen::Carte,
            Montant(0),
            QUAND,
        )
        .unwrap();
        assert_eq!(quantite_de(&s, "Pain"), 0);
    }

    #[test]
    fn le_journal_se_verifie_et_detecte_une_alteration() {
        let mut s = stock_europe();
        for article in [("Pain", 1), ("Lait", 1), ("Savon", 1)] {
            s.encaisser(&demande(&[article]), Moyen::Carte, Montant(0), QUAND)
                .unwrap();
        }
        assert_eq!(s.verifier_journal(), Ok(()));
        let intact = s.pieces.clone();

        s.pieces[1].total = Montant(1);
        assert_eq!(
            s.verifier_journal(),
            Err(ErreurStock::JournalAltere { numero: 2 })
        );
        s.pieces = intact.clone();

        s.pieces[0].lignes[0].quantite = 9;
        assert_eq!(
            s.verifier_journal(),
            Err(ErreurStock::JournalAltere { numero: 1 })
        );
        s.pieces = intact.clone();

        // Une pièce retirée du milieu : la suivante ne se raccroche plus à la chaîne.
        s.pieces.remove(1);
        assert_eq!(
            s.verifier_journal(),
            Err(ErreurStock::JournalAltere { numero: 3 })
        );
        s.pieces = intact.clone();

        // Deux pièces échangées.
        s.pieces.swap(0, 1);
        assert!(matches!(
            s.verifier_journal(),
            Err(ErreurStock::JournalAltere { .. })
        ));
        s.pieces = intact.clone();

        // L'empreinte précédente falsifiée.
        s.pieces[2].hash_precedent = zeros();
        assert_eq!(
            s.verifier_journal(),
            Err(ErreurStock::JournalAltere { numero: 3 })
        );
        s.pieces = intact;
        assert_eq!(s.verifier_journal(), Ok(()));
    }

    #[test]
    fn un_journal_vide_est_valide() {
        assert_eq!(Stock::nouveau().verifier_journal(), Ok(()));
    }

    #[test]
    fn l_empreinte_est_celle_du_format_documente() {
        // Valeurs calculées une fois avec `hashlib` (Python) ET `shasum -a 256` sur le texte
        // exact : empreinte précédente + « \n » + texte canonique de la pièce.
        let piece = Piece {
            numero: 1,
            genre: GenrePiece::Vente,
            horodatage: Horodatage {
                utc: 1_700_000_000,
                decalage_minutes: 60,
            },
            lignes: vec![LigneVente {
                nom: "Pain".into(),
                quantite: 3,
                prix: Montant(120),
                taux_bp: 550,
            }],
            moyen: Moyen::Carte,
            total: Montant(360),
            recu: Montant(360),
            rendu: Montant(0),
            hash_precedent: zeros(),
            hash: String::new(),
        };
        let hash1 = crate::journal::empreinte(&zeros(), &piece);
        assert_eq!(
            hash1,
            "cda9fc85affa82d56d8299d062cfe99128d8ea12e3647b53fc92fda60ba17096"
        );
        let annulation = Piece {
            numero: 2,
            genre: GenrePiece::Annulation { de: 1 },
            horodatage: Horodatage {
                utc: 1_700_000_100,
                decalage_minutes: 60,
            },
            total: Montant(-360),
            recu: Montant(-360),
            hash_precedent: hash1.clone(),
            ..piece
        };
        assert_eq!(
            crate::journal::empreinte(&hash1, &annulation),
            "c053caafab2020ad226a7a27b481a1ce03d95f48bd9df4918ce1b554033b1d80"
        );
    }

    #[test]
    fn un_montant_qui_deborde_ou_un_recu_enorme_est_refuse_sans_rien_changer() {
        let mut s = Stock::nouveau();
        produit(&mut s, "Cher", 5, Some(MONTANT_MAX.0), Some(0));
        produit(&mut s, "Pain", 5, Some(120), Some(550));
        let avant = etat_de(&s);
        assert_eq!(
            s.encaisser(&demande(&[("Cher", 2)]), Moyen::Carte, Montant(0), QUAND)
                .unwrap_err(),
            ErreurStock::MontantTropGrand
        );
        assert_eq!(
            s.encaisser(
                &demande(&[("Pain", 1)]),
                Moyen::Especes,
                Montant(MONTANT_MAX.0 + 1),
                QUAND
            )
            .unwrap_err(),
            ErreurStock::MontantTropGrand
        );
        assert_eq!(etat_de(&s), avant);
    }

    #[test]
    fn une_vente_gratuite_est_possible() {
        let mut s = Stock::nouveau();
        produit(&mut s, "Sac", 5, Some(0), Some(0));
        let p = s
            .encaisser(&demande(&[("Sac", 2)]), Moyen::Especes, Montant(0), QUAND)
            .unwrap();
        assert_eq!((p.total, p.rendu), (Montant(0), Montant(0)));
        assert_eq!(quantite_de(&s, "Sac"), 3);
    }

    // ---- Annuler ---------------------------------------------------------------------------

    const PLUS_TARD: Horodatage = Horodatage {
        utc: 1_700_000_500,
        decalage_minutes: 60,
    };

    #[test]
    fn annuler_cree_une_ecriture_inverse_et_remet_le_stock() {
        let mut s = stock_europe(); // Pain : 10
        let vente = s
            .encaisser(&demande(&[("Pain", 3)]), Moyen::Carte, Montant(0), QUAND)
            .unwrap()
            .clone();
        assert_eq!(quantite_de(&s, "Pain"), 7);

        let annulation = s.annuler(1, PLUS_TARD).unwrap().clone();
        assert_eq!(annulation.numero, 2);
        assert_eq!(annulation.genre, GenrePiece::Annulation { de: 1 });
        assert_eq!(annulation.total, Montant(-360));
        assert_eq!(
            (annulation.recu, annulation.rendu),
            (Montant(-360), Montant(0))
        );
        assert_eq!(annulation.moyen, Moyen::Carte);
        assert_eq!(annulation.horodatage, PLUS_TARD);
        assert_eq!(annulation.lignes, vente.lignes);
        assert_eq!(annulation.hash_precedent, vente.hash);
        assert_eq!(quantite_de(&s, "Pain"), 10);
        assert_eq!(
            s.historique().last(),
            Some(&Mouvement {
                genre: GenreStock::Entree,
                nom: "Pain".into(),
                quantite: 3
            })
        );
        assert_eq!(s.verifier_journal(), Ok(()));
    }

    #[test]
    fn annuler_garde_le_moyen_de_paiement_de_la_vente() {
        let mut s = stock_europe();
        s.encaisser(
            &demande(&[("Pain", 1)]),
            Moyen::Especes,
            Montant(500),
            QUAND,
        )
        .unwrap();
        assert_eq!(s.annuler(1, PLUS_TARD).unwrap().moyen, Moyen::Especes);
    }

    #[test]
    fn une_vente_ne_s_annule_qu_une_fois() {
        let mut s = stock_europe();
        s.encaisser(&demande(&[("Pain", 3)]), Moyen::Carte, Montant(0), QUAND)
            .unwrap();
        s.annuler(1, PLUS_TARD).unwrap();
        let avant = etat_de(&s);
        assert_eq!(s.annuler(1, PLUS_TARD), Err(ErreurStock::DejaAnnulee(1)));
        assert_eq!(etat_de(&s), avant);
    }

    #[test]
    fn on_n_annule_pas_une_annulation_ni_une_piece_inconnue() {
        let mut s = stock_europe();
        s.encaisser(&demande(&[("Pain", 3)]), Moyen::Carte, Montant(0), QUAND)
            .unwrap();
        s.annuler(1, PLUS_TARD).unwrap();
        let avant = etat_de(&s);
        assert_eq!(
            s.annuler(2, PLUS_TARD),
            Err(ErreurStock::AnnulationNonPermise(2))
        );
        assert_eq!(
            s.annuler(99, PLUS_TARD),
            Err(ErreurStock::PieceInconnue(99))
        );
        assert_eq!(etat_de(&s), avant);
    }

    #[test]
    fn une_annulation_qui_ferait_deborder_le_stock_est_refusee() {
        let mut s = Stock::nouveau();
        produit(&mut s, "Pain", 0, Some(100), Some(0));
        s.enregistrer_entree("Pain", u32::MAX).unwrap();
        s.encaisser(&demande(&[("Pain", 2)]), Moyen::Carte, Montant(0), QUAND)
            .unwrap();
        s.enregistrer_entree("Pain", 2).unwrap(); // le stock est de nouveau à u32::MAX
        let avant = etat_de(&s);
        assert_eq!(
            s.annuler(1, PLUS_TARD),
            Err(ErreurStock::QuantiteTropGrande)
        );
        assert_eq!(etat_de(&s), avant);
    }

    #[test]
    fn un_produit_vendu_ne_se_renomme_ni_ne_se_supprime() {
        let mut s = stock_europe();
        s.encaisser(&demande(&[("Pain", 1)]), Moyen::Carte, Montant(0), QUAND)
            .unwrap();
        assert_eq!(
            s.renommer_produit("Pain", "Pain complet"),
            Err(ErreurStock::ProduitVendu("Pain".into()))
        );
        assert_eq!(
            s.renommer_produit(" pain ", "PAIN"),
            Err(ErreurStock::ProduitVendu("Pain".into()))
        );
        assert_eq!(
            s.supprimer_produit("Pain"),
            Err(ErreurStock::ProduitVendu("Pain".into()))
        );
        assert_eq!(quantite_de(&s, "Pain"), 9);
        // Un produit jamais vendu reste modifiable.
        s.renommer_produit("Lait", "Lait entier").unwrap();
        s.supprimer_produit("Savon").unwrap();
    }

    #[test]
    fn la_devise_et_le_profil_ne_changent_plus_apres_la_premiere_vente() {
        let mut s = stock_europe();
        // Avant toute vente, tout se règle librement.
        let usd = Reglages {
            profil: Profil::EtatsUnis,
            devise: Devise::Usd,
            taux_defaut_bp: 825,
        };
        s.definir_reglages(usd).unwrap();
        s.definir_reglages(Reglages::default()).unwrap();

        s.encaisser(&demande(&[("Pain", 1)]), Moyen::Carte, Montant(0), QUAND)
            .unwrap();
        // Après une vente : changer la devise ou le profil changerait le sens des montants déjà
        // enregistrés (1 250 centimes deviendraient 1 250 francs).
        assert_eq!(
            s.definir_reglages(usd),
            Err(ErreurStock::ReglagesVerrouilles)
        );
        assert_eq!(
            s.definir_reglages(Reglages {
                devise: Devise::Xof,
                ..Reglages::default()
            }),
            Err(ErreurStock::ReglagesVerrouilles)
        );
        assert_eq!(
            s.definir_reglages(Reglages {
                profil: Profil::EtatsUnis,
                ..Reglages::default()
            }),
            Err(ErreurStock::ReglagesVerrouilles)
        );
        assert_eq!(s.reglages(), Reglages::default());
        // Le taux par défaut, lui, peut toujours changer.
        let nouveau_taux = Reglages {
            taux_defaut_bp: 550,
            ..Reglages::default()
        };
        s.definir_reglages(nouveau_taux).unwrap();
        assert_eq!(s.reglages().taux_defaut_bp, 550);
    }

    // ---- Détail des taxes d'une pièce déjà enregistrée ------------------------------------

    #[test]
    fn le_detail_des_taxes_d_une_piece_est_celui_du_devis() {
        for s in [stock_europe(), stock_us()] {
            let d = s
                .devis(&demande(&[("Pain", 3), ("Lait", 2), ("Savon", 1)]))
                .unwrap();
            let (groupes, total) = detail_taxes(&d.lignes, s.reglages().profil).unwrap();
            assert_eq!(groupes, d.groupes);
            assert_eq!(total, d.total);
        }
        assert_eq!(
            detail_taxes(&[], Profil::Europe),
            Ok((Vec::new(), Montant::ZERO))
        );
    }

    #[test]
    fn le_moyen_de_paiement_par_defaut_est_les_especes() {
        assert_eq!(Moyen::default(), Moyen::Especes);
    }
}
