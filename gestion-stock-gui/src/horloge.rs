//! L'heure réelle, vue de la fenêtre. La bibliothèque, elle, ne lit jamais l'horloge : l'heure
//! lui est donnée, ce qui rend toutes ses règles testables avec une heure fixe.

use gestion_stock::horodatage::Horodatage;

/// L'instant présent (UTC) avec le décalage de l'heure locale de cet ordinateur.
pub fn maintenant() -> Horodatage {
    let maintenant = chrono::Local::now();
    Horodatage {
        utc: maintenant.timestamp(),
        decalage_minutes: maintenant.offset().local_minus_utc() / 60,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn l_horloge_reelle_donne_une_heure_plausible() {
        let h = maintenant();
        assert!(h.utc > 1_700_000_000, "l'heure UTC est postérieure à 2023");
        assert!(h.utc < 4_000_000_000, "et antérieure à 2096");
        assert!(
            h.decalage_minutes.abs() <= 14 * 60,
            "un décalage horaire réel"
        );
    }
}
