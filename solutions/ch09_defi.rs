// Solution du défi du chapitre 9 (le code du chapitre + les deux ajouts du défi).
use std::fmt;

trait Animal {
    fn nom(&self) -> String;
    fn cri(&self) -> String;

    // Méthode avec implémentation par défaut
    fn presentation(&self) -> String {
        format!("{} fait « {} »", self.nom(), self.cri())
    }
}

struct Chien;
struct Chat {
    nom: String,
}

// DÉFI 1 : une nouvelle espèce
struct Vache;

impl Animal for Chien {
    fn nom(&self) -> String {
        String::from("Le chien")
    }

    fn cri(&self) -> String {
        String::from("ouaf")
    }
}

impl Animal for Chat {
    fn nom(&self) -> String {
        self.nom.clone()
    }

    fn cri(&self) -> String {
        String::from("miaou")
    }

    // On peut aussi remplacer l'implémentation par défaut
    fn presentation(&self) -> String {
        format!("{} ronronne puis dit « {} »", self.nom, self.cri())
    }
}

impl Animal for Vache {
    fn nom(&self) -> String {
        String::from("La vache")
    }

    fn cri(&self) -> String {
        String::from("meuh")
    }
}

// Générique avec contrainte de trait : T doit implémenter Animal
fn fais_parler<T: Animal>(animal: &T) {
    println!("{}", animal.presentation());
}

// Générique : fonctionne pour tout type comparable et copiable
fn plus_grand<T: PartialOrd + Copy>(valeurs: &[T]) -> T {
    let mut max = valeurs[0];
    for &v in valeurs {
        if v > max {
            max = v;
        }
    }
    max
}

// Un struct générique
struct Paire<T> {
    a: T,
    b: T,
}

impl<T: fmt::Display> fmt::Display for Paire<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.a, self.b)
    }
}

// DÉFI 2 : affiche chaque élément sur sa propre ligne
fn affiche_tout<T: fmt::Display>(elements: &[T]) {
    for element in elements {
        println!("* {element}");
    }
}

fn main() {
    let chien = Chien;
    let chat = Chat {
        nom: String::from("Mimi"),
    };
    fais_parler(&chien);
    fais_parler(&chat);

    // dyn Trait : une collection d'animaux de types différents
    let zoo: Vec<Box<dyn Animal>> = vec![Box::new(Chien), Box::new(chat), Box::new(Vache)];
    for animal in &zoo {
        println!("- {}", animal.presentation());
    }

    println!("{}", plus_grand(&[3, 9, 2]));
    println!("{}", plus_grand(&[1.5, 0.2, 7.25]));
    println!("{}", plus_grand(&['x', 'b', 'z']));

    let p = Paire { a: 1, b: 2 };
    println!("{p}");
    let q = Paire {
        a: "gauche",
        b: "droite",
    };
    println!("{q}");

    affiche_tout(&[10, 20, 30]);
    affiche_tout(&["un", "deux"]);
    affiche_tout(&[Paire { a: 1, b: 2 }, Paire { a: 3, b: 4 }]);
}
