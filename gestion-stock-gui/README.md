# Gestion de stock : l'interface graphique

Une caisse de petit supermarché (la maquette « POS Kora Marché »), en cinq écrans :

- **Vente** : un champ pour scanner ou taper un article, le ticket en cours, le total, le paiement (espèces, carte, autre) et une barre de fonctions (rechercher, supprimer l'article, annuler la vente, menu) ;
- **Recherche** : trouver un article par son nom, voir s'il en reste, le mettre au ticket ;
- **Historique** : les tickets de la journée (ou d'un autre jour, ou retrouvés par leur numéro), le détail de chacun, et le remboursement ;
- **Stock** : voir et chercher les produits, les ajouter avec leur prix, enregistrer des entrées et des sorties, être prévenu quand un produit est bientôt épuisé, renommer ou supprimer un produit, consulter l'historique ;
- **Réglages** : Europe ou États-Unis, devise, taux de taxe par défaut.

On passe de l'un à l'autre par le **Menu** (Historique, Stock, Réglages) ; chaque écran a un bouton « Retour à la vente ».

> **Ce programme n'est pas une caisse enregistreuse certifiée.** Lis la section « Caisse et conformité » avant de t'en servir pour de vrais clients.

La fenêtre utilise le même cœur que le programme en ligne de commande (`../gestion-stock`) et lit le même format de fichier.

## Apparence

La maquette est dans `../docs/superpowers/specs/2026-10-21-kora-marche-pos/maquette/`. Un en-tête sombre avec le nom du magasin, le numéro de caisse et l'heure ; un fond gris très clair, des cartes blanches plates (un fin bord, pas d'ombre), le bleu `#1E56C8` comme seule couleur d'accent, de grandes cibles (56 à 84 points). La police est **IBM Plex Sans**, en quatre graisses pour une vraie hiérarchie ; ses chiffres ont tous la même largeur, ce qui aligne les montants. La fenêtre s'ouvre en **1366 × 768** points (minimum 640 × 420). Sous 1100 points de large, ou quand le texte est agrandi, l'écran de vente passe en une seule colonne qui défile (le champ de scan d'abord) ; dans une fenêtre basse, la colonne du total et du paiement défile d'un bloc. Le nom du magasin et le numéro de caisse sont pour l'instant fixes (« Kora Marché », caisse 01) : ce seront des réglages.

Les couleurs sont toutes dans `src/palette.rs`, où le contraste de chacune est vérifié par des tests. Pour **voir** la fenêtre sans l'ouvrir, un outil dessine l'interface dans des images :

```bash
APERCU_DOSSIER=/chemin/de/sortie UPDATE_SNAPSHOTS=true cargo test --test apercu -- --ignored
```

## Suggestions de produits

Dans « Ajouter un produit », des produits courants de boutique sont proposés sous le champ du nom : on les **choisit d'un clic** au lieu de les taper. Ils se filtrent pendant la frappe (sans tenir compte des majuscules ni des accents : « the » trouve « Thé vert »). Choisir un produit remplit le nom et le seuil d'alerte conseillé, sauf si un seuil est déjà tapé. Un produit déjà en stock n'est jamais proposé.

La liste de départ est dans `src/catalogue.rs` : c'est une proposition, à adapter à la boutique (les produits les plus courants en premier).

## Vendre

L'écran **Vente** est celui qui s'ouvre au lancement. Le champ du haut a toujours le focus : un lecteur de codes-barres USB tape le code puis Entrée, comme une personne qui tape un nom. Un nom (ou un morceau de nom d'au moins trois lettres, sans tenir compte des majuscules ni des accents) qui ne correspond qu'à **un** produit le met au ticket ; une quantité s'écrit devant : `3 x pain`, `3*pain` ou `3×pain`. Si plusieurs produits correspondent, la **recherche** s'ouvre avec ce texte ; si aucun, une fenêtre « Article introuvable » le dit. Un produit épuisé, sans prix, ou dont tout le stock est déjà au ticket n'est pas ajouté, et un message dit pourquoi. On clique une ligne pour la choisir : « − » et « + » changent alors sa quantité (« − » est grisé à une unité : on supprime la ligne avec « Supprimer l'article »). Les codes-barres arrivent plus tard : aujourd'hui on cherche par le nom. Un produit n'a de prix qu'une fois que tu le lui as donné (écran Stock).

**Raccourcis.** `F2` ouvre la recherche ; `Suppr` supprime l'article choisi, **seulement si le champ de scan est vide** (sinon c'est une touche d'édition) ; `Échap` efface le champ, puis propose d'annuler la vente (rien n'entre au journal : la vente n'a pas eu lieu) ; dans une fenêtre, `Échap` la ferme.

**Payer.** Trois boutons : Espèces, Carte, Autre. En espèces, une fenêtre demande la somme donnée par le client (ou un montant rapide : « Exact », puis des arrondis) : la **monnaie à rendre** s'affiche en direct, et « Valider » reste grisé tant que la somme ne couvre pas le total (le champ n'est pas reformaté pendant la frappe). Par carte ou « Autre » (chèque, virement, mobile money…), **le programme n'est relié à aucun terminal de paiement** et ne voit aucun numéro de carte : la personne à la caisse **confirme à la main** ce que le terminal affiche (« Paiement accepté » ou « Paiement refusé » ; refusé, on peut réessayer ou payer en espèces).

**Valider** fait tout ou rien : la vente est validée en entier (stock suffisant, prix connus) ou refusée avec un message dans la fenêtre, sans rien changer. Le stock baisse et la fenêtre « Vente terminée » montre la monnaie à rendre ; « Voir le ticket » en affiche le détail, avec son numéro (`01-0001` : caisse, puis numéro du journal), l'heure locale, la TVA par taux et la mention « non fiscal ». « Nouvelle vente » la ferme ; la vente, elle, reste dans le journal.

**Europe ou États-Unis** (écran Réglages) :

| | Europe | États-Unis |
|---|---|---|
| Prix saisis | TTC (TVA comprise) | hors taxe |
| Ticket | « Total TTC », puis « dont TVA 5,5 % : … » pour chaque taux | « Sous-total », « Taxe 8 % : … » pour chaque taux, puis « Total » |
| Calcul | la TVA contenue dans le TTC : TTC × taux ÷ (100 % + taux) | la taxe ajoutée à la base : base × taux |

La taxe est calculée **une fois par taux**, sur le total des lignes de ce taux (pas ligne par ligne), arrondie au centime le plus proche, **0,5 arrondi vers le haut**. Chaque produit peut avoir son taux (de 0 à 100 %, deux décimales au plus, par exemple 5,5) ; sans taux propre, c'est le **taux par défaut** des réglages qui s'applique (20 % au départ). Le programme ne connaît ni les taux de chaque pays, État ou commune, ni les exonérations : c'est à toi de les saisir.

**Devises :** euro (EUR), dollar (USD), livre sterling (GBP) et franc CFA (XOF, sans centimes). Les montants s'écrivent à l'usage de la devise (« 12,50 € », « $12.50 », « 15 000 FCFA »). Tu peux taper la virgule ou le point.

**Réglages figés.** Dès la première vente, la devise et le mode de prix ne changent plus (les montants du journal perdraient leur sens) ; le taux par défaut reste modifiable. Un produit qui figure dans une vente ne peut plus être renommé ni supprimé.

**Rembourser un ticket** (écran Historique, bouton « Rembourser ») demande une confirmation. Rien n'est effacé : une **écriture d'annulation** (montant négatif, avec son propre numéro) est ajoutée au journal et apparaît dans l'historique comme une ligne « Remboursement » ; le stock est remis, et le ticket d'origine passe à « Remboursée ». Un ticket ne se rembourse qu'une fois, en entier ; on ne rembourse pas un remboursement. Il n'y a pas encore de remboursement partiel ni d'échange. L'historique montre la journée en cours ; les boutons ‹ et › vont aux autres jours, et taper un numéro dans « N° de ticket » retrouve un ticket quel que soit son jour.

## Le journal de caisse

Chaque vente et chaque annulation est une **pièce** du journal, numérotée 1, 2, 3… **sans trou** (ventes et annulations dans la même suite). Chaque pièce garde ses lignes avec le prix et le taux **du moment** : changer un prix plus tard ne change pas un ticket passé.

Les pièces sont **chaînées** : chacune porte une empreinte SHA-256 calculée sur son contenu (numéro, genre, date, montants, moyen de paiement, lignes) et sur l'empreinte de la pièce précédente. À l'ouverture et après chaque action, le programme recalcule toute la chaîne.

| Détecté | Pas détecté |
|---|---|
| une pièce modifiée à la main (montant, ligne, date, moyen…) | un fichier entièrement réécrit **avec les empreintes recalculées** |
| une pièce supprimée, ajoutée ou déplacée au milieu du journal ; un trou dans la numérotation | les **dernières** pièces supprimées (la chaîne qui reste est valide), ou le remplacement du fichier par une ancienne copie |
| | une modification du stock lui-même (lignes `P` et `M`) : la chaîne ne couvre que le journal des ventes |

Si le journal ne correspond plus, un bandeau l'écrit (écrans Vente et Historique) et les **ventes et les remboursements sont bloqués** ; le stock reste utilisable. Restaure une copie du fichier pour vendre à nouveau.

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

Pour utiliser un autre fichier, indique son chemin complet dans la variable d'environnement `STOCK_FICHIER` (elle a la priorité). Le chemin utilisé est affiché dans le **Menu** et dans **Réglages**, et sur l'écran d'erreur si le fichier est illisible.

**Tu utilises déjà le programme en ligne de commande ?** Son fichier `stock.txt` est dans le dossier où tu le lançais, pas à l'endroit ci-dessus : la fenêtre afficherait un stock vide. Copie ce fichier à l'emplacement par défaut, ou lance la fenêtre avec `STOCK_FICHIER=/chemin/vers/ton/stock.txt`.

**Format du fichier.** Il est écrit au **format version 2** (prix, réglages, journal des ventes). La fenêtre et le programme en ligne de commande de ce dépôt le lisent ; l'**ancienne version** du programme en ligne de commande ne sait pas le relire. Les fichiers de l'ancien format se lisent sans perte, et sont réécrits en version 2 au premier enregistrement : **garde une copie de ton `stock.txt` avant de passer à la nouvelle version.** Le programme en ligne de commande ne vend pas, mais il conserve le journal. Ne modifie jamais à la main les lignes `S;` et `L;` (le journal de caisse) : la fenêtre le détecterait et bloquerait les ventes.

Si le fichier n'existe pas encore, un stock vide est utilisé et la fenêtre le dit. Si le fichier est abîmé, un écran d'erreur s'affiche et le fichier n'est **jamais** modifié.

**Une seule fenêtre à la fois.** Si le fichier a été modifié ailleurs (une autre fenêtre, le programme en ligne de commande) depuis l'ouverture de la tienne, elle refuse d'enregistrer, affiche un message et ne change rien : ferme-la et rouvre-la pour voir le stock à jour. Sans cette protection, une fenêtre périmée effacerait les mouvements faits ailleurs.

**Sauvegarde conseillée :** copie régulièrement ce fichier (sur une clé USB, par exemple). C'est la seule chose à conserver.

## Accessibilité

Le but est le niveau AA des WCAG 2.1, qui est la référence de la norme européenne EN 301 549 et de la Section 508 américaine. **Ce programme n'a pas été audité par un tiers et ne fait l'objet d'aucune déclaration de conformité.** Voici ce qui est fait et comment c'est vérifié :

| Critère WCAG 2.1 | État | Comment |
|---|---|---|
| 1.4.3 Contraste (texte ≥ 4,5:1) | Tenu | des tests dessinent la fenêtre avec la vraie palette et vérifient chaque texte réellement peint sur son vrai fond, sur chaque écran et chaque fenêtre : vente (ticket vide et rempli, confirmations), paiements, recherche, historique (avec la fenêtre de remboursement et le journal altéré), stock et réglages ; ils s'ajoutent aux tests de la palette (`src/palette.rs`) |
| 1.4.11 Contraste des composants (≥ 3:1) | Tenu pour les champs | le contour des **champs de saisie** est `#848B96` (au moins 3,07:1 sur le fond de page comme sur les cartes) ; la maquette avait `#AEB5BF` (2,07:1), assombri au minimum. Les boutons secondaires gardent le contour clair de la maquette (`#D5DAE0`, 1,41:1) : ils sont identifiés par leur texte, pas par leur bord. Le focus clavier est un anneau de 2 points, dessiné aussi sur les boutons à contour fixe (test) ; sa visibilité reste à vérifier à la main |
| 1.4.1 Couleur seule | Tenu | les états sont écrits : « ALERTE : stock bas », « Rupture de stock », « Plus que 3 en stock », « Prix manquant », « Payée », « Remboursée », « Remboursement » ; la ligne choisie du ticket et du historique est annoncée « sélectionnée » (état du bouton) en plus d'être colorée |
| 1.4.4 Redimensionnement du texte | Tenu | zoom au clavier d'egui (`Cmd`/`Ctrl` + `+`, `−`, `0`) ; à 200 % la vente, l'historique et la recherche passent en une seule colonne sans déborder à l'horizontale (tests) |
| 1.4.10 Reflow | Partiel | une seule colonne quand la place manque ; les noms longs sont coupés par des points de suspension (dans le ticket, la recherche et l'historique) au lieu de passer à la ligne |
| 2.1.1 Clavier | À vérifier à la main | tout se fait avec `Tab`, `Maj+Tab`, `Espace`, `Entrée` |
| 3.3.1 / 3.3.3 Erreurs | Tenu | messages « Erreur : … » en français qui disent quoi faire ; un bouton reste grisé tant que son champ est vide |
| 3.3.2 Étiquettes | Tenu | chaque champ est relié à son libellé ; vérifié par des tests qui interrogent le **rôle** (champ de saisie) et le nom |
| 4.1.2 Nom, rôle, valeur | Partiel | noms et rôles vérifiés par des tests : le champ de scan (nom = son texte d'invite), les lignes du ticket (« Choisir Pain »), « − » et « + » (« Retirer une unité de Pain »), les cases de la barre de fonctions (« Rechercher (F2) »), les fenêtres (rôle « dialogue » nommé), « Rembourser le ticket 01-0001 » ; le rendu avec un lecteur d'écran reste à vérifier à la main |
| 4.1.3 Messages d'état | **Non couvert** | egui 0.35 n'a pas de région « live » : un lecteur d'écran n'annonce pas tout seul les messages éphémères en bas de l'écran, ni « Ajouté ✓ » sur la recherche |
| 2.4.3 Ordre du focus | **Limite connue** | l'ordre de `Tab` suit l'ordre de dessin : sur la vente, la barre de fonctions et la colonne de paiement viennent avant le ticket ; en fenêtre étroite (une colonne), le champ de scan passe en premier |
| 1.3.1 Information et relations | **Limite connue** | la liste est faite de cartes, pas d'un vrai tableau : un lecteur d'écran lit chaque ligne comme une suite de textes (nom, état, seuil, quantité) sans structure de tableau ; chaque valeur est écrite en toutes lettres (« Seuil : 5 », « en stock ») pour rester compréhensible |

**Autres limites connues :** un seul thème clair ; les textes de l'interface sont dans `src/textes.rs` mais les messages de résultat sont dans `src/etat.rs` et ceux de la bibliothèque dans `../gestion-stock/src/erreur.rs` (une traduction devra toucher ces trois endroits) ; la qualité de la lecture par un lecteur d'écran dépend d'egui et d'AccessKit ; les boutons des fenêtres, de la barre de fonctions et de la recherche sont dessinés à la main (ils gardent le focus clavier avec son anneau et un nom complet, mais pas toutes les finesses d'un bouton egui) ; `Suppr` ne supprime un article que si le champ de scan est vide ; un contrôle grisé (par exemple « Valider » tant que la somme ne couvre pas le total) est dispensé de contraste par les WCAG, et la raison est écrite à côté.

## Vie privée

Le programme ne collecte aucune donnée personnelle, n'ouvre aucune connexion réseau et n'envoie aucune télémétrie : aucune dépendance réseau n'est présente, sur aucune plateforme (vérifié avec `cargo tree --target all`). Tout reste dans le fichier local ci-dessus. Les noms de produits, quantités, prix, mouvements et ventes (date, lignes, total, moyen de paiement) sont les seules informations enregistrées. Aucun nom de client, aucune adresse et aucun numéro de carte ne sont demandés ni enregistrés.

## Licences

La liste complète des dépendances, pour **toutes** les plateformes (Windows, macOS, Linux), est dans [LICENCES-TIERS.txt](LICENCES-TIERS.txt). Elles sont sous licences permissives (MIT, Apache-2.0, Zlib, BSD, ISC, BSL-1.0, Unicode et assimilées).

- Deux dépendances sont proposées avec une option copyleft **en plus** d'une option permissive : `self_cell` (« Apache-2.0 OR GPL-2.0-only ») et `r-efi` (« MIT OR Apache-2.0 OR LGPL-2.1-or-later »). On utilise toujours l'option permissive.
- Deux familles de polices sont **intégrées au programme** : **IBM Plex Sans** (Regular, Medium, SemiBold et Bold ; licence SIL OFL 1.1, copyright 2017 IBM Corp. avec le nom réservé « Plex » ; le texte de la licence est dans `assets/polices/LICENCE-IBM-Plex-OFL-1.1.txt`), tirée de la version 1.1.0 officielle (`ibm-plex-sans.zip`, SHA-256 `fb365d910566e6d199cc2c15579a7dd9a267128e18431a394ed81f1970c69200` ; les empreintes des quatre fichiers sont dans `src/police.rs`), et les polices par défaut d'egui (OFL-1.1 et Ubuntu-font-1.0), gardées en secours pour les symboles.
- Si tu distribues le programme à d'autres personnes, joins les mentions de licence des dépendances (MIT, Apache-2.0, OFL, Ubuntu-font) et celle d'IBM Plex Sans.
- Le code des deux programmes de ce dépôt n'a pas encore de licence choisie : c'est à son auteur de la décider.

La liste se régénère avec `cargo tree --prefix none --format "{p} {l}" -e normal --target all`.

## Essai manuel

1. Lance le programme, ajoute un produit avec un seuil, fais une entrée, puis une sortie qui passe sous le seuil : l'état affiche « ALERTE : stock bas ».
2. Fais tout **sans la souris** : `Tab` pour avancer, `Maj+Tab` pour reculer, `Espace` ou `Entrée` pour activer. Vérifie que tu vois toujours où est le focus.
3. Agrandis le texte avec `Cmd` + `+` (macOS) ou `Ctrl` + `+`, jusqu'à 200 % : la fenêtre passe en une colonne et l'alerte reste visible. Reviens avec `Cmd`/`Ctrl` + `0`.
4. Sur macOS, active VoiceOver (`Cmd+F5`) et vérifie que les champs et les boutons sont annoncés avec leur nom.
5. Provoque des erreurs : sortie trop grande, nom avec `;` : un message clair s'affiche et rien ne change. Un bouton est grisé tant que son champ est vide.
6. Ferme puis rouvre : les données sont toujours là, au chemin affiché dans le Menu et dans Réglages.
7. Ouvre deux fenêtres sur le même fichier, enregistre dans la première, puis dans la seconde : la seconde refuse et le dit.
8. **Vente en espèces avec rendu.** Dans un fichier d'essai (`STOCK_FICHIER=/tmp/essai.txt`), écran Stock (Menu → Stock) : ajoute « Pain » avec le prix 1,20 et le taux 5,5, puis une entrée de 10. Retour à la vente : tape `2 x pain` puis Entrée, clique « Espèces », tape 5 comme montant reçu : « Monnaie à rendre » affiche 2,60 €. Valide : « Vente terminée » montre la monnaie ; « Voir le ticket » donne le n° 01-0001, « Total TTC : 2,40 € », « TVA 5,5 % incluse : 0,13 € » et la mention « non fiscal » ; le stock est passé à 8.
9. **Remboursement.** Menu → Historique : choisis le ticket 01-0001 et clique « Rembourser ». La fenêtre demande confirmation (« Annuler » ne change rien) ; après confirmation, une ligne « Remboursement » (01-0002, montant négatif) apparaît, le ticket 01-0001 passe à « Remboursée » et le stock est revenu à 10.
10. **Réglages.** Avec un fichier d'essai **neuf** (après une vente, devise et mode de prix sont figés) : Menu → Réglages, choisis États-Unis, USD et le taux 8,25, enregistre : à la vente, les montants sont en $ et la taxe s'ajoute au prix hors taxe.
11. **Journal modifié à la main.** Ferme la fenêtre, ouvre le fichier d'essai, change un montant dans une ligne `L;` ou `S;`, relance : un bandeau le dit, « Espèces », « Carte », « Autre » et « Rembourser » sont grisés, le stock reste utilisable. Remets le fichier d'origine.
12. **Tout sans la souris**, jusqu'à la vente : tape un nom dans le champ de scan puis Entrée, `Tab` jusqu'à « Carte », `Espace`, `Tab` jusqu'à « Paiement accepté », `Entrée`.

## Pour les développeurs

```bash
cargo test                                  # tous les tests
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

La logique est dans `src/etat.rs` (testable sans fenêtre : vente, remboursement, historique, détail d'un ticket, réglages, rendu de monnaie) et `src/scan.rs` (ce que veut dire le texte du champ de scan). Le dessin est dans `src/ui.rs` (l'application, les écrans, les messages) et `src/ui/` : un module par écran (`vente.rs`, `paiement.rs`, `ticket.rs`, `recherche.rs`, `historique.rs`, `stock.rs`, `reglages.rs`), l'en-tête et le Menu (`entete.rs`) et les éléments communs (`composants.rs`). Les couleurs sont dans `src/palette.rs`, la police dans `src/police.rs`. Les calculs d'argent, de taxe et de journal sont dans la bibliothèque (`../gestion-stock/src/monnaie.rs`, `vente.rs`, `journal.rs`). Dans les tests (`src/ui/essais.rs`), `textes_peints` dessine l'application avec la vraie palette et relève chaque texte peint avec son fond : c'est ce test qui garantit le contraste réel, pas seulement celui des constantes.
