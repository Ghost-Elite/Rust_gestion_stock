// Solution du défi du chapitre 6 (le code du chapitre + les deux ajouts du défi).
#[derive(Debug)]
struct Rectangle {
    largeur: f64,
    hauteur: f64,
}

impl Rectangle {
    // Fonction associée (comme un constructeur)
    fn new(largeur: f64, hauteur: f64) -> Self {
        Self { largeur, hauteur }
    }

    // Méthode : le premier paramètre est &self
    fn aire(&self) -> f64 {
        self.largeur * self.hauteur
    }

    fn est_carre(&self) -> bool {
        self.largeur == self.hauteur
    }

    // Méthode qui modifie : &mut self
    fn agrandir(&mut self, facteur: f64) {
        self.largeur *= facteur;
        self.hauteur *= facteur;
    }

    // DÉFI 2 : le périmètre
    fn perimetre(&self) -> f64 {
        2.0 * (self.largeur + self.hauteur)
    }
}

enum Forme {
    Cercle(f64),
    Rectangle { largeur: f64, hauteur: f64 },
    Point,
    // DÉFI 1 : une nouvelle variante
    Triangle { base: f64, hauteur: f64 },
}

impl Forme {
    fn aire(&self) -> f64 {
        match self {
            Forme::Cercle(rayon) => std::f64::consts::PI * rayon * rayon,
            Forme::Rectangle { largeur, hauteur } => largeur * hauteur,
            Forme::Point => 0.0,
            // Le compilateur refuse de compiler tant que ce bras manque
            Forme::Triangle { base, hauteur } => base * hauteur / 2.0,
        }
    }
}

fn main() {
    let mut r = Rectangle::new(3.0, 4.0);
    println!("{r:?}");
    println!("aire = {}", r.aire());
    println!("périmètre = {}", r.perimetre());
    println!("carré ? {}", r.est_carre());
    r.agrandir(2.0);
    println!(
        "après agrandissement : {r:?}, aire = {}, périmètre = {}",
        r.aire(),
        r.perimetre()
    );

    let formes = [
        Forme::Cercle(1.0),
        Forme::Rectangle {
            largeur: 2.0,
            hauteur: 5.0,
        },
        Forme::Point,
        Forme::Triangle {
            base: 6.0,
            hauteur: 3.0,
        },
    ];
    for forme in &formes {
        println!("aire = {:.2}", forme.aire());
    }
}
