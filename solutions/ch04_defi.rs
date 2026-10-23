// Solution du défi du chapitre 4.
fn main() {
    let texte = String::from("ownership");
    // On doit récupérer la String : la fonction en avait pris la propriété
    let (texte, longueur) = longueur_et_texte(texte);
    println!("« {texte} » fait {longueur} caractères");
}

fn longueur_et_texte(s: String) -> (String, usize) {
    // On calcule la longueur AVANT de déplacer `s` dans le tuple.
    // Écrire `(s, s.len())` ne compile pas : `s` est déjà déplacée
    // quand on appelle `s.len()`.
    let longueur = s.len();
    (s, longueur)
}
