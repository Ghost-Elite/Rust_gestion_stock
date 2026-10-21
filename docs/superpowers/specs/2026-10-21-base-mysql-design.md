# Base de données MySQL pour la caisse Kora Marché — conception

> Statut : **approuvé** (choix (a) du § 9 : la caisse refuse de vendre quand le serveur est injoignable). Plan 1 : `docs/superpowers/plans/2026-10-21-base-mysql-couche-stockage.md`.
>
> **Avancement.** Plan 1 (la couche de stockage) : **fait** — schéma et migrations, `DepotMysql` (même scénario de règles que le fichier), transactions et ordre fixe des verrous (un interblocage a été trouvé et corrigé par les tests de concurrence), reprise après coupure, import du `stock.txt` (60 000 pièces testées), commandes `stock migrer / importer / verifier`. Reste à faire **avec l'accord de la personne** : créer la vraie base `kora_marche` et son compte `kora` sur le MariaDB de XAMPP, écrire `base.conf` et importer le vrai `stock.txt`. Plan 2 (l'interface graphique et les commandes du fichier sur la base, « En ligne / Hors ligne ») : **à écrire**.
> Cette conception prolonge `2026-10-21-kora-marche-pos-design.md` (étapes B, C et D de la caisse) : elle doit être faite **avant l'étape B**, parce que B change le format des données.

## 1. Intention

Aujourd'hui toutes les données (produits, mouvements de stock, réglages, journal des ventes) sont dans un **seul fichier texte** (`stock.txt`, format version 2) que le programme lit en entier au démarrage et réécrit en entier à chaque opération. C'est simple et sûr pour **une seule caisse**, mais :

- deux postes (deux caisses, un bureau de gestion) ne peuvent pas partager le même fichier sans s'écraser l'un l'autre ;
- la maquette montre des choses qu'un fichier ne donne pas : « En ligne / Hors ligne », le nom du caissier (« Caisse 04 · Awa Ndiaye »), des tickets numérotés par caisse (« 04-0186 »), un code superviseur, des cartes fidélité, une ouverture et une clôture de caisse ;
- la personne qui développe ce projet veut **une base MySQL** pour « les produits et le reste », et XAMPP est déjà installé sur sa machine.

**But** : les produits, le stock, les ventes et les réglages vivent dans une base MySQL partagée ; plusieurs postes peuvent vendre en même temps sans jamais vendre le même dernier article deux fois, sans trou dans la numérotation des tickets, et sans casser la détection de modification du journal.

## 2. Ce qui existe et qui doit rester vrai

Le cœur (`gestion-stock`, tests unitaires en mémoire) porte les règles du métier. **Elles ne sont pas réécrites** : la base doit les faire respecter telles quelles.

- Un nom de produit est unique **sans tenir compte de la casse ni des espaces autour** (`riz 5kg` = `Riz 5KG`), mais **les accents comptent** (`thé` ≠ `the`). Il ne contient ni `;` ni saut de ligne, et n'est pas vide.
- Le stock ne devient jamais négatif. Une vente est refusée **en entier** si un seul article manque : rien ne change, pas même le numéro du ticket.
- Les montants sont des entiers en plus petite unité de la devise (`Montant(i64)`, limite `MONTANT_MAX`). Les taux sont en centièmes de pour cent (`taux_bp`, au plus 10 000).
- Le journal des ventes (« pièces ») est **append-only** : numérotation continue 1, 2, 3… sans trou, ventes et annulations confondues ; chaque pièce porte l'empreinte SHA-256 de la précédente (`journal.rs`) ; une annulation est **totale**, par écriture inverse, et une vente ne s'annule qu'une fois ; une annulation ne s'annule pas.
- Un produit qui figure dans le journal ne peut plus être ni renommé ni supprimé. Renommer un produit remplace aussi son nom dans l'historique des mouvements.
- Devise et profil (Europe / États-Unis) sont figés dès qu'une pièce existe ; le taux par défaut peut changer.
- L'empreinte d'une pièce se calcule sur le **texte canonique** de `journal.rs` (numéro, genre, heure, décalage, total, moyen, reçu, rendu, vente annulée, puis nom, quantité, prix et taux de chaque ligne) : une pièce importée ou écrite par la base doit donner exactement la même empreinte que dans le fichier.

## 3. Objectifs, non-objectifs, anti-objectifs

**Objectifs**
1. Une base `kora_marche` (MySQL/MariaDB de XAMPP) qui contient produits, mouvements, réglages et journal, avec un schéma versionné (migrations SQL dans le dépôt).
2. Plusieurs postes en même temps, sans survente ni trou de numéro, grâce à des **transactions**.
3. Le journal reste protégé par ses empreintes ; `verifier_journal` continue de dire quelle pièce ne correspond plus.
4. Le stock actuel (`stock.txt`) s'importe **sans perte** : mêmes numéros, mêmes empreintes.
5. L'interface graphique montre « En ligne » ou « Hors ligne » à partir de l'état réel de la connexion.
6. Le mode fichier actuel **continue de marcher** quand aucune base n'est configurée.

**Non-objectifs (ce document)**
- Les tables des étapes B, C et D (utilisateurs et rôles, code superviseur, remises, codes-barres, cartes fidélité, ouverture, dépôt et clôture de caisse, rapport). Chaque étape ajoutera ses **migrations** ; le schéma ci-dessous ne les prépare pas à l'avance.
- Vendre quand le serveur est injoignable, puis synchroniser plus tard (voir § 8, question ouverte).
- Chiffrer la liaison réseau (TLS) et gérer plusieurs magasins.
- Les images de produits.

**Anti-objectifs** (ce qui serait un échec même si les tests passent)
- Perdre ou réécrire une vente déjà enregistrée ; autoriser deux tickets de même numéro dans une caisse.
- Toucher aux autres bases de XAMPP (`kaytegue`, `phpmyadmin`, `mysql`…) ou au compte `root`.
- Mettre un mot de passe dans le dépôt Git, dans un message d'erreur ou dans un journal.
- Que la caisse se fige sans message quand le serveur ne répond pas.

## 4. Mes choix (à relire : ce sont les miens, pas les vôtres)

1. **Serveur** : le MariaDB 10.4 de XAMPP (compatible MySQL ; c'est lui qui est installé). Il n'est lancé qu'avec votre accord. Base `kora_marche`, jeu de caractères `utf8mb4`, moteur InnoDB.
2. **Compte** : un compte dédié `kora`, limité à la base `kora_marche` (lecture, écriture, pas de création de base). Le compte `root` de XAMPP n'est utilisé que pour créer la base et le compte, une fois.
3. **Client Rust** : la bibliothèque `mysql` (synchrone, v28). Le programme n'a pas d'exécuteur asynchrone (egui est synchrone) : `sqlx` obligerait à en ajouter un pour rien.
4. **Un journal par caisse** : chaque poste a un numéro de caisse (`01`, `04`…), sa propre numérotation continue et sa propre chaîne d'empreintes. Cela donne les tickets `04-0186` de la maquette, évite qu'une caisse bloque les autres pendant l'écriture, et laisse la porte ouverte à une vente hors ligne plus tard (une caisse est la seule à écrire sa propre chaîne). Le stock, lui, est partagé.
5. **Les règles du cœur sont réutilisées, pas dupliquées** : pour chaque opération, la base ouvre une transaction, **verrouille** les lignes concernées, les charge dans un `Stock` partiel, appelle le code existant (`encaisser`, `annuler`, `enregistrer_entree`…), puis écrit la différence et valide. Ainsi la règle « pas de survente » reste celle qui est déjà testée, et elle est revérifiée **dans** la transaction (deux postes qui vendent le dernier article : l'un réussit, l'autre reçoit « stock insuffisant »).
6. **Lecture** : l'interface garde une copie en mémoire (produits, historique) rafraîchie en arrière-plan à intervalle court ; **écriture** : toujours par la base. Les écrans de lecture ne changent presque pas.
7. **Où est la configuration** : un fichier `base.conf` (hôte, port, base, utilisateur, mot de passe, numéro de caisse) hors du dépôt, dans `~/.config/kora-marche/` (ou le chemin de `KORA_BASE_CONF`). Sans ce fichier : mode fichier, comme aujourd'hui. Le programme avertit si le fichier est lisible par d'autres utilisateurs.
8. **Fuseau et heure** : les pièces gardent leur heure sous la forme actuelle (`utc` + `décalage en minutes`, deux entiers), parce que l'empreinte les couvre ; la base n'en fait ni `DATETIME` ni `TIMESTAMP`.

## 5. Le schéma (version 1)

Tout est en `utf8mb4`. Les clés étrangères sont déclarées. Les montants et quantités gardent les types du cœur (`i64`, `u32`).

| Table | Colonnes principales | Remarques |
|---|---|---|
| `schema_version` | `version` (clé), `applique_le` | une ligne par migration appliquée |
| `reglage` | `cle` (clé), `valeur` | `profil`, `devise`, `taux_defaut_bp` |
| `caisse` | `code` (clé, 2 chiffres), `nom`, `dernier_numero`, `dernier_hash` | la ligne de tête de chaque journal, **verrouillée** pour écrire une pièce |
| `produit` | `id`, `nom`, `nom_cle` (unique), `quantite`, `seuil`, `prix` (NULL), `taux_bp` (NULL) | `nom_cle` = nom nettoyé en minuscules **calculé par le programme**, collation binaire (`utf8mb4_bin`) : même unicité que le mode fichier, accents compris. Une collation `_ci` ferait `thé` = `the` |
| `mouvement` | `id`, `produit_nom`, `produit_cle`, `genre` (entrée / sortie), `quantite`, `caisse`, `piece_numero` (NULL), `cree_le` | `produit_nom` est le nom en texte (comme dans le fichier) : renommer un produit le remplace partout ; `produit_cle` (même calcul que `nom_cle`, indexé) sert à retrouver ses mouvements |
| `piece` | `caisse`, `numero`, `genre` (V / A), `annule_numero` (NULL), `utc`, `decalage_minutes`, `total`, `moyen`, `recu`, `rendu`, `hash_precedent`, `hash` | clé `(caisse, numero)` ; ni modification ni suppression par le programme |
| `piece_ligne` | `caisse`, `numero`, `rang`, `nom`, `nom_cle`, `quantite`, `prix`, `taux_bp` | le texte canonique de l'empreinte lit ces lignes dans l'ordre de `rang` ; `nom_cle` (indexé) sert à savoir si un produit a déjà été vendu |

Contraintes : `produit.quantite >= 0` (colonne non signée), `piece.annule_numero` unique par caisse (une vente ne s'annule qu'une fois), les réglages devise/profil ne se changent pas s'il existe une pièce (le cœur le vérifie, la base ne fait que stocker).

## 6. Le déroulement d'une opération (exemple : encaisser)

1. Ouvrir une transaction (`READ COMMITTED` suffit, les verrous font le travail).
2. Verrouiller la ligne `caisse` du poste (`SELECT … FOR UPDATE`) : on lit `dernier_numero` et `dernier_hash`.
3. Verrouiller les lignes `produit` du panier, **toujours dans l'ordre alphabétique de `nom_cle`** (deux postes ne s'attendent jamais l'un l'autre en croix).
4. Construire un `Stock` partiel avec ces produits, les réglages et la tête du journal ; appeler `Stock::encaisser` (inchangé).
5. Écrire : nouvelle quantité de chaque produit, un `mouvement` par ligne, la `piece` et ses `piece_ligne`, puis la nouvelle tête dans `caisse`.
6. Valider. En cas d'erreur de règle (stock insuffisant, montant insuffisant) : annuler la transaction, **rien** n'a changé, le message est celui du cœur.
7. Un blocage détecté par le serveur (deadlock) : la transaction, annulée, est rejouée (trois essais en tout) ; au troisième échec, afficher l'erreur. Une coupure : voir ci-dessous. **Ordre des verrous, partout** : la ligne `caisse` d'abord, puis les produits dans l'ordre de leur clé. Les entrées et sorties de stock prennent aussi la ligne `caisse` en premier (le mouvement la référence par clé étrangère : le serveur la verrouillerait sinon après le produit, ce qui provoque des interblocages avec une vente — trouvé par les tests de concurrence). Le poste ne réessaie jamais une vente dont il n'a pas de réponse sûre : il relit la tête de sa caisse pour savoir si la pièce est entrée avant d'annoncer un échec.

Annuler une vente suit le même schéma (verrou de la caisse, des produits de la pièce et de la pièce d'origine). Ajouter, renommer, supprimer un produit, définir un prix, enregistrer une entrée ou une sortie : une transaction chacun.

## 7. Importer le fichier actuel

Une commande `stock importer <fichier>` lit un `stock.txt` (le cœur sait déjà le faire), **vérifie sa chaîne** avec `verifier_journal`, puis écrit tout dans la base dans **une seule transaction**, dans la caisse `01` : produits, mouvements, réglages, pièces avec leurs empreintes **telles quelles**. Elle refuse si la base n'est pas vide ou si la chaîne est altérée, et ne modifie jamais le fichier. Après l'import, `verifier_journal` sur la base doit donner le même résultat que sur le fichier.

## 8. L'interface graphique

- Au lancement : `base.conf` présent → connexion ; sinon mode fichier. La fenêtre unique « un seul programme à la fois sur ce fichier » ne vaut que pour le mode fichier.
- **Pas de gel** : délai de connexion 3 s, de lecture et d'écriture 5 s, message en français et en clair en cas d'échec (« Le serveur ne répond pas… »), jamais de mot de passe dans le texte.
- **« En ligne / Hors ligne »** dans l'en-tête de la vente, à partir de l'état de la dernière communication (une vérification légère toutes les 10 s, hors du fil de l'interface).
- **Quand le serveur est injoignable** : la vente en cours reste à l'écran, les boutons de paiement sont grisés avec la raison écrite, rien n'est perdu. *(Question ouverte : voir § 9.)*
- Le numéro du ticket affiche `<caisse>-<numéro sur 4 chiffres>` (`04-0186`), comme aujourd'hui avec `01`.

## 9. Ce qui reste à votre décision

**Si le serveur est injoignable au moment d'encaisser, que doit faire la caisse ?**
- **(a) Refuser de vendre** et le dire (mon choix pour cette version) : aucun risque de numéros ou de stock incohérents ; la caisse s'arrête tant que le serveur ne répond pas.
- **(b) Continuer à vendre** et synchroniser plus tard (la maquette montre « Hors ligne · 3 ventes à synchroniser ») : la caisse ne s'arrête jamais, mais il faut une file locale de ventes, une règle quand le stock du serveur n'a plus assez au moment de la synchronisation, et un travail de plus. Le choix (a) n'empêche pas de faire (b) ensuite : le journal par caisse est fait pour ça.

## 10. Découpage en plans

1. **Plan 1 — la couche de stockage** (`gestion-stock`) : trait `Depot` avec deux réalisations (fichier, MySQL), migrations, opérations transactionnelles, import, configuration, tests d'intégration. L'interface graphique ne change pas encore.
2. **Plan 2 — l'interface sur la base** (`gestion-stock-gui`) : `Etat` sur un `Depot`, copie rafraîchie, « En ligne / Hors ligne », messages d'erreur, retrait de la fenêtre unique en mode base.
3. Ensuite l'étape B de la caisse (format et tables de remises, codes-barres, Mobile money, code superviseur, utilisateurs).

## 11. Tests

- Les règles du cœur restent testées **en mémoire**, sans serveur (aucun changement).
- Les tests de la base sont **ignorés par défaut** (`#[ignore]`) et se lancent avec `KORA_TEST_URL` (une URL qui a le droit de créer une base) : chaque test crée une base jetable `kora_marche_test_<n>` et la supprime ensuite.
- Cas obligatoires : deux fils de vente sur le dernier article (un seul réussit, aucun trou de numéro) ; deux caisses qui vendent le même produit en même temps (stock juste, numéros continus dans chaque caisse) ; coupure au milieu d'une transaction (rien d'enregistré) ; chaîne vérifiée après des ventes concurrentes ; import d'un fichier à 60 000 pièces dans un temps raisonnable ; unicité `thé` / `the` ; renommage d'un produit jamais vendu ; refus de renommer un produit vendu.

## 12. Risques à connaître

- **XAMPP est configuré pour le développement** : le compte `root` n'a pas de mot de passe et le serveur écoute sur toutes les interfaces du réseau. Avant de brancher un deuxième poste, il faut mettre un mot de passe à `root` et, si un seul poste sert de serveur, limiter l'écoute à `127.0.0.1`. Ce document ne le fait pas à votre place.
- La liaison n'est pas chiffrée (TLS hors périmètre) : à réserver à un réseau local de confiance.
- Sans sauvegarde, la base devient l'unique copie des ventes : une sauvegarde régulière (`mysqldump`) sera documentée dans le README et proposée comme commande.
- Le mode fichier et le mode base ne se mélangent pas : un stock importé dans la base ne se modifie plus dans le fichier (le programme le dit).

## 13. Ce qui est laissé au constructeur

La forme exacte du trait `Depot` et la manière de construire un `Stock` partiel ; les noms de colonnes de détail et les index ; le format de `base.conf` ; la stratégie de rafraîchissement de la copie en mémoire (période, colonnes de version) ; la vérification de la chaîne au démarrage (en arrière-plan, sur la caisse locale) ; les messages d'erreur, qui suivent le ton du reste de l'application.
