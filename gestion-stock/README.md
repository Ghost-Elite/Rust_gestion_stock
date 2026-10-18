# Gestion de stock

Un petit programme en ligne de commande pour suivre le stock d'une boutique : produits, entrées de marchandise, sorties (ventes, pertes), alertes quand un produit est bientôt épuisé, et historique des mouvements.

Tout est écrit en Rust, sans bibliothèque externe. Les données sont gardées dans un simple fichier texte.

## Installation

```bash
cargo build --release
```

Le programme est alors dans `target/release/stock`. Copie-le où tu veux et lance-le depuis un terminal.

## Les commandes

Un nom de produit qui contient des espaces se met entre guillemets. Les majuscules n'ont pas d'importance : `riz 5kg` et `Riz 5KG` désignent le même produit.

| Commande | Ce qu'elle fait | Exemple |
|---|---|---|
| `stock ajouter <nom> [--seuil N]` | Crée un produit (stock à 0). Le seuil d'alerte vaut 0 par défaut. | `stock ajouter "Riz 5kg" --seuil 5` |
| `stock entree <nom> <quantité>` | Enregistre une entrée de marchandise. | `stock entree "Riz 5kg" 12` |
| `stock sortie <nom> <quantité>` | Enregistre une sortie. Refusée si le stock ne suffit pas. | `stock sortie "Riz 5kg" 2` |
| `stock lister` | Affiche tout le stock. | `stock lister` |
| `stock alertes` | Affiche seulement les produits en alerte (quantité inférieure ou égale au seuil). | `stock alertes` |
| `stock historique [N]` | Affiche les N derniers mouvements, du plus récent au plus ancien (10 par défaut). | `stock historique 20` |
| `stock aide` | Affiche la liste des commandes (aussi : `stock` sans rien). | `stock aide` |

Quand une sortie fait passer un produit sous son seuil, le programme affiche une ligne `ATTENTION : stock bas pour « … »`.

En cas d'erreur, le programme affiche `Erreur : …` et ne change rien aux données.

## Où sont les données ?

Dans le fichier `stock.txt`, dans le dossier où tu lances le programme. Pour utiliser un autre fichier, indique son chemin dans la variable d'environnement `STOCK_FICHIER` :

```bash
STOCK_FICHIER=/chemin/vers/mon-stock.txt stock lister
```

Le fichier est du texte lisible. Si tu as fait une faute de frappe dans un nom de produit, tu peux l'ouvrir et la corriger à la main (une ligne `P;nom;quantité;seuil` par produit).

## Sauvegarde conseillée

Copie régulièrement `stock.txt` (sur une clé USB, par exemple) : c'est la seule chose à conserver. Le programme écrit d'abord dans un fichier temporaire puis le renomme, pour que le fichier ne soit pas abîmé si l'ordinateur s'éteint au mauvais moment.

## Pour les développeurs

```bash
cargo test                          # tous les tests
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

Le cœur (`src/stock.rs`, `src/stockage.rs`) n'affiche rien et ne lit pas le terminal : il peut être réutilisé tel quel par une autre interface, par exemple une interface graphique.
