// Solution du défi du chapitre 8.
use std::collections::HashMap;

fn mots_longs(phrase: &str) -> Vec<String> {
    phrase
        .split_whitespace()
        // chars().count() compte les caractères ; len() compterait les octets
        .filter(|mot| mot.chars().count() > 4)
        .map(|mot| mot.to_uppercase())
        .collect()
}

fn frequence_lettres(texte: &str) -> HashMap<char, usize> {
    let mut frequences = HashMap::new();
    for lettre in texte.chars().filter(|c| c.is_alphabetic()) {
        *frequences.entry(lettre).or_insert(0) += 1;
    }
    frequences
}

fn main() {
    println!("{:?}", mots_longs("le petit chat dort sur le canapé"));

    let frequences = frequence_lettres("bonjour le monde");
    let mut triees: Vec<_> = frequences.into_iter().collect();
    triees.sort();
    println!("{triees:?}");
}
