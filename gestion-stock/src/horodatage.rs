//! Un instant et sa date locale, sans bibliothèque de dates.
//!
//! Ce module ne lit JAMAIS l'horloge : l'heure vient de l'extérieur (le programme qui appelle).
//! C'est ce qui rend tout le reste testable avec une heure fixe.

/// Un instant : les secondes écoulées depuis le 1er janvier 1970 (UTC), et le décalage de
/// l'heure locale de la boutique par rapport à UTC au moment où l'instant a été relevé
/// (60 pour la France en hiver, -300 pour New York en hiver, 0 pour Dakar).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Horodatage {
    pub utc: i64,
    pub decalage_minutes: i32,
}

/// Une date et une heure civiles.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DateHeure {
    pub annee: i32,
    pub mois: u32,
    pub jour: u32,
    pub heure: u32,
    pub minute: u32,
    pub seconde: u32,
}

impl Horodatage {
    /// La date et l'heure locales : l'instant UTC décalé, puis converti en date civile.
    pub fn local(self) -> DateHeure {
        let secondes = self.utc + i64::from(self.decalage_minutes) * 60;
        let jours = secondes.div_euclid(86_400);
        let reste = secondes.rem_euclid(86_400);
        let (annee, mois, jour) = date_civile(jours);
        DateHeure {
            annee,
            mois,
            jour,
            heure: (reste / 3600) as u32,
            minute: ((reste % 3600) / 60) as u32,
            seconde: (reste % 60) as u32,
        }
    }
}

impl DateHeure {
    /// `14/11/2023 22:13`
    pub fn en_francais(self) -> String {
        format!(
            "{:02}/{:02}/{} {:02}:{:02}",
            self.jour, self.mois, self.annee, self.heure, self.minute
        )
    }

    /// `2023-11-14`
    pub fn jour_iso(self) -> String {
        format!("{:04}-{:02}-{:02}", self.annee, self.mois, self.jour)
    }
}

/// Convertit un nombre de jours depuis le 1er janvier 1970 en (année, mois, jour). C'est
/// l'algorithme classique de Howard Hinnant, valable aussi avant 1970 et qui gère les années
/// bissextiles (2100 n'en est pas une, 2000 en était une).
fn date_civile(jours: i64) -> (i32, u32, u32) {
    let z = jours + 719_468;
    let ere = z.div_euclid(146_097);
    let jour_de_l_ere = z.rem_euclid(146_097); // 0..=146096
    let annee_de_l_ere = (jour_de_l_ere - jour_de_l_ere / 1460 + jour_de_l_ere / 36_524
        - jour_de_l_ere / 146_096)
        / 365; // 0..=399
    let annee = annee_de_l_ere + ere * 400;
    let jour_de_l_annee =
        jour_de_l_ere - (365 * annee_de_l_ere + annee_de_l_ere / 4 - annee_de_l_ere / 100); // 0..=365
    let mp = (5 * jour_de_l_annee + 2) / 153; // 0..=11, en commençant en mars
    let jour = jour_de_l_annee - (153 * mp + 2) / 5 + 1; // 1..=31
    let mois = if mp < 10 { mp + 3 } else { mp - 9 }; // 1..=12
    let annee = if mois <= 2 { annee + 1 } else { annee };
    (annee as i32, mois as u32, jour as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn local(utc: i64, decalage_minutes: i32) -> DateHeure {
        Horodatage {
            utc,
            decalage_minutes,
        }
        .local()
    }

    fn date(annee: i32, mois: u32, jour: u32, heure: u32, minute: u32, seconde: u32) -> DateHeure {
        DateHeure {
            annee,
            mois,
            jour,
            heure,
            minute,
            seconde,
        }
    }

    #[test]
    fn dates_connues() {
        assert_eq!(local(0, 0), date(1970, 1, 1, 0, 0, 0));
        assert_eq!(local(1_700_000_000, 0), date(2023, 11, 14, 22, 13, 20));
        // 29 février d'une année bissextile.
        assert_eq!(local(1_709_164_800, 0), date(2024, 2, 29, 0, 0, 0));
        // 2100 n'est pas bissextile : le lendemain du 28 février est le 1er mars.
        assert_eq!(local(4_107_542_400, 0), date(2100, 3, 1, 0, 0, 0));
        assert_eq!(local(4_107_542_400 - 1, 0), date(2100, 2, 28, 23, 59, 59));
    }

    #[test]
    fn avant_1970() {
        assert_eq!(local(-1, 0), date(1969, 12, 31, 23, 59, 59));
        assert_eq!(local(-86_400, 0), date(1969, 12, 31, 0, 0, 0));
    }

    #[test]
    fn le_decalage_change_l_heure_et_parfois_le_jour() {
        let l = |d| local(1_700_000_000, d);
        assert_eq!((l(60).jour, l(60).heure), (14, 23));
        // Trois heures de plus : on passe minuit.
        assert_eq!((l(180).jour, l(180).heure, l(180).minute), (15, 1, 13));
        // Cinq heures de moins (côte est des États-Unis).
        assert_eq!((l(-300).jour, l(-300).heure), (14, 17));
        // Un décalage d'une demi-heure (Inde : +5 h 30).
        assert_eq!((l(330).heure, l(330).minute), (3, 43));
    }

    #[test]
    fn formats() {
        let d = local(1_700_000_000, 0);
        assert_eq!(d.en_francais(), "14/11/2023 22:13");
        assert_eq!(d.jour_iso(), "2023-11-14");
        let tot = local(1_704_067_200 + 3600 * 5 + 60 * 7, 0); // 1er janvier 2024, 05:07
        assert_eq!(tot.en_francais(), "01/01/2024 05:07");
        assert_eq!(tot.jour_iso(), "2024-01-01");
    }

    #[test]
    fn toutes_les_dates_d_une_annee_sont_distinctes_et_ordonnees() {
        // Chaque jour de 2024 (bissextile : 366 jours) donne un jour ISO différent, dans l'ordre.
        let debut = 1_704_067_200_i64; // 2024-01-01 00:00:00 UTC
        let mut precedent = String::new();
        for jour in 0..366 {
            let iso = local(debut + jour * 86_400, 0).jour_iso();
            assert!(iso > precedent, "{iso} n'est pas après {precedent}");
            precedent = iso;
        }
        assert_eq!(precedent, "2024-12-31");
    }
}
