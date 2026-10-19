//! Le journal des ventes : une chaîne de pièces dont chacune porte l'empreinte de la précédente.
//!
//! L'empreinte d'une pièce est le SHA-256, en hexadécimal, de
//! `<empreinte précédente>` + `\n` + `<texte canonique de la pièce>`, où le texte canonique est
//! `numéro;genre;heure;décalage;total;moyen;reçu;rendu;vente annulée` suivi d'une ligne
//! `nom;quantité;prix;taux` par ligne de la pièce, chacune terminée par `\n`. Changer un seul
//! caractère d'une pièce, ou retirer ou déplacer une pièce, change toutes les empreintes
//! suivantes : c'est ce qui permet de détecter une modification.

use crate::vente::{GenrePiece, Piece};
use sha2::{Digest, Sha256};

/// L'empreinte « précédente » de la toute première pièce.
pub(crate) const HASH_INITIAL: &str =
    "0000000000000000000000000000000000000000000000000000000000000000";

fn texte_canonique(piece: &Piece) -> String {
    let (genre, annulee) = match piece.genre {
        GenrePiece::Vente => ("V", String::new()),
        GenrePiece::Annulation { de } => ("A", de.to_string()),
    };
    let mut texte = format!(
        "{};{};{};{};{};{};{};{};{}\n",
        piece.numero,
        genre,
        piece.horodatage.utc,
        piece.horodatage.decalage_minutes,
        piece.total.0,
        piece.moyen.code(),
        piece.recu.0,
        piece.rendu.0,
        annulee
    );
    for ligne in &piece.lignes {
        texte.push_str(&format!(
            "{};{};{};{}\n",
            ligne.nom, ligne.quantite, ligne.prix.0, ligne.taux_bp
        ));
    }
    texte
}

/// L'empreinte de `piece` chaînée à `hash_precedent` (64 caractères hexadécimaux minuscules).
pub(crate) fn empreinte(hash_precedent: &str, piece: &Piece) -> String {
    let mut sha = Sha256::new();
    sha.update(hash_precedent.as_bytes());
    sha.update(b"\n");
    sha.update(texte_canonique(piece).as_bytes());
    sha.finalize()
        .iter()
        .map(|octet| format!("{octet:02x}"))
        .collect()
}
