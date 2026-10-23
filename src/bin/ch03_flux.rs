use std::{num::ParseIntError, print};

// Chapitre 3 : tape ici le code du guide (voir GUIDE.md).
fn main() {
    let nombre = 7;

    let parite = if nombre % 2 == 0 { "pair" } else { "impair" };

    print!("{nombre} est {parite}");

    let mut essais = 0;
    let resultat = loop {
        essais += 1;
        if essais == 5 {
            break essais * 10;
        };
    };
    print!("\nloop terminé avec {resultat}\n");

    let mut compte_a_rebours = 3;
    while compte_a_rebours > 0 {
        println!("{compte_a_rebours}...");
        compte_a_rebours -= 1;
    }

    println!("Decollage !");

    for i in 1..=5 {
        print!("{i} x 3 = {} \n", i * 3);
    }

    for note in [4, 10, 14, 18] {
        println!("{note}/20 -> {}", mention(note));
    }

    fn mention(note: u32) -> &'static str {
        match note {
            0..=9 => "insuffisant",
            10..=13 => "passable",
            14..=16 => "bien",
            17..=20 => "tres bien",
            _ => "note invalide",
        }
    }
}
