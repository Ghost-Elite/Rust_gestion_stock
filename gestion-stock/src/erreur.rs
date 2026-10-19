use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErreurStock {
    /// Nom vide, ou contenant `;` ou un saut de ligne.
    NomInvalide(String),
    /// Contient le nom déjà enregistré (écriture d'origine).
    ProduitExistant(String),
    /// Contient le nom tel que tapé (nettoyé).
    ProduitInconnu(String),
    QuantiteNulle,
    QuantiteTropGrande,
    StockInsuffisant {
        nom: String,
        disponible: u32,
        demandee: u32,
    },
    FichierCorrompu {
        ligne: usize,
    },
    Io(String),
    Utilisation(String),
    /// La saisie (nettoyée) n'est pas un montant valide.
    MontantInvalide(String),
    /// La saisie (nettoyée) n'est pas un taux valide.
    TauxInvalide(String),
    MontantTropGrand,
    /// Aucun article dans le panier.
    PanierVide,
    /// Ce produit n'a pas de prix : on ne peut pas le vendre.
    PrixManquant(String),
}

impl fmt::Display for ErreurStock {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ErreurStock::NomInvalide(nom) => write!(
                f,
                "Nom de produit invalide : « {nom} » (il doit être non vide, sans « ; » ni saut de ligne)."
            ),
            ErreurStock::ProduitExistant(nom) => write!(f, "Le produit « {nom} » existe déjà."),
            ErreurStock::ProduitInconnu(nom) => write!(f, "Produit inconnu : « {nom} »."),
            ErreurStock::QuantiteNulle => write!(f, "La quantité doit être supérieure à zéro."),
            ErreurStock::QuantiteTropGrande => write!(
                f,
                "La quantité totale dépasserait la limite autorisée ({}).",
                u32::MAX
            ),
            ErreurStock::StockInsuffisant {
                nom,
                disponible,
                demandee,
            } => write!(
                f,
                "Stock insuffisant pour « {nom} » : {disponible} disponible(s), {demandee} demandé(s)."
            ),
            ErreurStock::FichierCorrompu { ligne } => {
                write!(f, "Fichier de données corrompu à la ligne {ligne}.")
            }
            ErreurStock::Io(message) => write!(f, "{message}"),
            ErreurStock::Utilisation(message) => write!(f, "{message}"),
            ErreurStock::MontantInvalide(saisie) => {
                write!(f, "Montant invalide : « {saisie} » (exemple : 12,50).")
            }
            ErreurStock::TauxInvalide(saisie) => {
                write!(f, "Taux invalide : « {saisie} » (exemple : 5,5).")
            }
            ErreurStock::MontantTropGrand => {
                write!(f, "Le montant dépasse la limite autorisée.")
            }
            ErreurStock::PanierVide => write!(f, "Le panier est vide."),
            ErreurStock::PrixManquant(nom) => write!(
                f,
                "Le produit « {nom} » n'a pas de prix : donnez-lui un prix avant de le vendre."
            ),
        }
    }
}

impl std::error::Error for ErreurStock {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn message_stock_insuffisant() {
        let e = ErreurStock::StockInsuffisant {
            nom: "Riz 5kg".into(),
            disponible: 3,
            demandee: 5,
        };
        assert_eq!(
            e.to_string(),
            "Stock insuffisant pour « Riz 5kg » : 3 disponible(s), 5 demandé(s)."
        );
    }

    #[test]
    fn message_fichier_corrompu() {
        assert_eq!(
            ErreurStock::FichierCorrompu { ligne: 2 }.to_string(),
            "Fichier de données corrompu à la ligne 2."
        );
    }
}
