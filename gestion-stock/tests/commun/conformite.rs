//! Le scénario unique que tout dépôt (fichier, MySQL) doit réussir : les règles du métier sont
//! celles du cœur, quel que soit l'endroit où les données vivent.

use gestion_stock::depot::Depot;
use gestion_stock::erreur::ErreurStock;
use gestion_stock::horodatage::Horodatage;
use gestion_stock::monnaie::{MONTANT_MAX, Montant};
use gestion_stock::reglages::{Profil, Reglages};
use gestion_stock::stock::{Genre, Produit};
use gestion_stock::vente::{GenrePiece, Moyen};

const QUAND: Horodatage = Horodatage {
    utc: 1_700_000_000,
    decalage_minutes: 60,
};

fn panier(articles: &[(&str, u32)]) -> Vec<(String, u32)> {
    articles.iter().map(|(n, q)| (n.to_string(), *q)).collect()
}

/// `(nom, quantité, seuil, prix, taux)` d'un produit.
type LigneProduit = (String, u32, u32, Option<i64>, Option<u32>);

/// Tous les produits, dans l'ordre du cœur.
fn produits(depot: &mut dyn Depot) -> Vec<LigneProduit> {
    depot
        .instantane()
        .unwrap()
        .produits()
        .into_iter()
        .map(|p: &Produit| {
            (
                p.nom.clone(),
                p.quantite,
                p.seuil,
                p.prix.map(|m| m.0),
                p.taux_bp,
            )
        })
        .collect()
}

/// À lancer sur un dépôt NEUF (aucun produit, aucune vente), réglages d'origine.
pub fn parcourir_les_regles(depot: &mut dyn Depot) {
    // ---- Les produits : unicité sans casse ni espaces, accents comptés ----
    depot.ajouter_produit("Riz 5kg", 5).unwrap();
    let existe = ErreurStock::ProduitExistant("Riz 5kg".to_string());
    assert_eq!(depot.ajouter_produit("riz 5KG", 0).unwrap_err(), existe);
    assert_eq!(depot.ajouter_produit("  Riz 5kg ", 0).unwrap_err(), existe);
    depot.ajouter_produit("Thé", 0).unwrap();
    depot.ajouter_produit("the", 0).unwrap(); // un autre produit : les accents comptent
    depot.ajouter_produit("Pain 🥖", 0).unwrap();
    for mauvais in ["", "   ", "a;b", "a\nb"] {
        assert!(
            matches!(
                depot.ajouter_produit(mauvais, 0),
                Err(ErreurStock::NomInvalide(_))
            ),
            "« {mauvais:?} » devrait être refusé"
        );
    }

    // ---- Entrées et sorties ----
    let riz = depot.enregistrer_entree("riz 5kg", 10).unwrap();
    assert_eq!((riz.nom.as_str(), riz.quantite), ("Riz 5kg", 10));
    assert_eq!(
        depot.enregistrer_entree("Riz 5kg", 0).unwrap_err(),
        ErreurStock::QuantiteNulle
    );
    assert_eq!(
        depot.enregistrer_sortie("Riz 5kg", 11).unwrap_err(),
        ErreurStock::StockInsuffisant {
            nom: "Riz 5kg".to_string(),
            disponible: 10,
            demandee: 11
        }
    );
    assert_eq!(depot.enregistrer_sortie("Riz 5kg", 4).unwrap().quantite, 6);
    assert_eq!(
        depot.enregistrer_entree("Inconnu", 1).unwrap_err(),
        ErreurStock::ProduitInconnu("Inconnu".to_string())
    );
    assert_eq!(
        depot.enregistrer_entree("Riz 5kg", u32::MAX).unwrap_err(),
        ErreurStock::QuantiteTropGrande
    );
    depot.enregistrer_entree("Thé", 5).unwrap();
    depot.enregistrer_entree("the", 1).unwrap();
    depot.enregistrer_entree("Pain 🥖", 10).unwrap();

    // ---- Prix et taux ----
    depot
        .definir_prix("Riz 5kg", Some(Montant(350)), Some(550))
        .unwrap();
    depot.definir_prix("thé", Some(Montant(250)), None).unwrap();
    depot
        .definir_prix("Pain 🥖", Some(Montant(120)), Some(550))
        .unwrap();
    assert!(matches!(
        depot.definir_prix("Riz 5kg", Some(Montant(-1)), None),
        Err(ErreurStock::MontantInvalide(_))
    ));
    assert_eq!(
        depot
            .definir_prix("Riz 5kg", Some(Montant(MONTANT_MAX.0 + 1)), None)
            .unwrap_err(),
        ErreurStock::MontantTropGrand
    );
    assert!(matches!(
        depot.definir_prix("Riz 5kg", Some(Montant(100)), Some(10_001)),
        Err(ErreurStock::TauxInvalide(_))
    ));
    assert_eq!(
        produits(depot),
        [
            ("Pain 🥖".to_string(), 10, 0, Some(120), Some(550)),
            ("Riz 5kg".to_string(), 6, 5, Some(350), Some(550)),
            ("the".to_string(), 1, 0, None, None),
            ("Thé".to_string(), 5, 0, Some(250), None),
        ],
        "un prix refusé ne change rien"
    );

    // ---- Les ventes : numérotation continue, refus en entier ----
    let v1 = depot
        .encaisser(
            &panier(&[("Riz 5kg", 2)]),
            Moyen::Especes,
            Montant(1000),
            QUAND,
        )
        .unwrap();
    assert_eq!((v1.numero, v1.genre), (1, GenrePiece::Vente));
    assert_eq!(
        (v1.total, v1.recu, v1.rendu),
        (Montant(700), Montant(1000), Montant(300))
    );
    let v2 = depot
        .encaisser(&panier(&[("thé", 1)]), Moyen::Carte, Montant(0), QUAND)
        .unwrap();
    assert_eq!((v2.numero, v2.total), (2, Montant(250)));
    assert_eq!(
        v2.hash_precedent, v1.hash,
        "chaque pièce porte l'empreinte de la précédente"
    );
    // Refusées : rien ne change, pas même le numéro du ticket suivant.
    assert!(matches!(
        depot.encaisser(
            &panier(&[("Thé", 1), ("Riz 5kg", 100)]),
            Moyen::Carte,
            Montant(0),
            QUAND
        ),
        Err(ErreurStock::StockInsuffisant { .. })
    ));
    assert!(matches!(
        depot.encaisser(
            &panier(&[("Riz 5kg", 1)]),
            Moyen::Especes,
            Montant(100),
            QUAND
        ),
        Err(ErreurStock::MontantInsuffisant { .. })
    ));
    assert!(matches!(
        depot.encaisser(&panier(&[("the", 1)]), Moyen::Carte, Montant(0), QUAND),
        Err(ErreurStock::PrixManquant(_))
    ));
    assert_eq!(
        depot
            .encaisser(&[], Moyen::Carte, Montant(0), QUAND)
            .unwrap_err(),
        ErreurStock::PanierVide
    );
    let v3 = depot
        .encaisser(&panier(&[("Pain 🥖", 3)]), Moyen::Carte, Montant(0), QUAND)
        .unwrap();
    assert_eq!(v3.numero, 3, "aucun trou après les ventes refusées");

    // ---- L'annulation : totale, une seule fois, jamais d'une annulation ----
    let annulation = depot.annuler(1, QUAND).unwrap();
    assert_eq!(
        (annulation.numero, annulation.genre),
        (4, GenrePiece::Annulation { de: 1 })
    );
    assert_eq!(annulation.total, Montant(-700));
    assert_eq!(
        depot.annuler(1, QUAND).unwrap_err(),
        ErreurStock::DejaAnnulee(1)
    );
    assert_eq!(
        depot.annuler(4, QUAND).unwrap_err(),
        ErreurStock::AnnulationNonPermise(4)
    );
    assert_eq!(
        depot.annuler(99, QUAND).unwrap_err(),
        ErreurStock::PieceInconnue(99)
    );

    // ---- Renommer et supprimer ----
    depot.ajouter_produit("Sel", 0).unwrap();
    depot.enregistrer_entree("Sel", 3).unwrap();
    depot.renommer_produit("Sel", "Sel fin").unwrap();
    depot.renommer_produit("Sel fin", "SEL FIN").unwrap(); // la casse seule : permis
    assert_eq!(
        depot.renommer_produit("SEL FIN", "riz 5kg").unwrap_err(),
        ErreurStock::ProduitExistant("Riz 5kg".to_string())
    );
    assert_eq!(
        depot.renommer_produit("Riz 5kg", "Riz").unwrap_err(),
        ErreurStock::ProduitVendu("Riz 5kg".to_string()),
        "un produit vendu garde son nom"
    );
    assert_eq!(
        depot.supprimer_produit("Riz 5kg").unwrap_err(),
        ErreurStock::ProduitVendu("Riz 5kg".to_string())
    );
    assert!(
        depot
            .instantane()
            .unwrap()
            .historique()
            .iter()
            .any(|m| m.nom == "SEL FIN"),
        "le nouveau nom remplace l'ancien dans l'historique"
    );
    depot.supprimer_produit("SEL FIN").unwrap();
    assert_eq!(
        depot.supprimer_produit("SEL FIN").unwrap_err(),
        ErreurStock::ProduitInconnu("SEL FIN".to_string())
    );

    // ---- Les réglages : figés par la première vente, sauf le taux par défaut ----
    let en_dollars = Reglages {
        profil: Profil::EtatsUnis,
        devise: gestion_stock::monnaie::Devise::Usd,
        taux_defaut_bp: 825,
    };
    assert_eq!(
        depot.definir_reglages(en_dollars).unwrap_err(),
        ErreurStock::ReglagesVerrouilles
    );
    let nouveau_taux = Reglages {
        taux_defaut_bp: 1000,
        ..Reglages::default()
    };
    depot.definir_reglages(nouveau_taux).unwrap();

    // ---- Le journal et la vue d'ensemble ----
    depot.verifier_journal().unwrap();
    let stock = depot.instantane().unwrap();
    stock.verifier_journal().unwrap();
    assert_eq!(stock.reglages(), nouveau_taux);
    let pieces = stock.pieces();
    assert_eq!(pieces.len(), 4);
    assert_eq!(pieces[0], v1);
    assert_eq!(pieces[1], v2);
    assert_eq!(pieces[2], v3);
    assert_eq!(pieces[3], annulation);
    assert_eq!(
        produits(depot),
        [
            ("Pain 🥖".to_string(), 7, 0, Some(120), Some(550)),
            ("Riz 5kg".to_string(), 6, 5, Some(350), Some(550)),
            ("the".to_string(), 1, 0, None, None),
            ("Thé".to_string(), 4, 0, Some(250), None),
        ]
    );
    // 6 entrées ou sorties à la main + 3 ventes + 1 remise en stock par l'annulation, puis l'entrée
    // de Sel, renommée depuis.
    let historique = stock.historique();
    assert_eq!(historique.len(), 10);
    let mouvement = |i: usize| {
        (
            historique[i].genre,
            historique[i].nom.as_str(),
            historique[i].quantite,
        )
    };
    assert_eq!(mouvement(5), (Genre::Sortie, "Riz 5kg", 2), "la vente 1");
    assert_eq!(
        mouvement(8),
        (Genre::Entree, "Riz 5kg", 2),
        "la remise en stock de l'annulation"
    );
    assert_eq!(mouvement(9), (Genre::Entree, "SEL FIN", 3));
}
