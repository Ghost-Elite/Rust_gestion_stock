//! Les montants d'argent. Toujours en entiers, en unités mineures (centimes) : un nombre à
//! virgule se trompe sur les centimes (0,1 + 0,2 n'est pas 0,3), un entier jamais.

use crate::erreur::ErreurStock;

/// Un montant en unités mineures (centimes ; pour le FCFA, des francs entiers).
/// Négatif pour un remboursement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Montant(pub i64);

/// Au-delà (10 millions d'unités), on refuse : c'est presque sûrement une erreur de saisie,
/// et cela garde de la marge avant tout dépassement d'entier.
pub const MONTANT_MAX: Montant = Montant(1_000_000_000);

fn dans_les_limites(valeur: i64) -> Result<Montant, ErreurStock> {
    if valeur.unsigned_abs() > MONTANT_MAX.0.unsigned_abs() {
        Err(ErreurStock::MontantTropGrand)
    } else {
        Ok(Montant(valeur))
    }
}

impl Montant {
    pub const ZERO: Montant = Montant(0);

    pub fn ajouter(self, autre: Montant) -> Result<Montant, ErreurStock> {
        let somme = self
            .0
            .checked_add(autre.0)
            .ok_or(ErreurStock::MontantTropGrand)?;
        dans_les_limites(somme)
    }

    pub fn multiplier(self, quantite: u32) -> Result<Montant, ErreurStock> {
        let produit = self
            .0
            .checked_mul(i64::from(quantite))
            .ok_or(ErreurStock::MontantTropGrand)?;
        dans_les_limites(produit)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Devise {
    Eur,
    Usd,
    Gbp,
    Xof,
}

impl Devise {
    pub fn code(self) -> &'static str {
        match self {
            Devise::Eur => "EUR",
            Devise::Usd => "USD",
            Devise::Gbp => "GBP",
            Devise::Xof => "XOF",
        }
    }

    pub fn depuis_code(code: &str) -> Option<Devise> {
        match code {
            "EUR" => Some(Devise::Eur),
            "USD" => Some(Devise::Usd),
            "GBP" => Some(Devise::Gbp),
            "XOF" => Some(Devise::Xof),
            _ => None,
        }
    }

    /// Le FCFA n'a pas de centimes.
    pub fn decimales(self) -> u32 {
        match self {
            Devise::Xof => 0,
            _ => 2,
        }
    }

    /// Le symbole de la devise : `€`, `$`, `£`, `FCFA`.
    pub fn symbole(self) -> &'static str {
        match self {
            Devise::Eur => "€",
            Devise::Usd => "$",
            Devise::Gbp => "£",
            Devise::Xof => "FCFA",
        }
    }

    /// `$12.50` mais `12,50 €` : l'usage change avec la devise.
    pub fn symbole_avant(self) -> bool {
        matches!(self, Devise::Usd | Devise::Gbp)
    }

    fn separateur_decimal(self) -> char {
        match self {
            Devise::Eur | Devise::Xof => ',',
            Devise::Usd | Devise::Gbp => '.',
        }
    }

    fn separateur_milliers(self) -> char {
        match self {
            Devise::Eur | Devise::Xof => '\u{a0}',
            Devise::Usd | Devise::Gbp => ',',
        }
    }

    /// Le nombre sans symbole : `1234,50`, avec ou sans séparateur de milliers.
    fn nombre(self, montant: Montant, milliers: bool) -> String {
        let absolu = montant.0.unsigned_abs();
        let diviseur = 10u64.pow(self.decimales());
        let (entier, fraction) = (absolu / diviseur, absolu % diviseur);
        let chiffres = entier.to_string();
        let mut texte = String::new();
        for (i, c) in chiffres.chars().enumerate() {
            if milliers && i > 0 && (chiffres.len() - i) % 3 == 0 {
                texte.push(self.separateur_milliers());
            }
            texte.push(c);
        }
        if self.decimales() > 0 {
            texte.push(self.separateur_decimal());
            texte.push_str(&format!(
                "{fraction:0largeur$}",
                largeur = self.decimales() as usize
            ));
        }
        texte
    }

    /// Le nombre seul, pour l'affichage en grand (le symbole se place à part) : `4,90`,
    /// `17 025`, `1,234.56`, avec les séparateurs de milliers de la devise et le signe moins.
    pub fn formater_nombre(self, montant: Montant) -> String {
        let signe = if montant.0 < 0 { "-" } else { "" };
        format!("{signe}{}", self.nombre(montant, true))
    }

    /// Pour l'affichage : `12,50 €`, `$1,234.50`, `15 000 FCFA`.
    pub fn formater(self, montant: Montant) -> String {
        let signe = if montant.0 < 0 { "-" } else { "" };
        let nombre = self.nombre(montant, true);
        if self.symbole_avant() {
            format!("{signe}{}{nombre}", self.symbole())
        } else {
            format!("{signe}{nombre}\u{a0}{}", self.symbole())
        }
    }

    /// Pour pré-remplir un champ de saisie : `1234,50`, relisible par `lire`.
    pub fn formater_brut(self, montant: Montant) -> String {
        let signe = if montant.0 < 0 { "-" } else { "" };
        format!("{signe}{}", self.nombre(montant, false))
    }

    /// Lit un montant tapé : des chiffres, avec au plus un séparateur décimal (virgule ou
    /// point) et pas plus de décimales que la devise n'en a. Jamais de signe ni d'espace.
    pub fn lire(self, texte: &str) -> Result<Montant, ErreurStock> {
        let saisie = texte.trim();
        let invalide = || ErreurStock::MontantInvalide(saisie.to_string());
        let (entier, fraction) = decouper(saisie).ok_or_else(invalide)?;
        let decimales = self.decimales() as usize;
        if fraction.len() > decimales {
            return Err(invalide());
        }
        let entier: u64 = entier.parse().map_err(|_| ErreurStock::MontantTropGrand)?;
        let fraction = format!("{fraction:0<decimales$}");
        let fraction: u64 = if decimales == 0 {
            0
        } else {
            fraction.parse().map_err(|_| invalide())?
        };
        let valeur = entier
            .checked_mul(10u64.pow(self.decimales()))
            .and_then(|e| e.checked_add(fraction))
            .ok_or(ErreurStock::MontantTropGrand)?;
        let valeur = i64::try_from(valeur).map_err(|_| ErreurStock::MontantTropGrand)?;
        dans_les_limites(valeur)
    }

    /// `20 %`, `5,5 %`, `8.25 %` selon l'usage de la devise.
    pub fn formater_taux(self, taux_bp: u32) -> String {
        format!("{}\u{a0}%", self.formater_taux_brut(taux_bp))
    }

    /// Le nombre seul, sans le signe %, tel qu'on le retape dans un champ : `20`, `5,5`, `8.25`.
    pub fn formater_taux_brut(self, taux_bp: u32) -> String {
        let (entier, centiemes) = (taux_bp / 100, taux_bp % 100);
        if centiemes == 0 {
            entier.to_string()
        } else if centiemes % 10 == 0 {
            format!("{entier}{}{}", self.separateur_decimal(), centiemes / 10)
        } else {
            format!("{entier}{}{centiemes:02}", self.separateur_decimal())
        }
    }
}

/// Sépare « 12,50 » en (« 12 », « 50 »). `None` si ce ne sont pas que des chiffres, s'il y a
/// plus d'un séparateur ou un séparateur sans rien après.
fn decouper(saisie: &str) -> Option<(&str, &str)> {
    let (entier, fraction) = match saisie.find([',', '.']) {
        Some(i) => (&saisie[..i], &saisie[i + 1..]),
        None => (saisie, ""),
    };
    let separateur_sans_decimales = saisie.len() > entier.len() && fraction.is_empty();
    let que_des_chiffres = |t: &str| t.bytes().all(|b| b.is_ascii_digit());
    if entier.is_empty()
        || !que_des_chiffres(entier)
        || !que_des_chiffres(fraction)
        || separateur_sans_decimales
    {
        return None;
    }
    Some((entier, fraction))
}

/// Lit un taux de TVA ou de taxe tapé (« 5,5 ») et le rend en centièmes de pour cent (550).
/// Entre 0 et 100 %, avec au plus deux décimales.
pub fn lire_taux(texte: &str) -> Result<u32, ErreurStock> {
    let saisie = texte.trim();
    let invalide = || ErreurStock::TauxInvalide(saisie.to_string());
    let (entier, fraction) = decouper(saisie).ok_or_else(invalide)?;
    if fraction.len() > 2 {
        return Err(invalide());
    }
    let entier: u32 = entier.parse().map_err(|_| invalide())?;
    let fraction: u32 = format!("{fraction:0<2}").parse().map_err(|_| invalide())?;
    let taux = entier
        .checked_mul(100)
        .and_then(|e| e.checked_add(fraction))
        .ok_or_else(invalide)?;
    if taux > 10_000 {
        return Err(invalide());
    }
    Ok(taux)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::erreur::ErreurStock;

    #[test]
    fn formats_exacts() {
        assert_eq!(Devise::Eur.formater(Montant(1250)), "12,50\u{a0}€");
        assert_eq!(
            Devise::Eur.formater(Montant(123450)),
            "1\u{a0}234,50\u{a0}€"
        );
        assert_eq!(Devise::Eur.formater(Montant(5)), "0,05\u{a0}€");
        assert_eq!(Devise::Eur.formater(Montant(-1250)), "-12,50\u{a0}€");
        assert_eq!(Devise::Usd.formater(Montant(123450)), "$1,234.50");
        assert_eq!(Devise::Usd.formater(Montant(-1250)), "-$12.50");
        assert_eq!(Devise::Gbp.formater(Montant(1250)), "£12.50");
        assert_eq!(
            Devise::Xof.formater(Montant(15000)),
            "15\u{a0}000\u{a0}FCFA"
        );
        assert_eq!(Devise::Xof.formater(Montant(0)), "0\u{a0}FCFA");
    }

    #[test]
    fn lecture_des_montants_saisis() {
        assert_eq!(Devise::Eur.lire("12,50"), Ok(Montant(1250)));
        assert_eq!(Devise::Eur.lire("12.5"), Ok(Montant(1250)));
        assert_eq!(Devise::Eur.lire(" 12 "), Ok(Montant(1200)));
        assert_eq!(Devise::Eur.lire("0,05"), Ok(Montant(5)));
        assert_eq!(Devise::Xof.lire("1500"), Ok(Montant(1500)));
        for mauvais in [
            "", "abc", "-3", "1 000", "1e3", "12,505", "12,", ",5", "1,2,3",
        ] {
            assert!(
                matches!(
                    Devise::Eur.lire(mauvais),
                    Err(ErreurStock::MontantInvalide(_))
                ),
                "{mauvais:?}"
            );
        }
        assert!(matches!(
            Devise::Xof.lire("15,5"),
            Err(ErreurStock::MontantInvalide(_))
        ));
        assert_eq!(Devise::Eur.lire("10000000"), Ok(Montant(1_000_000_000)));
        assert_eq!(
            Devise::Eur.lire("10000000,01"),
            Err(ErreurStock::MontantTropGrand)
        );
        assert_eq!(
            Devise::Eur.lire("99999999999999999999999"),
            Err(ErreurStock::MontantTropGrand)
        );
    }

    #[test]
    fn le_message_d_erreur_reprend_la_saisie() {
        let e = Devise::Eur.lire(" abc ").unwrap_err();
        assert_eq!(
            e.to_string(),
            "Montant invalide : « abc » (exemple : 12,50)."
        );
    }

    #[test]
    fn le_format_brut_se_relit_a_l_identique() {
        assert_eq!(Devise::Eur.formater_brut(Montant(123450)), "1234,50");
        assert_eq!(Devise::Usd.formater_brut(Montant(123450)), "1234.50");
        assert_eq!(Devise::Xof.formater_brut(Montant(15000)), "15000");
        assert_eq!(Devise::Eur.formater_brut(Montant(-1250)), "-12,50");
        for devise in [Devise::Eur, Devise::Usd, Devise::Gbp, Devise::Xof] {
            for centimes in [0, 5, 99, 100, 1250, 123450, 99_999_999, 1_000_000_000] {
                let montant = Montant(centimes);
                assert_eq!(
                    devise.lire(&devise.formater_brut(montant)),
                    Ok(montant),
                    "{devise:?} {centimes}"
                );
            }
        }
    }

    #[test]
    fn codes_de_devise() {
        for devise in [Devise::Eur, Devise::Usd, Devise::Gbp, Devise::Xof] {
            assert_eq!(Devise::depuis_code(devise.code()), Some(devise));
        }
        assert_eq!(Devise::depuis_code("JPY"), None);
        assert_eq!(Devise::depuis_code("eur"), None);
        assert_eq!(
            (
                Devise::Eur.decimales(),
                Devise::Usd.decimales(),
                Devise::Xof.decimales()
            ),
            (2, 2, 0)
        );
    }

    #[test]
    fn taux() {
        assert_eq!(lire_taux("20"), Ok(2000));
        assert_eq!(lire_taux("5,5"), Ok(550));
        assert_eq!(lire_taux("8.25"), Ok(825));
        assert_eq!(lire_taux("0"), Ok(0));
        assert_eq!(lire_taux("100"), Ok(10000));
        for mauvais in ["", "abc", "-1", "100,01", "5,555", "1,"] {
            assert!(
                matches!(lire_taux(mauvais), Err(ErreurStock::TauxInvalide(_))),
                "{mauvais:?}"
            );
        }
        assert_eq!(Devise::Eur.formater_taux(2000), "20\u{a0}%");
        assert_eq!(Devise::Eur.formater_taux(550), "5,5\u{a0}%");
        assert_eq!(Devise::Usd.formater_taux(825), "8.25\u{a0}%");
        assert_eq!(Devise::Usd.formater_taux(0), "0\u{a0}%");
    }

    #[test]
    fn arithmetique_verifiee() {
        assert_eq!(Montant(100).multiplier(3), Ok(Montant(300)));
        assert_eq!(Montant(100).ajouter(Montant(-30)), Ok(Montant(70)));
        assert_eq!(
            Montant(MONTANT_MAX.0).multiplier(2),
            Err(ErreurStock::MontantTropGrand)
        );
        assert_eq!(
            Montant(MONTANT_MAX.0).ajouter(Montant(1)),
            Err(ErreurStock::MontantTropGrand)
        );
        assert_eq!(
            Montant(i64::MAX).multiplier(u32::MAX),
            Err(ErreurStock::MontantTropGrand)
        );
        assert_eq!(
            Montant(i64::MAX).ajouter(Montant(1)),
            Err(ErreurStock::MontantTropGrand)
        );
    }

    #[test]
    fn le_taux_brut_se_relit_tel_quel_dans_un_champ_de_saisie() {
        assert_eq!(Devise::Eur.formater_taux_brut(550), "5,5");
        assert_eq!(Devise::Eur.formater_taux_brut(2000), "20");
        assert_eq!(Devise::Eur.formater_taux_brut(0), "0");
        assert_eq!(Devise::Usd.formater_taux_brut(825), "8.25");
        for devise in [Devise::Eur, Devise::Usd, Devise::Gbp, Devise::Xof] {
            for taux in [0, 5, 550, 825, 1000, 2000, 10000] {
                let texte = devise.formater_taux_brut(taux);
                assert_eq!(lire_taux(&texte), Ok(taux), "{devise:?} {texte}");
            }
        }
        // `formater_taux` ajoute seulement l'espace insécable et le signe %.
        assert_eq!(Devise::Eur.formater_taux(550), "5,5\u{a0}%");
    }

    #[test]
    fn le_nombre_se_formate_sans_symbole_avec_les_milliers() {
        assert_eq!(Devise::Xof.formater_nombre(Montant(17_025)), "17\u{a0}025");
        assert_eq!(Devise::Eur.formater_nombre(Montant(490)), "4,90");
        assert_eq!(Devise::Usd.formater_nombre(Montant(259)), "2.59");
        assert_eq!(Devise::Usd.formater_nombre(Montant(123_456)), "1,234.56");
        assert_eq!(Devise::Eur.formater_nombre(Montant(-250)), "-2,50");
    }

    #[test]
    fn le_symbole_et_sa_place_sont_publics() {
        assert_eq!(Devise::Xof.symbole(), "FCFA");
        assert_eq!(Devise::Eur.symbole(), "\u{20ac}");
        assert_eq!(Devise::Usd.symbole(), "$");
        assert_eq!(Devise::Gbp.symbole(), "\u{a3}");
        assert!(Devise::Usd.symbole_avant() && Devise::Gbp.symbole_avant());
        assert!(!Devise::Eur.symbole_avant() && !Devise::Xof.symbole_avant());
    }

    #[test]
    fn nombre_et_symbole_recomposent_le_montant_formate() {
        for devise in [Devise::Eur, Devise::Usd, Devise::Gbp, Devise::Xof] {
            for montant in [Montant(0), Montant(5), Montant(1250), Montant(123_450)] {
                let nombre = devise.formater_nombre(montant);
                let recompose = if devise.symbole_avant() {
                    format!("{}{nombre}", devise.symbole())
                } else {
                    format!("{nombre}\u{a0}{}", devise.symbole())
                };
                assert_eq!(
                    recompose,
                    devise.formater(montant),
                    "{devise:?} {montant:?}"
                );
            }
        }
    }
}
