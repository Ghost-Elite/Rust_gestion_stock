# Interface graphique de gestion de stock — Plan d'implémentation

> **Pour les agents :** SOUS-COMPÉTENCE REQUISE : utiliser superpowers:subagent-driven-development (recommandé) ou superpowers:executing-plans pour exécuter ce plan tâche par tâche. Les étapes utilisent la syntaxe à cocher (`- [ ]`).

**Objectif :** Une fenêtre sobre et accessible pour gérer le stock d'un petit commerçant (voir, chercher, ajouter, entrées, sorties, alertes, renommer, supprimer, historique), qui réutilise la bibliothèque `gestion_stock` du programme en ligne de commande.

**Architecture :** Un nouveau programme `gestion-stock-gui/`, voisin de `gestion-stock/` (dépendance par chemin), découpé en quatre morceaux : `palette` (couleurs + calcul du contraste), `etat` (toute la logique, sans aucune fenêtre, donc testable), `ui` (le dessin avec egui, très fin) et `main` (ouvre la fenêtre). La bibliothèque apprend d'abord à renommer et à supprimer un produit (Task 1).

**Stack technique :** Rust 1.93, édition 2024, `eframe`/`egui` **0.35** (les versions 0.36 et 0.37 exigent Rust 1.95, que cette machine n'a pas), `egui_kittest` 0.35 pour tester l'interface via l'API d'accessibilité.

**Spec :** pas de document de conception séparé (l'utilisateur a choisi d'avancer sans questions : « quelque chose de modéré qui respecte les normes européennes et américaines »). Les exigences sont dans la section « Exigences » ci-dessous.

## Exigences

- **Ce que l'utilisateur a demandé :** une interface graphique, modérée, qui respecte les normes européennes et américaines.
- **Mon interprétation (hypothèse, à corriger si elle est fausse) :**
  1. **Accessibilité** : viser WCAG 2.1 niveau AA, la référence de la norme européenne EN 301 549 (et de l'European Accessibility Act) et de la Section 508 / ADA aux États-Unis. Ni audit ni certification : le README publie les limites connues.
  2. **Vie privée** : aucune donnée personnelle, aucune connexion réseau, aucune télémétrie. Tout reste dans un fichier local (compatible RGPD / CCPA par conception).
  3. **Licences** : aucune dépendance sous licence GPL ou AGPL, vérifié et listé.
  4. **« Modéré »** : une seule fenêtre sobre, un seul thème clair, les mêmes fonctions que le terminal plus renommer et supprimer (un commerçant qui se trompe de nom ne peut pas éditer un fichier texte).
- **Fonctions :**
  - Liste des produits (nom, quantité, seuil, état), triée par nom, avec recherche et case « seulement les alertes ».
  - Formulaire d'ajout (nom, seuil facultatif).
  - Sur le produit sélectionné : entrée, sortie, renommer, supprimer (avec confirmation).
  - Historique des 20 derniers mouvements, du plus récent au plus ancien.
  - Pied de page : message du dernier résultat et chemin du fichier de données.
- **Critères d'accessibilité visés et comment ils sont vérifiés :**

| Critère WCAG 2.1 | Comment il est tenu | Vérification |
|---|---|---|
| 1.4.3 Contraste (texte ≥ 4,5:1) | palette à valeurs fixes | test `palette` (Task 2) |
| 1.4.11 Contraste des composants (≥ 3:1) | bordures, focus | test `palette` (Task 2) |
| 1.4.1 Couleur seule | l'alerte est écrite : « ALERTE : stock bas » | test `ui` (Task 4) |
| 1.4.4 Redimensionnement du texte | zoom clavier d'egui (Ctrl/Cmd + et −) | test `ui` + essai manuel |
| 2.1.1 Clavier | tout se fait avec Tab, Espace, Entrée | essai manuel |
| 3.3.1 / 3.3.3 Erreurs | « Erreur : … » en français, avec la façon de corriger | tests `etat` (Task 3) |
| 3.3.2 Étiquettes | chaque champ relié à son libellé (`labelled_by`) | tests `ui` (Task 4) |
| 4.1.2 Nom, rôle, valeur | noms accessibles via AccessKit | tests `ui` + essai VoiceOver |
| 4.1.3 Messages d'état | **non couvert** : egui 0.35 n'a pas de région « live » | documenté dans le README |

- **Hors périmètre :** autres langues (tous les textes sont dans un seul fichier pour que ce soit facile plus tard), thème sombre, impression ou export, usage simultané du terminal et de la fenêtre sur le même fichier, prix et dates, installeur ou signature d'application, audit d'accessibilité par un tiers.

## Contraintes globales

- Rust 1.93, `edition = "2024"`. Dépendances du programme graphique : `gestion-stock = { path = "../gestion-stock" }` et `eframe = "0.35"` (fonctionnalités par défaut) ; en développement seulement : `egui_kittest = "0.35"`. Rien d'autre.
- La bibliothèque `gestion-stock` reste sans dépendance et n'est modifiée que par la Task 1.
- Aucune connexion réseau dans le programme (voir la vérification de la Task 5).
- Aucune licence GPL ni AGPL parmi les dépendances (voir la Task 5).
- Tous les textes visibles sont en français et définis dans `src/textes.rs`.
- Pas de `unwrap()`, `expect()` ni de panique dans le code hors tests.
- Une action qui échoue ne change ni le fichier de données ni la liste affichée : seul le message change.
- Fichier de données par défaut, **jamais le dossier courant** (un double-clic lance le programme depuis un dossier imprévisible) : macOS `~/Library/Application Support/GestionStock/stock.txt` ; Windows `%APPDATA%\GestionStock\stock.txt` ; Linux `$XDG_DATA_HOME/gestion-stock/stock.txt`, sinon `~/.local/share/gestion-stock/stock.txt`. La variable `STOCK_FICHIER` (si non vide) a la priorité, comme dans le programme en ligne de commande. Le chemin utilisé est toujours affiché dans la fenêtre.
- `cargo clippy --all-targets -- -D warnings` et `cargo fmt --check` passent dans les deux dossiers.
- Les deux programmes lisent et écrivent le même format de fichier (celui de `gestion-stock/src/stockage.rs`).
- **Branche :** créer `feature/interface-graphique` à partir de `feature/gestion-stock` (qui n'est pas encore fusionnée). La fusion des deux branches est une décision de l'utilisateur, à la fin.
- **Pédagogie :** à la fin de chaque tâche, expliquer en quelques lignes les notions de Rust ou d'egui utilisées, avec le chapitre de `GUIDE.md` qui couvre ce qui existe déjà, et proposer un « casse-le exprès ».
- **Premier build long :** compiler `eframe` prend plusieurs minutes la première fois ; lancer les commandes `cargo` avec un délai d'au moins 10 minutes.

## Points de vigilance

Ce que la spec ne détaille pas mais qu'un vrai commerçant va rencontrer, du plus probable au moins probable. Chacun a son test dans la tâche indiquée.

1. **Lancement par double-clic et fichier introuvable ou abîmé** : le chemin par défaut est stable et affiché ; un fichier abîmé donne un écran d'erreur lisible et n'est jamais écrasé (Tasks 3, 4 et 5).
2. **Échec de sauvegarde** (disque plein, dossier verrouillé) : la liste affichée ne doit jamais montrer un stock différent de celui du fichier (Task 3).
3. **Champs vides ou invalides** : quantité vide, `0`, `-3`, `abc`, `2,5`, trop grande ; nom vide ou avec `;` → message clair en français, aucune panique (Task 3).
4. **Sélection périmée** : le produit sélectionné est renommé, supprimé ou masqué par la recherche → l'interface n'agit jamais sur un produit disparu et ne plante pas (Task 4).
5. **Couleur seule et contraste** : l'alerte reste lisible sans la couleur ; textes, contours, focus et sélection tiennent les seuils WCAG (Tasks 2 et 4).

---

### Task 1 : La bibliothèque apprend à renommer et à supprimer

**Fichiers :**
- Modifier : `gestion-stock/src/stock.rs` (méthodes, `Clone`, tests dans le module `tests` existant)

**Interfaces :**
- Consomme : `Stock`, `Produit`, `Mouvement`, `ErreurStock::{NomInvalide, ProduitExistant, ProduitInconnu}`, `nettoyer_nom` (fonction privée du même fichier) de la version actuelle.
- Produit :

```rust
#[derive(Debug, Default, Clone)]
pub struct Stock { /* champs inchangés */ }

impl Stock {
    /// Ordre des vérifications : nom invalide, produit inconnu, nom déjà pris par un AUTRE produit.
    /// Changer seulement la casse ou les espaces du même produit est permis.
    /// Le nouveau nom (nettoyé) remplace aussi le nom dans tous les mouvements de l'historique.
    pub fn renommer_produit(&mut self, ancien: &str, nouveau: &str) -> Result<(), ErreurStock>
    /// Retire le produit ; l'historique est conservé tel quel.
    pub fn supprimer_produit(&mut self, nom: &str) -> Result<(), ErreurStock>
}
```

- [ ] **Étape 1 : Créer la branche**

Run : `git checkout -b feature/interface-graphique`
Attendu : `Switched to a new branch 'feature/interface-graphique'`.

- [ ] **Étape 2 : Écrire les tests qui échouent** dans le module `tests` de `stock.rs` (la fonction d'aide `stock_avec_riz()` existe déjà : produit « Riz 5kg », seuil 5)

```rust
#[test] fn renommer_change_le_nom_du_produit_et_celui_de_l_historique() {
    let mut s = stock_avec_riz();
    s.enregistrer_entree("Riz 5kg", 12).unwrap();
    s.renommer_produit("riz 5KG", "  Riz basmati 5kg ").unwrap();
    assert_eq!(s.produits()[0].nom, "Riz basmati 5kg");
    assert_eq!(s.produits()[0].quantite, 12);
    assert_eq!(s.historique()[0].nom, "Riz basmati 5kg");
    assert!(s.enregistrer_entree("Riz basmati 5kg", 1).is_ok());
}
#[test] fn renommer_autorise_le_changement_de_casse_du_meme_produit() {
    let mut s = stock_avec_riz();
    s.renommer_produit("Riz 5kg", "RIZ 5KG").unwrap();
    assert_eq!(s.produits()[0].nom, "RIZ 5KG");
}
#[test] fn renommer_refuse_le_nom_d_un_autre_produit() {
    let mut s = stock_avec_riz();
    s.ajouter_produit("Sucre", 0).unwrap();
    assert_eq!(s.renommer_produit("Sucre", " riz 5KG "), Err(ErreurStock::ProduitExistant("Riz 5kg".into())));
    let noms: Vec<&str> = s.produits().iter().map(|p| p.nom.as_str()).collect();
    assert_eq!(noms, ["Riz 5kg", "Sucre"]);
}
#[test] fn renommer_refuse_un_nom_invalide_ou_un_produit_inconnu() {
    let mut s = stock_avec_riz();
    for nom in ["", "   ", "a;b", "a\nb"] {
        assert!(matches!(s.renommer_produit("Riz 5kg", nom), Err(ErreurStock::NomInvalide(_))), "{nom:?}");
    }
    assert_eq!(s.renommer_produit("Sel", "Sel fin"), Err(ErreurStock::ProduitInconnu("Sel".into())));
    assert_eq!(s.produits()[0].nom, "Riz 5kg");
}
#[test] fn supprimer_retire_le_produit_et_garde_l_historique() {
    let mut s = stock_avec_riz();
    s.enregistrer_entree("Riz 5kg", 3).unwrap();
    s.supprimer_produit(" RIZ 5KG ").unwrap();
    assert!(s.produits().is_empty());
    assert_eq!(s.historique().len(), 1);
    assert_eq!(s.supprimer_produit("RIZ 5KG"), Err(ErreurStock::ProduitInconnu("RIZ 5KG".into())));
}
#[test] fn un_clone_est_independant_de_l_original() {
    let s = stock_avec_riz();
    let mut copie = s.clone();
    copie.enregistrer_entree("Riz 5kg", 7).unwrap();
    assert_eq!(s.produits()[0].quantite, 0);
    assert_eq!(copie.produits()[0].quantite, 7);
}
```

- [ ] **Étape 3 : Lancer les tests et vérifier qu'ils échouent**

Run : `cd gestion-stock && cargo test --lib`
Attendu : erreurs de compilation (`renommer_produit`, `supprimer_produit` introuvables, `Clone` non implémenté).

- [ ] **Étape 4 : Implémenter** `renommer_produit` et `supprimer_produit` dans `impl Stock` et ajouter `Clone` au `derive` de `Stock`

S'appuyer sur `nettoyer_nom` et `position` existants. Pour le conflit de nom, comparer la position du produit trouvé pour `nouveau` avec celle de `ancien` : seul un autre produit est un conflit. Pour l'historique, remplacer le nom de chaque `Mouvement` dont le nom (sans tenir compte de la casse) est celui de l'ancien produit.

- [ ] **Étape 5 : Lancer les tests et vérifier qu'ils passent**

Run : `cd gestion-stock && cargo test`
Attendu : tous PASS (46 tests de bibliothèque, 8 tests de bout en bout).

- [ ] **Étape 6 : Valider et commiter**

Run : `cd gestion-stock && cargo fmt && cargo clippy --all-targets -- -D warnings`
Attendu : aucune sortie d'erreur.

```bash
git add gestion-stock docs/superpowers/plans/2026-10-18-interface-graphique.md
git commit -m "feat(stock): renommer et supprimer un produit (pour l'interface graphique)"
```

---

### Task 2 : Le projet graphique et sa palette accessible

**Fichiers :**
- Créer : `gestion-stock-gui/Cargo.toml`, `gestion-stock-gui/.gitignore` (contient `/target`)
- Créer : `gestion-stock-gui/src/lib.rs` (`pub mod palette;` — chaque tâche suivante ajoute son `pub mod`)
- Créer : `gestion-stock-gui/src/main.rs` (placeholder `fn main() {}`, remplacé à la Task 5)
- Créer : `gestion-stock-gui/src/palette.rs`

**Interfaces :**
- Produit :

```rust
pub type Rgb = [u8; 3];
// Valeurs de départ (à ajuster seulement si un test de contraste échoue) :
pub const FOND: Rgb = [255, 255, 255];
pub const FOND_PANNEAU: Rgb = [243, 244, 246];
pub const TEXTE: Rgb = [17, 24, 39];
pub const BORDURE: Rgb = [107, 114, 128];
pub const BOUTON_FOND: Rgb = [30, 64, 175];
pub const BOUTON_SURVOL_FOND: Rgb = [23, 37, 84];
pub const BOUTON_TEXTE: Rgb = [255, 255, 255];
pub const ALERTE_FOND: Rgb = [254, 226, 226];
pub const ALERTE_TEXTE: Rgb = [153, 27, 27];
pub const SELECTION_FOND: Rgb = [219, 234, 254];
pub const SELECTION_TEXTE: Rgb = [17, 24, 39];
pub const FOCUS: Rgb = [37, 99, 235];

/// Rapport de contraste WCAG entre deux couleurs : de 1,0 à 21,0.
pub fn rapport_contraste(a: Rgb, b: Rgb) -> f64
/// Le style egui complet, construit à partir des constantes ci-dessus.
pub fn visuals() -> eframe::egui::Visuals
```

- [ ] **Étape 1 : Créer le projet**

`Cargo.toml` : paquet `gestion-stock-gui`, édition 2024, `[dependencies]` avec `gestion-stock = { path = "../gestion-stock" }` et `eframe = "0.35"`, `[dev-dependencies]` avec `egui_kittest = "0.35"`, et une section `[[bin]]` `name = "stock-gui"`, `path = "src/main.rs"` (le programme s'appelle `stock-gui`). Ajouter les autres fichiers listés ci-dessus.

- [ ] **Étape 2 : Écrire les tests qui échouent** (`#[cfg(test)] mod tests` dans `palette.rs`)

```rust
fn rgb(c: eframe::egui::Color32) -> Rgb { [c.r(), c.g(), c.b()] }

#[test] fn contraste_noir_sur_blanc_vaut_21() {
    assert!((rapport_contraste([0, 0, 0], [255, 255, 255]) - 21.0).abs() < 0.01);
}
#[test] fn contraste_est_symetrique() {
    let (a, b) = ([30, 64, 175], [254, 226, 226]);
    assert!((rapport_contraste(a, b) - rapport_contraste(b, a)).abs() < 1e-9);
}
#[test] fn le_gris_767676_passe_le_seuil_aa_et_le_gris_777777_echoue() {
    assert!(rapport_contraste([0x76; 3], FOND) >= 4.5);
    assert!(rapport_contraste([0x77; 3], FOND) < 4.5);
}
#[test] fn les_textes_de_la_palette_atteignent_4_5() {
    let paires = [("texte/fond", TEXTE, FOND), ("texte/panneau", TEXTE, FOND_PANNEAU),
        ("bouton", BOUTON_TEXTE, BOUTON_FOND), ("bouton survolé", BOUTON_TEXTE, BOUTON_SURVOL_FOND),
        ("alerte", ALERTE_TEXTE, ALERTE_FOND), ("sélection", SELECTION_TEXTE, SELECTION_FOND)];
    for (nom, texte, fond) in paires {
        assert!(rapport_contraste(texte, fond) >= 4.5, "{nom}: {}", rapport_contraste(texte, fond));
    }
}
#[test] fn les_contours_et_le_focus_de_la_palette_atteignent_3() {
    let paires = [("bordure/fond", BORDURE, FOND), ("bordure/panneau", BORDURE, FOND_PANNEAU),
        ("focus/fond", FOCUS, FOND), ("focus/panneau", FOCUS, FOND_PANNEAU), ("bouton/fond", BOUTON_FOND, FOND)];
    for (nom, a, b) in paires {
        assert!(rapport_contraste(a, b) >= 3.0, "{nom}: {}", rapport_contraste(a, b));
    }
}
#[test] fn le_style_egui_respecte_les_seuils_dans_tous_les_etats() {
    let v = visuals();
    // Texte des libellés sur les fonds de fenêtre, de panneau et de champ de saisie.
    for fond in [v.panel_fill, v.window_fill, v.extreme_bg_color] {
        assert!(rapport_contraste(rgb(v.widgets.noninteractive.fg_stroke.color), rgb(fond)) >= 4.5);
    }
    // Boutons : texte sur fond, dans les trois états. egui peint les boutons avec `weak_bg_fill`
    // (et les curseurs avec `bg_fill`) : les deux doivent passer.
    for w in [v.widgets.inactive, v.widgets.hovered, v.widgets.active] {
        assert!(rapport_contraste(rgb(w.fg_stroke.color), rgb(w.bg_fill)) >= 4.5);
        assert!(rapport_contraste(rgb(w.fg_stroke.color), rgb(w.weak_bg_fill)) >= 4.5);
    }
    // Ligne sélectionnée : texte sur fond de sélection.
    assert!(rapport_contraste(rgb(v.selection.stroke.color), rgb(v.selection.bg_fill)) >= 4.5);
    // Bordures des composants sur le fond de panneau.
    assert!(rapport_contraste(rgb(v.widgets.inactive.bg_stroke.color), rgb(v.panel_fill)) >= 3.0);
}
```

- [ ] **Étape 3 : Lancer les tests et vérifier qu'ils échouent**

Run : `cd gestion-stock-gui && cargo test --lib`
Attendu : erreurs de compilation (`rapport_contraste`, `visuals` et les constantes non définis). Le premier lancement télécharge et compile `eframe` : c'est long.

- [ ] **Étape 4 : Implémenter `palette.rs`**

`rapport_contraste` : luminance relative WCAG (chaque canal : `c/255`, puis `c/12,92` si `c <= 0,03928` sinon `((c+0,055)/1,055)^2,4` ; `L = 0,2126 R + 0,7152 G + 0,0722 B`) ; rapport `(L_clair + 0,05) / (L_sombre + 0,05)`. `visuals()` : partir de `Visuals::light()` et remplacer, depuis les constantes : `panel_fill`, `window_fill`, `extreme_bg_color` (= `FOND` pour les champs de saisie), `widgets.noninteractive` (texte `TEXTE`, fond `FOND_PANNEAU`, contour `BORDURE`), `widgets.inactive` / `hovered` / `active` / `open` (fond `BOUTON_FOND` puis `BOUTON_SURVOL_FOND`, texte `BOUTON_TEXTE`, contour `BORDURE`), `selection` (fond `SELECTION_FOND`, couleur de trait `SELECTION_TEXTE`) et ne **pas** utiliser `override_text_color` (il écraserait le texte blanc des boutons). Si le test `le_style_egui…` échoue parce que egui utilise un champ autrement que prévu, lire les sources `egui-0.35.0` (dans `~/.cargo/registry/src/`), corriger l'implémentation et noter la décision.

- [ ] **Étape 5 : Lancer les tests et vérifier qu'ils passent**

Run : `cd gestion-stock-gui && cargo test --lib`
Attendu : 6 tests PASS.

- [ ] **Étape 6 : Valider et commiter**

Run : `cd gestion-stock-gui && cargo fmt && cargo clippy --all-targets -- -D warnings`
Attendu : aucune sortie d'erreur.

```bash
git add gestion-stock-gui
git commit -m "feat(gui): projet graphique et palette accessible (contraste WCAG AA)"
```

---

### Task 3 : Le modèle `Etat` (toute la logique, sans fenêtre)

**Fichiers :**
- Créer : `gestion-stock-gui/src/etat.rs`
- Modifier : `gestion-stock-gui/src/lib.rs` (ajouter `pub mod etat;`)

**Interfaces :**
- Consomme : `gestion_stock::stock::{Stock, Genre}` (dont `renommer_produit`, `supprimer_produit`, `Clone` de la Task 1), `gestion_stock::stockage::{charger, sauvegarder}`, `gestion_stock::erreur::ErreurStock`.
- Produit :

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Retour { Succes(String), Erreur(String) }

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LigneProduit { pub nom: String, pub quantite: u32, pub seuil: u32, pub en_alerte: bool }

#[derive(Debug)]
pub struct Etat { /* stock, chemin privés */ pub retour: Option<Retour> }

impl Etat {
    /// Crée le dossier du fichier s'il manque. Fichier absent => stock vide + message (voir ci-dessous).
    pub fn ouvrir(chemin: PathBuf) -> Result<Etat, ErreurStock>
    pub fn chemin(&self) -> &Path
    /// Produits triés par nom ; `recherche` : texte contenu dans le nom, sans casse ni espaces autour.
    pub fn lignes(&self, recherche: &str, alertes_seulement: bool) -> Vec<LigneProduit>
    /// Du plus récent au plus ancien, ex. « Entrée +12 Riz 5kg » / « Sortie -2 Riz 5kg ».
    pub fn historique(&self, limite: usize) -> Vec<String>
    // Chaque action renvoie true si elle a réussi ; dans tous les cas `retour` est mis à jour.
    pub fn ajouter(&mut self, nom: &str, seuil: &str) -> bool      // seuil vide = 0
    pub fn entree(&mut self, nom: &str, quantite: &str) -> bool
    pub fn sortie(&mut self, nom: &str, quantite: &str) -> bool
    pub fn renommer(&mut self, ancien: &str, nouveau: &str) -> bool
    pub fn supprimer(&mut self, nom: &str) -> bool
}
```

**Règle commune des actions :** appliquer l'action sur une **copie** du stock ; si elle réussit, sauvegarder la copie ; si la sauvegarde réussit, la copie remplace le stock ; sinon le stock n'est pas touché, `retour` = `Erreur(message)`.

**Textes exacts de `retour` :**

| Cas | Texte |
|---|---|
| `ajouter` | `Succes("Produit « Riz 5kg » ajouté (seuil d'alerte : 5).")` (nom nettoyé) |
| `entree` | `Succes("Entrée enregistrée : +3 Riz 5kg, stock actuel : 15.")` |
| `sortie` | `Succes("Sortie enregistrée : -2 Riz 5kg, stock actuel : 13.")` ; si le produit est ensuite en alerte, on ajoute ` ATTENTION : stock bas (seuil d'alerte : 5).` |
| `renommer` | `Succes("Produit renommé : « Riz 5kg » devient « Riz basmati 5kg ».")` |
| `supprimer` | `Succes("Produit « Riz 5kg » supprimé (l'historique est conservé).")` |
| ouverture, fichier absent | `Succes("Aucun fichier de données trouvé : un stock vide est utilisé. Le fichier sera créé au premier enregistrement.")` |
| ouverture, fichier présent | `None` |
| quantité vide | `Erreur("Quantité manquante. Saisissez un nombre entier positif, par exemple 12.")` |
| quantité illisible | `Erreur("Quantité invalide : « abc ». Saisissez un nombre entier positif, par exemple 12.")` |
| seuil illisible | `Erreur("Seuil d'alerte invalide : « abc ». Saisissez un nombre entier positif, par exemple 5, ou laissez vide pour 0.")` |
| erreur de la bibliothèque | `Erreur(e.to_string())` (déjà en français) |

- [ ] **Étape 1 : Écrire les tests qui échouent** (`#[cfg(test)] mod tests` dans `etat.rs`)

```rust
// aides :
//   fn chemin_temp(nom: &str) -> PathBuf   temp_dir()/gestion-stock-gui-{pid}-{nom}.txt, supprimé s'il existe
//   fn etat_avec_riz(nom: &str) -> Etat    ouvrir + ajouter("Riz 5kg", "5") + entree("Riz 5kg", "12")  (stock 12)

#[test] fn ouvrir_un_fichier_absent_donne_un_stock_vide_et_le_dit() {
    let e = Etat::ouvrir(chemin_temp("absent")).unwrap();
    assert!(e.lignes("", false).is_empty());
    assert_eq!(e.retour, Some(Retour::Succes("Aucun fichier de données trouvé : un stock vide est utilisé. Le fichier sera créé au premier enregistrement.".into())));
}
#[test] fn ouvrir_cree_le_dossier_des_donnees() {
    let dossier = std::env::temp_dir().join(format!("gestion-stock-gui-{}-dossier", std::process::id()));
    let _ = std::fs::remove_dir_all(&dossier);
    let mut e = Etat::ouvrir(dossier.join("stock.txt")).unwrap();
    assert!(e.ajouter("Riz", ""));
    assert!(dossier.join("stock.txt").exists());
    std::fs::remove_dir_all(&dossier).unwrap();
}
#[test] fn ouvrir_un_fichier_abime_renvoie_l_erreur_sans_toucher_au_fichier() {
    let c = chemin_temp("abime");
    std::fs::write(&c, "P;Riz;12;5\nn'importe quoi\n").unwrap();
    assert_eq!(Etat::ouvrir(c.clone()).unwrap_err(), ErreurStock::FichierCorrompu { ligne: 2 });
    assert_eq!(std::fs::read_to_string(&c).unwrap(), "P;Riz;12;5\nn'importe quoi\n");
}
#[test] fn ajouter_valide_sauvegarde_et_confirme() {
    let c = chemin_temp("ajout");
    let mut e = Etat::ouvrir(c.clone()).unwrap();
    assert!(e.ajouter("  Riz 5kg ", "5"));
    assert_eq!(e.retour, Some(Retour::Succes("Produit « Riz 5kg » ajouté (seuil d'alerte : 5).".into())));
    assert_eq!(e.lignes("", false), [LigneProduit { nom: "Riz 5kg".into(), quantite: 0, seuil: 5, en_alerte: true }]);
    assert!(std::fs::read_to_string(&c).unwrap().contains("P;Riz 5kg;0;5"));
}
#[test] fn ajouter_sans_seuil_utilise_zero() {
    let mut e = Etat::ouvrir(chemin_temp("sans-seuil")).unwrap();
    assert!(e.ajouter("Sel", "  "));
    assert_eq!(e.lignes("", false)[0].seuil, 0);
}
#[test] fn saisies_invalides_de_l_ajout_ne_changent_rien() {
    for (nom, seuil) in [("", "5"), ("a;b", "5"), ("Riz", "abc"), ("Riz", "-1"), ("Riz", "2,5")] {
        let mut e = Etat::ouvrir(chemin_temp("invalide")).unwrap();
        assert!(!e.ajouter(nom, seuil), "{nom:?} {seuil:?}");
        assert!(matches!(e.retour, Some(Retour::Erreur(_))));
        assert!(e.lignes("", false).is_empty());
    }
    let mut e = Etat::ouvrir(chemin_temp("seuil")).unwrap();
    e.ajouter("Riz", "abc");
    assert_eq!(e.retour, Some(Retour::Erreur("Seuil d'alerte invalide : « abc ». Saisissez un nombre entier positif, par exemple 5, ou laissez vide pour 0.".into())));
}
#[test] fn entree_et_sortie_confirment_et_signalent_le_stock_bas() {
    let mut e = etat_avec_riz("mouvements");
    assert!(e.entree("riz 5kg", "3"));
    assert_eq!(e.retour, Some(Retour::Succes("Entrée enregistrée : +3 Riz 5kg, stock actuel : 15.".into())));
    assert!(e.sortie("Riz 5kg", "2"));
    assert_eq!(e.retour, Some(Retour::Succes("Sortie enregistrée : -2 Riz 5kg, stock actuel : 13.".into())));
    assert!(e.sortie("Riz 5kg", "9"));
    assert_eq!(e.retour, Some(Retour::Succes("Sortie enregistrée : -9 Riz 5kg, stock actuel : 4. ATTENTION : stock bas (seuil d'alerte : 5).".into())));
}
#[test] fn quantites_invalides_donnent_un_message_clair() {
    let mut e = etat_avec_riz("quantites");
    for q in ["", "  ", "abc", "-3", "2,5", "99999999999"] {
        assert!(!e.entree("Riz 5kg", q), "{q:?}");
        assert!(!e.sortie("Riz 5kg", q), "{q:?}");
        assert!(matches!(&e.retour, Some(Retour::Erreur(m)) if m.starts_with("Quantité ")), "{q:?}");
    }
    e.entree("Riz 5kg", "");
    assert_eq!(e.retour, Some(Retour::Erreur("Quantité manquante. Saisissez un nombre entier positif, par exemple 12.".into())));
    e.entree("Riz 5kg", "abc");
    assert_eq!(e.retour, Some(Retour::Erreur("Quantité invalide : « abc ». Saisissez un nombre entier positif, par exemple 12.".into())));
    assert!(!e.sortie("Riz 5kg", "0"));
    assert_eq!(e.retour, Some(Retour::Erreur("La quantité doit être supérieure à zéro.".into())));
    assert_eq!(e.lignes("", false)[0].quantite, 12);
}
#[test] fn sortie_trop_grande_refusee_sans_toucher_au_fichier() {
    let mut e = etat_avec_riz("trop-grande");
    let avant = std::fs::read_to_string(e.chemin()).unwrap();
    assert!(!e.sortie("Riz 5kg", "99"));
    assert_eq!(e.retour, Some(Retour::Erreur("Stock insuffisant pour « Riz 5kg » : 12 disponible(s), 99 demandé(s).".into())));
    assert_eq!(e.lignes("", false)[0].quantite, 12);
    assert_eq!(std::fs::read_to_string(e.chemin()).unwrap(), avant);
}
#[test] fn echec_de_sauvegarde_laisse_la_liste_affichee_inchangee() {
    let c = chemin_temp("sauvegarde-impossible");
    let mut e = Etat::ouvrir(c.clone()).unwrap();
    let bloc = PathBuf::from(format!("{}.tmp", c.display()));   // un DOSSIER à la place du fichier temporaire
    std::fs::create_dir(&bloc).unwrap();
    assert!(!e.ajouter("Sel", "1"));
    assert!(matches!(&e.retour, Some(Retour::Erreur(m)) if m.contains(&c.display().to_string())));
    assert!(e.lignes("", false).is_empty());
    std::fs::remove_dir(&bloc).unwrap();
}
#[test] fn renommer_met_a_jour_la_liste_et_l_historique() {
    let mut e = etat_avec_riz("renommer");
    assert!(e.renommer("riz 5kg", " Riz basmati "));
    assert_eq!(e.retour, Some(Retour::Succes("Produit renommé : « riz 5kg » devient « Riz basmati ».".into())));
    assert_eq!(e.lignes("", false)[0].nom, "Riz basmati");
    assert_eq!(e.historique(10)[0], "Entrée +12 Riz basmati");
    assert!(!e.renommer("Sel", "Sel fin"));
    assert!(matches!(e.retour, Some(Retour::Erreur(_))));
}
#[test] fn supprimer_retire_le_produit_et_garde_l_historique() {
    let mut e = etat_avec_riz("supprimer");
    assert!(e.supprimer("RIZ 5KG"));
    assert_eq!(e.retour, Some(Retour::Succes("Produit « RIZ 5KG » supprimé (l'historique est conservé).".into())));
    assert!(e.lignes("", false).is_empty());
    assert_eq!(e.historique(10).len(), 1);
    assert!(!e.supprimer("Riz 5kg"));
}
#[test] fn lignes_filtrent_par_recherche_et_par_alerte() {
    let mut e = etat_avec_riz("filtres");          // Riz 5kg : 12, seuil 5 (pas en alerte)
    e.ajouter("Riz basmati", "5");                  // 0, seuil 5 (en alerte)
    e.ajouter("Sucre", "0");
    e.entree("Sucre", "4");
    assert_eq!(e.lignes("  RIZ ", false).len(), 2);
    assert_eq!(e.lignes("sucre", false).len(), 1);
    assert_eq!(e.lignes("zzz", false).len(), 0);
    let alertes: Vec<String> = e.lignes("", true).into_iter().map(|l| l.nom).collect();
    assert_eq!(alertes, ["Riz basmati"]);
}
#[test] fn historique_du_plus_recent_au_plus_ancien_avec_limite() {
    let mut e = etat_avec_riz("historique");
    e.sortie("Riz 5kg", "2");
    assert_eq!(e.historique(10), ["Sortie -2 Riz 5kg", "Entrée +12 Riz 5kg"]);
    assert_eq!(e.historique(1), ["Sortie -2 Riz 5kg"]);
}
```

- [ ] **Étape 2 : Lancer les tests et vérifier qu'ils échouent**

Run : `cd gestion-stock-gui && cargo test --lib etat`
Attendu : erreurs de compilation (`Etat`, `Retour`, `LigneProduit` non définis).

- [ ] **Étape 3 : Implémenter `etat.rs`**

Un `fn appliquer(&mut self, action: impl FnOnce(&mut Stock) -> Result<String, ErreurStock>) -> bool` privé porte la règle commune (copie, action, sauvegarde, remplacement) ; les cinq actions publiques ne font que lire leurs champs de saisie puis appeler `appliquer`. Une fonction privée lit un entier positif depuis le texte saisi (`trim`, vide → message « manquante », échec de `parse::<u32>()` → message « invalide »). `ouvrir` fait `std::fs::create_dir_all` sur le dossier du fichier (erreur → `ErreurStock::Io` en français, comme `stockage`), puis `charger`.

- [ ] **Étape 4 : Lancer les tests et vérifier qu'ils passent**

Run : `cd gestion-stock-gui && cargo test --lib`
Attendu : tous PASS (6 de `palette`, 15 de `etat`).

- [ ] **Étape 5 : Valider et commiter**

Run : `cd gestion-stock-gui && cargo fmt && cargo clippy --all-targets -- -D warnings`
Attendu : aucune sortie d'erreur.

```bash
git add gestion-stock-gui/src/etat.rs gestion-stock-gui/src/lib.rs
git commit -m "feat(gui): modèle Etat (actions, messages d'erreur, sauvegarde sûre)"
```

---

### Task 4 : L'interface (egui) et ses tests d'accessibilité

**Fichiers :**
- Créer : `gestion-stock-gui/src/textes.rs`, `gestion-stock-gui/src/ui.rs`
- Modifier : `gestion-stock-gui/src/lib.rs` (ajouter `pub mod textes; pub mod ui;`)

**Interfaces :**
- Consomme : `etat::{Etat, Retour, LigneProduit}`, `palette::{ALERTE_FOND, ALERTE_TEXTE}` et `Etat::lignes / historique / ajouter / entree / sortie / renommer / supprimer / chemin`.
- Produit :

```rust
// textes.rs — constantes &str (tous les textes visibles ; le « {} » des gabarits est rempli avec format!)
pub const TITRE_FENETRE: &str = "Gestion de stock";
pub const RECHERCHE: &str = "Rechercher un produit";
pub const ALERTES_SEULEMENT: &str = "Afficher seulement les produits en alerte";
pub const COLONNE_PRODUIT: &str = "Produit";   pub const COLONNE_STOCK: &str = "En stock";
pub const COLONNE_SEUIL: &str = "Seuil";       pub const COLONNE_ETAT: &str = "État";
pub const ETAT_ALERTE: &str = "ALERTE : stock bas";   pub const ETAT_CORRECT: &str = "Correct";
pub const SECTION_AJOUT: &str = "Ajouter un produit";
pub const CHAMP_NOM: &str = "Nom du produit";
pub const CHAMP_SEUIL: &str = "Seuil d'alerte (facultatif)";
pub const BOUTON_AJOUTER: &str = "Ajouter le produit";
pub const AUCUNE_SELECTION: &str = "Aucun produit sélectionné. Choisissez un produit dans la liste.";
pub const CHAMP_QUANTITE: &str = "Quantité";
pub const BOUTON_ENTREE: &str = "Enregistrer une entrée";
pub const BOUTON_SORTIE: &str = "Enregistrer une sortie";
pub const CHAMP_NOUVEAU_NOM: &str = "Nouveau nom";
pub const BOUTON_RENOMMER: &str = "Renommer le produit";
pub const BOUTON_SUPPRIMER: &str = "Supprimer le produit";
pub const BOUTON_CONFIRMER_SUPPRESSION: &str = "Confirmer la suppression";
pub const BOUTON_ANNULER: &str = "Annuler";
pub const SECTION_HISTORIQUE: &str = "Derniers mouvements";
pub const ERREUR_OUVERTURE_TITRE: &str = "Impossible d'ouvrir le fichier de données";
pub const ERREUR_OUVERTURE_AIDE: &str = "Rien n'a été modifié. Corrigez le fichier, ou indiquez un autre emplacement avec la variable STOCK_FICHIER, puis relancez le programme.";
pub const BOUTON_QUITTER: &str = "Quitter";
pub fn produit_selectionne(nom: &str, quantite: u32) -> String   // « Produit sélectionné : {nom} ({quantite} en stock) »
pub fn confirmation_suppression(nom: &str, quantite: u32) -> String   // « Supprimer « {nom} » ? Il reste {quantite} en stock. L'historique est conservé. »
pub fn pied_de_page_donnees(chemin: &str) -> String   // « Données : {chemin} »

// ui.rs
pub struct Application {
    pub etat: Option<Etat>,                 // None = écran d'erreur d'ouverture
    pub erreur_ouverture: Option<String>,
    pub recherche: String, pub alertes_seulement: bool,
    pub selection: Option<String>,          // nom du produit sélectionné
    pub saisie_nom: String, pub saisie_seuil: String,
    pub saisie_quantite: String, pub saisie_nouveau_nom: String,
    pub confirmer_suppression: bool,
}
impl Application {
    pub fn nouvelle(etat: Etat) -> Application
    pub fn en_erreur(message: String) -> Application
}
/// Dessine toute la fenêtre dans le `Ui` donné (panneaux compris) : permet de la tester sans fenêtre.
pub fn dessiner(ui: &mut eframe::egui::Ui, app: &mut Application)
impl eframe::App for Application { fn ui(&mut self, ui: &mut eframe::egui::Ui, _frame: &mut eframe::Frame) /* appelle dessiner */ }
```

**Règles de dessin :**
- Disposition : pied de page en bas (message + chemin) ; panneau à droite (ajout, puis commandes du produit sélectionné) ; centre : recherche, case des alertes, tableau (`egui::Grid` dans un `ScrollArea`), puis historique.
- Chaque champ de saisie est relié à son libellé : `let l = ui.label(TEXTE); champ.labelled_by(l.id)`. Chaque bouton porte son texte complet comme nom.
- Tableau : le nom est un `selectable_label` ; l'état est écrit (`ETAT_ALERTE` ou `ETAT_CORRECT`) ; une ligne en alerte a en plus le fond `ALERTE_FOND` et le texte `ALERTE_TEXTE` (la couleur n'est jamais seule).
- Cliquer un nom : `selection = Some(nom)`, vider `saisie_nouveau_nom`, `confirmer_suppression = false`.
- À chaque image : si `selection` désigne un produit absent de `etat.lignes("", false)` (supprimé, renommé), la remettre à `None`.
- Sans sélection : seulement `AUCUNE_SELECTION` à la place des commandes du produit. Avec sélection : `produit_selectionne`, champ `CHAMP_QUANTITE`, boutons entrée / sortie, champ `CHAMP_NOUVEAU_NOM` + bouton renommer, bouton supprimer.
- Après succès : entrée/sortie vident `saisie_quantite` ; ajouter vide `saisie_nom` et `saisie_seuil` ; renommer met `selection` au nouveau nom nettoyé et vide `saisie_nouveau_nom` ; supprimer met `selection` à `None`. Après échec : rien n'est vidé.
- Supprimer : le premier clic affiche `confirmation_suppression` avec les boutons `BOUTON_CONFIRMER_SUPPRESSION` et `BOUTON_ANNULER` (`confirmer_suppression = true`) ; seul « Confirmer » supprime.
- Pied de page : `etat.retour` — un succès en texte normal, une erreur précédée de `Erreur : ` sur fond `ALERTE_FOND` / texte `ALERTE_TEXTE` — puis `pied_de_page_donnees(chemin)`.
- Écran d'erreur d'ouverture (`etat == None`) : `ERREUR_OUVERTURE_TITRE`, le message, `ERREUR_OUVERTURE_AIDE` et le bouton `BOUTON_QUITTER` (envoie `ViewportCommand::Close`).
- Lire les sources `eframe-0.35.0` / `egui-0.35.0` (dans `~/.cargo/registry/src/`) pour les API exactes des panneaux (`show_inside`) et des `Grid`.

- [ ] **Étape 1 : Écrire les tests qui échouent** (`#[cfg(test)] mod tests` dans `ui.rs`, avec `egui_kittest::{Harness, kittest::Queryable}`)

```rust
// aides :
//   fn app_avec_produits(nom: &str) -> Application   fichier temporaire ; via Etat : « Riz 5kg » (seuil 5, stock 3, en alerte)
//                                                     et « Sucre » (seuil 0, stock 10, correct)
//   fn harness(app: Application) -> Harness<'static, Application>   Harness::new_ui_state(|ui, app| dessiner(ui, app), app) puis run()
// Pour taper dans un champ : harness.get_by_label(X).type_text("...") puis harness.run().
// Pour cliquer : harness.get_by_label(X).click() puis harness.run().

#[test] fn les_commandes_sans_selection_ont_un_nom_accessible() {
    let h = harness(app_avec_produits("sans-selection"));
    for nom in [RECHERCHE, ALERTES_SEULEMENT, CHAMP_NOM, CHAMP_SEUIL, BOUTON_AJOUTER] { h.get_by_label(nom); }
    assert!(h.query_by_label(BOUTON_ENTREE).is_none());
    h.get_by_label(AUCUNE_SELECTION);
}
#[test] fn choisir_un_produit_affiche_ses_commandes() {
    let mut h = harness(app_avec_produits("selection"));
    h.get_by_label("Riz 5kg").click(); h.run();
    assert_eq!(h.state().selection.as_deref(), Some("Riz 5kg"));
    for nom in [CHAMP_QUANTITE, BOUTON_ENTREE, BOUTON_SORTIE, CHAMP_NOUVEAU_NOM, BOUTON_RENOMMER, BOUTON_SUPPRIMER] { h.get_by_label(nom); }
    h.get_by_label(&produit_selectionne("Riz 5kg", 3));
}
#[test] fn ajouter_un_produit_par_l_interface() {
    let mut h = harness(app_avec_produits("ajout"));
    h.get_by_label(CHAMP_NOM).type_text("Café 250g"); h.run();
    h.get_by_label(CHAMP_SEUIL).type_text("2"); h.run();
    h.get_by_label(BOUTON_AJOUTER).click(); h.run();
    let lignes = h.state().etat.as_ref().unwrap().lignes("café", false);
    assert_eq!((lignes[0].nom.as_str(), lignes[0].seuil), ("Café 250g", 2));
    assert!(h.state().saisie_nom.is_empty() && h.state().saisie_seuil.is_empty());
}
#[test] fn une_sortie_trop_grande_affiche_une_erreur_lisible_et_garde_la_saisie() {
    let mut h = harness(app_avec_produits("sortie-erreur"));
    h.get_by_label("Riz 5kg").click(); h.run();
    h.get_by_label(CHAMP_QUANTITE).type_text("5"); h.run();
    h.get_by_label(BOUTON_SORTIE).click(); h.run();
    h.get_by_label_contains("Erreur : Stock insuffisant pour « Riz 5kg » : 3 disponible(s), 5 demandé(s).");
    assert_eq!(h.state().saisie_quantite, "5");
}
#[test] fn l_alerte_est_ecrite_et_pas_seulement_coloree() {
    let h = harness(app_avec_produits("alerte"));
    assert_eq!(h.get_all_by_label(ETAT_ALERTE).count(), 1);     // Riz 5kg
    assert_eq!(h.get_all_by_label(ETAT_CORRECT).count(), 1);    // Sucre
}
#[test] fn la_suppression_demande_une_confirmation() {
    let mut h = harness(app_avec_produits("suppression"));
    h.get_by_label("Riz 5kg").click(); h.run();
    h.get_by_label(BOUTON_SUPPRIMER).click(); h.run();
    h.get_by_label_contains("Supprimer « Riz 5kg » ? Il reste 3 en stock.");
    assert_eq!(h.state().etat.as_ref().unwrap().lignes("", false).len(), 2);
    h.get_by_label(BOUTON_ANNULER).click(); h.run();
    assert_eq!(h.state().etat.as_ref().unwrap().lignes("", false).len(), 2);
    h.get_by_label(BOUTON_SUPPRIMER).click(); h.run();
    h.get_by_label(BOUTON_CONFIRMER_SUPPRESSION).click(); h.run();
    assert_eq!(h.state().etat.as_ref().unwrap().lignes("", false).len(), 1);
    assert_eq!(h.state().selection, None);
}
#[test] fn une_selection_disparue_est_abandonnee_sans_planter() {
    let mut app = app_avec_produits("fantome");
    app.selection = Some("Fantôme".into());
    let h = harness(app);
    assert_eq!(h.state().selection, None);
    h.get_by_label(AUCUNE_SELECTION);
}
#[test] fn la_recherche_qui_masque_le_produit_selectionne_ne_plante_pas() {
    let mut h = harness(app_avec_produits("recherche"));
    h.get_by_label("Riz 5kg").click(); h.run();
    h.get_by_label(RECHERCHE).type_text("sucre"); h.run();
    assert!(h.query_by_label("Riz 5kg").is_none());
    h.get_by_label(BOUTON_ENTREE);            // la sélection (masquée mais existante) reste utilisable
}
#[test] fn l_ecran_d_erreur_d_ouverture_est_lisible() {
    let h = harness(Application::en_erreur("Fichier de données corrompu à la ligne 2.".into()));
    h.get_by_label(ERREUR_OUVERTURE_TITRE);
    h.get_by_label_contains("ligne 2");
    h.get_by_label(ERREUR_OUVERTURE_AIDE);
    h.get_by_label(BOUTON_QUITTER);
}
#[test] fn le_zoom_au_clavier_est_actif() {
    assert!(eframe::egui::Context::default().options(|o| o.zoom_with_keyboard));
}
```

(Si l'API de `egui_kittest` 0.35 diffère de ces appels — noms de méthodes, façon de lire l'état — lire `egui_kittest-0.35.0` et `kittest-0.4.0` dans `~/.cargo/registry/src/`, adapter les tests **sans en affaiblir les assertions**, et noter la décision.)

- [ ] **Étape 2 : Lancer les tests et vérifier qu'ils échouent**

Run : `cd gestion-stock-gui && cargo test --lib ui`
Attendu : erreurs de compilation (`Application`, `dessiner`, constantes de `textes` non définis).

- [ ] **Étape 3 : Implémenter `textes.rs` puis `ui.rs`** selon les règles de dessin ci-dessus

- [ ] **Étape 4 : Lancer les tests et vérifier qu'ils passent**

Run : `cd gestion-stock-gui && cargo test --lib`
Attendu : tous PASS (6 de `palette`, 15 de `etat`, 9 de `ui`).

- [ ] **Étape 5 : Valider et commiter**

Run : `cd gestion-stock-gui && cargo fmt && cargo clippy --all-targets -- -D warnings`
Attendu : aucune sortie d'erreur.

```bash
git add gestion-stock-gui/src
git commit -m "feat(gui): interface egui (liste, ajout, mouvements, renommer, supprimer, historique)"
```

---

### Task 5 : Le programme `stock-gui`, le chemin par défaut, le README et les vérifications

**Fichiers :**
- Créer : `gestion-stock-gui/src/chemin.rs`
- Modifier : `gestion-stock-gui/src/lib.rs` (ajouter `pub mod chemin;`), `gestion-stock-gui/src/main.rs`
- Créer : `gestion-stock-gui/README.md`, `gestion-stock-gui/LICENCES-TIERS.txt`

**Interfaces :**
- Consomme : `Etat::ouvrir`, `Application::{nouvelle, en_erreur}`, `palette::visuals`, `textes::TITRE_FENETRE`.
- Produit :

```rust
// chemin.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Systeme { MacOs, Windows, Autre }
pub fn systeme_courant() -> Systeme                       // cfg!(target_os = ...)
/// Dossier de données de l'utilisateur ; `var` lit une variable d'environnement (vide = absente).
pub fn chemin_donnees(systeme: Systeme, var: impl Fn(&str) -> Option<String>) -> Option<PathBuf>
/// Priorité : STOCK_FICHIER non vide, puis le dossier de données, puis « stock.txt ».
pub fn choisir(stock_fichier: Option<String>, donnees: Option<PathBuf>) -> PathBuf
pub fn chemin_par_defaut() -> PathBuf                     // lit l'environnement réel et appelle les deux fonctions ci-dessus
```

- [ ] **Étape 1 : Écrire les tests qui échouent** (`#[cfg(test)] mod tests` dans `chemin.rs`)

```rust
fn env<'a>(paires: &'a [(&'a str, &'a str)]) -> impl Fn(&str) -> Option<String> + 'a {
    move |cle| paires.iter().find(|(k, _)| *k == cle).map(|(_, v)| v.to_string())
}

#[test] fn macos_utilise_application_support() {
    assert_eq!(chemin_donnees(Systeme::MacOs, env(&[("HOME", "/Users/awa")])),
        Some(PathBuf::from("/Users/awa/Library/Application Support/GestionStock/stock.txt")));
}
#[test] fn windows_utilise_appdata() {
    assert_eq!(chemin_donnees(Systeme::Windows, env(&[("APPDATA", "C:\\Users\\Awa\\AppData\\Roaming")])),
        Some(PathBuf::from("C:\\Users\\Awa\\AppData\\Roaming").join("GestionStock").join("stock.txt")));
}
#[test] fn linux_prefere_xdg_data_home() {
    assert_eq!(chemin_donnees(Systeme::Autre, env(&[("XDG_DATA_HOME", "/home/awa/.data"), ("HOME", "/home/awa")])),
        Some(PathBuf::from("/home/awa/.data/gestion-stock/stock.txt")));
}
#[test] fn linux_sans_xdg_utilise_local_share() {
    assert_eq!(chemin_donnees(Systeme::Autre, env(&[("HOME", "/home/awa")])),
        Some(PathBuf::from("/home/awa/.local/share/gestion-stock/stock.txt")));
}
#[test] fn une_variable_vide_est_ignoree() {
    assert_eq!(chemin_donnees(Systeme::Autre, env(&[("XDG_DATA_HOME", ""), ("HOME", "/home/awa")])),
        Some(PathBuf::from("/home/awa/.local/share/gestion-stock/stock.txt")));
}
#[test] fn sans_variable_il_n_y_a_pas_de_dossier_de_donnees() {
    assert_eq!(chemin_donnees(Systeme::MacOs, env(&[])), None);
    assert_eq!(chemin_donnees(Systeme::Windows, env(&[])), None);
    assert_eq!(chemin_donnees(Systeme::Autre, env(&[])), None);
}
#[test] fn choisir_respecte_la_priorite() {
    let donnees = Some(PathBuf::from("/d/stock.txt"));
    assert_eq!(choisir(Some("/x/mon.txt".into()), donnees.clone()), PathBuf::from("/x/mon.txt"));
    assert_eq!(choisir(Some("".into()), donnees.clone()), PathBuf::from("/d/stock.txt"));
    assert_eq!(choisir(None, donnees), PathBuf::from("/d/stock.txt"));
    assert_eq!(choisir(None, None), PathBuf::from("stock.txt"));
}
```

- [ ] **Étape 2 : Lancer les tests et vérifier qu'ils échouent**

Run : `cd gestion-stock-gui && cargo test --lib chemin`
Attendu : erreurs de compilation (`chemin_donnees`, `choisir`, `Systeme` non définis).

- [ ] **Étape 3 : Implémenter `chemin.rs`**

`systeme_courant` : `cfg!(target_os = "macos")` → `MacOs`, `cfg!(windows)` → `Windows`, sinon `Autre`. Une variable d'environnement vide compte comme absente. `chemin_par_defaut` appelle `choisir(std::env::var("STOCK_FICHIER").ok(), chemin_donnees(systeme_courant(), |c| std::env::var(c).ok()))`.

- [ ] **Étape 4 : Lancer les tests et vérifier qu'ils passent**

Run : `cd gestion-stock-gui && cargo test --lib`
Attendu : tous PASS (6 + 15 + 9 + 7 = 37 tests).

- [ ] **Étape 5 : Écrire `main.rs`**

```rust
fn main() -> eframe::Result
```

`chemin_par_defaut()`, puis `Etat::ouvrir` : succès → `Application::nouvelle`, échec → `Application::en_erreur(e.to_string())` (jamais d'écrasement du fichier). `eframe::run_native` avec : titre `TITRE_FENETRE`, taille initiale 960×640, taille minimale 640×420 (`ViewportBuilder`), et un créateur qui applique `cc.egui_ctx.set_visuals(palette::visuals())` puis renvoie l'application.

- [ ] **Étape 6 : Écrire `README.md`** (français, court) avec ces sections

À quoi ça sert ; lancer (`cargo run --release`) ; où sont les données (chemins par défaut selon le système, `STOCK_FICHIER`, chemin affiché dans la fenêtre, même format que le programme en ligne de commande, sauvegarde conseillée) ; **Accessibilité** (cible WCAG 2.1 AA / EN 301 549 / Section 508, tableau de la section « Exigences » du plan avec l'état de chaque critère, **limites connues** : pas de région « live » pour les messages d'état, qualité du lecteur d'écran dépendante d'egui/AccessKit, aucun audit externe, pas de déclaration de conformité) ; **Vie privée** (aucune donnée personnelle, aucun réseau, aucune télémétrie, fichier local) ; **Licences** (renvoi à `LICENCES-TIERS.txt`) ; **Essai manuel** (liste ci-dessous).

Liste d'essai manuel à mettre dans le README :
1. Lancer le programme, ajouter un produit avec un seuil, faire une entrée, puis une sortie qui passe sous le seuil : l'état affiche « ALERTE : stock bas ».
2. Tout faire **sans la souris** : `Tab` pour avancer, `Maj+Tab` pour reculer, `Espace` ou `Entrée` pour activer.
3. Agrandir le texte avec `Cmd` + `+` (macOS) ou `Ctrl` + `+`, puis revenir avec `Cmd`/`Ctrl` + `0`.
4. Sur macOS, activer VoiceOver (`Cmd+F5`) et vérifier que les champs et les boutons sont annoncés avec leur nom.
5. Provoquer des erreurs : quantité vide, `abc`, sortie trop grande, nom avec `;` → un message clair, rien ne change.
6. Fermer et rouvrir : les données sont toujours là, au chemin affiché en bas.

- [ ] **Étape 7 : Vérifications de conformité**

Run : `cd gestion-stock-gui && cargo tree -e normal | grep -iE 'reqwest|hyper|ureq|curl|openssl|rustls|isahc|attohttpc|tungstenite' ; echo "fin-reseau"`
Attendu : seule la ligne `fin-reseau` s'affiche (aucune dépendance réseau). Si une ligne apparaît, ne pas continuer sans noter la décision (le code du programme n'ouvre aucune connexion ; la dépendance indirecte est à documenter dans le README).

Run : `cd gestion-stock-gui && cargo tree --prefix none --format "{p} {l}" -e normal | sort -u > LICENCES-TIERS.txt && grep -ic 'gpl' LICENCES-TIERS.txt`
Attendu : `0` (aucune licence GPL, AGPL ni LGPL). Un résultat différent : lire les lignes concernées et noter la décision.

- [ ] **Étape 8 : Vérification finale et essai de démarrage**

Run : `cd gestion-stock-gui && cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test`
Attendu : aucune erreur ni avertissement, tous les tests PASS (37).

Run : `cd gestion-stock && cargo test`
Attendu : 46 tests de bibliothèque et 8 tests de bout en bout PASS (le programme en ligne de commande n'est pas cassé).

Run (démarrage, quelques secondes ; on lance le binaire lui-même pour pouvoir l'arrêter proprement) : `cd gestion-stock-gui && cargo build --quiet && (STOCK_FICHIER="$(mktemp -d)/essai.txt" ./target/debug/stock-gui & PID=$!; sleep 8; kill $PID 2>/dev/null; wait $PID 2>/dev/null; echo "démarrage terminé")`
Attendu : une fenêtre « Gestion de stock » s'ouvre brièvement, aucun message d'erreur dans le terminal. (Le test complet à la main est celui du README ; il revient à l'utilisateur.)

```bash
git add gestion-stock-gui
git commit -m "feat(gui): programme stock-gui, chemin de données par défaut, README et vérifications"
```
