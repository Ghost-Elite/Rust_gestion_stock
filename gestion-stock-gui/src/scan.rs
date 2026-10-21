//! Ce que veut dire le texte tapé (ou scanné) dans le champ de l'écran de vente. Aucune fenêtre
//! ici : on n'a besoin que de la liste des produits, donc tout se teste.
//!
//! Formes acceptées : un nom, ou une partie de nom d'au moins trois caractères (sans casse ni
//! accents), éventuellement précédé d'une quantité : `3 x pain`, `3*pain`, `3×pain`.

use crate::catalogue::normaliser;
use crate::etat::LigneProduit;

/// Ce qu'il faut faire du texte tapé.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResultatScan {
    /// Rien n'est tapé.
    Vide,
    /// Un seul produit correspond : on le met au ticket, en cette quantité.
    Ajouter { nom: String, quantite: u32 },
    /// Plusieurs produits correspondent : on ouvre la recherche avec ce texte.
    Plusieurs,
    /// Aucun produit ne correspond (ou le texte est trop court) ; le texte, pour le message.
    Introuvable(String),
}

/// Le préfixe de quantité : de un à trois chiffres, puis `*`, `x`, `X` ou `×`. Renvoie la quantité
/// (`0` vaut 1) et le reste du texte ; sans préfixe, la quantité est 1 et le texte est entier.
fn lire_prefixe(saisie: &str) -> (u32, &str) {
    let chiffres = saisie.chars().take_while(char::is_ascii_digit).count();
    if chiffres == 0 || chiffres > 3 {
        return (1, saisie);
    }
    // Les chiffres ASCII font un octet chacun : `chiffres` est une position valide.
    let mut suite = saisie[chiffres..].trim_start().chars();
    if !matches!(suite.next(), Some('*' | 'x' | 'X' | '\u{d7}')) {
        return (1, saisie);
    }
    let nom = suite.as_str().trim_start();
    if nom.is_empty() {
        return (1, saisie);
    }
    let quantite = saisie[..chiffres].parse::<u32>().unwrap_or(1).max(1);
    (quantite, nom)
}

/// Analyse le texte tapé contre la liste des produits.
pub fn analyser(saisie: &str, produits: &[LigneProduit]) -> ResultatScan {
    let saisie = saisie.trim();
    if saisie.is_empty() {
        return ResultatScan::Vide;
    }
    let (quantite, texte) = lire_prefixe(saisie);
    let cherche = normaliser(texte);
    if cherche.chars().count() < 3 {
        return ResultatScan::Introuvable(saisie.to_string());
    }
    // Un nom qui est exactement ce qui est tapé l'emporte sur ceux qui le contiennent.
    let normalises: Vec<(&LigneProduit, String)> = produits
        .iter()
        .map(|produit| (produit, normaliser(&produit.nom)))
        .collect();
    if let [(produit, _)] = normalises
        .iter()
        .filter(|(_, nom)| *nom == cherche)
        .collect::<Vec<_>>()
        .as_slice()
    {
        return ResultatScan::Ajouter {
            nom: produit.nom.clone(),
            quantite,
        };
    }
    let trouves: Vec<&LigneProduit> = normalises
        .iter()
        .filter(|(_, nom)| nom.contains(&cherche))
        .map(|(produit, _)| *produit)
        .collect();
    match trouves.as_slice() {
        [] => ResultatScan::Introuvable(saisie.to_string()),
        [produit] => ResultatScan::Ajouter {
            nom: produit.nom.clone(),
            quantite,
        },
        _ => ResultatScan::Plusieurs,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gestion_stock::monnaie::Montant;

    fn produit(nom: &str) -> LigneProduit {
        LigneProduit {
            nom: nom.to_string(),
            quantite: 10,
            seuil: 0,
            en_alerte: false,
            prix: Some(Montant(100)),
            taux_bp: None,
        }
    }

    fn produits() -> Vec<LigneProduit> {
        ["Pain", "Savon", "Lait", "Laitue", "Thé vert"]
            .into_iter()
            .map(produit)
            .collect()
    }

    fn ajouter(nom: &str, quantite: u32) -> ResultatScan {
        ResultatScan::Ajouter {
            nom: nom.to_string(),
            quantite,
        }
    }

    #[test]
    fn un_texte_vide_ou_fait_d_espaces_est_vide() {
        assert_eq!(analyser("", &produits()), ResultatScan::Vide);
        assert_eq!(analyser("   \t ", &produits()), ResultatScan::Vide);
    }

    #[test]
    fn un_nom_unique_s_ajoute_sans_casse_ni_accents() {
        assert_eq!(analyser("pain", &produits()), ajouter("Pain", 1));
        assert_eq!(analyser("PAIN", &produits()), ajouter("Pain", 1));
        assert_eq!(analyser("  sav ", &produits()), ajouter("Savon", 1));
        assert_eq!(analyser("the", &produits()), ajouter("Thé vert", 1));
    }

    #[test]
    fn le_prefixe_de_quantite_se_lit() {
        for saisie in [
            "3 x pain",
            "3*pain",
            "3×pain",
            "3 X pain",
            "3x pain",
            "3 *  pain",
        ] {
            assert_eq!(
                analyser(saisie, &produits()),
                ajouter("Pain", 3),
                "{saisie}"
            );
        }
    }

    #[test]
    fn zero_vaut_un_et_quatre_chiffres_ne_sont_pas_une_quantite() {
        assert_eq!(analyser("0 x pain", &produits()), ajouter("Pain", 1));
        assert_eq!(analyser("999 x pain", &produits()), ajouter("Pain", 999));
        assert_eq!(
            analyser("1000 x pain", &produits()),
            ResultatScan::Introuvable("1000 x pain".to_string())
        );
    }

    #[test]
    fn moins_de_trois_caracteres_ou_aucun_produit_est_introuvable() {
        assert_eq!(
            analyser("pa", &produits()),
            ResultatScan::Introuvable("pa".to_string())
        );
        assert_eq!(
            analyser("zzz", &produits()),
            ResultatScan::Introuvable("zzz".to_string())
        );
        // Une quantité sans nom n'est pas un article.
        assert_eq!(
            analyser("3 x", &produits()),
            ResultatScan::Introuvable("3 x".to_string())
        );
    }

    #[test]
    fn plusieurs_produits_correspondants_ouvrent_la_recherche() {
        assert_eq!(analyser("lai", &produits()), ResultatScan::Plusieurs);
        assert_eq!(analyser("2 x lai", &produits()), ResultatScan::Plusieurs);
        // Mais le nom entier d'un produit l'emporte sur les noms qui le contiennent seulement.
        assert_eq!(analyser("lait", &produits()), ajouter("Lait", 1));
        assert_eq!(analyser("2 x LAIT", &produits()), ajouter("Lait", 2));
    }
}
