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

Pour utiliser un autre fichier, indique son chemin complet dans la variable d'environnement `STOCK_FICHIER` (elle a la priorité). Le chemin utilisé est toujours affiché en bas de la fenêtre, et sur l'écran d'erreur si le fichier est illisible.

**Tu utilises déjà le programme en ligne de commande ?** Son fichier `stock.txt` est dans le dossier où tu le lançais, pas à l'endroit ci-dessus : la fenêtre afficherait un stock vide. Copie ce fichier à l'emplacement par défaut, ou lance la fenêtre avec `STOCK_FICHIER=/chemin/vers/ton/stock.txt`.

Si le fichier n'existe pas encore, un stock vide est utilisé et la fenêtre le dit. Si le fichier est abîmé, un écran d'erreur s'affiche et le fichier n'est **jamais** modifié.

**Une seule fenêtre à la fois.** Si le fichier a été modifié ailleurs (une autre fenêtre, le programme en ligne de commande) depuis l'ouverture de la tienne, elle refuse d'enregistrer, affiche un message et ne change rien : ferme-la et rouvre-la pour voir le stock à jour. Sans cette protection, une fenêtre périmée effacerait les mouvements faits ailleurs.

**Sauvegarde conseillée :** copie régulièrement ce fichier (sur une clé USB, par exemple). C'est la seule chose à conserver.

## Accessibilité

Le but est le niveau AA des WCAG 2.1, qui est la référence de la norme européenne EN 301 549 et de la Section 508 américaine. **Ce programme n'a pas été audité par un tiers et ne fait l'objet d'aucune déclaration de conformité.** Voici ce qui est fait et comment c'est vérifié :

| Critère WCAG 2.1 | État | Comment |
|---|---|---|
| 1.4.3 Contraste (texte ≥ 4,5:1) | Tenu | un test dessine la fenêtre avec la vraie palette et vérifie chaque texte réellement peint sur son vrai fond (`src/ui.rs`), en plus des tests de la palette (`src/palette.rs`) |
| 1.4.11 Contraste des composants (≥ 3:1) | Tenu pour les bordures | bordures testées ; la visibilité du focus clavier reste à vérifier à la main |
| 1.4.1 Couleur seule | Tenu | l'alerte est écrite : « ALERTE : stock bas » |
| 1.4.4 Redimensionnement du texte | Tenu | zoom au clavier d'egui (`Cmd`/`Ctrl` + `+`, `−`, `0`) ; à 200 % la fenêtre passe en une seule colonne et l'alerte reste visible (test) |
| 1.4.10 Reflow | Partiel | une seule colonne quand la place manque ; le tableau (exempté par la norme) défile horizontalement au besoin |
| 2.1.1 Clavier | À vérifier à la main | tout se fait avec `Tab`, `Maj+Tab`, `Espace`, `Entrée` |
| 3.3.1 / 3.3.3 Erreurs | Tenu | messages « Erreur : … » en français qui disent quoi faire ; un bouton reste grisé tant que son champ est vide |
| 3.3.2 Étiquettes | Tenu | chaque champ est relié à son libellé ; vérifié par des tests qui interrogent le **rôle** (champ de saisie) et le nom |
| 4.1.2 Nom, rôle, valeur | Partiel | noms et rôles vérifiés par des tests ; le rendu avec un lecteur d'écran reste à vérifier à la main |
| 4.1.3 Messages d'état | **Non couvert** | egui 0.35 n'a pas de région « live » : un lecteur d'écran n'annonce pas tout seul les messages du bas de la fenêtre |
| 2.4.3 Ordre du focus | **Limite connue** | l'ordre de `Tab` suit l'ordre de dessin : en disposition à deux colonnes, le panneau de droite (ajout, commandes) vient avant la recherche et la liste |
| 1.3.1 Information et relations | **Limite connue** | le tableau est une grille d'egui : un lecteur d'écran n'y voit ni tableau, ni lignes, ni colonnes |

**Autres limites connues :** `Entrée` ne valide pas un formulaire (il faut activer le bouton) ; après « Supprimer le produit » au clavier, le bouton disparaît et le focus est perdu ; un seul thème clair ; les textes de l'interface sont dans `src/textes.rs` mais les messages de résultat sont dans `src/etat.rs` et ceux de la bibliothèque dans `../gestion-stock/src/erreur.rs` (une traduction devra toucher ces trois endroits) ; la qualité de la lecture par un lecteur d'écran dépend d'egui et d'AccessKit.

## Vie privée

Le programme ne collecte aucune donnée personnelle, n'ouvre aucune connexion réseau et n'envoie aucune télémétrie : aucune dépendance réseau n'est présente, sur aucune plateforme (vérifié avec `cargo tree --target all`). Tout reste dans le fichier local ci-dessus. Les noms de produits, quantités et mouvements sont les seules informations enregistrées.

## Licences

La liste complète des dépendances, pour **toutes** les plateformes (Windows, macOS, Linux), est dans [LICENCES-TIERS.txt](LICENCES-TIERS.txt). Elles sont sous licences permissives (MIT, Apache-2.0, Zlib, BSD, ISC, BSL-1.0, Unicode et assimilées).

- Deux dépendances sont proposées avec une option copyleft **en plus** d'une option permissive : `self_cell` (« Apache-2.0 OR GPL-2.0-only ») et `r-efi` (« MIT OR Apache-2.0 OR LGPL-2.1-or-later »). On utilise toujours l'option permissive.
- Les polices par défaut d'egui sont **intégrées au programme** ; elles sont sous les licences OFL-1.1 et Ubuntu-font-1.0.
- Si tu distribues le programme à d'autres personnes, joins les mentions de licence des dépendances (MIT, Apache-2.0, OFL, Ubuntu-font).
- Le code des deux programmes de ce dépôt n'a pas encore de licence choisie : c'est à son auteur de la décider.

La liste se régénère avec `cargo tree --prefix none --format "{p} {l}" -e normal --target all`.

## Essai manuel

1. Lance le programme, ajoute un produit avec un seuil, fais une entrée, puis une sortie qui passe sous le seuil : l'état affiche « ALERTE : stock bas ».
2. Fais tout **sans la souris** : `Tab` pour avancer, `Maj+Tab` pour reculer, `Espace` ou `Entrée` pour activer. Vérifie que tu vois toujours où est le focus.
3. Agrandis le texte avec `Cmd` + `+` (macOS) ou `Ctrl` + `+`, jusqu'à 200 % : la fenêtre passe en une colonne et l'alerte reste visible. Reviens avec `Cmd`/`Ctrl` + `0`.
4. Sur macOS, active VoiceOver (`Cmd+F5`) et vérifie que les champs et les boutons sont annoncés avec leur nom.
5. Provoque des erreurs : sortie trop grande, nom avec `;` : un message clair s'affiche et rien ne change. Un bouton est grisé tant que son champ est vide.
6. Ferme puis rouvre : les données sont toujours là, au chemin affiché en bas.
7. Ouvre deux fenêtres sur le même fichier, enregistre dans la première, puis dans la seconde : la seconde refuse et le dit.

## Pour les développeurs

```bash
cargo test                                  # tous les tests
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

La logique est dans `src/etat.rs` (testable sans fenêtre), le dessin dans `src/ui.rs`, les couleurs dans `src/palette.rs`. Dans les tests de `src/ui.rs`, `textes_peints` dessine l'application avec la vraie palette et relève chaque texte peint avec son fond : c'est ce test qui garantit le contraste réel, pas seulement celui des constantes.
