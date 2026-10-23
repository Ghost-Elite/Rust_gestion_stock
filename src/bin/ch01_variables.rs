// Chapitre 1 : tape ici le code du guide (voir GUIDE.md).

use std::{print, println};

const MAX_POINTS: u32 = 100;

fn main() {
    print!("Bonjour Rust ! \n");

    let age = 30;

    println!("J'ai {age} ans");

    let mut score = 0;

    println!("Score de depart : {score}");

    score += 10;

    println!("Score apres un bonus : {score}");

    // Shadowing (masquage) : on redeclare une variable avec le meme nom
    // grace a un nouveau `let`. La nouvelle variable "cache" l'ancienne.
    //
    // Ici, `espaces` est d'abord un &str (le texte " ").
    let espaces = "   ";
    // Puis `espaces` devient un usize (la longueur du texte = 1).
    // Le nom est reutilise, mais le type change : c'est permis avec `let`.
    // L'ancienne valeur (le &str) n'est plus accessible a partir d'ici.
    let espaces = espaces.len();
    //
    // Difference avec `mut` : avec `let mut espaces = " ";` suivi de
    // `espaces = espaces.len();`, on aurait une erreur de compilation,
    // car `mut` permet de changer la valeur mais pas le type.
    println!(" Il y a {espaces} espaces");

    println!("Maximum : {MAX_POINTS}");
}
