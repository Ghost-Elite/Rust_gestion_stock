// Solution du défi du chapitre 5.
fn plus_grand(valeurs: &[i32]) -> i32 {
    let mut max = valeurs[0];
    for &v in valeurs {
        if v > max {
            max = v;
        }
    }
    max
}

// Il faut `&mut String` : on veut modifier la String de l'appelant.
fn met_en_majuscules(texte: &mut String) {
    *texte = texte.to_uppercase();
}

fn main() {
    let notes = [12, 17, 9, 14];
    println!("plus grande note : {}", plus_grand(&notes));

    let mut cri = String::from("rust");
    met_en_majuscules(&mut cri);
    println!("{cri}");
}
