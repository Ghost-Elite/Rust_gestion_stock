//! Les réglages de la boutique : comment les prix sont comptés, et dans quelle devise.

use crate::monnaie::Devise;

/// Deux façons de compter, qui ne sont pas qu'une question d'affichage :
/// - **Europe** : le prix affiché est TTC, la TVA est incluse (et détaillée par taux sur le ticket) ;
/// - **États-Unis** : le prix affiché est hors taxe, la taxe de vente s'ajoute à l'encaissement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Profil {
    Europe,
    EtatsUnis,
}

impl Profil {
    /// Le nom utilisé dans le fichier de données.
    pub fn code(self) -> &'static str {
        match self {
            Profil::Europe => "europe",
            Profil::EtatsUnis => "etats-unis",
        }
    }

    pub fn depuis_code(code: &str) -> Option<Profil> {
        match code {
            "europe" => Some(Profil::Europe),
            "etats-unis" => Some(Profil::EtatsUnis),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Reglages {
    pub profil: Profil,
    pub devise: Devise,
    /// Le taux de TVA (ou de taxe) des produits qui n'ont pas le leur, en centièmes de pour
    /// cent : 2000 pour 20 %.
    pub taux_defaut_bp: u32,
}

impl Default for Reglages {
    /// Valeurs de départ : Europe, euro, 20 %. À vérifier par le commerçant.
    fn default() -> Self {
        Reglages {
            profil: Profil::Europe,
            devise: Devise::Eur,
            taux_defaut_bp: 2000,
        }
    }
}
