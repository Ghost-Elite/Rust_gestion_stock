use crate::erreur::ErreurStock;
use crate::monnaie::{MONTANT_MAX, Montant};
use crate::reglages::Reglages;
use crate::vente::Piece;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Produit {
    pub nom: String,
    pub quantite: u32,
    pub seuil: u32,
    /// Le prix de vente (TTC en Europe, hors taxe aux États-Unis) ; sans prix, pas de vente.
    pub prix: Option<Montant>,
    /// Le taux de TVA ou de taxe de ce produit, en centièmes de pour cent ; sans taux, celui
    /// des réglages.
    pub taux_bp: Option<u32>,
}

impl Produit {
    /// Vrai quand il reste au plus `seuil` unités : il est temps de réapprovisionner.
    pub fn en_alerte(&self) -> bool {
        self.quantite <= self.seuil
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Genre {
    Entree,
    Sortie,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mouvement {
    pub genre: Genre,
    pub nom: String,
    pub quantite: u32,
}

#[derive(Debug, Default, Clone)]
pub struct Stock {
    pub(crate) produits: Vec<Produit>,
    pub(crate) historique: Vec<Mouvement>,
    pub(crate) reglages: Reglages,
    pub(crate) pieces: Vec<Piece>,
}

/// Retire les espaces autour du nom et refuse ce qui casserait le fichier de données.
fn nettoyer_nom(nom: &str) -> Result<String, ErreurStock> {
    let propre = nom.trim();
    if propre.is_empty() || propre.contains([';', '\n', '\r']) {
        return Err(ErreurStock::NomInvalide(propre.to_string()));
    }
    Ok(propre.to_string())
}

impl Stock {
    pub fn nouveau() -> Stock {
        Stock::default()
    }

    /// Cherche un produit sans tenir compte de la casse ni des espaces autour du nom.
    fn position(&self, nom: &str) -> Option<usize> {
        let cherche = nom.trim().to_lowercase();
        self.produits
            .iter()
            .position(|p| p.nom.to_lowercase() == cherche)
    }

    /// Vérifications communes aux entrées et aux sorties : quantité non nulle, produit connu.
    fn trouver(&self, nom: &str, quantite: u32) -> Result<usize, ErreurStock> {
        if quantite == 0 {
            return Err(ErreurStock::QuantiteNulle);
        }
        self.chercher(nom)
    }

    /// Position du produit, ou `ProduitInconnu` avec le nom tel que tapé (nettoyé).
    pub(crate) fn chercher(&self, nom: &str) -> Result<usize, ErreurStock> {
        self.position(nom)
            .ok_or_else(|| ErreurStock::ProduitInconnu(nom.trim().to_string()))
    }

    /// Applique un mouvement déjà validé : nouvelle quantité + ligne d'historique.
    fn appliquer(
        &mut self,
        i: usize,
        genre: Genre,
        quantite: u32,
        nouvelle_quantite: u32,
    ) -> Result<&Produit, ErreurStock> {
        let produit = &mut self.produits[i];
        produit.quantite = nouvelle_quantite;
        self.historique.push(Mouvement {
            genre,
            nom: produit.nom.clone(),
            quantite,
        });
        Ok(&self.produits[i])
    }

    pub fn ajouter_produit(&mut self, nom: &str, seuil: u32) -> Result<(), ErreurStock> {
        let nom = nettoyer_nom(nom)?;
        if let Some(i) = self.position(&nom) {
            return Err(ErreurStock::ProduitExistant(self.produits[i].nom.clone()));
        }
        self.produits.push(Produit {
            nom,
            quantite: 0,
            seuil,
            prix: None,
            taux_bp: None,
        });
        Ok(())
    }

    /// Donne (ou retire, avec `None`) le prix et le taux d'un produit. Rien ne change si une
    /// valeur est refusée : prix négatif ou au-delà de la limite, taux au-delà de 100 %.
    pub fn definir_prix(
        &mut self,
        nom: &str,
        prix: Option<Montant>,
        taux_bp: Option<u32>,
    ) -> Result<(), ErreurStock> {
        let i = self.chercher(nom)?;
        if let Some(prix) = prix {
            if prix.0 < 0 {
                return Err(ErreurStock::MontantInvalide(
                    self.reglages.devise.formater(prix),
                ));
            }
            if prix > MONTANT_MAX {
                return Err(ErreurStock::MontantTropGrand);
            }
        }
        if let Some(taux) = taux_bp {
            self.verifier_taux(taux)?;
        }
        self.produits[i].prix = prix;
        self.produits[i].taux_bp = taux_bp;
        Ok(())
    }

    fn verifier_taux(&self, taux_bp: u32) -> Result<(), ErreurStock> {
        if taux_bp > 10_000 {
            return Err(ErreurStock::TauxInvalide(
                self.reglages.devise.formater_taux(taux_bp),
            ));
        }
        Ok(())
    }

    pub fn reglages(&self) -> Reglages {
        self.reglages
    }

    /// Une fois qu'une vente est enregistrée, la devise et le profil sont figés : les montants du
    /// journal sont des entiers en unités mineures, et les relire dans une autre devise (ou en
    /// hors taxe au lieu de TTC) leur ferait dire autre chose. Le taux par défaut peut changer.
    pub fn definir_reglages(&mut self, reglages: Reglages) -> Result<(), ErreurStock> {
        self.verifier_taux(reglages.taux_defaut_bp)?;
        if !self.pieces.is_empty()
            && (reglages.devise != self.reglages.devise || reglages.profil != self.reglages.profil)
        {
            return Err(ErreurStock::ReglagesVerrouilles);
        }
        self.reglages = reglages;
        Ok(())
    }

    pub fn enregistrer_entree(
        &mut self,
        nom: &str,
        quantite: u32,
    ) -> Result<&Produit, ErreurStock> {
        let i = self.trouver(nom, quantite)?;
        let total = self.produits[i]
            .quantite
            .checked_add(quantite)
            .ok_or(ErreurStock::QuantiteTropGrande)?;
        self.appliquer(i, Genre::Entree, quantite, total)
    }

    pub fn enregistrer_sortie(
        &mut self,
        nom: &str,
        quantite: u32,
    ) -> Result<&Produit, ErreurStock> {
        let i = self.trouver(nom, quantite)?;
        let produit = &self.produits[i];
        let reste = produit.quantite.checked_sub(quantite).ok_or_else(|| {
            ErreurStock::StockInsuffisant {
                nom: produit.nom.clone(),
                disponible: produit.quantite,
                demandee: quantite,
            }
        })?;
        self.appliquer(i, Genre::Sortie, quantite, reste)
    }

    /// Un produit qui figure dans une pièce du journal garde son nom pour toujours : le journal
    /// ne se modifie jamais, et une annulation doit pouvoir retrouver le produit à remettre en stock.
    fn verifier_non_vendu(&self, i: usize) -> Result<(), ErreurStock> {
        let nom = &self.produits[i].nom;
        let cherche = nom.to_lowercase();
        let vendu = self
            .pieces
            .iter()
            .flat_map(|piece| piece.lignes.iter())
            .any(|ligne| ligne.nom.to_lowercase() == cherche);
        if vendu {
            return Err(ErreurStock::ProduitVendu(nom.clone()));
        }
        Ok(())
    }

    /// Ordre des vérifications : nom invalide, produit inconnu, produit déjà vendu, nom déjà
    /// pris par un AUTRE produit.
    /// Changer seulement la casse ou les espaces du même produit est permis.
    /// Le nouveau nom remplace aussi l'ancien dans tout l'historique.
    pub fn renommer_produit(&mut self, ancien: &str, nouveau: &str) -> Result<(), ErreurStock> {
        let nouveau = nettoyer_nom(nouveau)?;
        let i = self.chercher(ancien)?;
        self.verifier_non_vendu(i)?;
        if let Some(j) = self.position(&nouveau)
            && j != i
        {
            return Err(ErreurStock::ProduitExistant(self.produits[j].nom.clone()));
        }
        let ancien_nom = self.produits[i].nom.to_lowercase();
        for mouvement in &mut self.historique {
            if mouvement.nom.to_lowercase() == ancien_nom {
                mouvement.nom = nouveau.clone();
            }
        }
        self.produits[i].nom = nouveau;
        Ok(())
    }

    /// Retire le produit ; l'historique est conservé tel quel.
    pub fn supprimer_produit(&mut self, nom: &str) -> Result<(), ErreurStock> {
        let i = self.chercher(nom)?;
        self.verifier_non_vendu(i)?;
        self.produits.remove(i);
        Ok(())
    }

    pub fn produits(&self) -> Vec<&Produit> {
        let mut liste: Vec<&Produit> = self.produits.iter().collect();
        liste.sort_by_key(|p| p.nom.to_lowercase());
        liste
    }

    pub fn produits_en_alerte(&self) -> Vec<&Produit> {
        self.produits()
            .into_iter()
            .filter(|p| p.en_alerte())
            .collect()
    }

    pub fn historique(&self) -> &[Mouvement] {
        &self.historique
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stock_avec_riz() -> Stock {
        let mut s = Stock::nouveau();
        s.ajouter_produit("Riz 5kg", 5).unwrap();
        s
    }

    #[test]
    fn ajouter_refuse_les_noms_invalides() {
        let mut s = Stock::nouveau();
        for nom in ["", "   ", "Riz;5kg", "Riz\n5kg"] {
            assert!(
                matches!(s.ajouter_produit(nom, 0), Err(ErreurStock::NomInvalide(_))),
                "{nom:?}"
            );
        }
    }

    #[test]
    fn ajouter_refuse_un_doublon_sans_tenir_compte_de_la_casse() {
        let mut s = Stock::nouveau();
        s.ajouter_produit("Riz", 0).unwrap();
        assert_eq!(
            s.ajouter_produit("  riz ", 0),
            Err(ErreurStock::ProduitExistant("Riz".into()))
        );
    }

    #[test]
    fn ajouter_garde_l_ecriture_d_origine_nettoyee() {
        let mut s = Stock::nouveau();
        s.ajouter_produit("  Riz 5kg ", 5).unwrap();
        assert_eq!(s.produits()[0].nom, "Riz 5kg");
    }

    #[test]
    fn entree_retrouve_le_produit_malgre_la_casse_et_les_espaces() {
        let mut s = stock_avec_riz();
        assert_eq!(s.enregistrer_entree(" RIZ 5KG ", 12).unwrap().quantite, 12);
    }

    #[test]
    fn sortie_diminue_la_quantite() {
        let mut s = stock_avec_riz();
        s.enregistrer_entree("Riz 5kg", 12).unwrap();
        assert_eq!(s.enregistrer_sortie("Riz 5kg", 8).unwrap().quantite, 4);
    }

    #[test]
    fn sortie_trop_grande_refusee_et_rien_ne_change() {
        let mut s = stock_avec_riz();
        s.enregistrer_entree("Riz 5kg", 3).unwrap();
        assert_eq!(
            s.enregistrer_sortie("Riz 5kg", 5).unwrap_err(),
            ErreurStock::StockInsuffisant {
                nom: "Riz 5kg".into(),
                disponible: 3,
                demandee: 5
            }
        );
        assert_eq!(s.produits()[0].quantite, 3);
        assert_eq!(s.historique().len(), 1); // seule l'entrée est enregistrée
    }

    #[test]
    fn quantite_nulle_refusee_pour_entree_et_sortie() {
        let mut s = stock_avec_riz();
        assert_eq!(
            s.enregistrer_entree("Riz 5kg", 0).unwrap_err(),
            ErreurStock::QuantiteNulle
        );
        assert_eq!(
            s.enregistrer_sortie("Riz 5kg", 0).unwrap_err(),
            ErreurStock::QuantiteNulle
        );
    }

    #[test]
    fn produit_inconnu() {
        let mut s = stock_avec_riz();
        assert_eq!(
            s.enregistrer_entree("Sucre", 1).unwrap_err(),
            ErreurStock::ProduitInconnu("Sucre".into())
        );
    }

    #[test]
    fn entree_qui_depasse_u32_max_refusee_sans_rien_changer() {
        let mut s = stock_avec_riz();
        s.enregistrer_entree("Riz 5kg", u32::MAX).unwrap();
        assert_eq!(
            s.enregistrer_entree("Riz 5kg", 1).unwrap_err(),
            ErreurStock::QuantiteTropGrande
        );
        assert_eq!(s.produits()[0].quantite, u32::MAX);
        assert_eq!(s.historique().len(), 1);
    }

    #[test]
    fn alerte_quand_quantite_inferieure_ou_egale_au_seuil() {
        let mut s = stock_avec_riz(); // seuil 5
        s.enregistrer_entree("Riz 5kg", 6).unwrap();
        assert!(s.produits_en_alerte().is_empty());
        s.enregistrer_sortie("Riz 5kg", 1).unwrap(); // 5 <= 5
        assert_eq!(s.produits_en_alerte().len(), 1);
    }

    #[test]
    fn produits_tries_par_nom_sans_tenir_compte_de_la_casse() {
        let mut s = Stock::nouveau();
        for nom in ["citron", "Banane", "abricot"] {
            s.ajouter_produit(nom, 0).unwrap();
        }
        let noms: Vec<&str> = s.produits().iter().map(|p| p.nom.as_str()).collect();
        assert_eq!(noms, ["abricot", "Banane", "citron"]);
    }

    #[test]
    fn historique_garde_l_ordre_et_le_nom_d_origine() {
        let mut s = stock_avec_riz();
        s.enregistrer_entree("riz 5kg", 12).unwrap();
        s.enregistrer_sortie("RIZ 5KG", 2).unwrap();
        assert_eq!(
            s.historique(),
            [
                Mouvement {
                    genre: Genre::Entree,
                    nom: "Riz 5kg".into(),
                    quantite: 12
                },
                Mouvement {
                    genre: Genre::Sortie,
                    nom: "Riz 5kg".into(),
                    quantite: 2
                },
            ]
        );
    }

    #[test]
    fn renommer_change_le_nom_du_produit_et_celui_de_l_historique() {
        let mut s = stock_avec_riz();
        s.enregistrer_entree("Riz 5kg", 12).unwrap();
        s.renommer_produit("riz 5KG", "  Riz basmati 5kg ").unwrap();
        assert_eq!(s.produits()[0].nom, "Riz basmati 5kg");
        assert_eq!(s.produits()[0].quantite, 12);
        assert_eq!(s.historique()[0].nom, "Riz basmati 5kg");
        assert!(s.enregistrer_entree("Riz basmati 5kg", 1).is_ok());
    }

    #[test]
    fn renommer_autorise_le_changement_de_casse_du_meme_produit() {
        let mut s = stock_avec_riz();
        s.renommer_produit("Riz 5kg", "RIZ 5KG").unwrap();
        assert_eq!(s.produits()[0].nom, "RIZ 5KG");
    }

    #[test]
    fn renommer_refuse_le_nom_d_un_autre_produit() {
        let mut s = stock_avec_riz();
        s.ajouter_produit("Sucre", 0).unwrap();
        assert_eq!(
            s.renommer_produit("Sucre", " riz 5KG "),
            Err(ErreurStock::ProduitExistant("Riz 5kg".into()))
        );
        let noms: Vec<&str> = s.produits().iter().map(|p| p.nom.as_str()).collect();
        assert_eq!(noms, ["Riz 5kg", "Sucre"]);
    }

    #[test]
    fn renommer_refuse_un_nom_invalide_ou_un_produit_inconnu() {
        let mut s = stock_avec_riz();
        for nom in ["", "   ", "a;b", "a\nb"] {
            assert!(
                matches!(
                    s.renommer_produit("Riz 5kg", nom),
                    Err(ErreurStock::NomInvalide(_))
                ),
                "{nom:?}"
            );
        }
        assert_eq!(
            s.renommer_produit("Sel", "Sel fin"),
            Err(ErreurStock::ProduitInconnu("Sel".into()))
        );
        assert_eq!(s.produits()[0].nom, "Riz 5kg");
    }

    #[test]
    fn supprimer_retire_le_produit_et_garde_l_historique() {
        let mut s = stock_avec_riz();
        s.enregistrer_entree("Riz 5kg", 3).unwrap();
        s.supprimer_produit(" RIZ 5KG ").unwrap();
        assert!(s.produits().is_empty());
        assert_eq!(s.historique().len(), 1);
        assert_eq!(
            s.supprimer_produit("RIZ 5KG"),
            Err(ErreurStock::ProduitInconnu("RIZ 5KG".into()))
        );
    }

    #[test]
    fn un_clone_est_independant_de_l_original() {
        let s = stock_avec_riz();
        let mut copie = s.clone();
        copie.enregistrer_entree("Riz 5kg", 7).unwrap();
        assert_eq!(s.produits()[0].quantite, 0);
        assert_eq!(copie.produits()[0].quantite, 7);
    }

    #[test]
    fn definir_prix_et_taux_sur_un_produit_connu() {
        use crate::monnaie::Montant;
        let mut s = stock_avec_riz();
        s.definir_prix(" RIZ 5KG ", Some(Montant(350)), Some(550))
            .unwrap();
        let p = &s.produits()[0];
        assert_eq!((p.prix, p.taux_bp), (Some(Montant(350)), Some(550)));
        s.definir_prix("Riz 5kg", None, None).unwrap();
        let p = &s.produits()[0];
        assert_eq!((p.prix, p.taux_bp), (None, None));
    }

    #[test]
    fn definir_prix_refuse_les_valeurs_impossibles_et_ne_change_rien() {
        use crate::monnaie::{MONTANT_MAX, Montant};
        let mut s = stock_avec_riz();
        s.definir_prix("Riz 5kg", Some(Montant(350)), Some(550))
            .unwrap();
        assert_eq!(
            s.definir_prix("Sel", Some(Montant(100)), None),
            Err(ErreurStock::ProduitInconnu("Sel".into()))
        );
        assert!(matches!(
            s.definir_prix("Riz 5kg", None, Some(10_001)),
            Err(ErreurStock::TauxInvalide(_))
        ));
        assert!(matches!(
            s.definir_prix("Riz 5kg", Some(Montant(-1)), None),
            Err(ErreurStock::MontantInvalide(_))
        ));
        assert_eq!(
            s.definir_prix("Riz 5kg", Some(Montant(MONTANT_MAX.0 + 1)), None),
            Err(ErreurStock::MontantTropGrand)
        );
        let p = &s.produits()[0];
        assert_eq!((p.prix, p.taux_bp), (Some(Montant(350)), Some(550)));
    }

    #[test]
    fn les_reglages_par_defaut_sont_europe_eur_vingt_pour_cent() {
        use crate::monnaie::Devise;
        use crate::reglages::Profil;
        let r = Stock::nouveau().reglages();
        assert_eq!(
            (r.profil, r.devise, r.taux_defaut_bp),
            (Profil::Europe, Devise::Eur, 2000)
        );
    }

    #[test]
    fn definir_reglages_les_conserve_et_refuse_un_taux_au_dela_de_cent_pour_cent() {
        use crate::monnaie::Devise;
        use crate::reglages::{Profil, Reglages};
        let mut s = Stock::nouveau();
        let voulus = Reglages {
            profil: Profil::EtatsUnis,
            devise: Devise::Usd,
            taux_defaut_bp: 825,
        };
        s.definir_reglages(voulus).unwrap();
        assert_eq!(s.reglages(), voulus);
        let trop = Reglages {
            taux_defaut_bp: 10_001,
            ..voulus
        };
        assert!(matches!(
            s.definir_reglages(trop),
            Err(ErreurStock::TauxInvalide(_))
        ));
        assert_eq!(s.reglages(), voulus);
    }
}
