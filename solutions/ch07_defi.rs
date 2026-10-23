// Solution du défi du chapitre 7.
fn division(a: f64, b: f64) -> Result<f64, String> {
    if b == 0.0 {
        Err(String::from("division par zéro"))
    } else {
        Ok(a / b)
    }
}

// Version avec une boucle : c'est ce qu'on écrit naturellement au chapitre 7.
// Clippy te proposerait `find` (voir ci-dessous), d'où l'`allow` pour le faire taire ici.
#[allow(clippy::manual_find)]
fn premier_pair(valeurs: &[i32]) -> Option<i32> {
    for &v in valeurs {
        if v % 2 == 0 {
            return Some(v);
        }
    }
    None
}

// Version avec un itérateur (chapitre 8) : même résultat, en une ligne.
fn premier_pair_iterateur(valeurs: &[i32]) -> Option<i32> {
    valeurs.iter().find(|&&v| v % 2 == 0).copied()
}

fn main() {
    println!("{:?}", division(10.0, 4.0));
    println!("{:?}", division(1.0, 0.0));
    println!("{:?}", premier_pair(&[1, 3, 8, 5, 6]));
    println!("{:?}", premier_pair(&[1, 3, 5]));
    println!("{:?}", premier_pair_iterateur(&[1, 3, 8, 5, 6]));
}
