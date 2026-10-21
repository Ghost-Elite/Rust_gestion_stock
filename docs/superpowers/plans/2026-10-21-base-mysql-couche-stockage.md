# Base MySQL — plan 1 : la couche de stockage — Plan de réalisation

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Une base MySQL (`kora_marche`, MariaDB de XAMPP) qui porte produits, stock, réglages et journal des ventes par caisse, derrière un trait `Depot` à deux réalisations (fichier, MySQL), avec import du `stock.txt` actuel, sans que l'interface graphique change.

**Architecture:** Le cœur (`Stock`) garde toutes les règles du métier. `DepotMysql` ouvre une transaction par opération, verrouille les lignes concernées, les charge dans un `Stock` **partiel**, appelle la méthode existante du cœur, puis écrit la différence. Un jeu de scénarios unique (« conformité ») tourne contre `DepotFichier` et `DepotMysql` : les deux obéissent aux mêmes règles.

**Tech Stack:** Rust 1.93, édition 2024, crate `mysql` 28 (synchrone), `sha2` déjà présent. MariaDB 10.4.28 (XAMPP).

**Spec:** `docs/superpowers/specs/2026-10-21-base-mysql-design.md` (§ 5 : le schéma ; § 6 : le déroulement d'une opération ; § 7 : l'import). Le plan 2 (l'interface graphique sur la base) viendra ensuite, séparément.

## Global Constraints

- Tout se passe dans la crate `gestion-stock` (`/Users/cherifly/rust-apprentissage/gestion-stock`) ; `gestion-stock-gui` ne doit pas cesser de compiler ni de passer ses tests (vérifier avec `cargo build` et `cargo test` dans `gestion-stock-gui` après chaque tâche qui touche `ErreurStock`).
- Dépendances : `sha2` et `mysql = "28"` (fonctionnalités par défaut, **pas de TLS**). Aucune autre.
- Compatibilité **MariaDB 10.4** : pas de `RETURNING`, pas de syntaxe propre à MySQL 8. Moteur InnoDB, `utf8mb4`.
- `max_allowed_packet` de XAMPP vaut **1 Mo** : chaque requête reste sous 512 Kio (insertions groupées par paquets de 500 lignes au plus).
- Un nom de produit est unique sans tenir compte de la casse ni des espaces autour, **accents comptés** : `nom_cle` = `nom.trim().to_lowercase()` calculé en Rust, collation `utf8mb4_bin`. Jamais de collation `_ci`.
- Pièces : l'heure reste `utc` (BIGINT) + `decalage_minutes` (SMALLINT) ; montants BIGINT signés ; quantités INT UNSIGNED. L'empreinte d'une pièce lue en base doit être identique à celle du fichier (texte canonique de `journal.rs`).
- Messages d'erreur en français, jamais de mot de passe dedans (ni dans `Debug`).
- `cargo test` (sans option) passe **sans serveur MySQL**. Les tests de la base sont `#[ignore]` et se lancent avec `KORA_TEST_URL=mysql://root@127.0.0.1:3306 cargo test -- --ignored` (le serveur est démarré par la personne, dans XAMPP).
- Messages de commit en français, terminés par `Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>`. `cargo clippy --all-targets -- -D warnings` et `cargo fmt --check` propres avant chaque commit.
- Le compte `root` et les bases `kaytegue`, `phpmyadmin`, `mysql`, `performance_schema` de XAMPP ne sont **jamais** modifiés ; les tests ne créent et ne suppriment que des bases `kora_marche_test_*`.

## Review Focus

Ce que la spec implique sans que les tests nominaux le couvrent, du plus probable au moins probable (chaque ligne a son test dans la tâche indiquée) :

1. **Coupure au milieu d'une transaction** : si la connexion tombe avant la validation, rien n'est enregistré, et le programme ne dit « échec » qu'après avoir relu la tête de sa caisse (Tâche 7).
2. **Accents et casse** : `Thé` et `the` sont deux produits, `Riz` et ` riz ` un seul ; un nom en emoji ou en caractères hors plan de base se relit à l'identique (Tâches 5 et 9).
3. **Gros journal** : 60 000 pièces s'importent dans une transaction sans dépasser le paquet de 1 Mo, et la chaîne se vérifie ensuite (Tâche 9).
4. **Le mot de passe ne fuit pas** : ni dans l'erreur de connexion, ni dans `{:?}` de la configuration (Tâches 1 et 5).
5. **Base pas à jour** : se connecter à une base vierge ou d'une version plus récente donne un message clair, pas une erreur SQL brute (Tâche 2).

---

### Task 1: Dépendance, erreur `Base` et configuration

**Files:**
- Modify: `gestion-stock/Cargo.toml`, `gestion-stock/src/lib.rs`, `gestion-stock/src/erreur.rs`
- Create: `gestion-stock/src/base/mod.rs`, `gestion-stock/src/base/configuration.rs`

**Interfaces:**
- Consumes: —
- Produces:
  - `ErreurStock::Base(String)` (message français prêt à afficher).
  - `base::configuration::Configuration { hote: String, port: u16, base: String, utilisateur: String, mot_de_passe: String, caisse: String }` ; `Debug` écrit `mot_de_passe: "***"`.
  - `Configuration::depuis_texte(texte: &str) -> Result<Configuration, ErreurStock>` — lignes `cle=valeur`, `#` commence un commentaire ; clés `hote` (défaut `127.0.0.1`), `port` (défaut 3306), `base` (défaut `kora_marche`), `utilisateur` (obligatoire), `mot_de_passe` (obligatoire), `caisse` (obligatoire, exactement deux chiffres) ; une clé inconnue ou répétée est refusée avec son numéro de ligne.
  - `Configuration::chemin_par_defaut() -> Option<PathBuf>` : `KORA_BASE_CONF`, sinon `$HOME/.config/kora-marche/base.conf`.
  - `Configuration::charger(chemin: &Path) -> Result<Option<Configuration>, ErreurStock>` : `None` si le fichier n'existe pas.
  - `Configuration::lisible_par_d_autres(chemin: &Path) -> bool` : sous Unix, vrai si les droits dépassent `0o600`.
  - `Configuration::options(&self) -> mysql::Opts` : délai de connexion 3 s, lecture et écriture 5 s.
  - `base::erreur_de_base(e: &mysql::Error, config: &Configuration) -> ErreurStock` : traduit en français (serveur injoignable, accès refusé, base inconnue, autre) sans jamais citer le mot de passe.

- [ ] **Step 1: Écrire les tests (rouges)** dans `configuration.rs` (`#[cfg(test)]`) : fichier complet lu ; défauts appliqués ; `caisse=1` et `caisse=ab` refusées ; clé inconnue, clé répétée et ligne sans `=` refusées avec le numéro de ligne ; commentaires et lignes vides ignorés ; `format!("{config:?}")` contient `***` et pas le mot de passe ; `charger` d'un chemin absent donne `Ok(None)` ; `lisible_par_d_autres` est faux pour `0o600`, vrai pour `0o644` (fichier temporaire, `#[cfg(unix)]`).
- [ ] **Step 2: Lancer** `cargo test configuration` — Expected : ne compile pas (module absent).
- [ ] **Step 3: Implémenter** : ajouter `mysql = "28"` aux dépendances ; `pub mod base;` dans `lib.rs` ; la variante `Base(String)` (affichage = le message tel quel) ; les fonctions ci-dessus. `erreur_de_base` : test unitaire sur un `mysql::Error` fabriqué à la main (accès refusé, injoignable) qui vérifie que le texte nomme l'hôte et le port mais pas le mot de passe `"secret"` placé dans la configuration.
- [ ] **Step 4: Lancer** `cargo test` — Expected : tout passe ; puis `cd ../gestion-stock-gui && cargo build` — Expected : compile (si un `match` sur `ErreurStock` devient non exhaustif, ajouter la branche `Base` qui affiche le message).
- [ ] **Step 5: Commit** — `feat(base): configuration de la base et erreur « Base »`.

### Task 2: Le schéma et les migrations

**Files:**
- Create: `gestion-stock/migrations/001_schema.sql`, `gestion-stock/src/base/migrations.rs`, `gestion-stock/base/creer_base.sql`, `gestion-stock/tests/commun/mod.rs`, `gestion-stock/tests/commun/base_jetable.rs`
- Modify: `gestion-stock/src/base/mod.rs`

**Interfaces:**
- Consumes: `Configuration`, `erreur_de_base` (tâche 1).
- Produces:
  - Les tables du spec § 5, plus deux colonnes de recherche ajoutées par ce plan : `piece_ligne.nom_cle` et `mouvement.produit_cle` (nom en minuscules, `utf8mb4_bin`, indexées), utiles pour « ce produit a-t-il été vendu ? » et pour renommer l'historique.
  - `base::migrations::VERSION_COURANTE: u32` (= 1) ; `base::migrer(config: &Configuration) -> Result<u32, ErreurStock>` : crée `schema_version` si besoin, applique les migrations manquantes dans l'ordre, une transaction par migration (DDL implicite en MariaDB : on vérifie `schema_version` avant), renvoie la version atteinte, **idempotente** ; refuse (« la base est plus récente que ce programme ») si la version dépasse la courante.
  - `base::version(config: &Configuration) -> Result<Option<u32>, ErreurStock>` : `None` si la base n'a pas de `schema_version`.
  - `creer_base.sql` : `CREATE DATABASE IF NOT EXISTS kora_marche CHARACTER SET utf8mb4`, `CREATE USER IF NOT EXISTS 'kora'@'localhost' IDENTIFIED BY '<à remplacer>'`, `GRANT ALL PRIVILEGES ON kora_marche.* TO 'kora'@'localhost'` — le fichier dit en tête de remplacer le mot de passe et qu'il se lance **une fois**, avec `root`, par la personne.
  - Test : `tests/commun/base_jetable.rs` : `BaseJetable::creer() -> Option<BaseJetable>` (`None` si `KORA_TEST_URL` absente → le test s'ignore avec un message), qui crée `kora_marche_test_<pid>_<compteur>`, applique `migrer`, expose `config() -> Configuration` (caisse `01`) et supprime la base dans `Drop`.

- [ ] **Step 1: Prérequis humain.** Demander à la personne de **démarrer MySQL dans le gestionnaire XAMPP** (l'assistant n'a pas `sudo`) puis vérifier : `/Applications/XAMPP/xamppfiles/bin/mysql -u root -e "select version()"`. Ne rien créer d'autre à la main.
- [ ] **Step 2: Écrire les tests (rouges)** `tests/migrations.rs` (`#[ignore]`) : base vierge → `version` = `None` ; après `migrer`, `Some(1)` et les sept tables existent (`information_schema`) ; deuxième `migrer` ne change rien ; une base dont `schema_version` contient 99 est refusée avec le message « plus récente » ; insérer deux produits `Thé` et `the` réussit, `Riz` et `riz` échoue (clé unique sur `nom_cle`) ; `piece` refuse deux fois `(caisse, numero)` ; `annule_numero` unique par caisse.
- [ ] **Step 3: Lancer** `KORA_TEST_URL=mysql://root@127.0.0.1:3306 cargo test --test migrations -- --ignored` — Expected : échec (fonctions absentes).
- [ ] **Step 4: Implémenter** le SQL (`include_str!` pour l'embarquer), `migrer`, `version`, `BaseJetable`.
- [ ] **Step 5: Lancer** le même test — Expected : passe ; puis `cargo test` — Expected : passe sans serveur (les tests de base sont ignorés).
- [ ] **Step 6: Commit** — `feat(base): schéma version 1 et migrations`.

### Task 3: Le cœur sur un `Stock` partiel

**Files:**
- Modify: `gestion-stock/src/stock.rs` (constructeur), `gestion-stock/src/vente.rs` (tests)
- Test: dans les modules `tests` de `stock.rs` et `vente.rs`

**Interfaces:**
- Consumes: `Stock` (champs `pub(crate)`), `Produit`, `Piece`, `Reglages`.
- Produces: `Stock::partiel(produits: Vec<Produit>, reglages: Reglages, pieces: Vec<Piece>) -> Stock` (`pub(crate)`, historique vide) ; et la garantie, par tests, que les règles existantes donnent le même résultat sur un stock partiel :
  - `encaisser` sur un stock dont `pieces` ne contient que **la dernière pièce** donne `numero = dernier + 1` et `hash_precedent = hash de la dernière` ;
  - `annuler(n)` sur un stock dont `pieces` contient la pièce `n`, éventuellement son annulation, et la dernière pièce donne la même pièce qu'un stock complet, et refuse `DejaAnnulee` si l'annulation y figure ;
  - `renommer_produit` et `supprimer_produit` donnent `ProduitVendu` si `pieces` contient **une seule** pièce dont une ligne porte ce nom (casse ignorée) ;
  - `definir_reglages` donne `ReglagesVerrouilles` dès que `pieces` contient une pièce quelconque.

- [ ] **Step 1: Écrire les tests (rouges)** décrits ci-dessus (un test par garantie, comparant au résultat d'un `Stock` complet construit par les mêmes appels).
- [ ] **Step 2: Lancer** `cargo test partiel` — Expected : ne compile pas (`Stock::partiel` absent).
- [ ] **Step 3: Implémenter** `Stock::partiel`. Si un test montre qu'une règle lit autre chose que ce que le stock partiel contient, corriger **le test ou le constructeur**, pas la règle : la règle du cœur ne change pas dans ce plan.
- [ ] **Step 4: Lancer** `cargo test` (tout) — Expected : passe, y compris les 105 tests existants.
- [ ] **Step 5: Commit** — `feat(stock): Stock partiel pour les transactions de la base`.

### Task 4: Le trait `Depot`, `DepotFichier` et la suite de conformité

**Files:**
- Create: `gestion-stock/src/depot.rs`, `gestion-stock/tests/commun/conformite.rs`, `gestion-stock/tests/depot_fichier.rs`
- Modify: `gestion-stock/src/lib.rs`, `gestion-stock/tests/commun/mod.rs`

**Interfaces:**
- Consumes: `Stock`, `stockage::{charger, sauvegarder}`.
- Produces (`pub trait Depot`, toutes les méthodes prennent `&mut self` et renvoient des valeurs possédées) :
  - `instantane(&mut self) -> Result<Stock, ErreurStock>` — copie complète (réglages, produits, mouvements, pièces de la caisse du dépôt) pour les écrans de lecture ;
  - `ajouter_produit(&mut self, nom: &str, seuil: u32) -> Result<(), ErreurStock>`
  - `definir_prix(&mut self, nom: &str, prix: Option<Montant>, taux_bp: Option<u32>) -> Result<(), ErreurStock>`
  - `definir_reglages(&mut self, reglages: Reglages) -> Result<(), ErreurStock>`
  - `enregistrer_entree(&mut self, nom: &str, quantite: u32) -> Result<Produit, ErreurStock>` et `enregistrer_sortie(...)` idem
  - `renommer_produit(&mut self, ancien: &str, nouveau: &str) -> Result<(), ErreurStock>` ; `supprimer_produit(&mut self, nom: &str) -> Result<(), ErreurStock>`
  - `encaisser(&mut self, demande: &[(String, u32)], moyen: Moyen, recu: Montant, quand: Horodatage) -> Result<Piece, ErreurStock>`
  - `annuler(&mut self, numero: u64, quand: Horodatage) -> Result<Piece, ErreurStock>`
  - `verifier_journal(&mut self) -> Result<(), ErreurStock>`
- `DepotFichier::ouvrir(chemin: &Path) -> Result<DepotFichier, ErreurStock>` : chaque opération s'applique à une **copie** du stock, qui n'est sauvegardée puis adoptée que si tout réussit (comportement actuel : rien ne change en cas d'erreur).
- `tests/commun/conformite.rs` : `pub fn parcourir_les_regles(depot: &mut dyn Depot)` — le scénario unique : produit ajouté et doublon refusé (casse, espaces), `Thé` ≠ `the`, nom invalide (`;`, vide), entrée et sortie, sortie au-delà du stock refusée sans rien changer, prix et taux (valeurs refusées), vente en espèces / carte, vente refusée en entier si un article manque (le numéro du ticket suivant ne saute pas), numérotation 1, 2, 3 sans trou, annulation totale, double annulation refusée, annulation d'une annulation refusée, renommage refusé d'un produit vendu et permis d'un produit jamais vendu (l'historique prend le nouveau nom), suppression, réglages verrouillés après une vente (le taux par défaut reste modifiable), `verifier_journal` vrai après toutes ces opérations, `instantane` cohérent avec ce que chaque opération a renvoyé.

- [ ] **Step 1: Écrire** `parcourir_les_regles` et `tests/depot_fichier.rs` qui l'appelle sur un `DepotFichier` temporaire.
- [ ] **Step 2: Lancer** `cargo test --test depot_fichier` — Expected : ne compile pas (`Depot`, `DepotFichier` absents).
- [ ] **Step 3: Implémenter** le trait et `DepotFichier` (délégation au cœur, sauvegarde atomique existante).
- [ ] **Step 4: Lancer** `cargo test` — Expected : passe.
- [ ] **Step 5: Commit** — `feat(depot): trait Depot, dépôt fichier et suite de conformité`.

### Task 5: `DepotMysql` — connexion, instantané, produits et réglages

**Files:**
- Create: `gestion-stock/src/base/depot_mysql.rs`, `gestion-stock/tests/depot_mysql.rs`
- Modify: `gestion-stock/src/base/mod.rs`

**Interfaces:**
- Consumes: `Configuration`, `migrer`, `version`, `erreur_de_base`, `Stock::partiel`, `Depot`.
- Produces:
  - `DepotMysql::connecter(config: &Configuration) -> Result<DepotMysql, ErreurStock>` : vérifie que la version du schéma est la courante, sinon `Base("La base n'est pas à jour : lancez « stock migrer ».")` (ou « pas encore créée ») ; vérifie que la ligne `caisse` du poste existe et la crée avec `dernier_numero = 0` et `dernier_hash` = 64 zéros si elle manque.
  - `impl Depot for DepotMysql` : cette tâche réalise `instantane`, `ajouter_produit`, `definir_prix`, `definir_reglages` ; les autres méthodes renvoient provisoirement `Err(ErreurStock::Base("pas encore réalisé".into()))` (tâches 6 et 7).
  - Chaque opération : `START TRANSACTION`, verrous `SELECT … FOR UPDATE` (produits dans l'ordre de `nom_cle`), appel du cœur sur `Stock::partiel`, écriture de la différence, `COMMIT` ; toute erreur de règle fait `ROLLBACK` et renvoie l'erreur du cœur **inchangée**.
  - Détails de comportement : l'unicité est vérifiée par le cœur **et** garantie par l'index unique (une violation de clé → `ProduitExistant` avec le nom déjà en base) ; les réglages absents de la table valent `Reglages::default()` ; `definir_reglages` charge **une** pièce quelconque de la base pour que le cœur verrouille devise et profil.

- [ ] **Step 1: Écrire les tests (rouges)** `tests/depot_mysql.rs` (`#[ignore]`) : `connecter` sur une base vierge (sans migration) → message « pas encore créée » ; sur une base migrée → `Ok` ; la ligne `caisse` `01` existe ; `connecter` avec un mauvais mot de passe → erreur qui **ne contient pas** ce mot de passe ; produit ajouté puis relu par `instantane` ; `Thé` et `the` coexistent, `Riz` puis ` riz ` → `ProduitExistant` ; produit au nom en emoji (`"Pain 🥖"`) relu à l'identique ; prix et taux relus ; réglages relus ; une règle refusée ne laisse aucune ligne (compter avant/après).
- [ ] **Step 2: Lancer** `KORA_TEST_URL=mysql://root@127.0.0.1:3306 cargo test --test depot_mysql -- --ignored` — Expected : échec (type absent).
- [ ] **Step 3: Implémenter** `DepotMysql` (une connexion `mysql::Conn`, reconnexion automatique d'une fois au début d'une opération si la connexion est tombée).
- [ ] **Step 4: Lancer** le même test — Expected : passe ; `cargo test` — Expected : passe sans serveur.
- [ ] **Step 5: Commit** — `feat(base): DepotMysql, connexion, produits et réglages`.

### Task 6: `DepotMysql` — entrées, sorties, renommer, supprimer

**Files:**
- Modify: `gestion-stock/src/base/depot_mysql.rs`, `gestion-stock/tests/depot_mysql.rs`

**Interfaces:**
- Consumes: tâche 5.
- Produces: `enregistrer_entree`, `enregistrer_sortie`, `renommer_produit`, `supprimer_produit` réalisés.
  - Entrée et sortie : un `mouvement` (nom en texte, `produit_cle`, `caisse`, `piece_numero` NULL) par opération.
  - Renommer : le produit, **et** `UPDATE mouvement SET produit_nom = nouveau WHERE produit_cle = ancienne_cle` (puis `produit_cle` du nouveau nom) ; refus `ProduitVendu` si une `piece_ligne` porte la même `nom_cle` (une pièce témoin est chargée dans le `Stock` partiel pour que le cœur décide).
  - Supprimer : le produit disparaît, les mouvements restent.

- [ ] **Step 1: Écrire les tests (rouges)** : entrée puis sortie, stock juste ; sortie au-delà du stock → `StockInsuffisant` et la quantité en base n'a pas bougé ; renommage d'un produit jamais vendu : `instantane().historique()` porte le nouveau nom ; renommage de `Riz` en `riz` (même produit, casse) permis ; renommage vers un nom pris → `ProduitExistant` ; produit vendu → `ProduitVendu` (le test insère une pièce avec la ligne SQL directement, la vente par `DepotMysql` n'existe pas encore) ; suppression garde l'historique.
- [ ] **Step 2: Lancer** `--test depot_mysql -- --ignored` — Expected : échec (« pas encore réalisé »).
- [ ] **Step 3: Implémenter.**
- [ ] **Step 4: Lancer** — Expected : passe ; `cargo test` — Expected : passe.
- [ ] **Step 5: Commit** — `feat(base): entrées, sorties, renommage et suppression en base`.

### Task 7: `DepotMysql` — encaisser, annuler, vérifier le journal

**Files:**
- Modify: `gestion-stock/src/base/depot_mysql.rs`, `gestion-stock/tests/depot_mysql.rs`
- Create: `gestion-stock/tests/commun/pannes.rs` (aide de test : tuer une connexion depuis une seconde)

**Interfaces:**
- Consumes: tâches 5 et 6.
- Produces: `encaisser`, `annuler`, `verifier_journal` réalisés, et `instantane` qui relit les pièces de la caisse avec leurs lignes.
  - `encaisser` : verrous dans l'ordre (ligne `caisse`, puis produits par `nom_cle`) ; `Stock::partiel` avec la **dernière pièce** de la caisse comme seule pièce ; écriture des quantités, des mouvements (`piece_numero` rempli), de la pièce et de ses lignes (`rang` à partir de 0, `nom_cle` rempli), puis de la tête (`dernier_numero`, `dernier_hash`).
  - `annuler(n)` : verrous de la caisse, de la pièce `n`, de son éventuelle annulation, des produits de la pièce ; `annule_numero` renseigné.
  - `verifier_journal` : relit **toute** la chaîne de la caisse par ordre de numéro (lecture en flux, pas tout en mémoire), recalcule chaque empreinte avec le code de `journal.rs`, compare aussi la tête de `caisse` ; `JournalAltere { numero }` à la première pièce qui ne correspond plus.
  - Reprise après coupure : sur une erreur de communication pendant ou après `COMMIT`, l'opération se reconnecte et **relit la tête de la caisse** : si la pièce attendue est entrée, elle est renvoyée comme un succès ; sinon l'erreur « Le serveur ne répond pas » est renvoyée. Jamais de seconde tentative d'écriture.

- [ ] **Step 1: Écrire les tests (rouges)** : `DepotMysql` passe `parcourir_les_regles` (appelée depuis `tests/depot_mysql.rs`) ; l'empreinte d'une pièce vendue par la base est égale à celle qu'un `Stock` en mémoire donne pour les mêmes opérations ; annulation : stock remis, `annule_numero` renseigné, deuxième annulation refusée ; **coupure** : avec l'aide `pannes`, tuer la connexion juste avant la validation (point d'accroche de test) → `encaisser` renvoie une erreur `Base`, **aucune** ligne n'a été écrite (quantités, mouvements, pièce, tête inchangés), et l'opération suivante réussit avec le numéro attendu (pas de trou) ; altérer une ligne `piece_ligne` par SQL → `verifier_journal` donne `JournalAltere` avec le bon numéro.
- [ ] **Step 2: Lancer** — Expected : échec.
- [ ] **Step 3: Implémenter** (le point d'accroche de test est un champ `#[cfg(test)]`/`pub(crate)` rappelé juste avant `COMMIT`).
- [ ] **Step 4: Lancer** `--test depot_mysql -- --ignored` — Expected : tout passe, y compris la suite de conformité ; `cargo test` — Expected : passe.
- [ ] **Step 5: Commit** — `feat(base): encaisser, annuler et vérifier le journal en base`.

### Task 8: La concurrence

**Files:**
- Create: `gestion-stock/tests/concurrence.rs` (`#[ignore]`)

**Interfaces:**
- Consumes: `DepotMysql` complet, `BaseJetable`.
- Produces: rien de nouveau dans le code de production ; ce test **prouve** la promesse du spec. Si un test échoue, corriger l'ordre des verrous ou la transaction dans `depot_mysql.rs`, pas le test.

- [ ] **Step 1: Écrire les tests** (chaque fil a sa propre connexion) : (1) 8 fils vendent chacun 1 unité du **dernier** exemplaire d'un produit dans la même caisse : exactement 1 réussit, 7 reçoivent `StockInsuffisant`, la quantité finale est 0, les numéros de pièces vont de 1 sans trou ; (2) deux caisses (`01` et `04`) vendent chacune 50 fois le même produit (stock 100) en parallèle : stock final 0, numéros 1 à 50 dans chaque caisse, `verifier_journal` vrai dans les deux ; (3) deux fils vendent des paniers qui contiennent les mêmes deux produits dans des ordres différents (pas d'interblocage : tout se termine en moins de 10 s) ; (4) entrées et ventes concurrentes sur le même produit : quantité finale = départ + entrées − ventes.
- [ ] **Step 2: Lancer** `KORA_TEST_URL=… cargo test --test concurrence -- --ignored` — Expected : passe (ces tests ne sont pas rouges au départ : ils vérifient le travail des tâches 5 à 7 ; s'ils échouent, ce sont des défauts à corriger).
- [ ] **Step 3: Répéter** 5 fois de suite (`for i in 1..5`) — Expected : toujours vert (un test de concurrence qui passe « parfois » est un défaut).
- [ ] **Step 4: Commit** — `test(base): concurrence de plusieurs fils et de deux caisses`.

### Task 9: Importer un `stock.txt`, et les commandes `migrer`, `importer`, `verifier`

**Files:**
- Create: `gestion-stock/src/base/import.rs`, `gestion-stock/tests/import_mysql.rs`
- Modify: `gestion-stock/src/base/mod.rs`, `gestion-stock/src/commandes.rs`, `gestion-stock/src/main.rs`, `gestion-stock/README.md`

**Interfaces:**
- Consumes: `charger`, `Stock::verifier_journal`, `DepotMysql`, `migrer`.
- Produces:
  - `base::importer(config: &Configuration, fichier: &Path) -> Result<ResumeImport, ErreurStock>` avec `ResumeImport { produits: usize, mouvements: usize, pieces: usize }` : lit le fichier, **vérifie sa chaîne** (sinon `JournalAltere`), refuse si la base contient déjà un produit, un mouvement ou une pièce, puis écrit tout **dans une seule transaction** : réglages, produits, mouvements (par paquets de 500 lignes au plus par requête), pièces et lignes avec leurs empreintes **telles quelles** dans la caisse de la configuration ; compare ensuite `verifier_journal` en base avec celui du fichier. Ne modifie jamais le fichier.
  - Commandes (`Commande::Migrer`, `Commande::Importer { fichier: String }`, `Commande::VerifierBase`) : `stock migrer`, `stock importer <fichier>`, `stock verifier` — elles lisent `Configuration::charger(chemin_par_defaut())` et disent clairement « Aucune base configurée : créez `…/base.conf` » si le fichier manque. Les commandes existantes (mode fichier) ne changent pas.
  - README : section « La base MySQL (XAMPP) » (créer la base avec `creer_base.sql`, écrire `base.conf` avec droits `600`, `stock migrer`, `stock importer`, sauvegarde `mysqldump`), et correction de la phrase « Tout est écrit… la seule bibliothèque externe est `sha2` ».

- [ ] **Step 1: Écrire les tests (rouges)** `tests/import_mysql.rs` (`#[ignore]`) : un `stock.txt` construit par `DepotFichier` (produits, ventes, une annulation, un produit au nom `"Pain 🥖"`) s'importe ; `instantane()` du `DepotMysql` donne les mêmes produits, mouvements et pièces (mêmes empreintes) ; `verifier_journal` est vrai ; un deuxième import est refusé (« la base n'est pas vide ») ; un fichier à chaîne altérée est refusé **avant** toute écriture (la base reste vide) ; un fichier de **60 000 pièces** (généré par le test, une ligne chacune) s'importe sans erreur de paquet et en moins de 60 s, puis `verifier_journal` est vrai. Dans `commandes.rs` (tests sans serveur) : `analyser` reconnaît `migrer`, `importer fichier.txt`, `verifier` et refuse `importer` sans fichier avec le message d'utilisation habituel.
- [ ] **Step 2: Lancer** `--test import_mysql -- --ignored` et `cargo test commandes` — Expected : échec (fonctions et commandes absentes).
- [ ] **Step 3: Implémenter** `importer`, les trois commandes et `main.rs` (ces trois commandes ne touchent pas à `stock.txt` : elles n'appellent pas `charger` du mode fichier, sauf `importer` pour son fichier source).
- [ ] **Step 4: Lancer** les tests de la base et `cargo test` — Expected : passent. Puis, **avec l'accord de la personne**, créer la vraie base : `mysql -u root < creer_base.sql` (mot de passe remplacé), écrire `~/.config/kora-marche/base.conf` (`chmod 600`), `cargo run --bin stock -- migrer` — Expected : « version 1 ».
- [ ] **Step 5: Commit** — `feat(base): import du stock.txt, commandes migrer, importer et verifier`.

### Task 10: Vérification finale et mise à jour de la documentation

**Files:**
- Modify: `docs/superpowers/specs/2026-10-21-base-mysql-design.md` (section « Avancement »), `docs/superpowers/specs/2026-10-21-kora-marche-pos-design.md`

**Interfaces:**
- Consumes: tout ce qui précède.
- Produces: la preuve que le plan est terminé.

- [ ] **Step 1: Lancer** dans `gestion-stock` : `cargo test`, `KORA_TEST_URL=… cargo test -- --ignored`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check` — Expected : tout vert ; puis dans `gestion-stock-gui` : `cargo test` et `cargo clippy --all-targets -- -D warnings` — Expected : 211 tests verts, aucun avertissement.
- [ ] **Step 2: Vérifier à la main** : `stock importer gestion-stock/stock.txt` sur la vraie base (avec l'accord de la personne), puis `stock verifier` — Expected : le journal est cohérent ; le fichier `stock.txt` n'a pas changé (même empreinte `shasum` avant et après).
- [ ] **Step 3: Mettre à jour** les deux documents (le plan 1 est fait, le plan 2 reste) et commiter — `docs: couche de stockage MySQL terminée (plan 1)`.
