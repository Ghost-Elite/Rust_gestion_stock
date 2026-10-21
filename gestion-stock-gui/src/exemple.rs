//! Les articles d'exemple de la maquette « Kora Marché » : quinze produits avec leur prix en FCFA
//! et leur stock, pour essayer la caisse (scan simulé, paiements simulés) sans rien saisir.
//!
//! Ils se chargent par un bouton, jamais tout seuls, et seulement dans une caisse qui n'a encore
//! aucune vente : les montants du journal sont figés dans leur devise.

/// `(nom, prix en FCFA, stock)`. « Lait caillé 500 g » est en rupture, comme dans la maquette.
pub const ARTICLES_EXEMPLE: [(&str, i64, u32); 15] = [
    ("Riz parfumé 5 kg", 4750, 61),
    ("Huile d'arachide 1 L", 1600, 40),
    ("Lait en poudre 400 g", 2350, 34),
    ("Lait en poudre 900 g", 4900, 12),
    ("Lait concentré sucré 397 g", 850, 3),
    ("Lait UHT 1 L", 1100, 48),
    ("Lait caillé 500 g", 900, 0),
    ("Sucre en morceaux 1 kg", 800, 75),
    ("Eau minérale 1,5 L", 400, 230),
    ("Baguette", 175, 140),
    ("Spaghetti 500 g", 650, 30),
    ("Concentré de tomate 400 g", 700, 30),
    ("Yaourt nature × 4", 1500, 22),
    ("Poulet entier", 3400, 9),
    ("Café moulu 250 g", 1900, 5),
];

/// Le seuil d'alerte donné à chaque article d'exemple.
pub const SEUIL_EXEMPLE: u32 = 2;

/// Le taux de TVA des réglages d'exemple, en centièmes de pour cent (18 %).
pub const TAUX_EXEMPLE_BP: u32 = 1800;
