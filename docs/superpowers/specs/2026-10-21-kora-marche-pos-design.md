# Caisse Kora Marché — refonte de l'interface et des fonctions de caisse

Date : 2026-10-21 · Statut : **brouillon, en attente d'approbation**

## 1. Intention

L'interface actuelle (`gestion-stock-gui`, egui) est une application de gestion de stock avec un écran de vente par tuiles, en quatre onglets, en indigo, police Inter. La personne qui développe ce projet a conçu une **maquette de caisse de supermarché, « POS Kora Marché »**, et veut que l'application **prenne ce design**, avec **toutes les fonctions que la maquette montre**, y compris celles que l'application n'a pas encore (remise, carte fidélité, mise en attente, code superviseur, ouverture et clôture de caisse, rapport de fin de journée).

Réponse donnée à la question de périmètre : « **Tout, comme la maquette** ».

Cela prolonge la feuille de route déjà écrite dans `docs/superpowers/plans/2026-10-19-vente.md` : ce plan réservait à des chantiers suivants « la caisse (ouverture, clôtures X/Z, mouvements d'espèces) », les remises et les images. Les remises et la caisse entrent donc ici. Les images de produits, elles, restent hors périmètre (voir § 3).

## 2. La référence visuelle

- Fichier d'origine : `gestion-stock/Caisse supermarché — POS Kora Marché.html` (une planche de 7 écrans en 1366 × 768, chaque écran est un bundle imbriqué).
- Pour travailler sans l'ouvrir : `docs/superpowers/specs/2026-10-21-kora-marche-pos/maquette/` contient, pour chaque écran, **une capture** (`N-nom.png`, 1366 × 768, rendue avec Chrome) et **le gabarit lisible** (`N-nom.html`, avec les couleurs, tailles et textes exacts, et le petit script qui donne les comportements : formules, états, règles de saisie).
- Les écrans : `1-vente` (vente et encaissement, avec toutes ses fenêtres : espèces, terminal carte/mobile, vente terminée, confirmation, remise, article introuvable), `2-especes`, `3-carte`, `4-ouverture`, `5-historique`, `6-rapport`, `7-recherche`.
- **Les scripts de la maquette sont la spécification des comportements** (par exemple la lecture de la saisie du scan, le calcul des montants rapides, la règle des 10 % de remise). Ce document les résume ; en cas de doute, le gabarit fait foi, sauf là où le § 9 dit le contraire.
- Les images de produits de la maquette **ne sont pas dans le fichier** (les captures les montrent cassées) : elles n'existent pas.

## 3. Objectifs, non-objectifs, anti-objectifs

**Objectifs**
1. Les 7 écrans de la maquette, fidèles à son apparence : mêmes couleurs, mêmes proportions, même police, mêmes textes en français.
2. Toutes les fonctions que ces écrans montrent, réellement branchées au modèle (rien qui ressemble à un bouton sans rien derrière).
3. Les écrans **Stock** et **Réglages**, que la maquette ne dessine pas, **reprennent le même langage visuel** et restent accessibles (depuis le « Menu »).
4. Un usage à la caisse : cibles tactiles larges, scan au clavier, raccourcis F2, F4, F5, F8, Suppr, Échap, tout faisable sans souris.
5. Tout ce qui existe déjà continue de marcher : stock, journal chaîné, numérotation sans trou, annulation par écriture inverse, TVA par taux, quatre devises, lecture sans perte des fichiers actuels.

**Non-objectifs**
- Matériel : imprimante de tickets, tiroir-caisse, terminal de paiement, lecteur de codes-barres relié autrement que par le clavier (un lecteur de code-barres USB se comporte comme un clavier : il tape le code puis Entrée).
- Réseau, synchronisation, mode hors ligne, plusieurs caisses, plusieurs comptes de caissiers.
- Certification de caisse (NF 525, TSE…). Le ticket garde la mention « Ticket non fiscal » et le README garde sa section « Caisse et conformité ».
- Images de produits (chantier à part, déjà annoncé dans le plan de la vente). Les lignes et la recherche montrent un **repère de remplacement** (carré gris arrondi avec les initiales du produit).
- Remboursement partiel, échange (la maquette ne les montre pas).

**Anti-objectifs** (un échec même si « ça marche »)
- Casser l'intégrité du journal : les pièces déjà enregistrées doivent rester valides, la numérotation des tickets reste **sans trou**.
- Perdre ou rendre illisible un fichier de données existant.
- Afficher quelque chose de faux : « tiroir ouvert », « ticket imprimé », « En ligne » alors que rien de tel n'existe.
- Un rendu « gabarit par défaut d'egui » : si l'écran ne ressemble pas à la capture, c'est raté.
- Perdre l'accessibilité acquise (contraste AA vérifié par des tests, noms et rôles des contrôles, zoom).

## 4. Contraintes héritées du projet (à respecter)

Reprises de `docs/superpowers/plans/2026-10-19-vente.md` et du code : Rust 1.93, édition 2024, egui/eframe 0.35 ; tout le métier dans la bibliothèque `gestion-stock`, la fenêtre au-dessus (`Etat`) ; **la bibliothèque ne lit jamais l'heure** (elle vient en paramètre) ; montants en entiers (`Montant(i64)`), arithmétique vérifiée, jamais de flottant ; pas de `unwrap`/`expect`/panique hors tests ; une opération qui échoue ne change **rien** ; une pièce du journal ne se modifie jamais ; textes de la fenêtre dans `textes.rs` ; dépendances licenciées MIT/Apache (et polices OFL) ; `cargo clippy --all-targets -- -D warnings` et `cargo fmt --check` passent ; le programme **n'ouvre aucune connexion réseau** (promesse écrite dans le README, section « Vie privée »).

`ui.rs` fait déjà 3 100 lignes, tests compris : la refonte doit le **découper en modules par écran** plutôt que d'y ajouter. Environ 90 tests de l'interface (libellés de tuiles, onglets, etc.) devront être réécrits avec les nouveaux écrans ; les garde-fous qu'ils portent (contraste AA de chaque texte peint, noms accessibles, pas de double encaissement, fenêtre étroite, zoom 200 %) **doivent survivre** à la réécriture.

## 5. Le langage visuel (jetons)

Unités : points egui = pixels CSS de la maquette à 1366 × 768. La fenêtre s'ouvre en **1366 × 768** (la maquette est dessinée à cette taille) ; la taille minimale actuelle (640 × 420) reste, avec une mise en page empilée dessous d'environ 1100 points de large (le détail est laissé au planificateur).

**Police** : IBM Plex Sans (licence SIL OFL 1.1), graisses 400, 500, 600, 700, **en remplacement d'Inter** (egui n'accepte qu'un fichier par famille : une famille par graisse, comme `police.rs` le fait déjà pour Inter semi-gras). Chiffres : la maquette demande des chiffres de largeur fixe (`tabular-nums`) pour aligner les montants ; egui ne sait pas activer cette option : le planificateur vérifie l'alignement des colonnes de montants (alignées à droite) et, si les chiffres de Plex ne sont pas de largeur fixe par défaut, le dit. La licence de la police s'ajoute à `LICENCES-TIERS.txt` avec l'empreinte du fichier, comme pour Inter.

**Couleurs** (toutes à mettre dans `palette.rs`, avec leurs tests de contraste) :

| Rôle | Valeur |
|---|---|
| Fond de fenêtre | `#F2F3F5` |
| Surface (cartes, fenêtres modales, champs) | `#FFFFFF` |
| Bord de carte | `#E1E4E8` · filet de ligne `#EEF0F3` |
| Bord de bouton / de champ | `#D5DAE0` / `#AEB5BF` (`#C9CED6` pour − et +) |
| Texte principal / secondaire / discret | `#16191D` / `#4A515B` / `#5B636E` |
| Texte et fond d'un contrôle grisé | `#6B7280` sur `#E3E6EA` (icône `#9AA1AB`) |
| En-tête | fond `#16191D` · texte `#FFFFFF` · sous-texte `#C4CAD3` |
| Accent (boutons principaux, focus, champs actifs) | `#1E56C8` · survol `#163F94` · ligne sélectionnée `#EAF0FC` |
| Succès | texte `#12653F` sur `#E8F5EE` · bouton validé `#12784A` |
| Erreur / alerte | texte `#A61B12` sur `#FDECEA` · bord `#E8B4AE` · bouton de danger `#B42318` |
| Avertissement (stock bas, « Remboursée ») | texte `#8A4106` |
| Point de statut dans l'en-tête | `#6FD3A0` (ouverte) · `#F5B661` (avertissement) · `#8B929C` (fermée) |
| Voile derrière une fenêtre modale | noir-bleu `#16191D` à 50 % |

Règle : **le contraste AA (4,5:1 pour le texte, 3:1 pour les contours) reste vérifié par les tests existants, sur les couleurs de la maquette.** Si une paire de la maquette échoue, on assombrit le moins possible et on le **liste dans le README** ; on ne baisse jamais le seuil.

**Formes et tailles principales** (les autres sont dans les gabarits) : en-tête 56 ; marge extérieure et écart 16 ; colonne de paiement 420 ; cartes à coins de 12 ; fenêtres modales à coins de 16, marge 28, largeur 520 (espèces), 480 (terminal, vente terminée), 460 (confirmations, remise, remboursement) ; champ de scan 56 de haut, bord de 2 en accent ; ligne de ticket 68 minimum ; boutons − et + 44 × 44 ; total en 64 gras ; bouton Espèces 84 de haut, Carte et Mobile money 72 ; barre de fonctions : 7 cases de 64 de haut ; focus clavier : contour de 2 en accent, décalé de 2.

## 6. Navigation

Plus d'onglets. L'**écran de vente** est l'écran principal. Depuis lui :

- **Rechercher** (F2) → écran Recherche. **Menu** → un panneau qui mène à : Historique, Rapport de caisse, Ouverture / Clôture de la caisse, Dépôt au coffre, **Stock**, **Réglages**.
- Chaque écran secondaire a l'en-tête de la maquette avec un bouton blanc « Retour à la vente » (Historique ajoute « Rapport de caisse » ; Rapport ajoute « Imprimer » et « Clôturer la caisse »).
- La caisse **fermée** montre l'écran Ouverture ; on ne peut pas vendre tant qu'elle est fermée. Cet écran a lui aussi un bouton **Menu** (Réglages, Stock, Historique, Rapport de caisse ; pas de Dépôt tant que la caisse est fermée) : sans lui, un premier lancement ne pourrait pas régler la devise, le fonds de caisse, le code superviseur ni les produits avant la première ouverture.

## 7. Les écrans et leurs règles

Les captures et gabarits ont le détail ; voici ce que la maquette décide et ce que l'application doit en faire.

### 7.1 Vente (`1-vente`)
- **En-tête** : nom du magasin (gras), « Caisse NN · Nom du caissier », à droite l'état de la caisse (point + texte) et l'heure locale (HH:MM, mise à jour).
- **Champ de scan** (toujours actif au démarrage et après chaque action) : accepte un **code-barres exact** ou une partie de nom (≥ 3 caractères, accepté si **un seul** produit correspond), avec un préfixe de quantité `3 x`, `3*`, `3×`. Entrée valide. Rien de trouvé → fenêtre « Article introuvable » (« Scanner à nouveau » / « Rechercher »). Pas de bouton « Simuler un scan » (voir § 9).
- **Ticket en cours** (colonne de gauche) : une ligne par article (repère, nom, « qté × prix · remise N % », total de ligne). **Une seule ligne est sélectionnée** ; seule la ligne sélectionnée montre − quantité +. Pied : « N articles » et « Carte fidélité : NOM » ou « Pas de carte fidélité ». État vide : « Prêt pour une nouvelle vente / Scannez le premier article pour commencer. »
- **Colonne de paiement** : Sous-total, Remises (en vert, si > 0), TOTAL en grand avec l'unité, ligne « TVA 18 % incluse : … » (en général : **une ligne par taux**, comme le ticket actuel ; en profil États-Unis, la taxe ajoutée). Boutons **Espèces** (grand, plein), **Carte** et **Mobile money** (contour). Les trois sont grisés si le ticket est vide.
- **Barre de fonctions** (7 cases, avec raccourci écrit) : Rechercher F2 · Carte fidélité F4 · Remise F5 (grisée sans ligne sélectionnée) · Supprimer l'article Suppr (idem) · Mettre en attente F8 (devient « Reprendre la vente ») · Annuler la vente Échap (rouge) · Menu.
- **Fenêtres** : Espèces (§ 7.2) ; terminal (§ 7.3) ; vente terminée (coche verte, « Moyen · total », monnaie à rendre en grand pour les espèces, bouton « Nouvelle vente » ; **plus un lien « Voir le ticket »** qui affiche le ticket actuel de l'application, avec sa TVA par taux et « Ticket non fiscal ») ; confirmation « Annuler la vente ? » / « Supprimer cet article ? » (boutons « Non, revenir » / « Oui, … » en rouge) ; remise ; article introuvable.
- **Remise** : par **ligne**, choix parmi Aucune, 5, 10, 15, 20 %. Au-delà de **10 %** : champ « Code superviseur (au-delà de 10 %) » obligatoire, « Appliquer » grisé tant qu'il n'est pas bon. Calcul : remise de ligne = ligne brute × % arrondie à la moitié supérieure ; la TVA se calcule ensuite par taux sur les lignes remisées (règle actuelle du projet).
- **Mise en attente** : une seule vente en attente à la fois, **en mémoire seulement** (perdue si l'application se ferme). Reprendre n'est possible que si le ticket en cours est vide (« Terminez d'abord la vente en cours »).
- **Message éphémère** (toast) en bas, 3 secondes : « Aucune vente en cours », « Vente mise en attente », « Vente reprise », « Vente annulée ».
- Annuler la vente = vider le ticket en cours après confirmation (**avant** tout paiement) : rien n'entre au journal.

### 7.2 Paiement en espèces (`2-especes`)
« À payer », champ « Montant reçu » (chiffres, espaces de milliers à l'affichage), 4 montants rapides : **Exact** puis les 3 prochains arrondis supérieurs au total (pas : 1 000, 5 000, 10 000, 20 000, 50 000 en FCFA ; pas adaptés pour les autres devises, au choix du planificateur), puis bandeau **« Monnaie à rendre »** (vert) ou **« Il manque »** (rouge), Retour / Valider (grisé tant que le reçu ne couvre pas le total).

### 7.3 Carte et Mobile money (`3-carte`)
Fenêtre « Paiement par carte » / « Paiement mobile » avec le montant et un bandeau d'état : **en attente** (bleu : « Présentez la carte sur le terminal » ; mobile : « Le client scanne le QR code ou valide sur son téléphone »), **acceptée** (vert), **refusée** (rouge : « Carte refusée », « Aucun montant n'a été débité. Proposez un autre moyen de paiement. » ; boutons « Réessayer » et « Payer en espèces »). Comme il n'y a aucun terminal relié, **le caissier confirme à la main** ce que le terminal affiche (§ 9). Annuler ferme la fenêtre.

### 7.4 Recherche (`7-recherche`)
Titre, champ (sans accents ni majuscules, comme la recherche actuelle), compteur (« 6 articles »), liste : repère, nom, état du stock (« En stock » vert ; « Plus que N en stock » ambre si 1 à 5 ; « Rupture de stock » rouge), prix, bouton « Ajouter » (bleu) → « Ajouté ✓ » (vert) ; « Indisponible » (grisé) en rupture. « Aucun article trouvé. Vérifiez l'orthographe ou le code. » Un produit **sans prix** n'est pas vendable : il apparaît « Prix manquant » et son bouton est grisé (règle actuelle). Ajouter met l'article au ticket sans quitter la recherche ; « Retour à la vente » revient.

### 7.5 Ouverture de caisse (`4-ouverture`)
« Comptez l'argent du tiroir avant la première vente. » Grille de coupures (nombre de billets/pièces saisi pour chacune ; en FCFA : 10 000, 5 000, 2 000, 1 000, 500 et une ligne « Pièces » où l'on saisit le **montant total** des pièces), puis **Compté / Attendu / Écart**. Écart nul : vert, bouton « Ouvrir la caisse » ; écart non nul : rouge, « Écart détecté : recomptez ou prévenez votre responsable avant d'ouvrir. », bouton **« Ouvrir quand même (écart signalé) »**. L'ouverture est **enregistrée au journal** avec le compté, l'attendu et l'écart.

### 7.6 Historique (`5-historique`)
« Ventes du jour » (journée locale) : liste N° · heure · moyen · total · statut, filtre « N° de ticket » ; la ligne choisie ouvre à droite le ticket (lignes avec repère, quantité, nom, total ; total ; en-tête « HH:MM · Moyen · N articles · Statut »). **Réimprimer** (§ 9) et **Rembourser** : fenêtre « Rembourser le ticket … ? » avec le montant « par le même moyen de paiement » et le **code superviseur** ; l'action écrit l'**annulation** déjà existante (écriture inverse, stock remis) ; le ticket passe à « Remboursée » (ambre). Statuts : **Payée** (vert), **Remboursée** (ambre). Rembourser est grisé pour un ticket déjà remboursé et pour une annulation.

**Autres jours et annulations** (ajouté après relecture, voir § 9.12) : la liste montre par défaut la **journée locale en cours**, avec un sélecteur de jour (précédent, suivant, date). Le filtre « N° de ticket » cherche dans **tous les jours**, et on peut rembourser un ticket ancien (la liste actuelle des ventes le permet : cela ne doit pas se perdre). Les **annulations** apparaissent comme des lignes à part, avec leur propre numéro (aucun trou dans la suite des numéros), le libellé « Remboursement » en ambre et un total négatif ; elles n'ont pas de bouton Rembourser.

### 7.7 Rapport de caisse (`6-rapport`)
« Rapport de fin de journée », date longue et plage « ouverture → maintenant (ou clôture) ». Deux cartes. **Ventes** : total, nombre de tickets, panier moyen ; par moyen (Espèces, Carte, Mobile money, **Autre**) ; remises ; remboursements (nombre et montant) ; dont TVA. **Tiroir** : fonds d'ouverture + ventes en espèces − remboursements en espèces − dépôts au coffre = **Attendu** ; **Compté** ; **Écart** (vert si nul, rouge sinon). « Imprimer » (§ 9). **« Clôturer la caisse »** ouvre le comptage (la même grille de coupures que l'ouverture), enregistre la **clôture** au journal (compté, attendu, écart) et revient à l'écran d'ouverture. Avant la clôture, « Compté » n'est pas affiché (« — »).

**Ce que couvre un rapport** (ajouté après relecture, voir § 9.12) : **une séance**, de l'ouverture à la clôture (ou à maintenant si elle est ouverte), pas un jour civil : deux séances le même jour, ou une séance qui passe minuit, donnent deux rapports. Le Menu ouvre le rapport de la séance en cours, ou de la **dernière séance clôturée** (avec Compté et Écart) ; un sélecteur (précédente, suivante) parcourt les séances passées. Le fichier d'impression se nomme `rapport-AAAA-MM-JJ-HHMM.txt` (date et heure de l'**ouverture**), donc une seconde séance n'écrase pas la première. **« Fonds d'ouverture » = le montant compté à l'ouverture** (ce qu'il y a vraiment dans le tiroir) : l'écart d'ouverture n'est compté qu'une fois, à l'ouverture, et n'est pas répété dans l'écart de clôture.

### 7.8 Dépôt au coffre
La maquette l'utilise dans le rapport (« − Dépôts au coffre ») sans écran pour le créer. Fenêtre simple depuis le Menu : montant (≤ espèces attendues dans le tiroir), confirmation ; enregistré au journal.

### 7.9 Stock et Réglages (pas de maquette)
Mêmes en-tête, cartes, boutons et champs que la maquette. Stock : ce qu'il fait aujourd'hui **plus le champ « Code-barres »** (optionnel, unique) à l'ajout et dans les actions du produit. Réglages : ce qu'ils font aujourd'hui **plus** : nom du magasin, numéro de caisse, nom du caissier (affichés dans l'en-tête), fonds de caisse prévu à l'ouverture, **code superviseur** (§ 8), liste des cartes fidélité (§ 8).

## 8. Ce que le modèle doit apprendre

Le format de fichier passe en **version 3**. Un fichier v2 (ou plus ancien) se lit **sans perte** et se réécrit en v3 au premier enregistrement ; le programme en ligne de commande de ce dépôt apprend à lire v3 ; l'ancienne version ne le lit pas (le README le dit, avec le conseil de copier le fichier avant).

- **Produit** : un code-barres optionnel, unique (comparaison exacte).
- **Ligne de vente** : une **remise** (en pour cent, 0 à 100 ; la maquette n'en propose que 0, 5, 10, 15, 20), figée comme le prix et le taux.
- **Moyen de paiement** : nouveau **Mobile money** ; « Autre » reste (chèques, virements, bons, avoirs). Les codes existants (E, C, X) gardent leur sens.
- **Séances de caisse** : événements **Ouverture** (compté, attendu, écart), **Dépôt** (montant), **Clôture** (compté, attendu, écart). **Ils ont leur propre numérotation** (les numéros de tickets montrés aux clients restent la suite sans trou ventes + annulations), mais **tous sont couverts par la chaîne d'empreintes**, et la vérification d'intégrité les inclut. Une vente demande une séance ouverte ; un fichier qui n'a jamais eu de séance démarre « caisse fermée ».
- **Les empreintes des pièces déjà écrites ne changent pas** : un journal v2 intact reste valide après passage en v3 (les champs nouveaux n'entrent dans l'empreinte que quand ils sont présents).
- **Réglages** : nom du magasin, numéro de caisse, nom du caissier, fonds de caisse prévu, **code superviseur** (4 à 8 chiffres, conservé sous forme d'**empreinte salée**, jamais en clair).
- **Code superviseur** : exigé pour une remise > 10 % et pour un remboursement. Tant qu'aucun code n'est défini, ces actions sont refusées avec un message qui renvoie à Réglages. Le **premier** code se définit librement ; ensuite, **le changer ou le retirer demande le code actuel** (sinon un caissier le remplacerait pour contourner le contrôle). Code oublié : la procédure de secours (retirer le code dans le fichier de données, hors de la chaîne d'empreintes) s'écrit dans le README. Il n'y a **pas de blocage après des échecs** (limite connue, à écrire dans le README). Comme le journal chaîné, ce code **ne protège pas contre quelqu'un qui édite le fichier à la main**.
- **Réglages figés** : aujourd'hui la devise et le mode de prix se figent à la première **vente**. Désormais la devise, le profil **et le numéro de caisse** se figent dès le **premier enregistrement du journal de n'importe quel genre** (ouverture, dépôt ou vente) : un comptage en euros ne doit pas être suivi d'un passage en FCFA, et changer le numéro de caisse renommerait tous les tickets passés. Restent modifiables : le taux par défaut, le nom du magasin, le nom du caissier, le fonds de caisse prévu.
- **Cartes fidélité** : une liste (nom, numéro de carte) dans le fichier ; le caissier en rattache **une** à la vente (F4 : recherche par nom ou numéro, ou « Retirer la carte ») ; le nom est écrit **sur le ticket et dans la pièce du journal**. **Identification seulement : pas de points, pas de réduction.** Cela contredit la phrase actuelle du README (« aucun nom de client n'est enregistré ») : la section « Vie privée » est à réécrire.
- **Numéro de ticket affiché** : `NN-NNNN` = numéro de caisse sur 2 chiffres, tiret, numéro du journal sur 4 chiffres (plus si besoin) ; le numéro interne reste celui du journal.

## 9. Mes choix (à relire, ils ne viennent pas de toi)

1. **Pas d'indicateur « En ligne / Hors ligne · N ventes à synchroniser »** : il n'y a aucun réseau. L'en-tête montre « Caisse ouverte » / « Caisse fermée ».
2. **Les boutons « Simuler… » de la maquette** sont des aides de prototype. « Simuler un scan » disparaît (un lecteur USB tape dans le champ). Pour carte et mobile, les boutons deviennent de **vraies confirmations manuelles** : « Paiement accepté » et « Paiement refusé » (le caissier lit le terminal).
3. **Pas de « tiroir ouvert » ni de « ticket imprimé »** : aucun matériel. La vente terminée dit « Ticket enregistré » et propose « Voir le ticket ».
4. **Imprimer / Réimprimer** (historique, rapport) : l'application **écrit un fichier texte** dans un dossier `tickets/` à côté du fichier de données (`ticket-04-0186.txt`, `rapport-AAAA-MM-JJ.txt`) et affiche son chemin. Pas de pilote d'imprimante.
5. **Statut « Annulée »** (ticket barré de la maquette) : aucune action ne le produit (après paiement, le seul retour est le remboursement). Seuls **Payée** et **Remboursée** existent.
6. **Valeurs de départ** : magasin « Kora Marché », caisse 1, caissier vide (l'en-tête n'affiche alors que « Caisse 01 »), fonds de caisse prévu 0, devise et profil inchangés (Europe/EUR/20 % tant qu'on n'a pas réglé autrement ; la maquette est en FCFA, TVA 18 % incluse).
7. **« Attendu » à l'ouverture** = le **fonds de caisse prévu** des Réglages (pas le reste de la veille).
8. **Coupures par devise** (listes en dur, au choix du planificateur) ; la ligne « Pièces » est un montant total.
9. **Mise en attente en mémoire seulement.**
10. **Ordre de livraison** (chaque étape laisse l'application utilisable, tests verts) :
    - **Étape A — apparence** : police, jetons, en-tête, écrans Vente, paiements, Recherche, Historique (avec remboursement sans code), Stock et Réglages restylés, Menu. Fonctions **existantes** seulement ; moyens Espèces / Carte / Autre ; le scan sert par nom.
    - **Étape B — vente** : format v3, code-barres, Mobile money, remises, code superviseur, mise en attente.
    - **Étape C — caisse** : séances (ouverture, dépôt, clôture), rapport, « Imprimer ».
    - **Étape D — fidélité** : cartes, F4, ticket.
11. **Fenêtre** par défaut 1366 × 768 ; mise en page empilée sous environ 1100 points de large pour garder l'usage en petite fenêtre et à 200 % de zoom.
12. **Réponses aux questions de la relecture** (le relecteur n'avait que ce document) : un rapport couvre une **séance** et reste consultable après la clôture (§ 7.7) ; le fonds d'ouverture est le **montant compté** (§ 7.7) ; le code superviseur se **change avec l'ancien code**, avec une procédure de secours écrite (§ 8) ; l'**historique garde l'accès aux anciens jours** et montre les annulations comme lignes « Remboursement » (§ 7.6) ; la devise, le profil et le **numéro de caisse se figent au premier enregistrement du journal**, pas seulement à la première vente (§ 8) ; l'écran **Ouverture a un Menu** vers Réglages, Stock, Historique, Rapport (§ 6).

## 10. Vérification

- Tous les tests existants qui portent un garde-fou (contraste AA, noms accessibles, double encaissement, zoom 200 %, fenêtre basse) sont **réécrits** pour les nouveaux écrans, pas supprimés.
- `tests/apercu.rs` (outil de capture) produit une image **par écran de la maquette**, à 1366 × 768, qu'on compare **à l'œil** aux captures du dossier `maquette/`. Le planificateur décide s'il garde ces captures dans le dépôt.
- Tests de modèle : lecture d'un fichier v2 puis réécriture en v3 sans perte ; chaîne d'empreintes valide après v2 → v3 ; remises (arrondi, TVA par taux sur lignes remisées) ; séances ; code superviseur (jamais en clair dans le fichier).
- `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check`, `cargo test` dans les deux dossiers.
- README et `LICENCES-TIERS.txt` mis à jour (apparence, fonctions, format v3, conformité, vie privée, accessibilité, nouvelle police).

## 11. Laissé au planificateur

Le découpage exact en modules ; la représentation des remises et des événements dans le fichier ; le calcul des montants rapides hors FCFA ; les listes de coupures ; la mise en page empilée ; la manière de faire tenir les chiffres alignés avec la police ; le sort des captures d'aperçu dans le dépôt ; l'ordre fin des tâches à l'intérieur de chaque étape.
