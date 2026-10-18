# Gestion de stock pour petit commerçant — Plan d'implémentation

> **Pour les agents :** SOUS-COMPÉTENCE REQUISE : utiliser superpowers:subagent-driven-development (recommandé) ou superpowers:executing-plans pour exécuter ce plan tâche par tâche. Les étapes utilisent la syntaxe à cocher (`- [ ]`).

**Objectif :** Un programme en ligne de commande, en Rust, qui permet à un petit commerçant de suivre son stock (produits, entrées, sorties, alertes de stock bas, historique) avec des données gardées dans un fichier texte.

**Architecture :** Une bibliothèque (`gestion_stock`) contient toute la logique, sans aucune lecture/écriture : `Stock` (les règles métier), `stockage` (lire/écrire le fichier), `commandes` (analyser les arguments et produire le texte à afficher). Un `main.rs` très court relie le tout. Cette séparation permet de tester la logique sans lancer de programme.

**Stack technique :** Rust 1.93, édition 2024, **aucune dépendance externe**.

**Spec :** pas de document de conception séparé (l'utilisateur a choisi d'avancer sans questions détaillées). Les exigences sont dans la section « Exigences » ci-dessous ; les hypothèses sur le commerçant sont signalées comme telles.

## Exigences

- **Pour qui (hypothèse) :** un commerçant de quartier, seul, avec quelques centaines de produits. Il veut savoir à tout moment ce qu'il a en rayon, ce qui est bientôt épuisé, et ce qui est entré ou sorti.
- **Ce que l'utilisateur a demandé :** gérer le stock avec les entrées et les sorties.
- **Étape suivante annoncée par l'utilisateur : une interface graphique.** Elle réutilisera la bibliothèque `gestion_stock` et aura son propre plan, après celui-ci. Ce plan-ci se limite au terminal, mais la bibliothèque est écrite pour être réutilisable (voir les contraintes globales).
- **Hors périmètre de la V1 (idées pour plus tard) :** prix et valeur du stock, dates des mouvements, menu interactif, supprimer ou renommer un produit (le fichier est du texte, il se corrige à la main), quantités décimales (kg, litres), plusieurs utilisateurs, import/export tableur.

## Contraintes globales

- Rust 1.93, `edition = "2024"`, section `[dependencies]` **vide** pour cette version en ligne de commande (l'interface graphique ajoutera ses propres dépendances, dans son propre programme).
- Les modules `erreur`, `stock` et `stockage` n'affichent rien (pas de `println!`) et ne lisent pas le terminal : seuls `commandes` (texte pour le terminal) et `main.rs` le font. C'est ce qui permet à l'interface graphique de réutiliser le même cœur.
- Le projet vit dans `gestion-stock/` à la racine du dépôt. La bibliothèque s'appelle `gestion_stock`, le programme s'appelle `stock`.
- Fichier de données : la variable d'environnement `STOCK_FICHIER` si elle existe, sinon `stock.txt` dans le dossier courant.
- Quantités et seuils : `u32`.
- Un nom de produit est « nettoyé » (espaces au début et à la fin retirés), jamais vide, sans `;` ni saut de ligne. Deux noms sont identiques s'ils ne diffèrent que par la casse (`Riz` = `riz`). Le produit garde l'écriture de sa création.
- Tous les messages destinés à l'utilisateur sont en français.
- Pas de `unwrap()`, `expect()` ni de panique dans le code hors tests : toute erreur remonte en `ErreurStock`.
- Une commande qui échoue ne modifie jamais le fichier de données.
- La sauvegarde est atomique : on écrit dans `<fichier>.tmp` puis on renomme.
- `cargo clippy -- -D warnings` et `cargo fmt --check` passent.
- **Pédagogie :** à la fin de chaque tâche, expliquer en quelques lignes les notions de Rust utilisées, avec le chapitre de `GUIDE.md` qui les couvre, et proposer un « casse-le exprès » à l'utilisateur.

## Points de vigilance

Entrées que la spec ne détaille pas mais qu'un vrai commerçant va rencontrer, de la plus probable à la moins probable. Chacune a son test dans la tâche indiquée.

1. **Casse et espaces dans les noms** : il tape `riz 5kg` ou ` Riz 5KG ` pour le produit créé `Riz 5kg` → c'est le même produit (Tâche 1).
2. **Sortie plus grande que le stock** : refusée, le stock et l'historique ne changent pas (Tâche 1).
3. **Quantité invalide** : `0`, `-3`, `abc`, `2,5` → message clair, pas de panique (Tâches 1 et 3).
4. **Accents et caractères spéciaux** : `Café 250g` survit à une sauvegarde puis un rechargement ; un nom avec `;` est refusé car il casserait le fichier (Tâches 1 et 2).
5. **Fichier absent ou abîmé** : premier lancement = stock vide ; ligne illisible = erreur avec le numéro de ligne, pas de panique, pas d'écrasement des données (Tâches 2 et 4).

---

### Task 1 : Le cœur — `Stock`, `Produit`, `ErreurStock`

**Fichiers :**
- Créer : `gestion-stock/Cargo.toml`
- Créer : `gestion-stock/.gitignore` (contient `/target` et `stock.txt`)
- Créer : `gestion-stock/src/lib.rs` (`pub mod erreur; pub mod stock; pub mod stockage; pub mod commandes;` — les deux derniers modules sont créés vides ici pour que la bibliothèque compile, remplis aux tâches 2 et 3)
- Créer : `gestion-stock/src/main.rs` (placeholder `fn main() {}`, remplacé à la tâche 4)
- Créer : `gestion-stock/src/erreur.rs`
- Créer : `gestion-stock/src/stock.rs`
- Créer : `gestion-stock/src/stockage.rs`, `gestion-stock/src/commandes.rs` (vides)

**Interfaces :**
- Produit (utilisé par toutes les tâches suivantes) :

```rust
// erreur.rs
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErreurStock {
    NomInvalide(String),
    ProduitExistant(String),   // contient le nom déjà enregistré (écriture d'origine)
    ProduitInconnu(String),    // contient le nom tel que tapé (nettoyé)
    QuantiteNulle,
    QuantiteTropGrande,
    StockInsuffisant { nom: String, disponible: u32, demandee: u32 },
    FichierCorrompu { ligne: usize },
    Io(String),
    Utilisation(String),
}
// + impl std::fmt::Display for ErreurStock + impl std::error::Error for ErreurStock

// stock.rs
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Produit { pub nom: String, pub quantite: u32, pub seuil: u32 }
impl Produit { pub fn en_alerte(&self) -> bool }          // quantite <= seuil

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Genre { Entree, Sortie }

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mouvement { pub genre: Genre, pub nom: String, pub quantite: u32 }

#[derive(Debug, Default)]
pub struct Stock { pub(crate) produits: Vec<Produit>, pub(crate) historique: Vec<Mouvement> }
impl Stock {
    pub fn nouveau() -> Stock
    pub fn ajouter_produit(&mut self, nom: &str, seuil: u32) -> Result<(), ErreurStock>
    pub fn enregistrer_entree(&mut self, nom: &str, quantite: u32) -> Result<&Produit, ErreurStock>
    pub fn enregistrer_sortie(&mut self, nom: &str, quantite: u32) -> Result<&Produit, ErreurStock>
    pub fn produits(&self) -> Vec<&Produit>           // triés par nom, sans tenir compte de la casse
    pub fn produits_en_alerte(&self) -> Vec<&Produit> // idem, seulement ceux en alerte
    pub fn historique(&self) -> &[Mouvement]          // du plus ancien au plus récent
}
```

- [ ] **Étape 1 : Créer le projet**

Depuis la racine du dépôt : `cargo new gestion-stock --vcs none`, puis remplacer `Cargo.toml` par un paquet `gestion-stock` (édition 2024, `[dependencies]` vide) avec une section `[[bin]]` `name = "stock"`, `path = "src/main.rs"`. Ajouter `src/lib.rs`, `.gitignore` et les fichiers vides listés ci-dessus.

- [ ] **Étape 2 : Écrire les tests qui échouent** dans `stock.rs` (`#[cfg(test)] mod tests`) et `erreur.rs`

```rust
// stock.rs — aides : fn stock_avec_riz() -> Stock  (produit "Riz 5kg", seuil 5)

#[test] fn ajouter_refuse_les_noms_invalides() {
    let mut s = Stock::nouveau();
    for nom in ["", "   ", "Riz;5kg", "Riz\n5kg"] {
        assert!(matches!(s.ajouter_produit(nom, 0), Err(ErreurStock::NomInvalide(_))), "{nom:?}");
    }
}
#[test] fn ajouter_refuse_un_doublon_sans_tenir_compte_de_la_casse() {
    let mut s = Stock::nouveau();
    s.ajouter_produit("Riz", 0).unwrap();
    assert_eq!(s.ajouter_produit("  riz ", 0), Err(ErreurStock::ProduitExistant("Riz".into())));
}
#[test] fn ajouter_garde_l_ecriture_d_origine_nettoyee() {
    let mut s = Stock::nouveau();
    s.ajouter_produit("  Riz 5kg ", 5).unwrap();
    assert_eq!(s.produits()[0].nom, "Riz 5kg");
}
#[test] fn entree_retrouve_le_produit_malgre_la_casse_et_les_espaces() {
    let mut s = stock_avec_riz();
    assert_eq!(s.enregistrer_entree(" RIZ 5KG ", 12).unwrap().quantite, 12);
}
#[test] fn sortie_diminue_la_quantite() {
    let mut s = stock_avec_riz();
    s.enregistrer_entree("Riz 5kg", 12).unwrap();
    assert_eq!(s.enregistrer_sortie("Riz 5kg", 8).unwrap().quantite, 4);
}
#[test] fn sortie_trop_grande_refusee_et_rien_ne_change() {
    let mut s = stock_avec_riz();
    s.enregistrer_entree("Riz 5kg", 3).unwrap();
    assert_eq!(
        s.enregistrer_sortie("Riz 5kg", 5).unwrap_err(),
        ErreurStock::StockInsuffisant { nom: "Riz 5kg".into(), disponible: 3, demandee: 5 }
    );
    assert_eq!(s.produits()[0].quantite, 3);
    assert_eq!(s.historique().len(), 1); // seule l'entrée est enregistrée
}
#[test] fn quantite_nulle_refusee_pour_entree_et_sortie() {
    let mut s = stock_avec_riz();
    assert_eq!(s.enregistrer_entree("Riz 5kg", 0).unwrap_err(), ErreurStock::QuantiteNulle);
    assert_eq!(s.enregistrer_sortie("Riz 5kg", 0).unwrap_err(), ErreurStock::QuantiteNulle);
}
#[test] fn produit_inconnu() {
    let mut s = stock_avec_riz();
    assert_eq!(s.enregistrer_entree("Sucre", 1).unwrap_err(), ErreurStock::ProduitInconnu("Sucre".into()));
}
#[test] fn entree_qui_depasse_u32_max_refusee_sans_rien_changer() {
    let mut s = stock_avec_riz();
    s.enregistrer_entree("Riz 5kg", u32::MAX).unwrap();
    assert_eq!(s.enregistrer_entree("Riz 5kg", 1).unwrap_err(), ErreurStock::QuantiteTropGrande);
    assert_eq!(s.produits()[0].quantite, u32::MAX);
    assert_eq!(s.historique().len(), 1);
}
#[test] fn alerte_quand_quantite_inferieure_ou_egale_au_seuil() {
    let mut s = stock_avec_riz();                       // seuil 5
    s.enregistrer_entree("Riz 5kg", 6).unwrap();
    assert!(s.produits_en_alerte().is_empty());
    s.enregistrer_sortie("Riz 5kg", 1).unwrap();        // 5 <= 5
    assert_eq!(s.produits_en_alerte().len(), 1);
}
#[test] fn produits_tries_par_nom_sans_tenir_compte_de_la_casse() {
    let mut s = Stock::nouveau();
    for nom in ["citron", "Banane", "abricot"] { s.ajouter_produit(nom, 0).unwrap(); }
    let noms: Vec<&str> = s.produits().iter().map(|p| p.nom.as_str()).collect();
    assert_eq!(noms, ["abricot", "Banane", "citron"]);
}
#[test] fn historique_garde_l_ordre_et_le_nom_d_origine() {
    let mut s = stock_avec_riz();
    s.enregistrer_entree("riz 5kg", 12).unwrap();
    s.enregistrer_sortie("RIZ 5KG", 2).unwrap();
    assert_eq!(s.historique(), [
        Mouvement { genre: Genre::Entree, nom: "Riz 5kg".into(), quantite: 12 },
        Mouvement { genre: Genre::Sortie, nom: "Riz 5kg".into(), quantite: 2 },
    ]);
}

// erreur.rs
#[test] fn message_stock_insuffisant() {
    let e = ErreurStock::StockInsuffisant { nom: "Riz 5kg".into(), disponible: 3, demandee: 5 };
    assert_eq!(e.to_string(), "Stock insuffisant pour « Riz 5kg » : 3 disponible(s), 5 demandé(s).");
}
#[test] fn message_fichier_corrompu() {
    assert_eq!(
        ErreurStock::FichierCorrompu { ligne: 2 }.to_string(),
        "Fichier de données corrompu à la ligne 2."
    );
}
```

- [ ] **Étape 3 : Lancer les tests et vérifier qu'ils échouent**

Run : `cd gestion-stock && cargo test --lib`
Attendu : erreurs de compilation (`ErreurStock`, `Stock`… non définis).

- [ ] **Étape 4 : Implémenter `ErreurStock` (avec `Display` en français et `impl std::error::Error`) dans `erreur.rs`, puis `Produit`, `Genre`, `Mouvement`, `Stock` dans `stock.rs`** avec les signatures exactes de la section Interfaces

Une fonction privée `fn nettoyer_nom(nom: &str) -> Result<String, ErreurStock>` valide le nom (règles des contraintes globales). Une autre `fn position(&self, nom: &str) -> Option<usize>` cherche un produit sans tenir compte de la casse (comparaison de `to_lowercase()`). Dans `enregistrer_entree` / `enregistrer_sortie`, tout est vérifié (quantité non nulle, produit connu, `checked_add` / stock suffisant) **avant** de modifier quoi que ce soit, de sorte qu'une erreur ne laisse aucune trace ; l'historique reçoit le nom d'origine du produit.

- [ ] **Étape 5 : Lancer les tests et vérifier qu'ils passent**

Run : `cd gestion-stock && cargo test --lib`
Attendu : tous les tests PASS (12 dans `stock`, 2 dans `erreur`).

- [ ] **Étape 6 : Valider et commiter**

Run : `cd gestion-stock && cargo fmt && cargo clippy -- -D warnings`
Attendu : aucune sortie d'erreur.

```bash
git add gestion-stock docs/superpowers/plans/2026-10-18-gestion-stock.md
git commit -m "feat(stock): cœur de gestion de stock (produits, entrées, sorties, alertes)"
```

---

### Task 2 : Sauvegarde et chargement du fichier

**Fichiers :**
- Modifier : `gestion-stock/src/stockage.rs`

**Interfaces :**
- Consomme : `Stock` (champs `pub(crate) produits` / `historique`), `Produit`, `Mouvement`, `Genre`, `ErreurStock::{FichierCorrompu, Io, NomInvalide, ProduitExistant}` de la tâche 1.
- Produit :

```rust
pub fn charger(chemin: &Path) -> Result<Stock, ErreurStock>      // fichier absent => Ok(Stock vide)
pub fn sauvegarder(stock: &Stock, chemin: &Path) -> Result<(), ErreurStock>
```

**Format du fichier** (texte UTF-8, une ligne par enregistrement, champs séparés par `;`) :

```text
P;Café 250g;7;2        ← produit : nom ; quantité ; seuil
M;E;Café 250g;10       ← mouvement : E (entrée) ou S (sortie) ; nom ; quantité
M;S;Café 250g;3
```

Les produits sont écrits d'abord, puis les mouvements dans l'ordre. Les lignes vides sont ignorées au chargement.

- [ ] **Étape 1 : Écrire les tests qui échouent** (`#[cfg(test)] mod tests` dans `stockage.rs`)

```rust
// aide : fn chemin_temp(nom: &str) -> PathBuf
//   std::env::temp_dir().join(format!("gestion-stock-{}-{nom}.txt", std::process::id()))

#[test] fn aller_retour_conserve_produits_et_historique() {
    let mut s = Stock::nouveau();
    s.ajouter_produit("Café 250g", 2).unwrap();
    s.enregistrer_entree("café 250g", 10).unwrap();
    s.enregistrer_sortie("Café 250g", 3).unwrap();
    let chemin = chemin_temp("aller-retour");
    sauvegarder(&s, &chemin).unwrap();
    let relu = charger(&chemin).unwrap();
    assert_eq!(relu.produits(), s.produits());
    assert_eq!(relu.historique(), s.historique());
    std::fs::remove_file(&chemin).unwrap();
}
#[test] fn fichier_absent_donne_un_stock_vide() {
    let s = charger(&chemin_temp("absent")).unwrap();
    assert!(s.produits().is_empty() && s.historique().is_empty());
}
#[test] fn ligne_illisible_signalee_avec_son_numero() {
    let chemin = chemin_temp("illisible");
    std::fs::write(&chemin, "P;Riz;12;5\nn'importe quoi\n").unwrap();
    assert_eq!(charger(&chemin).unwrap_err(), ErreurStock::FichierCorrompu { ligne: 2 });
    std::fs::remove_file(&chemin).unwrap();
}
#[test] fn nombre_non_numerique_signale() {
    let chemin = chemin_temp("non-numerique");
    std::fs::write(&chemin, "P;Riz;beaucoup;5\n").unwrap();
    assert_eq!(charger(&chemin).unwrap_err(), ErreurStock::FichierCorrompu { ligne: 1 });
    std::fs::remove_file(&chemin).unwrap();
}
#[test] fn produit_en_double_dans_le_fichier_signale() {
    let chemin = chemin_temp("double");
    std::fs::write(&chemin, "P;Riz;1;0\nP;riz;2;0\n").unwrap();
    assert_eq!(charger(&chemin).unwrap_err(), ErreurStock::FichierCorrompu { ligne: 2 });
    std::fs::remove_file(&chemin).unwrap();
}
#[test] fn lignes_vides_ignorees() {
    let chemin = chemin_temp("vides");
    std::fs::write(&chemin, "P;Riz;1;0\n\n").unwrap();
    assert_eq!(charger(&chemin).unwrap().produits().len(), 1);
    std::fs::remove_file(&chemin).unwrap();
}
#[test] fn la_sauvegarde_ne_laisse_pas_de_fichier_temporaire() {
    let chemin = chemin_temp("tmp");
    sauvegarder(&Stock::nouveau(), &chemin).unwrap();
    assert!(!chemin.with_extension("tmp").exists());
    std::fs::remove_file(&chemin).unwrap();
}
#[test] fn erreur_d_ecriture_devient_erreur_io() {
    let chemin = std::env::temp_dir().join("dossier-qui-n-existe-pas-gestion-stock").join("stock.txt");
    assert!(matches!(sauvegarder(&Stock::nouveau(), &chemin), Err(ErreurStock::Io(_))));
}
```

- [ ] **Étape 2 : Lancer les tests et vérifier qu'ils échouent**

Run : `cd gestion-stock && cargo test --lib stockage`
Attendu : erreurs de compilation (`charger`, `sauvegarder` non définis).

- [ ] **Étape 3 : Implémenter `charger` et `sauvegarder` dans `stockage.rs`**

`charger` : `std::fs::read_to_string`, une erreur « fichier introuvable » (`ErrorKind::NotFound`) donne un stock vide, toute autre erreur d'E/S devient `ErreurStock::Io(message)`. Une ligne qui ne se découpe pas en le bon nombre de champs, avec un nombre illisible ou un genre autre que `E`/`S`, donne `FichierCorrompu { ligne }` (numérotée à partir de 1, lignes vides comprises). Pour les lignes `P`, passer par `Stock::ajouter_produit` (nom invalide ou doublon → `FichierCorrompu`) puis fixer la quantité. `sauvegarder` : construire tout le texte, l'écrire dans `chemin.with_extension("tmp")`, puis `std::fs::rename` vers `chemin`.

- [ ] **Étape 4 : Lancer les tests et vérifier qu'ils passent**

Run : `cd gestion-stock && cargo test --lib`
Attendu : tous PASS (les 14 de la tâche 1 et les 8 de `stockage`).

- [ ] **Étape 5 : Valider et commiter**

Run : `cd gestion-stock && cargo fmt && cargo clippy -- -D warnings`
Attendu : aucune sortie d'erreur.

```bash
git add gestion-stock/src/stockage.rs
git commit -m "feat(stock): sauvegarde et chargement du stock dans un fichier texte"
```

---

### Task 3 : Les commandes — analyser les arguments et produire le texte

**Fichiers :**
- Modifier : `gestion-stock/src/commandes.rs`

**Interfaces :**
- Consomme : `Stock`, `Produit::en_alerte`, `Genre`, `ErreurStock` des tâches 1 et 2.
- Produit :

```rust
#[derive(Debug, PartialEq, Eq)]
pub enum Commande {
    Ajouter { nom: String, seuil: u32 },
    Entree { nom: String, quantite: u32 },
    Sortie { nom: String, quantite: u32 },
    Lister,
    Alertes,
    Historique { limite: usize },
    Aide,
}
impl Commande { pub fn modifie_le_stock(&self) -> bool }    // vrai pour Ajouter, Entree, Sortie
pub const AIDE: &str;
pub fn analyser(args: &[String]) -> Result<Commande, ErreurStock>        // args SANS le nom du programme
pub fn executer(commande: &Commande, stock: &mut Stock) -> Result<String, ErreurStock>
```

**Syntaxe des commandes** (le nom d'un produit avec des espaces se met entre guillemets) :

```text
stock ajouter "Riz 5kg" [--seuil 5]     (seuil par défaut : 0)
stock entree  "Riz 5kg" 12
stock sortie  "Riz 5kg" 2
stock lister                            (affiche tout le stock)
stock alertes                           (seulement les produits en alerte)
stock historique [10]                   (les N derniers mouvements, 10 par défaut)
stock aide                              (aussi : aucun argument)
```

Les noms de commande sont `ajouter`, `entree`, `sortie`, `lister`, `alertes`, `historique`, `aide`.

**Textes produits par `executer`** (exacts) :

| Commande | Texte |
|---|---|
| `Ajouter` | `Produit « Riz 5kg » ajouté (seuil d'alerte : 5).` |
| `Entree` | `Entrée enregistrée : +12 Riz 5kg, stock actuel : 12.` |
| `Sortie` | `Sortie enregistrée : -2 Riz 5kg, stock actuel : 3.` et, si le produit est ensuite en alerte, une deuxième ligne `ATTENTION : stock bas pour « Riz 5kg » (seuil d'alerte : 5).` |
| `Lister` / `Alertes` | une ligne par produit : `format!("{:<20} {:>6}  seuil {:>4}{}", nom, quantite, seuil, marque)` où `marque` vaut `"  ALERTE"` si le produit est en alerte, sinon `""`. Vide : `Aucun produit enregistré.` / `Aucun produit en alerte.` |
| `Historique` | du plus récent au plus ancien, `Entrée +12 Riz 5kg` ou `Sortie -2 Riz 5kg`. Vide : `Aucun mouvement enregistré.` |
| `Aide` | `AIDE` (liste les 7 commandes) |

- [ ] **Étape 1 : Écrire les tests qui échouent** (`#[cfg(test)] mod tests` dans `commandes.rs`)

```rust
// aides : fn args(v: &[&str]) -> Vec<String> { v.iter().map(|s| s.to_string()).collect() }
//         fn stock_avec_riz() -> Stock   (produit "Riz 5kg", seuil 5)

#[test] fn analyser_entree() {
    assert_eq!(analyser(&args(&["entree", "Riz 5kg", "12"])), Ok(Commande::Entree { nom: "Riz 5kg".into(), quantite: 12 }));
}
#[test] fn analyser_sortie() {
    assert_eq!(analyser(&args(&["sortie", "Riz 5kg", "2"])), Ok(Commande::Sortie { nom: "Riz 5kg".into(), quantite: 2 }));
}
#[test] fn analyser_ajouter_avec_et_sans_seuil() {
    assert_eq!(analyser(&args(&["ajouter", "Riz", "--seuil", "5"])), Ok(Commande::Ajouter { nom: "Riz".into(), seuil: 5 }));
    assert_eq!(analyser(&args(&["ajouter", "Riz"])), Ok(Commande::Ajouter { nom: "Riz".into(), seuil: 0 }));
}
#[test] fn analyser_refuse_les_quantites_invalides() {
    for q in ["abc", "-3", "2,5", ""] {
        assert!(matches!(analyser(&args(&["entree", "Riz", q])), Err(ErreurStock::Utilisation(_))), "{q:?}");
    }
}
#[test] fn analyser_refuse_les_arguments_manquants_ou_en_trop() {
    for a in [&["entree", "Riz"][..], &["sortie"], &["ajouter"], &["ajouter", "Riz", "--seuil"], &["entree", "Riz", "1", "2"], &["lister", "x"]] {
        assert!(matches!(analyser(&args(a)), Err(ErreurStock::Utilisation(_))), "{a:?}");
    }
}
#[test] fn analyser_commandes_simples_et_aide() {
    assert_eq!(analyser(&args(&[])), Ok(Commande::Aide));
    assert_eq!(analyser(&args(&["aide"])), Ok(Commande::Aide));
    assert_eq!(analyser(&args(&["lister"])), Ok(Commande::Lister));
    assert_eq!(analyser(&args(&["alertes"])), Ok(Commande::Alertes));
    assert_eq!(analyser(&args(&["historique"])), Ok(Commande::Historique { limite: 10 }));
    assert_eq!(analyser(&args(&["historique", "3"])), Ok(Commande::Historique { limite: 3 }));
}
#[test] fn analyser_refuse_une_commande_inconnue() {
    assert!(matches!(analyser(&args(&["voler"])), Err(ErreurStock::Utilisation(_))));
}
#[test] fn modifie_le_stock_seulement_pour_ajouter_entree_sortie() {
    assert!(Commande::Entree { nom: "x".into(), quantite: 1 }.modifie_le_stock());
    assert!(!Commande::Lister.modifie_le_stock());
    assert!(!Commande::Aide.modifie_le_stock());
}
#[test] fn executer_ajouter_entree_sortie_avec_alerte() {
    let mut s = Stock::nouveau();
    let c = |v: &[&str]| analyser(&args(v)).unwrap();
    assert_eq!(executer(&c(&["ajouter", "Riz 5kg", "--seuil", "5"]), &mut s).unwrap(), "Produit « Riz 5kg » ajouté (seuil d'alerte : 5).");
    assert_eq!(executer(&c(&["entree", "Riz 5kg", "12"]), &mut s).unwrap(), "Entrée enregistrée : +12 Riz 5kg, stock actuel : 12.");
    assert_eq!(executer(&c(&["sortie", "Riz 5kg", "2"]), &mut s).unwrap(), "Sortie enregistrée : -2 Riz 5kg, stock actuel : 10.");
    assert_eq!(
        executer(&c(&["sortie", "Riz 5kg", "6"]), &mut s).unwrap(),
        "Sortie enregistrée : -6 Riz 5kg, stock actuel : 4.\nATTENTION : stock bas pour « Riz 5kg » (seuil d'alerte : 5)."
    );
}
#[test] fn executer_lister_et_alertes() {
    let mut s = stock_avec_riz();
    s.ajouter_produit("Sucre", 0).unwrap();
    s.enregistrer_entree("Riz 5kg", 4).unwrap();        // en alerte (4 <= 5)
    s.enregistrer_entree("Sucre", 10).unwrap();
    let tout = executer(&Commande::Lister, &mut s).unwrap();
    let lignes: Vec<&str> = tout.lines().collect();
    assert_eq!(lignes.len(), 2);
    assert!(lignes[0].starts_with("Riz 5kg ") && lignes[0].ends_with("seuil    5  ALERTE"));
    assert!(lignes[1].starts_with("Sucre ") && !lignes[1].contains("ALERTE"));
    let alertes = executer(&Commande::Alertes, &mut s).unwrap();
    assert_eq!(alertes.lines().count(), 1);
    assert!(alertes.starts_with("Riz 5kg"));
}
#[test] fn executer_listes_vides() {
    let mut s = Stock::nouveau();
    assert_eq!(executer(&Commande::Lister, &mut s).unwrap(), "Aucun produit enregistré.");
    assert_eq!(executer(&Commande::Alertes, &mut s).unwrap(), "Aucun produit en alerte.");
    assert_eq!(executer(&Commande::Historique { limite: 10 }, &mut s).unwrap(), "Aucun mouvement enregistré.");
}
#[test] fn executer_historique_du_plus_recent_au_plus_ancien_avec_limite() {
    let mut s = stock_avec_riz();
    s.enregistrer_entree("Riz 5kg", 12).unwrap();
    s.enregistrer_sortie("Riz 5kg", 2).unwrap();
    assert_eq!(executer(&Commande::Historique { limite: 10 }, &mut s).unwrap(), "Sortie -2 Riz 5kg\nEntrée +12 Riz 5kg");
    assert_eq!(executer(&Commande::Historique { limite: 1 }, &mut s).unwrap(), "Sortie -2 Riz 5kg");
}
#[test] fn executer_remonte_les_erreurs_du_stock() {
    let mut s = stock_avec_riz();
    assert_eq!(executer(&Commande::Sortie { nom: "Riz 5kg".into(), quantite: 1 }, &mut s).unwrap_err(),
        ErreurStock::StockInsuffisant { nom: "Riz 5kg".into(), disponible: 0, demandee: 1 });
}
#[test] fn aide_cite_toutes_les_commandes() {
    for mot in ["ajouter", "entree", "sortie", "lister", "alertes", "historique", "aide"] {
        assert!(AIDE.contains(mot), "{mot}");
    }
}
```

- [ ] **Étape 2 : Lancer les tests et vérifier qu'ils échouent**

Run : `cd gestion-stock && cargo test --lib commandes`
Attendu : erreurs de compilation (`Commande`, `analyser`, `executer`, `AIDE` non définis).

- [ ] **Étape 3 : Implémenter `Commande`, `AIDE`, `analyser`, `executer` dans `commandes.rs`**

`analyser` fait un `match` sur `args` par motifs de tranche (`[commande, reste @ ..]`). Une quantité ou un seuil se lit avec `str::parse::<u32>()`, et l'échec devient `ErreurStock::Utilisation("quantité invalide : « abc »".to_string())` (écrire un message du même genre pour le seuil, l'argument manquant, l'argument en trop et la commande inconnue, chacun disant quoi faire, par exemple « Tapez `stock aide` »). `executer` ne fait que traduire la commande en appels à `Stock` et formater le texte du tableau ci-dessus ; elle ne lit ni n'écrit aucun fichier.

- [ ] **Étape 4 : Lancer les tests et vérifier qu'ils passent**

Run : `cd gestion-stock && cargo test --lib`
Attendu : tous PASS (22 tests des tâches 1 et 2, plus les 14 de `commandes`).

- [ ] **Étape 5 : Valider et commiter**

Run : `cd gestion-stock && cargo fmt && cargo clippy -- -D warnings`
Attendu : aucune sortie d'erreur.

```bash
git add gestion-stock/src/commandes.rs
git commit -m "feat(stock): commandes ajouter, entree, sortie, lister, alertes, historique"
```

---

### Task 4 : Le programme `stock` et ses tests de bout en bout

**Fichiers :**
- Modifier : `gestion-stock/src/main.rs`
- Créer : `gestion-stock/tests/cli.rs`
- Créer : `gestion-stock/README.md`

**Interfaces :**
- Consomme : `gestion_stock::commandes::{analyser, executer, Commande, AIDE}`, `gestion_stock::stockage::{charger, sauvegarder}`, `ErreurStock` (Display).
- Produit : le programme `stock` (code de sortie `0` si tout va bien, `1` en cas d'erreur ; résultats sur la sortie standard, erreurs sur la sortie d'erreur sous la forme `Erreur : <message>`).

- [ ] **Étape 1 : Écrire les tests qui échouent** dans `gestion-stock/tests/cli.rs`

```rust
// aides :
//   fn fichier(nom: &str) -> PathBuf  (dans std::env::temp_dir(), avec std::process::id(), supprimé au début du test)
//   fn lancer(f: &Path, args: &[&str]) -> std::process::Output {
//       Command::new(env!("CARGO_BIN_EXE_stock")).args(args).env("STOCK_FICHIER", f).output().unwrap()
//   }
//   fn sortie(o: &Output) -> String   (stdout en String)

#[test] fn scenario_complet_persiste_entre_deux_lancements() {
    let f = fichier("scenario");
    assert!(lancer(&f, &["ajouter", "Riz 5kg", "--seuil", "5"]).status.success());
    assert!(lancer(&f, &["entree", "Riz 5kg", "12"]).status.success());
    let o = lancer(&f, &["sortie", "Riz 5kg", "8"]);
    assert!(o.status.success());
    assert!(sortie(&o).contains("ATTENTION : stock bas pour « Riz 5kg »"));
    let liste = sortie(&lancer(&f, &["lister"]));
    assert!(liste.contains("Riz 5kg") && liste.contains("ALERTE"));
    let histo = sortie(&lancer(&f, &["historique"]));
    assert_eq!(histo.lines().collect::<Vec<_>>(), ["Sortie -8 Riz 5kg", "Entrée +12 Riz 5kg"]);
}
#[test] fn sortie_impossible_echoue_sans_toucher_au_fichier() {
    let f = fichier("echec");
    lancer(&f, &["ajouter", "Riz", "--seuil", "1"]);
    lancer(&f, &["entree", "Riz", "3"]);
    let avant = std::fs::read_to_string(&f).unwrap();
    let o = lancer(&f, &["sortie", "Riz", "5"]);
    assert_eq!(o.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&o.stderr).contains("Stock insuffisant"));
    assert_eq!(std::fs::read_to_string(&f).unwrap(), avant);
}
#[test] fn sans_argument_affiche_l_aide_et_ne_cree_pas_de_fichier() {
    let f = fichier("aide");
    let o = lancer(&f, &[]);
    assert!(o.status.success());
    assert!(sortie(&o).contains("historique"));
    assert!(!f.exists());
}
#[test] fn commande_inconnue_echoue_avec_le_code_1() {
    let f = fichier("inconnue");
    let o = lancer(&f, &["voler"]);
    assert_eq!(o.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&o.stderr).starts_with("Erreur : "));
    assert!(!f.exists());
}
#[test] fn fichier_abime_signale_la_ligne_et_reste_intact() {
    let f = fichier("abime");
    std::fs::write(&f, "P;Riz;12;5\nn'importe quoi\n").unwrap();
    let o = lancer(&f, &["entree", "Riz", "1"]);
    assert_eq!(o.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&o.stderr).contains("ligne 2"));
    assert_eq!(std::fs::read_to_string(&f).unwrap(), "P;Riz;12;5\nn'importe quoi\n");
}
```

(Chaque test supprime son fichier temporaire à la fin.)

- [ ] **Étape 2 : Lancer les tests et vérifier qu'ils échouent**

Run : `cd gestion-stock && cargo test --test cli`
Attendu : les 5 tests FAIL (le `main` placeholder n'affiche rien et ne crée aucun fichier).

- [ ] **Étape 3 : Implémenter `main` dans `main.rs`**

`fn main()` appelle `fn lancer() -> Result<(), ErreurStock>`, affiche `Erreur : {e}` sur `stderr` en cas d'échec et termine avec `std::process::exit(1)`. `lancer` : `analyser(&args)` (arguments = `std::env::args().skip(1)`) ; si la commande est `Aide`, afficher `AIDE` et s'arrêter sans toucher au fichier ; sinon `charger` le fichier (chemin = `STOCK_FICHIER` ou `stock.txt`), `executer`, et `sauvegarder` **uniquement** si `commande.modifie_le_stock()` et que `executer` a réussi, puis afficher le texte. Le message de `FichierCorrompu` (« … à la ligne N. », fixé par le test de la tâche 1) est affiché tel quel après `Erreur : `.

- [ ] **Étape 4 : Lancer toute la suite et vérifier qu'elle passe**

Run : `cd gestion-stock && cargo test`
Attendu : tous PASS (36 tests de la bibliothèque, 5 tests `cli`).

- [ ] **Étape 5 : Écrire `README.md`** (court, en français) : à quoi sert le programme, `cargo build --release` puis le programme `target/release/stock`, la liste des commandes de la tâche 3 avec un exemple chacune, l'emplacement du fichier de données (`STOCK_FICHIER` ou `stock.txt`), et la sauvegarde conseillée (copier `stock.txt`).

- [ ] **Étape 6 : Vérification finale et commit**

Run : `cd gestion-stock && cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test`
Attendu : aucune erreur ni avertissement, tous les tests PASS.

Run (essai à la main, dans un dossier sans `stock.txt`) : `cd gestion-stock && cargo run --quiet -- ajouter "Riz 5kg" --seuil 5 && cargo run --quiet -- lister; rm -f stock.txt`
Attendu : `Produit « Riz 5kg » ajouté (seuil d'alerte : 5).` puis une ligne `Riz 5kg …  0  seuil    5  ALERTE` (`stock.txt` est ignoré par git).

```bash
git add gestion-stock
git commit -m "feat(stock): programme en ligne de commande, tests de bout en bout et README"
```
