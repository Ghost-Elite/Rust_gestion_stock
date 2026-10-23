// Chapitre 2 : tape ici le code du guide (voir GUIDE.md).

use std::println;

fn main() {
    let entier: i32 = -42;
    let petit: u8 = 255;
    let prix: f64 = 19.99;
    let vrai: bool = true;
    let lettre: char = 'R';

    println!("===================Rust=================");

    println!("{entier} {petit} {prix} {vrai} {lettre}");

    let persone = ("Awa", 28, 1.68);

    let (nom, age, taille) = persone;
    println!("{nom} a {age} ans et mesure {taille}");
    println!("Premier element du tuple : {}", persone.0);

    let jours = ["lundi", "mardi", "mercredi"];

    println!("Premier jour : {}", jours[0]);
    println!("Nombre de jours : {}", jours.len());

    let somme = additionner(2, 3);
    println!("2 + 3 = {somme}");
    println!("Le carré de 7 est {}", carre(7));

    let temperature_c = 25;
    let temperature_f = celsius_vers_fahrenheit(temperature_c as f64);

    println!("{temperature_c} °C = {temperature_f}");
}

fn additionner(a: i32, b: i32) -> i32 {
    a + b
}

fn carre(x: i32) -> i32 {
    x * x
}

fn celsius_vers_fahrenheit(c: f64) -> f64 {
    c * 9.0 / 5.0 + 32.0
}
