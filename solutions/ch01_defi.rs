// Solution du défi du chapitre 1.
fn main() {
    // Un compteur mutable, incrémenté trois fois
    let mut compteur = 0;
    println!("compteur = {compteur}");
    compteur += 1;
    println!("compteur = {compteur}");
    compteur += 1;
    println!("compteur = {compteur}");
    compteur += 1;
    println!("compteur = {compteur}");

    // Bonus : le shadowing peut changer le type de la variable
    let compteur = compteur.to_string();
    println!("« {compteur} » a {} caractère(s)", compteur.len());
}
