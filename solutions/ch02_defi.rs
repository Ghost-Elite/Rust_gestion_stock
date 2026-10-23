// Solution du défi du chapitre 2.
fn est_pair(n: i32) -> bool {
    n % 2 == 0
}

fn moyenne(a: f64, b: f64) -> f64 {
    (a + b) / 2.0
}

// Bonus : une fonction peut renvoyer un tuple
fn min_max(a: i32, b: i32) -> (i32, i32) {
    if a < b { (a, b) } else { (b, a) }
}

fn main() {
    println!("4 est pair ? {}", est_pair(4));
    println!("7 est pair ? {}", est_pair(7));
    println!("moyenne de 10 et 15 : {}", moyenne(10.0, 15.0));

    let (min, max) = min_max(8, 3);
    println!("min = {min}, max = {max}");
}
