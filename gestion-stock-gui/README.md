# Gestion de stock : l'interface graphique

Une fenêtre sobre pour suivre le stock d'une boutique : voir et chercher les produits, les ajouter, enregistrer des entrées et des sorties, être prévenu quand un produit est bientôt épuisé, renommer ou supprimer un produit, consulter l'historique.

Elle utilise le même cœur que le programme en ligne de commande (`../gestion-stock`) et lit le même format de fichier.

## Lancer

```bash
cargo run --release
```

Le programme s'appelle `stock-gui` (dans `target/release/`). Le premier build est long (plusieurs minutes), les suivants sont rapides.

## Où sont les données ?

Dans un fichier texte, dans le dossier de données de l'utilisateur, **pas** dans le dossier d'où l'on lance le programme :

| Système | Fichier par défaut |
|---|---|
| macOS | `~/Library/Application Support/GestionStock/stock.txt` |
| Windows | `%APPDATA%\GestionStock\stock.txt` |
| Linux | `$XDG_DATA_HOME/gestion-stock/stock.txt`, sinon `~/.local/share/gestion-stock/stock.txt` |

Pour utiliser un autre fichier, indique son chemin complet dans la variable d'environnement `STOCK_FICHIER` (elle a la priorité). Le chemin utilisé est toujours affiché en bas de la fenêtre.

Si le fichier n'existe pas encore, un stock vide est utilisé et la fenêtre le dit. Si le fichier est abîmé, un écran d'erreur s'affiche et le fichier n'est **jamais** modifié.

**Sauvegarde conseillée :** copie régulièrement ce fichier (sur une clé USB, par exemple). C'est la seule chose à conserver.

## Accessibilité

Le but est le niveau AA des WCAG 2.1, qui est la référence de la norme européenne EN 301 549 et de la Section 508 américaine. **Ce programme n'a pas été audité par un tiers et ne fait l'objet d'aucune déclaration de conformité.** Voici ce qui est fait et comment c'est vérifié :

| Critère WCAG 2.1 | État | Comment |
|---|---|---|
| 1.4.3 Contraste (texte ≥ 4,5:1) | Tenu | palette à valeurs fixes, vérifiée par des tests (`src/palette.rs`) |
| 1.4.11 Contraste des composants (≥ 3:1) | Tenu | bordures et contours vérifiés par des tests |
| 1.4.1 Couleur seule | Tenu | l'alerte est écrite : « ALERTE : stock bas » |
| 1.4.4 Redimensionnement du texte | Tenu | zoom au clavier d'egui : `Cmd`/`Ctrl` + `+`, `−`, `0` |
| 2.1.1 Clavier | À vérifier à la main | tout se fait avec `Tab`, `Maj+Tab`, `Espace`, `Entrée` |
| 3.3.1 / 3.3.3 Erreurs | Tenu | messages « Erreur : … » en français, qui disent quoi faire |
| 3.3.2 Étiquettes | Tenu | chaque champ est relié à son libellé (vérifié par des tests) |
| 4.1.2 Nom, rôle, valeur | Partiel | noms accessibles via AccessKit, vérifiés par des tests ; le rendu avec un lecteur d'écran reste à vérifier à la main |
| 4.1.3 Messages d'état | **Non couvert** | egui 0.35 n'a pas de région « live » : un lecteur d'écran n'annonce pas tout seul les messages du bas de la fenêtre |

**Limites connues :** la qualité de la lecture par un lecteur d'écran dépend d'egui et d'AccessKit ; la visibilité du focus clavier est à vérifier à la main ; un seul thème clair ; textes en français uniquement (tous dans `src/textes.rs`, pour faciliter une traduction).

## Vie privée

Le programme ne collecte aucune donnée personnelle, n'ouvre aucune connexion réseau et n'envoie aucune télémétrie. Tout reste dans le fichier local ci-dessus. Les noms de produits, quantités et mouvements sont les seules informations enregistrées.

## Licences

Les dépendances sont sous licences permissives (MIT, Apache-2.0, Zlib, BSD, Unicode et assimilées). Aucune n'est uniquement sous GPL ou AGPL. Une seule, `self_cell`, est proposée en double licence « Apache-2.0 OR GPL-2.0-only » : on utilise l'option Apache-2.0, qui n'impose aucune obligation GPL. La liste complète est dans [LICENCES-TIERS.txt](LICENCES-TIERS.txt) (regénérable avec `cargo tree --prefix none --format "{p} {l}" -e normal | sort -u`).

## Essai manuel

1. Lance le programme, ajoute un produit avec un seuil, fais une entrée, puis une sortie qui passe sous le seuil : l'état affiche « ALERTE : stock bas ».
2. Fais tout **sans la souris** : `Tab` pour avancer, `Maj+Tab` pour reculer, `Espace` ou `Entrée` pour activer. Vérifie que tu vois toujours où est le focus.
3. Agrandis le texte avec `Cmd` + `+` (macOS) ou `Ctrl` + `+`, puis reviens avec `Cmd`/`Ctrl` + `0`.
4. Sur macOS, active VoiceOver (`Cmd+F5`) et vérifie que les champs et les boutons sont annoncés avec leur nom.
5. Provoque des erreurs : quantité vide, `abc`, sortie trop grande, nom avec `;` : un message clair s'affiche et rien ne change.
6. Ferme puis rouvre : les données sont toujours là, au chemin affiché en bas.

## Pour les développeurs

```bash
cargo test                                  # tous les tests
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

La logique est dans `src/etat.rs` (testable sans fenêtre), le dessin dans `src/ui.rs`, les couleurs dans `src/palette.rs`.
