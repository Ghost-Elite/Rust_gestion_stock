//! Des produits courants de petite boutique, proposés quand on ajoute un produit : on les
//! choisit d'un clic au lieu de les taper, avec un seuil d'alerte conseillé.
//!
//! C'est une liste de départ, à adapter à la boutique : les produits les plus courants sont en
//! premier (ce sont eux que l'on voit tant que rien n'est tapé).

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Suggestion {
    pub nom: &'static str,
    /// Le seuil d'alerte conseillé : en dessous, il est temps de réapprovisionner.
    pub seuil: u32,
}

const fn p(nom: &'static str, seuil: u32) -> Suggestion {
    Suggestion { nom, seuil }
}

pub const CATALOGUE: &[Suggestion] = &[
    // Épicerie de base
    p("Riz parfumé 5kg", 5),
    p("Sucre en poudre 1kg", 10),
    p("Huile de tournesol 1 L", 8),
    p("Farine de blé 1kg", 8),
    p("Sel fin 1kg", 6),
    p("Lait entier 1 L", 12),
    p("Café moulu 250g", 6),
    p("Pâtes alimentaires 500g", 12),
    p("Huile d'arachide 1 L", 8),
    p("Huile de palme 1 L", 6),
    p("Riz parfumé 25kg", 3),
    p("Farine de blé 25kg", 3),
    p("Sucre en morceaux 1kg", 6),
    p("Spaghetti 500g", 12),
    p("Couscous 1kg", 6),
    p("Semoule fine 1kg", 6),
    p("Mil 1kg", 6),
    p("Maïs 1kg", 6),
    p("Lentilles 500g", 6),
    p("Haricots secs 1kg", 6),
    // Conserves et cuisine
    p("Concentré de tomate 70g", 20),
    p("Cubes de bouillon x24", 10),
    p("Sardines à l'huile", 24),
    p("Thon en conserve", 12),
    p("Lait en poudre 400g", 8),
    p("Lait concentré sucré", 12),
    // Petit-déjeuner, boissons
    p("Café soluble 100g", 6),
    p("Thé vert 100g", 6),
    p("Thé noir x25 sachets", 6),
    p("Chocolat en poudre 400g", 4),
    p("Biscuits secs", 10),
    p("Pain de mie", 6),
    p("Œufs x12", 6),
    p("Beurre 250g", 6),
    p("Eau minérale 1,5 L", 24),
    p("Jus d'orange 1 L", 10),
    p("Boisson gazeuse 1,5 L", 12),
    // Hygiène et maison
    p("Savon de toilette", 12),
    p("Savon de Marseille", 8),
    p("Lessive en poudre 1kg", 6),
    p("Liquide vaisselle 500 ml", 6),
    p("Eau de Javel 1 L", 6),
    p("Papier toilette x6", 10),
    p("Mouchoirs en papier", 8),
    p("Dentifrice 75 ml", 8),
    p("Brosse à dents", 10),
    p("Shampoing 250 ml", 6),
    p("Sacs poubelle x20", 6),
    p("Allumettes", 24),
    p("Bougies x6", 10),
    p("Piles AA x4", 8),
    p("Ampoule LED", 6),
    // Fruits et légumes
    p("Oignons 1kg", 10),
    p("Pommes de terre 1kg", 10),
    p("Tomates 1kg", 8),
    p("Bananes 1kg", 8),
    p("Ail 250g", 6),
];

/// Minuscules, sans accents, espaces réduits : « Thé  VERT » et « the vert » se valent.
pub fn normaliser(texte: &str) -> String {
    let mut sortie = String::with_capacity(texte.len());
    for c in texte.to_lowercase().chars() {
        match c {
            'à' | 'â' | 'ä' => sortie.push('a'),
            'ç' => sortie.push('c'),
            'é' | 'è' | 'ê' | 'ë' => sortie.push('e'),
            'î' | 'ï' => sortie.push('i'),
            'ô' | 'ö' => sortie.push('o'),
            'ù' | 'û' | 'ü' => sortie.push('u'),
            'ÿ' => sortie.push('y'),
            'œ' => sortie.push_str("oe"),
            'æ' => sortie.push_str("ae"),
            autre => sortie.push(autre),
        }
    }
    sortie.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Les produits du catalogue à proposer pour ce que la personne a tapé.
/// - tous les mots tapés doivent se retrouver dans le nom (sans casse ni accents) ;
/// - ceux qui COMMENCENT par la saisie passent avant les autres ;
/// - un produit déjà en stock (`deja`) n'est jamais proposé : il serait refusé comme doublon ;
/// - un nom déjà tapé en entier n'est pas re-proposé.
pub fn suggerer<'a>(
    catalogue: &'a [Suggestion],
    saisie: &str,
    deja: &[String],
    limite: usize,
) -> Vec<&'a Suggestion> {
    let saisie = normaliser(saisie);
    let mots: Vec<&str> = saisie.split(' ').filter(|m| !m.is_empty()).collect();
    let presents: Vec<String> = deja.iter().map(|nom| normaliser(nom)).collect();
    let mut trouves: Vec<(bool, &Suggestion)> = catalogue
        .iter()
        .filter_map(|s| {
            let nom = normaliser(s.nom);
            let propose = nom != saisie
                && !presents.contains(&nom)
                && mots.iter().all(|mot| nom.contains(mot));
            propose.then(|| (nom.starts_with(&saisie), s))
        })
        .collect();
    // Tri stable : à égalité, on garde l'ordre du catalogue (les plus courants d'abord).
    trouves.sort_by_key(|(commence_par, _)| !commence_par);
    trouves.into_iter().map(|(_, s)| s).take(limite).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use gestion_stock::stock::Stock;

    fn noms<'a>(suggestions: &[&'a Suggestion]) -> Vec<&'a str> {
        suggestions.iter().map(|s| s.nom).collect()
    }

    #[test]
    fn la_normalisation_ignore_la_casse_et_les_accents() {
        assert_eq!(normaliser("Thé VERT"), "the vert");
        assert_eq!(normaliser("Café Crème Brûlée"), "cafe creme brulee");
        assert_eq!(normaliser("Œufs x12"), "oeufs x12");
        assert_eq!(normaliser("  Pâtes   500g "), "pates 500g");
    }

    #[test]
    fn sans_saisie_on_propose_les_premiers_produits_courants() {
        let s = suggerer(CATALOGUE, "", &[], 6);
        assert_eq!(s.len(), 6);
        assert_eq!(
            noms(&s),
            noms(&CATALOGUE.iter().take(6).collect::<Vec<_>>())
        );
    }

    #[test]
    fn on_filtre_sans_tenir_compte_de_la_casse_ni_des_accents() {
        assert!(noms(&suggerer(CATALOGUE, "the", &[], 20)).contains(&"Thé vert 100g"));
        assert!(noms(&suggerer(CATALOGUE, "CAFE", &[], 20)).contains(&"Café moulu 250g"));
        assert!(suggerer(CATALOGUE, "zzzz", &[], 20).is_empty());
    }

    #[test]
    fn tous_les_mots_tapes_doivent_se_retrouver_dans_le_nom() {
        let s = noms(&suggerer(CATALOGUE, "huile 1 l", &[], 20));
        assert!(s.contains(&"Huile de tournesol 1 L"));
        assert!(s.iter().all(|n| normaliser(n).contains("huile")));
        assert!(!s.contains(&"Sel fin 1kg"));
    }

    #[test]
    fn les_noms_qui_commencent_par_la_saisie_passent_avant_les_autres() {
        // « lait » est au début de « Lait entier 1 L » mais au milieu de « Chocolat au lait ».
        let catalogue = [
            Suggestion {
                nom: "Chocolat au lait",
                seuil: 4,
            },
            Suggestion {
                nom: "Lait entier 1 L",
                seuil: 12,
            },
        ];
        let s = suggerer(&catalogue, "lait", &[], 5);
        assert_eq!(noms(&s), ["Lait entier 1 L", "Chocolat au lait"]);
    }

    #[test]
    fn un_produit_deja_en_stock_n_est_jamais_propose() {
        let deja = vec!["CAFÉ MOULU 250G".to_string(), "Sel fin 1kg".to_string()];
        let s = noms(&suggerer(CATALOGUE, "", &deja, 100));
        assert!(!s.contains(&"Café moulu 250g"));
        assert!(!s.contains(&"Sel fin 1kg"));
        assert!(s.len() >= CATALOGUE.len() - 2);
    }

    #[test]
    fn un_nom_deja_tape_en_entier_n_est_pas_re_propose() {
        let s = noms(&suggerer(CATALOGUE, "Sel fin 1kg", &[], 20));
        assert!(!s.contains(&"Sel fin 1kg"));
    }

    #[test]
    fn la_limite_est_respectee() {
        assert_eq!(suggerer(CATALOGUE, "", &[], 3).len(), 3);
        assert_eq!(suggerer(CATALOGUE, "", &[], 0).len(), 0);
    }

    #[test]
    fn le_catalogue_est_sain() {
        assert!(
            CATALOGUE.len() >= 40,
            "un catalogue assez large pour servir"
        );
        let mut stock = Stock::nouveau();
        for s in CATALOGUE {
            assert!(s.seuil > 0 && s.seuil <= 100, "seuil de {}", s.nom);
            // Chaque nom doit être accepté par le stock : valide, et sans doublon
            // (casse et accents mis à part, que `ajouter_produit` ne confond pas).
            stock
                .ajouter_produit(s.nom, s.seuil)
                .unwrap_or_else(|e| panic!("« {} » refusé : {e}", s.nom));
        }
        let mut normalises: Vec<String> = CATALOGUE.iter().map(|s| normaliser(s.nom)).collect();
        normalises.sort();
        normalises.dedup();
        assert_eq!(normalises.len(), CATALOGUE.len(), "noms en double");
    }
}
