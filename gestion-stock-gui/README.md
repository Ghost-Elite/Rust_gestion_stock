# Gestion de stock : l'interface graphique

Une fenêtre sobre pour gérer une petite boutique, en quatre onglets :

- **Vente** : des tuiles de produits, un panier, l'encaissement (avec le rendu de monnaie) et le ticket ;
- **Stock** : voir et chercher les produits, les ajouter avec leur prix, enregistrer des entrées et des sorties, être prévenu quand un produit est bientôt épuisé, renommer ou supprimer un produit, consulter l'historique ;
- **Ventes** : le journal des ventes, où l'on annule une vente ;
- **Réglages** : Europe ou États-Unis, devise, taux de taxe par défaut.

> **Ce programme n'est pas une caisse enregistreuse certifiée.** Lis la section « Caisse et conformité » avant de t'en servir pour de vrais clients.

Elle utilise le même cœur que le programme en ligne de commande (`../gestion-stock`) et lit le même format de fichier.

## Apparence

Un fond gris très clair, des cartes blanches aux coins arrondis, une seule couleur d'accent (indigo), des puces pour les états et trois chiffres en haut : produits, produits en alerte, unités en stock. La police est **Inter**, avec deux graisses pour une vraie hiérarchie. Sous 800 points de large, ou quand le texte est agrandi, la fenêtre passe en une seule colonne. Elle s'ouvre en 1100 × 720 points (minimum 640 × 420) : l'écran de vente a besoin de hauteur pour montrer le panier et le paiement ensemble ; dans une fenêtre plus basse, la colonne du panier défile d'un bloc.

Les couleurs sont toutes dans `src/palette.rs`, où le contraste de chacune est vérifié par des tests. Pour **voir** la fenêtre sans l'ouvrir, un outil dessine l'interface dans des images :

```bash
APERCU_DOSSIER=/chemin/de/sortie UPDATE_SNAPSHOTS=true cargo test --test apercu -- --ignored
```

## Suggestions de produits

Dans « Ajouter un produit », des produits courants de boutique sont proposés sous le champ du nom : on les **choisit d'un clic** au lieu de les taper. Ils se filtrent pendant la frappe (sans tenir compte des majuscules ni des accents : « the » trouve « Thé vert »). Choisir un produit remplit le nom et le seuil d'alerte conseillé, sauf si un seuil est déjà tapé. Un produit déjà en stock n'est jamais proposé.

La liste de départ est dans `src/catalogue.rs` : c'est une proposition, à adapter à la boutique (les produits les plus courants en premier).

## Vendre

L'écran **Vente** est celui qui s'ouvre au lancement. Les produits sont des tuiles : un clic met une unité au panier. Une tuile est grisée, avec la raison **écrite** dessous, quand on ne peut pas vendre : « Épuisé », « Prix manquant » ou « Tout le stock est au panier ». Dans le panier, « − » et « + » changent la quantité, « Retirer » enlève la ligne, « Vider le panier » repart de zéro. Un produit n'a de prix qu'une fois que tu le lui as donné (onglet Stock).

**Payer.** Trois moyens : Espèces, Carte, Autre. En espèces, tape la somme donnée par le client (ou « Montant juste ») : le **rendu de monnaie** s'affiche en direct, et « Encaisser » reste grisé tant que la somme ne couvre pas le total. Par carte ou « Autre » (chèque, virement, mobile money…), le programme note seulement le moyen : **il n'est relié à aucun terminal de paiement** et ne voit aucun numéro de carte.

**Encaisser** fait tout ou rien : la vente est validée en entier (stock suffisant, prix connus) ou refusée avec un message, sans rien changer. Le stock baisse et le ticket s'affiche, avec son numéro et l'heure locale. « Nouvelle vente » (ou un clic sur un produit) le ferme ; la vente, elle, reste dans le journal.

**Europe ou États-Unis** (onglet Réglages) :

| | Europe | États-Unis |
|---|---|---|
| Prix saisis | TTC (TVA comprise) | hors taxe |
| Ticket | « Total TTC », puis « dont TVA 5,5 % : … » pour chaque taux | « Sous-total », « Taxe 8 % : … » pour chaque taux, puis « Total » |
| Calcul | la TVA contenue dans le TTC : TTC × taux ÷ (100 % + taux) | la taxe ajoutée à la base : base × taux |

La taxe est calculée **une fois par taux**, sur le total des lignes de ce taux (pas ligne par ligne), arrondie au centime le plus proche, **0,5 arrondi vers le haut**. Chaque produit peut avoir son taux (de 0 à 100 %, deux décimales au plus, par exemple 5,5) ; sans taux propre, c'est le **taux par défaut** des réglages qui s'applique (20 % au départ). Le programme ne connaît ni les taux de chaque pays, État ou commune, ni les exonérations : c'est à toi de les saisir.

**Devises :** euro (EUR), dollar (USD), livre sterling (GBP) et franc CFA (XOF, sans centimes). Les montants s'écrivent à l'usage de la devise (« 12,50 € », « $12.50 », « 15 000 FCFA »). Tu peux taper la virgule ou le point.

**Réglages figés.** Dès la première vente, la devise et le mode de prix ne changent plus (les montants du journal perdraient leur sens) ; le taux par défaut reste modifiable. Un produit qui figure dans une vente ne peut plus être renommé ni supprimé.

**Annuler une vente** (onglet Ventes) demande une confirmation. Rien n'est effacé : une **écriture d'annulation** (montant négatif, avec son propre numéro) est ajoutée au journal, le stock est remis, et la vente d'origine reste visible. Une vente ne s'annule qu'une fois, en entier ; on n'annule pas une annulation. Il n'y a pas encore de remboursement partiel ni d'échange : il faut annuler puis ressaisir la vente.

## Le journal de caisse

Chaque vente et chaque annulation est une **pièce** du journal, numérotée 1, 2, 3… **sans trou** (ventes et annulations dans la même suite). Chaque pièce garde ses lignes avec le prix et le taux **du moment** : changer un prix plus tard ne change pas un ticket passé.

Les pièces sont **chaînées** : chacune porte une empreinte SHA-256 calculée sur son contenu (numéro, genre, date, montants, moyen de paiement, lignes) et sur l'empreinte de la pièce précédente. À l'ouverture et après chaque action, le programme recalcule toute la chaîne.

| Détecté | Pas détecté |
|---|---|
| une pièce modifiée à la main (montant, ligne, date, moyen…) | un fichier entièrement réécrit **avec les empreintes recalculées** |
| une pièce supprimée, ajoutée ou déplacée au milieu du journal ; un trou dans la numérotation | les **dernières** pièces supprimées (la chaîne qui reste est valide), ou le remplacement du fichier par une ancienne copie |
| | une modification du stock lui-même (lignes `P` et `M`) : la chaîne ne couvre que le journal des ventes |

Si le journal ne correspond plus, un bandeau l'écrit (onglets Vente et Ventes) et les **ventes et les annulations sont bloquées** ; le stock reste utilisable. Restaure une copie du fichier pour vendre à nouveau.

**Ce n'est pas une protection contre une personne qui voudrait tricher** : la chaîne signale une erreur ou une modification maladroite, elle ne l'empêche pas. Pour un vrai contrôle, il faudrait garder les empreintes ailleurs que dans le même fichier.

## Caisse et conformité

**Ce programme n'est pas une caisse enregistreuse certifiée.** Il enregistre des ventes de façon ordonnée et vérifiable, mais il n'a été ni audité ni certifié par un organisme, et le ticket qu'il affiche porte la mention « Ticket non fiscal ». Dans plusieurs pays, **vendre à des particuliers avec un logiciel de caisse non conforme peut être illégal** : renseigne-toi auprès de ton administration fiscale ou de ton comptable avant de l'utiliser pour de vrais clients.

Ce que l'on sait des règles (**à vérifier** : elles changent, et ceci n'est ni un avis juridique ni un avis fiscal) :

- **France :** depuis 2018, un assujetti à la TVA qui encaisse avec un logiciel ou un système de caisse doit utiliser un produit **certifié ou attesté** (inaltérabilité, sécurisation, conservation, archivage) : article 286, I, 3° bis du Code général des impôts.
- **Allemagne :** un système de caisse électronique doit avoir un dispositif de sécurité technique (TSE) certifié et remettre un justificatif : § 146a de l'Abgabenordnung et Kassensicherungsverordnung (KassenSichV).
- **Autres pays de l'Union européenne :** plusieurs ont leurs propres règles (caisses certifiées, transmission en ligne des recettes).
- **Prix dans l'Union :** au détail, le prix annoncé au consommateur inclut la TVA (directive 98/6/CE sur l'indication des prix).
- **États-Unis :** il n'y a pas de certification fédérale des caisses ; la taxe de vente est fixée par les États et les collectivités locales (taux, exonérations et arrondis varient). Si tu acceptes des cartes, les règles PCI DSS concernent qui traite ou stocke des numéros de carte : ici, aucun numéro n'est jamais vu ni enregistré.

**Ce qui manque encore pour une vraie caisse :** l'ouverture et la clôture de caisse (fond de caisse, rapports X et Z, clôture numérotée), les entrées et sorties d'espèces, l'export des données pour un comptable, l'archivage par période, l'impression des tickets, les remises et les remboursements partiels, plusieurs postes ou vendeurs.

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

**Format du fichier.** Il est écrit au **format version 2** (prix, réglages, journal des ventes). La fenêtre et le programme en ligne de commande de ce dépôt le lisent ; l'**ancienne version** du programme en ligne de commande ne sait pas le relire. Les fichiers de l'ancien format se lisent sans perte, et sont réécrits en version 2 au premier enregistrement : **garde une copie de ton `stock.txt` avant de passer à la nouvelle version.** Le programme en ligne de commande ne vend pas, mais il conserve le journal. Ne modifie jamais à la main les lignes `S;` et `L;` (le journal de caisse) : la fenêtre le détecterait et bloquerait les ventes.

Si le fichier n'existe pas encore, un stock vide est utilisé et la fenêtre le dit. Si le fichier est abîmé, un écran d'erreur s'affiche et le fichier n'est **jamais** modifié.

**Une seule fenêtre à la fois.** Si le fichier a été modifié ailleurs (une autre fenêtre, le programme en ligne de commande) depuis l'ouverture de la tienne, elle refuse d'enregistrer, affiche un message et ne change rien : ferme-la et rouvre-la pour voir le stock à jour. Sans cette protection, une fenêtre périmée effacerait les mouvements faits ailleurs.

**Sauvegarde conseillée :** copie régulièrement ce fichier (sur une clé USB, par exemple). C'est la seule chose à conserver.

## Accessibilité

Le but est le niveau AA des WCAG 2.1, qui est la référence de la norme européenne EN 301 549 et de la Section 508 américaine. **Ce programme n'a pas été audité par un tiers et ne fait l'objet d'aucune déclaration de conformité.** Voici ce qui est fait et comment c'est vérifié :

| Critère WCAG 2.1 | État | Comment |
|---|---|---|
| 1.4.3 Contraste (texte ≥ 4,5:1) | Tenu | des tests dessinent la fenêtre avec la vraie palette et vérifient chaque texte réellement peint sur son vrai fond (`src/ui.rs`), sur chaque écran : stock, vente (panier, ticket), ventes (avec la confirmation d'annulation) et réglages ; ils s'ajoutent aux tests de la palette (`src/palette.rs`) |
| 1.4.11 Contraste des composants (≥ 3:1) | Tenu pour les bordures | bordures testées ; la visibilité du focus clavier reste à vérifier à la main |
| 1.4.1 Couleur seule | Tenu | l'alerte est écrite : « ALERTE : stock bas » ; « Épuisé », « Prix manquant » et « (bas) » sont écrits sous les tuiles ; l'onglet et le moyen de paiement choisis sont annoncés « sélectionné » (état du bouton) en plus d'être colorés |
| 1.4.4 Redimensionnement du texte | Tenu | zoom au clavier d'egui (`Cmd`/`Ctrl` + `+`, `−`, `0`) ; à 200 % la fenêtre passe en une seule colonne et l'alerte reste visible (test) |
| 1.4.10 Reflow | Partiel | une seule colonne quand la place manque, et les noms longs passent à la ligne ; l'alerte reste visible à 200 % et en petite fenêtre (test) |
| 2.1.1 Clavier | À vérifier à la main | tout se fait avec `Tab`, `Maj+Tab`, `Espace`, `Entrée` |
| 3.3.1 / 3.3.3 Erreurs | Tenu | messages « Erreur : … » en français qui disent quoi faire ; un bouton reste grisé tant que son champ est vide |
| 3.3.2 Étiquettes | Tenu | chaque champ est relié à son libellé ; vérifié par des tests qui interrogent le **rôle** (champ de saisie) et le nom |
| 4.1.2 Nom, rôle, valeur | Partiel | noms et rôles vérifiés par des tests, y compris sur la vente : les tuiles et les boutons « − » et « + » ont un nom complet (« Ajouter Pain au panier (1,20 €) », « Retirer une unité de Pain »), les onglets et les choix sont des boutons à bascule « sélectionné » ; le rendu avec un lecteur d'écran reste à vérifier à la main |
| 4.1.3 Messages d'état | **Non couvert** | egui 0.35 n'a pas de région « live » : un lecteur d'écran n'annonce pas tout seul les messages du bas de la fenêtre |
| 2.4.3 Ordre du focus | **Limite connue** | l'ordre de `Tab` suit l'ordre de dessin : en disposition à deux colonnes, le panneau de droite (ajout et commandes sur le stock, panier et paiement sur la vente) vient avant la recherche et la liste |
| 1.3.1 Information et relations | **Limite connue** | la liste est faite de cartes, pas d'un vrai tableau : un lecteur d'écran lit chaque ligne comme une suite de textes (nom, état, seuil, quantité) sans structure de tableau ; chaque valeur est écrite en toutes lettres (« Seuil : 5 », « en stock ») pour rester compréhensible |

**Autres limites connues :** `Entrée` ne valide pas un formulaire (il faut activer le bouton) ; après « Supprimer le produit » au clavier, le bouton disparaît et le focus est perdu ; un seul thème clair ; les textes de l'interface sont dans `src/textes.rs` mais les messages de résultat sont dans `src/etat.rs` et ceux de la bibliothèque dans `../gestion-stock/src/erreur.rs` (une traduction devra toucher ces trois endroits) ; la qualité de la lecture par un lecteur d'écran dépend d'egui et d'AccessKit ; les onglets sont des boutons à bascule et pas des « onglets » au sens de WAI-ARIA (egui n'a pas ce rôle) ; les tuiles de produits sont dessinées à la main (elles gardent le focus clavier avec son contour et un nom complet, mais pas toutes les finesses d'un bouton egui) ; un contrôle grisé (par exemple « Encaisser » tant que la somme ne couvre pas le total) est dispensé de contraste par les WCAG, et la raison est écrite à côté.

## Vie privée

Le programme ne collecte aucune donnée personnelle, n'ouvre aucune connexion réseau et n'envoie aucune télémétrie : aucune dépendance réseau n'est présente, sur aucune plateforme (vérifié avec `cargo tree --target all`). Tout reste dans le fichier local ci-dessus. Les noms de produits, quantités, prix, mouvements et ventes (date, lignes, total, moyen de paiement) sont les seules informations enregistrées. Aucun nom de client, aucune adresse et aucun numéro de carte ne sont demandés ni enregistrés.

## Licences

La liste complète des dépendances, pour **toutes** les plateformes (Windows, macOS, Linux), est dans [LICENCES-TIERS.txt](LICENCES-TIERS.txt). Elles sont sous licences permissives (MIT, Apache-2.0, Zlib, BSD, ISC, BSL-1.0, Unicode et assimilées).

- Deux dépendances sont proposées avec une option copyleft **en plus** d'une option permissive : `self_cell` (« Apache-2.0 OR GPL-2.0-only ») et `r-efi` (« MIT OR Apache-2.0 OR LGPL-2.1-or-later »). On utilise toujours l'option permissive.
- Deux polices sont **intégrées au programme** : **Inter** (Regular et SemiBold, licence SIL OFL 1.1, copyright The Inter Project Authors ; le texte de la licence est dans `assets/polices/LICENCE-Inter-OFL-1.1.txt`), tirée de la version 4.0 officielle (`Inter-4.0.zip`, SHA-256 `ff970a5d4561a04f102a7cb781adbd6ac4e9b6c460914c7a101f15acb7f7d1a4`), et les polices par défaut d'egui (OFL-1.1 et Ubuntu-font-1.0), gardées en secours pour les symboles.
- Si tu distribues le programme à d'autres personnes, joins les mentions de licence des dépendances (MIT, Apache-2.0, OFL, Ubuntu-font) et celle d'Inter.
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
8. **Vente en espèces avec rendu.** Dans un fichier d'essai (`STOCK_FICHIER=/tmp/essai.txt`), onglet Stock : ajoute « Pain » avec le prix 1,20 et le taux 5,5, puis une entrée de 10. Onglet Vente : clique deux fois sur « Pain », tape 5 comme montant reçu : « Rendu de monnaie : 2,60 € » s'affiche. Encaisse : le ticket n° 1 montre « Total TTC : 2,40 € », « dont TVA 5,5 % : 0,13 € » et la mention « non fiscal » ; le stock est passé à 8.
9. **Annulation.** Onglet Ventes : annule la vente n° 1. Le premier clic demande confirmation (« Garder la vente » ne change rien) ; après confirmation, une ligne « Annulation de la vente n° 1 » apparaît, la vente reste visible et le stock est revenu à 10.
10. **Réglages.** Avec un fichier d'essai **neuf** (après une vente, devise et mode de prix sont figés) : onglet Réglages, choisis États-Unis, USD et le taux 8,25, enregistre : à la vente, les montants sont en $ et la taxe s'ajoute au prix hors taxe.
11. **Journal modifié à la main.** Ferme la fenêtre, ouvre le fichier d'essai, change un montant dans une ligne `L;` ou `S;`, relance : un bandeau le dit, « Encaisser » et « Annuler la vente » sont grisés, le stock reste utilisable. Remets le fichier d'origine.
12. **Tout sans la souris**, jusqu'à l'encaissement : `Tab` jusqu'à une tuile, `Espace` pour l'ajouter, `Tab` jusqu'au montant reçu, puis « Encaisser ».

## Pour les développeurs

```bash
cargo test                                  # tous les tests
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

La logique est dans `src/etat.rs` (testable sans fenêtre : vente, annulation, réglages, rendu de monnaie), le dessin dans `src/ui.rs`, les couleurs dans `src/palette.rs`. Les calculs d'argent, de taxe et de journal sont dans la bibliothèque (`../gestion-stock/src/monnaie.rs`, `vente.rs`, `journal.rs`). Dans les tests de `src/ui.rs`, `textes_peints` dessine l'application avec la vraie palette et relève chaque texte peint avec son fond : c'est ce test qui garantit le contraste réel, pas seulement celui des constantes.
