# Caisse Kora Marché, étape A (apparence) — Plan d'implémentation

> **Pour les agents :** SOUS-COMPÉTENCE REQUISE : utiliser superpowers:executing-plans pour exécuter ce plan tâche par tâche, **dans la session, sans subagent** (demande de l'utilisateur). Les étapes utilisent la syntaxe à cocher (`- [ ]`).

**Objectif :** Habiller l'application existante (`gestion-stock-gui`) avec le design de la maquette « POS Kora Marché » : police IBM Plex Sans, couleurs de la maquette, écrans Vente (scan, ticket, paiement espèces / carte / autre), Recherche, Historique avec remboursement, et Stock et Réglages dans le même cadre, **avec les seules fonctions qui existent déjà**.

**Architecture :** `ui.rs` (3 100 lignes) est découpé en modules par écran (`src/ui/`), au-dessus de composants partagés et de jetons de couleur neufs. Les onglets disparaissent : un enum `Ecran` et une fenêtre « Menu » les remplacent. Le modèle (`Etat`) reçoit seulement de quoi afficher (historique, détail de ticket, heure, jour) ; **le format de fichier et le journal ne changent pas** (c'est l'étape B).

**Tech Stack :** Rust 1.93 (édition 2024), eframe/egui 0.35, egui_kittest 0.35, polices IBM Plex Sans (SIL OFL 1.1). Aucune dépendance nouvelle.

**Spec :** `docs/superpowers/specs/2026-10-21-kora-marche-pos-design.md` (§ 5 jetons, § 6 navigation, § 7.1 à 7.4, 7.6, 7.9, étape A du § 9.10). Les écrans de référence : `docs/superpowers/specs/2026-10-21-kora-marche-pos/maquette/` (`N-nom.png` = capture 1366 × 768, `N-nom.html` = dimensions, couleurs et textes exacts ; reprendre ces valeurs en points).

**Suite :** les étapes B (remises, code-barres, mobile money, code superviseur, mise en attente, format v3), C (caisse) et D (fidélité) auront chacune leur plan, écrit après celle-ci.

## Ce que l'étape A n'inclut PAS

Remise (F5), carte fidélité (F4), mise en attente (F8), code-barres, Mobile money, code superviseur, ouverture / dépôt / clôture / rapport, « Imprimer » / « Réimprimer », nom du magasin / caisse / caissier réglables, indicateur de caisse ouverte. Leurs boutons **n'apparaissent pas**. Les constantes de l'en-tête (« Kora Marché », caisse 1) sont dans `textes.rs` et deviennent des réglages à l'étape B.

## Global Constraints

- Rust 1.93, édition 2024 ; egui/eframe **0.35**, egui_kittest 0.35 ; aucune dépendance nouvelle.
- Tout le métier dans `gestion-stock` ; l'interface ne calcule rien d'autre que de l'affichage. La bibliothèque ne lit jamais l'heure.
- Montants en entiers, jamais de flottant ; pas de `unwrap`/`expect`/panique hors tests ; une opération qui échoue ne change rien.
- Tous les textes de la fenêtre dans `src/textes.rs`. Messages en français.
- **Contraste AA vérifié par les tests** : texte ≥ 4,5:1 sur son vrai fond (test `textes_peints` existant, conservé pour chaque écran), contours des **champs de saisie** ≥ 3:1. Le texte d'un contrôle grisé est dispensé.
- Chaque contrôle garde un **nom accessible** complet (`WidgetInfo`) et un rôle ; les écrans restent utilisables au clavier et à 200 % de zoom.
- Couleurs de la maquette (`§ 5` de la spec) : fond `#F2F3F5`, surface `#FFFFFF`, bord de carte `#E1E4E8`, texte `#16191D` / `#4A515B` / `#5B636E`, en-tête `#16191D` (sous-texte `#C4CAD3`), accent `#1E56C8` (survol `#163F94`, sélection `#EAF0FC`), succès `#12653F` sur `#E8F5EE` (bouton `#12784A`), alerte `#A61B12` sur `#FDECEA` (bord `#E8B4AE`, bouton danger `#B42318`), avertissement `#8A4106`, grisé `#6B7280` sur `#E3E6EA`, voile des fenêtres modales `#16191D` à 50 %.
- Fenêtre : **1366 × 768** au lancement, minimum **640 × 420** ; mise en page empilée sous **1100** points de large.
- Le programme n'ouvre **aucune connexion réseau** ; le ticket garde la mention « Ticket non fiscal : ce programme n'est pas une caisse certifiée. » (`MENTION_NON_CERTIFIE`).
- Vérifications avant chaque commit, dans `gestion-stock-gui/` : `cargo test`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check` ; dans `gestion-stock/` quand il est touché : les mêmes. Base de départ : **123 tests** verts dans `gestion-stock-gui`.

## Review Focus

Cinq cas que la spec laisse implicites et qui peuvent faire mal à une personne à la caisse (chacun a son test dans la tâche indiquée) :

1. **Texte de scan inattendu** : vide, espaces seuls, `0 x pain`, `1000 x pain`, `999 x pain` (plus que le stock), nom trouvé à plusieurs exemplaires, accents et majuscules. Aucun ne plante ni n'ajoute une mauvaise quantité (tâche 5 et 6).
2. **`Suppr` et `Échap` pendant la saisie** : le champ de scan a toujours le focus ; `Suppr` ne doit **jamais** supprimer un article quand on est en train d'effacer des caractères (tâche 6).
3. **Montant reçu absurde** : `abc`, `1e5`, `-5`, trop de décimales, un nombre énorme. « Valider » reste grisé, rien n'est écrit au journal, pas de panique (tâche 7).
4. **Rembourser deux fois / journal altéré** : un double clic ne crée qu'une écriture ; un journal modifié à la main grise « Rembourser » en le disant (tâche 9).
5. **Une journée sans vente, et des tickets d'autres jours** : l'historique vide un jour donné le dit, reste navigable, et un ticket d'hier reste remboursable par son numéro (tâche 9).

## Décisions de ce plan (elles précisent la spec, elles ne la contredisent pas)

- **Messages** : un `Retour::Succes` de l'état devient un message éphémère (3 s, en bas, au-dessus de la barre de fonctions) ; un `Retour::Erreur` reste un bandeau d'alerte sous l'en-tête jusqu'à la prochaine action réussie, et s'affiche aussi **dans** la fenêtre modale ouverte. Le pied de page disparaît ; le chemin du fichier de données (`pied_de_page_donnees`) se montre dans la fenêtre Menu et dans Réglages.
- **Barre de fonctions** : une grille dont le nombre de colonnes est le nombre de cases. À l'étape A, 4 cases : Rechercher F2, Supprimer l'article Suppr, Annuler la vente Échap, Menu.
- **Les trois boutons de paiement** : Espèces, Carte, **Autre** (le libellé « Mobile money » de la maquette arrive à l'étape B avec son moyen).
- **Raccourcis** : F2 ouvre la recherche. `Suppr` demande à supprimer l'article choisi **seulement si le champ de scan est vide**. `Échap` efface d'abord le champ de scan s'il contient du texte, sinon demande à annuler la vente ; dans une fenêtre modale, `Échap` la ferme.
- **Scan** : par nom seulement (les codes-barres arrivent à l'étape B). Un seul produit correspond (≥ 3 caractères, sans casse ni accents) → ajout ; plusieurs → l'écran Recherche s'ouvre avec le texte tapé ; aucun → fenêtre « Article introuvable ». Un préfixe de quantité `3 x`, `3*`, `3×` (1 à 3 chiffres ; `0` vaut 1).
- **Étiquette de TVA** : « TVA 5,5 % incluse : 0,19 € » (Europe) ; aux États-Unis « Taxe 8 % : … » comme aujourd'hui.
- **Recherche** : « Plus que N en stock » (ambre) pour 1 à 5 unités, « Rupture de stock » (rouge) à 0, sinon « En stock » (vert). Un produit sans prix : « Prix manquant », bouton « Indisponible » grisé.
- **Montants rapides** de l'espèces : « Exact » puis jusqu'à 3 arrondis supérieurs distincts ; pas en FCFA : 1 000, 5 000, 10 000, 20 000, 50 000 ; en EUR / USD / GBP (centimes) : 500, 1 000, 2 000, 5 000, 10 000. Le champ « Montant reçu » n'est pas reformaté pendant la frappe (limite d'egui) ; les montants formatés sont dans le bandeau dessous.
- **Historique** : pas de bouton « Réimprimer » (étape C). Numéro de ticket affiché `NN-NNNN` (`01-0001`) ; le numéro interne reste celui du journal. Les annulations sont des lignes « Remboursement » (ambre, total négatif).
- **Repère de produit** (images absentes) : carré arrondi `#F5F6F8` avec les initiales du produit.

## Structure des fichiers

| Fichier | Rôle |
|---|---|
| `gestion-stock/src/monnaie.rs` | + `Devise::{formater_nombre, symbole, symbole_avant}` publics |
| `gestion-stock-gui/assets/polices/IBMPlexSans-{Regular,Medium,SemiBold,Bold}.ttf`, `LICENCE-IBM-Plex-OFL-1.1.txt` | créés ; les fichiers Inter sont supprimés |
| `src/police.rs` | réécrit : `Graisse` |
| `src/palette.rs` | jetons de la maquette, `visuals()` |
| `src/textes.rs` | textes neufs, retrait de ceux des onglets |
| `src/scan.rs` | **nouveau** : analyse du texte de scan (sans fenêtre) |
| `src/etat.rs` | + heure et jour courants, recherche sans accents, historique, détail de ticket, message de remboursement |
| `src/ui.rs` | ne garde que `Application`, `Ecran`, `Modale`, `dessiner`, messages |
| `src/ui/composants.rs` | widgets partagés (cartes, champs, boutons, modale, repère) |
| `src/ui/entete.rs` | en-tête sombre et fenêtre Menu |
| `src/ui/vente.rs`, `paiement.rs`, `ticket.rs` | écran de vente ; fenêtres de paiement ; affichage d'un ticket |
| `src/ui/recherche.rs`, `historique.rs`, `stock.rs`, `reglages.rs` | les autres écrans |
| `src/ui/essais.rs` | aides de test (`#[cfg(test)]`) |
| `tests/apercu.rs` | captures 1366 × 768 par écran (outil, pas un test de non-régression) |

---

### Tâche 1 : Composants partagés et aides de test (refactor sans changement de comportement)

**Files :**
- Create : `gestion-stock-gui/src/ui/composants.rs`, `gestion-stock-gui/src/ui/essais.rs`
- Modify : `gestion-stock-gui/src/ui.rs`

**Interfaces :**
- Produces (`pub(crate)`, dans `composants`) : `carte`, `carte_marge`, `titre_de_carte`, `discret`, `puce`, `bandeau_alerte`, `champ`, `champ_en_ligne`, `Style`, `bouton`, `rempli`, `bouton_choix`, `gras`, `bouton_symbole` — signatures actuelles de `ui.rs`, inchangées.
- Produces (`pub(crate)`, `#[cfg(test)]`, dans `essais`) : `chemin_temp`, `harness`, `saisir`, `FIXE`, `etat_vendeur`, `app_vendeuse`, `app_vendeuse_avec_blocages`, `app_avec_ventes`, `app_journal_altere`, `harness_vente`, `FENETRE`, `cliquer`, `est_grise`, `nombre_de_pieces`, `quantite_en_stock`, `est_selectionne`, `TextePeint`, `textes_peints`, `verifier_contrastes`.

- [ ] **Step 1 :** Déclarer `mod composants;` et `#[cfg(test)] mod essais;` dans `ui.rs` (le fichier `ui.rs` et le dossier `ui/` coexistent, sans `mod.rs`). Déplacer dans `composants.rs`, en `pub(crate)`, les fonctions listées (aujourd'hui lignes 238-385, 435-454, 1413-1422 de `ui.rs`) ; `ui.rs` fait `use composants::*;`.
- [ ] **Step 2 :** Déplacer dans `essais.rs` les aides de test listées (aujourd'hui dans `mod tests` de `ui.rs`) ; `mod tests` fait `use super::essais::*;`. Les tests eux-mêmes ne bougent pas.
- [ ] **Step 3 :** `cd gestion-stock-gui && cargo test` — attendu : **123 passed**. Puis `cargo clippy --all-targets -- -D warnings` et `cargo fmt --check` : propres.
- [ ] **Step 4 :** Commit :

```bash
git add gestion-stock-gui/src
git commit -m "refactor(gui): composants partagés et aides de test dans des modules"
```

---

### Tâche 2 : Police IBM Plex Sans

**Files :**
- Create : `assets/polices/IBMPlexSans-{Regular,Medium,SemiBold,Bold}.ttf`, `assets/polices/LICENCE-IBM-Plex-OFL-1.1.txt`
- Delete : `assets/polices/Inter-Regular.ttf`, `Inter-SemiBold.ttf`, `LICENCE-Inter-OFL-1.1.txt`
- Modify : `src/police.rs`, `src/palette.rs` (police du titre), `src/ui.rs` et `src/ui/composants.rs` (appels à `famille_disponible`)

**Interfaces :**
- Produces (`police.rs`) : `enum Graisse { Normale, Moyenne, SemiGrasse, Grasse }` (400, 500, 600, 700) ; `fn famille(Graisse) -> FontFamily` ; `fn police(Graisse, f32) -> FontId` ; `fn famille_disponible(&Context, Graisse) -> FontFamily` (retombe sur `Proportional` si la famille n'est pas encore installée : première image) ; `fn definitions() -> FontDefinitions`. **Disparaissent** : `semi_gras`, `famille_semi_grasse`, `SEMI_GRAS`.

- [ ] **Step 1 :** Récupérer les polices (version officielle, SHA-256 de l'archive vérifié) :

```bash
cd /private/tmp/claude-501/-Users-cherifly-rust-apprentissage/c9caf10f-75ff-451b-89c1-31f3b29bf5e0/scratchpad/plex
curl -sL -o plex-sans.zip "https://github.com/IBM/plex/releases/download/%40ibm%2Fplex-sans%401.1.0/ibm-plex-sans.zip"
shasum -a 256 plex-sans.zip   # attendu : fb365d910566e6d199cc2c15579a7dd9a267128e18431a394ed81f1970c69200
unzip -oq plex-sans.zip 'ibm-plex-sans/fonts/complete/ttf/*' 'ibm-plex-sans/LICENSE.txt'
```

  Copier les quatre `.ttf` Regular, Medium, SemiBold, Bold dans `assets/polices/` et `LICENSE.txt` en `LICENCE-IBM-Plex-OFL-1.1.txt`. SHA-256 des fichiers (à écrire dans l'en-tête de `police.rs`, comme pour Inter) : Regular `975dcda3…afa5`, Medium `331c8639…f907`, SemiBold `a20caf82…7bb1`, Bold `9e6c74a8…fb2f` (valeurs complètes : recalculer avec `shasum -a 256`).
- [ ] **Step 2 : tests qui échouent** dans `police.rs` (remplacent ceux d'Inter) :

```rust
#[test]
fn plex_devient_la_police_par_defaut_et_garde_les_polices_d_egui_en_secours() {
    let d = definitions();
    let p = &d.families[&FontFamily::Proportional];
    assert_eq!(p[0], "IBM Plex Sans");
    assert!(p.len() > 1, "polices de secours : {p:?}");
}
#[test]
fn chaque_graisse_a_sa_famille_qui_retombe_sur_plex_regular() {
    let d = definitions();
    for g in [Graisse::Moyenne, Graisse::SemiGrasse, Graisse::Grasse] {
        let f = &d.families[&famille(g)];
        assert!(f.len() >= 2 && f.contains(&"IBM Plex Sans".to_string()));
    }
}
#[test]
fn les_quatre_polices_sont_de_vraies_polices_truetype() { /* 00 01 00 00, > 100_000 octets */ }
#[test]
fn les_chiffres_ont_tous_la_meme_largeur() {
    // Contexte egui + `set_fonts(definitions())`, puis mesurer "0".."9" avec
    // `ui.painter().layout_no_wrap(..).size().x` dans `ctx.run_ui`.
    // Attendu : 10 largeurs identiques (la maquette demande des chiffres de largeur fixe).
}
```

- [ ] **Step 3 :** `cargo test police` — attendu : échec de compilation (`Graisse` inconnu).
- [ ] **Step 4 :** Réécrire `police.rs` selon les signatures ci-dessus (`include_bytes!` des quatre fichiers ; une famille `Name("Plex-Medium")`, `"Plex-SemiBold"`, `"Plex-Bold"` par graisse, chacune suivie de `IBM Plex Sans` puis des polices d'egui). Remplacer les appels : `gras` (SemiGrasse), `chiffre`, `bouton_tuile`, `palette::appliquer` (titre : `police(Graisse::SemiGrasse, 22.0)`). Supprimer les fichiers Inter.
- [ ] **Step 5 :** `cargo test` — attendu : tout vert. **Si** `les_chiffres_ont_tous_la_meme_largeur` échoue (chiffres proportionnels), ne pas contourner : remplacer l'assertion par un commentaire daté et noter la limite dans le README (tâche 13) ; les montants sont déjà alignés à droite.
- [ ] **Step 6 :** Commit :

```bash
git add gestion-stock-gui
git commit -m "feat(gui): police IBM Plex Sans (quatre graisses) à la place d'Inter"
```

---

### Tâche 3 : Jetons de couleur de la maquette

**Files :**
- Modify : `src/palette.rs` (constantes, `visuals()`, `appliquer()`, tests), `src/ui.rs` et `src/ui/composants.rs` (cartes plates, rayons)

**Interfaces :**
- Produces (`palette.rs`, type `Rgb`) : conserve `FOND`, `SURFACE`, `TEXTE`, `TEXTE_DISCRET`, `FILET`, `ACCENT`, `ACCENT_SURVOL`, `ACCENT_TEXTE`, `ACCENT_DOUX`, `FOCUS`, `ALERTE_*`, `SUCCES_*`, `DANGER`, `BOUTON_*`, `SELECTION_*`, avec les **valeurs de la maquette** ; ajoute : `TEXTE_SECONDAIRE` (`4A515B`), `FOND_DOUX` (`F5F6F8`), `FILET_LIGNE` (`EEF0F3`), `BORDURE_BOUTON` (`D5DAE0`, décoratif), `ENTETE_FOND` (`16191D`), `ENTETE_TEXTE` (`FFFFFF`), `ENTETE_DISCRET` (`C4CAD3`), `SUCCES_BOUTON` (`12784A`), `ALERTE_BORD` (`E8B4AE`), `AVERTISSEMENT_TEXTE` (`8A4106`, remplace `SORTIE_TEXTE`), `GRISE_FOND` (`E3E6EA`), `GRISE_TEXTE` (`6B7280`), `VOILE` (`Color32` 22,25,29 à 50 %). `BORDURE` (contour des **champs**) devient `8B929C` (3,14:1 sur blanc : `AEB5BF` de la maquette n'atteint que 2,07:1, on assombrit le minimum). egui applique les mêmes `visuals` aux champs et aux boutons : `visuals().widgets.inactive.bg_stroke` prend `BORDURE` (champs) et `Style::Secondaire` de `composants::bouton` fixe **explicitement** `BORDURE_BOUTON` (`D5DAE0`).
- `ombre()` n'est plus utilisée (cartes plates : bord 1 `FILET`, rayon 12).

- [ ] **Step 1 : tests qui échouent** dans `palette.rs` (adapter la table existante) :
  - `les_textes_de_la_palette_atteignent_4_5` : paires (texte, fond) — `TEXTE`/`SURFACE`, `TEXTE`/`FOND`, `TEXTE_SECONDAIRE`/`SURFACE`, `TEXTE_DISCRET`/`SURFACE`, `TEXTE_DISCRET`/`FOND`, `TEXTE_DISCRET`/`ACCENT_DOUX`, `ENTETE_TEXTE`/`ENTETE_FOND`, `ENTETE_DISCRET`/`ENTETE_FOND`, `ACCENT`/`SURFACE`, `ACCENT`/`ACCENT_DOUX`, `ACCENT_TEXTE`/`ACCENT`, `SUCCES_TEXTE`/`SURFACE`, `SUCCES_TEXTE`/`SUCCES_FOND`, `ALERTE_TEXTE`/`SURFACE`, `ALERTE_TEXTE`/`ALERTE_FOND`, `AVERTISSEMENT_TEXTE`/`SURFACE`, `ACCENT_TEXTE`/`DANGER`, `ACCENT_TEXTE`/`SUCCES_BOUTON`. Rapports attendus (calculés) : ≥ 5,3 pour toutes.
  - `les_contours_de_champs_et_le_focus_atteignent_3`: `BORDURE`/`SURFACE` ≥ 3,0 ; `FOCUS`/`SURFACE` ≥ 3,0.
  - `les_valeurs_de_la_maquette_sont_reprises`: `FOND == [242,243,245]`, `ACCENT == [30,86,200]`, `ENTETE_FOND == [22,25,29]`.
- [ ] **Step 2 :** `cargo test palette` — attendu : échecs sur les valeurs.
- [ ] **Step 3 :** Mettre à jour les constantes ; `visuals()` : rayons de widgets 10, fenêtres 16, `window_stroke` `FILET`, plus d'ombre ; `appliquer()` : `interact_size.y = 48`, `button_padding (16, 10)`, styles de texte Heading `police(SemiGrasse, 22)`, Body/Button 16, Small 14. `carte`/`carte_marge` : rayon 12, plus d'ombre, marge par défaut 16.
- [ ] **Step 4 :** `cargo test` — attendu : tout vert (les tests de contraste de chaque écran, inchangés, valident la palette sur l'**ancienne mise en page**). Corriger toute paire qui échoue en assombrissant le minimum, et la noter pour le README.
- [ ] **Step 5 :** Générer un aperçu et le regarder : `APERCU_DOSSIER=<scratchpad>/apercu-t3 UPDATE_SNAPSHOTS=true cargo test --test apercu apercu_vente_panier -- --ignored`, puis lire l'image : fond gris `#F2F3F5`, cartes plates, bleu `#1E56C8`, police Plex.
- [ ] **Step 6 :** Commit :

```bash
git add gestion-stock-gui
git commit -m "feat(gui): couleurs de la maquette Kora Marché et cartes plates"
```

---

### Tâche 4 : Cadre de l'application — écrans, en-tête, Menu, messages

**Files :**
- Create : `src/ui/entete.rs`
- Modify : `src/ui.rs`, `src/textes.rs`, `tests/apercu.rs` (renommages seulement), `src/ui/composants.rs` (+ `modale`)

**Interfaces :**
- Produces (`ui.rs`) :

```rust
pub enum Ecran { #[default] Vente, Recherche, Historique, Stock, Reglages }
pub enum Modale { Menu /* les autres variantes sont ajoutées par les tâches 6 à 9 */ }
pub struct Toast { pub texte: String, pub depuis: Option<f64> }  // `depuis` : posé au premier affichage (heure egui)
// Application : `onglet` devient `ecran: Ecran` ; + `modale: Option<Modale>`, `toast: Option<Toast>`.
impl Application { pub fn message(&mut self, texte: impl Into<String>) }  // pose un toast
```
- Produces (`composants.rs`) : `fn modale<R>(ctx: &Context, id: &str, largeur: f32, nom_accessible: &str, contenu: impl FnOnce(&mut Ui) -> R) -> ModaleReponse<R>` (enveloppe `egui::Modal` : voile `VOILE`, cadre blanc, rayon 16, marge 28, `largeur`, `ferme: bool` si `Échap` ou clic sur le voile).
- Produces (`entete.rs`) : `fn entete(ui: &mut Ui, app: &mut Application)` ; `fn fenetre_menu(ctx: &Context, app: &mut Application)`.
- Produces (`etat.rs`) : `pub fn heure_locale(&self) -> String` (`"23:13"`, depuis l'horloge de l'état).
- Produces (`textes.rs`) : `NOM_MAGASIN = "Kora Marché"`, `NUMERO_CAISSE: u32 = 1`, `fn libelle_caisse() -> String` (`"Caisse 01"`), `fn numero_de_ticket(numero: u64) -> String` (`format!("{:02}-{:04}", NUMERO_CAISSE, numero)`), `BOUTON_RETOUR_VENTE = "Retour à la vente"`, `BOUTON_MENU = "Menu"`, `TITRE_MENU = "Menu"`, `ECRAN_HISTORIQUE = "Historique"`, `ECRAN_STOCK = "Stock"`, `ECRAN_REGLAGES = "Réglages"`, `BOUTON_FERMER = "Fermer"`. `TITRE_FENETRE` devient `"Kora Marché — Caisse"`. Les textes d'onglets (`ONGLET_*`, `NAVIGATION`) disparaissent.

- [ ] **Step 1 : tests qui échouent** (dans `ui.rs`, `mod tests`) :

```rust
#[test]
fn l_en_tete_montre_le_magasin_la_caisse_et_l_heure() {
    let h = harness_vente(app_vendeuse("entete"), FENETRE);
    h.get_by_label("Kora Marché");
    h.get_by_label("Caisse 01");
    h.get_by_label("23:13");               // horloge fixe : 14/11/2023 23:13 local
    assert_eq!(h.state().etat.as_ref().unwrap().heure_locale(), "23:13");
}
#[test]
fn le_menu_mene_aux_ecrans_et_le_retour_ramene_a_la_vente() { /* clic « Menu » (bouton TEMPORAIRE de l'en-tête de vente) ;
   boutons Historique, Stock, Réglages présents ; clic « Stock » → ecran == Stock, modale == None ;
   clic « Retour à la vente » → ecran == Vente ; idem depuis Réglages et Historique */ }
#[test]
fn un_succes_devient_un_message_ephemere_et_une_erreur_un_bandeau() { /* écran Stock : « Enregistrer une entrée » valide
   → le texte du retour est dans `app.toast`, et `etat.retour` est None ; une sortie trop grande → bandeau
   « Erreur : … » toujours présent à l'image suivante */ }
#[test]
fn tous_les_textes_peints_de_l_en_tete_et_du_menu_atteignent_4_5() { /* `textes_peints` sur Vente et sur Menu ouvert */ }
```

  Adapter aussi les tests qui citent `Onglet` : `Onglet::Stock → Ecran::Stock`, `Onglet::Ventes → Ecran::Historique`, `Onglet::Reglages → Ecran::Reglages` ; **supprimer** `les_onglets_sont_annonces_avec_leur_nom_et_leur_role`, `les_quatre_onglets_s_affichent_et_se_selectionnent`, `les_quatre_onglets_tiennent_dans_une_fenetre_etroite` (remplacés par les tests ci-dessus) ; les tests qui cliquent un onglet passent par `app.ecran = …`.
- [ ] **Step 2 :** `cargo test` — attendu : échec de compilation puis échecs des nouveaux tests.
- [ ] **Step 3 :** Implémenter. `dessiner` : en-tête sombre (hauteur 56, fond `ENTETE_FOND`, marge 20 ; gauche : `NOM_MAGASIN` 18 gras blanc puis `libelle_caisse()` 14 `ENTETE_DISCRET` ; droite : heure 18/600 (`Etat::heure_locale`), sauf sur Historique ; bouton blanc « Retour à la vente » (40 de haut, rayon 8) sur tous les écrans sauf Vente) ; **supprimer** la barre d'onglets, les trois « chiffres » restent dans l'écran Stock ; bandeau d'alerte pour `Retour::Erreur` sous l'en-tête ; transfert d'un `Retour::Succes` vers `app.toast` ; affichage du toast (pastille `#16191D`, texte blanc 15, centrée, 104 au-dessus du bas, 3 s, `request_repaint_after`). Pied de page supprimé. L'écran Vente reçoit un bouton « Menu » **temporaire** dans son en-tête (retiré à la tâche 6). `fenetre_menu` : titre, trois boutons de 56 de haut (Historique, Stock, Réglages — chacun change `ecran` et ferme), « Fermer », et la ligne « Données : chemin » en texte discret. Les aperçus de `tests/apercu.rs` : seulement les renommages `Onglet → Ecran` et `pixels_per_point = 1.0`.
- [ ] **Step 4 :** `cargo test` — attendu : tout vert.
- [ ] **Step 5 :** Commit :

```bash
git add gestion-stock-gui
git commit -m "feat(gui): en-tête Kora Marché, écrans, fenêtre Menu et messages éphémères"
```

---

### Tâche 5 : Briques sans fenêtre — montants, scan, jour, recherche

**Files :**
- Modify : `gestion-stock/src/monnaie.rs`, `gestion-stock-gui/src/etat.rs`, `src/lib.rs`, `src/textes.rs`
- Create : `gestion-stock-gui/src/scan.rs`

**Interfaces :**
- Produces (`gestion_stock::monnaie::Devise`) : `pub fn formater_nombre(self, Montant) -> String` (nombre avec séparateurs de milliers, **sans** symbole : `17 025` avec espace insécable, `4,90`, `2,59` / `$` en `2.59`), `pub fn symbole(self) -> &'static str`, `pub fn symbole_avant(self) -> bool`.
- Produces (`scan.rs`) :

```rust
pub enum ResultatScan {
    Vide,
    Ajouter { nom: String, quantite: u32 },
    Plusieurs,            // plusieurs produits correspondent : ouvrir la recherche
    Introuvable(String),  // texte nettoyé, pour le message
}
pub fn analyser(saisie: &str, produits: &[LigneProduit]) -> ResultatScan
```

- Produces (`etat.rs`) : `pub fn jour_courant(&self) -> String` (`jour_iso` : `"2023-11-14"`) ; `pub fn recherche_articles(&self, saisie: &str) -> Vec<LigneProduit>` (sous-chaîne, via `catalogue::normaliser` : sans casse ni accents ; saisie vide → tous).
- Produces (`textes.rs`) : `fn compteur_articles(n: usize) -> String` (`"0 article"`, `"1 article"`, `"2 articles"`), `libelle_taxe(Europe, taux)` = `"TVA {taux} incluse"` (les États-Unis gardent `"Taxe {taux}"`).

- [ ] **Step 1 : tests qui échouent.**
  - `monnaie.rs` : `formater_nombre` — `Xof`, 17 025 → `"17\u{a0}025"` ; `Eur`, 490 → `"4,90"` ; `Usd`, 259 → `"2.59"` ; `Usd`, 123_456 → `"1,234.56"` ; `symbole(Xof) == "FCFA"` ; `symbole_avant(Usd) == true`, `(Eur) == false`.
  - `scan.rs` (produits : Pain, Savon, Lait, Laitue, Thé vert ; tous vendables ou non, l'analyse ne regarde pas le stock) :

```rust
#[test] fn un_texte_vide_ou_fait_d_espaces_est_vide() { /* "" et "   " → Vide */ }
#[test] fn un_nom_unique_s_ajoute_sans_casse_ni_accents() { /* "pain", "PAIN", "the" → Ajouter{ Pain|Thé vert, 1 } */ }
#[test] fn le_prefixe_de_quantite_se_lit() { /* "3 x pain", "3*pain", "3×pain", "3 X pain" → quantite 3 */ }
#[test] fn zero_vaut_un_et_quatre_chiffres_ne_sont_pas_une_quantite() {
    /* "0 x pain" → quantite 1 ; "1000 x pain" → Introuvable("1000 x pain") ; "999 x pain" → quantite 999 */ }
#[test] fn moins_de_trois_caracteres_ou_aucun_produit_est_introuvable() { /* "pa" → Introuvable("pa") ; "zzz" → Introuvable("zzz") */ }
#[test] fn plusieurs_produits_correspondants_ouvrent_la_recherche() { /* "lai" (Lait, Laitue) → Plusieurs */ }
```

  - `etat.rs` : `jour_courant` = `"2023-11-14"` avec l'horloge fixe ; `recherche_articles("the")` trouve « Thé vert » ; `recherche_articles("")` renvoie tous les produits.
- [ ] **Step 2 :** `cd gestion-stock && cargo test monnaie` puis `cd ../gestion-stock-gui && cargo test` — attendu : échecs.
- [ ] **Step 3 :** Implémenter : `formater_nombre` réutilise `nombre(montant, true)` en rendant publique la mise en forme sans symbole ; `analyser` : trim ; préfixe `^(\d{1,3})\s*[*xX×]\s*(.+)$` lu à la main (pas de dépendance) ; `quantite 0 → 1` ; comparaison par `catalogue::normaliser`, ≥ 3 caractères, **un** produit → `Ajouter`, plusieurs → `Plusieurs`, sinon `Introuvable`. Déclarer `pub mod scan;` dans `lib.rs`.
- [ ] **Step 4 :** Dans `gestion-stock` : `cargo test`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check`. Dans `gestion-stock-gui` : idem. Les tests qui citent « dont TVA … » dans `ui.rs` sont mis à jour vers « TVA … incluse ».
- [ ] **Step 5 :** Commit :

```bash
git add gestion-stock gestion-stock-gui
git commit -m "feat: montants en grand, analyse du scan, jour courant et recherche sans accents"
```

---

### Tâche 6 : Écran de vente

> L'application ne sait pas encaisser entre cette tâche et la suivante : ne rien publier entre les deux.

**Files :**
- Create : `src/ui/vente.rs`
- Modify : `src/ui.rs` (`Application` : `scan: String`, `ligne_choisie: Option<String>`, `focus_scan: bool` ; `Modale::Confirmer(Confirmation)`, `Modale::Introuvable(String)` ; supprimer `recherche_vente`, `annulation_en_attente`), `src/ui/composants.rs` (`bouton_a_lignes`, `repere`), `src/textes.rs`, `tests/apercu.rs`
- Supprimer de `ui.rs` : `ecran_vente` (ancien), `tuiles`, `bouton_tuile`, `tuile`, `carte_produits`, `colonne_panier`, `carte_panier`, `carte_paiement`, `panier_empile`, `lignes_du_panier`, `ligne_du_panier`, `paiement`, `ticket` et leurs tests de tuiles

**Interfaces :**
- Produces (`composants.rs`) :

```rust
pub(crate) struct LigneBouton<'a> { pub texte: &'a str, pub taille: f32, pub graisse: Graisse, pub couleur: Rgb }
pub(crate) fn bouton_a_lignes(ui: &mut Ui, nom_accessible: &str, taille: Vec2,
    fond: Rgb, contour: Stroke, rayon: u8, lignes: &[LigneBouton], actif: bool) -> Response
    // dessiné à la main avec des galleys (comme l'ancien `bouton_tuile`) ; vrai contrôle : focus clavier
    // avec contour `FOCUS` de 2, curseur main, `WidgetInfo::labeled(Button, actif, nom_accessible)`.
pub(crate) fn repere(ui: &mut Ui, nom: &str, cote: f32)  // initiales (2 lettres max), fond FOND_DOUX, rayon 8
```

- Produces (`ui.rs`) : `pub enum Confirmation { AnnulerVente, SupprimerArticle }`.
- Produces (`vente.rs`) : `pub(super) fn ecran_vente(ui: &mut Ui, app: &mut Application)` ; `pub(super) fn ajouter_au_ticket(app: &mut Application, nom: &str, quantite: u32) -> bool` (vérifie prix, stock et ce qui est déjà au ticket ; en cas de refus pose un toast et renvoie `false`) ; `pub(super) fn raccourcis(ui: &mut Ui, app: &mut Application)`.
- Produces (`textes.rs`) : `CHAMP_SCAN = "Code-barres ou nom du produit"`, `PLACEHOLDER_SCAN = "Scannez un article ou tapez son nom"`, `TICKET_VIDE_TITRE = "Prêt pour une nouvelle vente"`, `TICKET_VIDE_AIDE = "Scannez le premier article pour commencer."`, `LIBELLE_TOTAL_GRAND = "TOTAL"`, `BOUTON_ESPECES = "Espèces"`, `BOUTON_CARTE = "Carte"`, `BOUTON_AUTRE = "Autre"`, `FONCTION_RECHERCHER = "Rechercher"`, `FONCTION_SUPPRIMER = "Supprimer l'article"`, `FONCTION_ANNULER = "Annuler la vente"`, `FONCTION_MENU = "Menu"`, `fn articles_au_ticket(n: u32) -> String` (`"16 articles"`, `"1 article"`), `TOAST_AUCUNE_VENTE = "Aucune vente en cours"`, `TOAST_VENTE_ANNULEE = "Vente annulée"`, `fn confirmation_titre(Confirmation)`, `fn confirmation_texte(Confirmation, n: u32, nom: &str)` (`"Tous les articles (7) seront retirés du ticket."`, `"Pain sera retiré du ticket."`), `OUI_ANNULER = "Oui, annuler"`, `OUI_SUPPRIMER = "Oui, supprimer"`, `NON_REVENIR = "Non, revenir"`, `INTROUVABLE_TITRE = "Article introuvable"`, `fn introuvable_texte(saisie: &str)` (`"Aucun article ne correspond à « zzz ». Scannez à nouveau ou recherchez l'article par son nom."`), `SCANNER_A_NOUVEAU = "Scanner à nouveau"`, `fn stock_limite(nom: &str, disponible: u32)` (`"Pain : il n'en reste que 10."`), `fn detail_de_ligne(quantite: u32, prix: &str)` (`"2 × 1,20 €"`), `fn fonction_nom(libelle: &str, raccourci: &str)` (nom accessible `"Rechercher (F2)"` ; « Menu » n'a pas de raccourci et garde le nom `"Menu"`).

Dimensions et styles : reprendre **telles quelles** celles de `maquette/1-vente.html` (en-tête 56, marges et écarts 16, colonne de paiement 420, champ de scan 56 avec contour de 2 en accent et rayon 10, ligne de ticket 68 minimum avec repère 52, boutons − et + 44 × 44, total 64/700, Espèces 84 de haut, Carte et Autre 72, barre de fonctions : cases de 64 de haut, écart 10, libellé 15/600 et raccourci 12/500 discret ; « Annuler la vente » en rouge `ALERTE_TEXTE` avec bord `ALERTE_BORD`).

- [ ] **Step 1 : tests qui échouent** (dans `vente.rs`, `mod tests`, avec `essais::*`) :

```rust
#[test]
fn scanner_un_nom_ajoute_une_unite_choisit_la_ligne_et_vide_le_champ() {
    let mut h = harness_vente(app_vendeuse("scan"), FENETRE);
    saisir(&mut h, CHAMP_SCAN, "pain");
    h.key_press(Key::Enter); h.run();
    assert_eq!(h.state().panier, [("Pain".to_string(), 1)]);
    assert_eq!(h.state().ligne_choisie.as_deref(), Some("Pain"));
    assert!(h.state().scan.is_empty());
}
```
  - `un_prefixe_de_quantite_multiplie` : `"3 x pain"` → `("Pain", 3)`.
  - `un_article_inconnu_ouvre_la_fenetre_introuvable` : `"zzz"` → `Modale::Introuvable("zzz")`, le texte `introuvable_texte("zzz")` est affiché ; « Scanner à nouveau » ferme ; « Rechercher » → `ecran == Recherche`.
  - `plusieurs_correspondances_ouvrent_la_recherche_avec_le_texte` : `"lai"` avec Lait et Laitue → `ecran == Recherche`, `recherche_articles == "lai"`.
  - `un_produit_epuise_ou_sans_prix_ne_s_ajoute_pas_et_dit_pourquoi` (`app_vendeuse_avec_blocages`) : `"sel"` → panier vide, toast `produit_non_vendable("Sel", EPUISE)` ; `"thé"` → toast `produit_non_vendable("Thé", PRIX_MANQUANT)`.
  - `on_ne_peut_pas_ajouter_plus_que_le_stock` : `"11 x pain"` → panier vide, toast `stock_limite("Pain", 10)` ; `"999 x pain"` idem ; avec `("Savon", 10)` déjà au ticket, `"savon"` → toast `stock_limite("Savon", 10)`.
  - `les_lignes_montrent_nom_detail_et_total` : Pain × 2 → étiquettes `"Pain"`, `"2 × 1,20 €"` (nbsp), `"2,40 €"`.
  - `une_seule_ligne_est_choisie_et_montre_moins_et_plus` : deux lignes, clic sur « Pain » → boutons `retirer_une_unite("Pain")` et `ajouter_une_unite("Pain")` présents, aucun pour Savon ; `+` grisé quand la quantité atteint le stock (port de `on_ne_peut_pas_mettre_au_panier_plus_que_le_stock`).
  - `le_total_en_grand_la_tva_incluse_et_le_sous_total` : Pain × 3 + Savon × 1 → texte `"6,10"` peint en 64 et `"€"` peint à part ; `"TVA 5,5 % incluse : 0,19 €"` et `"TVA 20 % incluse : 0,42 €"` (nbsp avant `%` et `€`). Aux États-Unis (port de `le_ticket_detaille_la_taxe_aux_etats_unis`, Pain 1.20 × 2) : `"Sous-total : $2.40"`, `"Taxe 8 % : $0.19"` avant le total `"2.59"`.
  - `les_boutons_de_paiement_sont_grises_sans_article_et_actifs_avec` : `est_grise(BOUTON_ESPECES|BOUTON_CARTE|BOUTON_AUTRE)` vrai ticket vide, faux après un ajout.
  - `un_ticket_devenu_invalide_bloque_le_paiement_et_le_dit` (port de `un_panier_devenu_invalide_est_signale_et_ne_s_encaisse_pas`, panier `("Fantôme", 1)`) : `PANIER_INVALIDE` affiché, les trois boutons grisés.
  - `la_barre_de_fonctions_a_quatre_cases_avec_leur_raccourci` : boutons `"Rechercher (F2)"`, `"Supprimer l'article (Suppr)"`, `"Annuler la vente (Échap)"`, `"Menu"` ; « Supprimer l'article » grisé sans ligne choisie.
  - `f2_ouvre_la_recherche` : `key_press(Key::F2)` → `ecran == Recherche`.
  - **Review Focus 2** `suppr_ne_supprime_un_article_que_si_le_champ_de_scan_est_vide` : ligne choisie, champ contenant `"ab"` → `key_press(Key::Delete)` → panier inchangé, `modale == None` ; champ vidé → `Delete` → `modale == Some(Confirmer(SupprimerArticle))`.
  - `echap_efface_d_abord_la_saisie_puis_propose_d_annuler_la_vente` : champ `"ab"` + `Échap` → champ vide, pas de modale ; `Échap` encore → `Confirmer(AnnulerVente)` ; ticket vide + `Échap` → toast `TOAST_AUCUNE_VENTE`.
  - `annuler_la_vente_vide_le_ticket_apres_confirmation_et_n_ecrit_rien_au_journal` : « Non, revenir » garde le ticket ; « Oui, annuler » → panier vide, `nombre_de_pieces == 0`, toast `TOAST_VENTE_ANNULEE`. Même chose pour « Supprimer l'article » (seule la ligne choisie part).
  - `un_nom_de_produit_tres_long_ne_chevauche_pas_le_total` : produit de 120 caractères ; le rectangle du nom finit avant celui du total de ligne.
  - `tous_les_textes_peints_de_l_ecran_vente_atteignent_4_5` (port) : ticket rempli, une ligne choisie, `verifier_contrastes` ; puis avec la fenêtre de confirmation ouverte.
- [ ] **Step 2 :** `cargo test` — attendu : échec de compilation.
- [ ] **Step 3 :** Implémenter `bouton_a_lignes` et `repere` ; puis `vente.rs` : mise en page à deux colonnes ≥ 1100 points (le mode étroit est la tâche 11) avec `Panel::top` (en-tête), `Panel::bottom` (barre de fonctions, marges 16), `Panel::right` (`exact_size(420)`, `resizable(false)`, aucun trait de séparation) et `CentralPanel` ; champ de scan : `TextEdit` plein, `labelled_by`, focus demandé quand `app.focus_scan` (au premier affichage et après toute action ; jamais pendant une fenêtre modale), validation par `lost_focus() && Entrée` puis `scan::analyser` ; ligne entière cliquable (`Sense::click` déclaré **avant** les boutons − et +, pour que ceux-ci gardent leurs clics) ; le bandeau d'intégrité du journal et `PANIER_INVALIDE` comme aujourd'hui ; `raccourcis` lit F2, Suppr (champ de scan vide seulement) et Échap via `ctx.input_mut(|i| i.consume_key(Modifiers::NONE, …))`, **hors** fenêtre modale ; confirmations et « Article introuvable » via `composants::modale` (largeur 460). Les boutons de paiement posent `Modale::Especes` / `Terminal` (tâche 7). Retirer le bouton « Menu » temporaire de l'en-tête (la case « Menu » de la barre ouvre `Modale::Menu`).
- [ ] **Step 4 :** `cargo test` — attendu : tout vert. Clippy et fmt propres.
- [ ] **Step 5 : contrôle visuel.** Réécrire dans `tests/apercu.rs` `apercu_vente_vide` et `apercu_vente_panier` à 1366 × 768 (ticket de 7 lignes comme la maquette, une ligne choisie), les générer, **lire** les PNG et les comparer à `maquette/1-vente.png` : en-tête, cartes, total en grand, bouton Espèces bleu plein, barre de fonctions. Corriger les écarts visibles (espacements, graisses, couleurs).
- [ ] **Step 6 :** Commit :

```bash
git add gestion-stock-gui
git commit -m "feat(gui): écran de vente Kora Marché (scan, ticket, totaux, barre de fonctions)"
```

---

### Tâche 7 : Paiement — espèces, carte ou autre, vente terminée, ticket

**Files :**
- Create : `src/ui/paiement.rs`, `src/ui/ticket.rs`
- Modify : `src/ui.rs` (`Modale::{Especes, Terminal { moyen: Moyen, refuse: bool }, VenteTerminee { ticket_visible: bool }}`), `src/etat.rs`, `src/textes.rs`, `src/ui/vente.rs`, `tests/apercu.rs`

**Interfaces :**
- Consumes : `composants::modale`, `Etat::{reglement_especes, encaisser, derniere_piece, detail_taxes, effacer_ticket}`, `Devise::{formater_nombre, symbole}`.
- Produces (`paiement.rs`) : `pub(super) fn fenetre_paiement(ctx: &Context, app: &mut Application)` (dessine la modale courante parmi les trois) ; `pub(crate) fn montants_rapides(devise: Devise, total: Montant) -> Vec<Montant>`.
- Produces (`etat.rs`) :

```rust
pub enum StatutPiece { Payee /* vente non annulée */, Remboursee /* vente annulée */, Remboursement /* pièce d'annulation */ }
pub struct LigneDetail { pub quantite: u32, pub nom: String, pub total: String }
pub struct DetailTicket {
    pub ticket: String,            // numero_de_ticket(numero)
    pub quand: String,             // "14/11/2023 23:13"
    pub moyen: Moyen,
    pub statut: StatutPiece,
    pub articles: u32,
    pub lignes: Vec<LigneDetail>,
    pub taxes: Vec<String>,        // « TVA 5,5 % incluse : 0,19 € » ou « Taxe 8 % : $0.19 », sans le total
    pub sous_total: Option<String>,// profil États-Unis seulement
    pub total: String,
    pub recu: Option<String>, pub rendu: Option<String>,   // espèces seulement
}
pub fn detail_ticket(&self, numero: u64) -> Option<DetailTicket>
```

- Produces (`ticket.rs`) : `pub(super) fn corps_du_ticket(ui: &mut Ui, detail: &DetailTicket, avec_mention: bool)` (lignes avec repère 40, quantité × nom, total ; filets pointillés ; taxes ; total 28 ; « Payé par », reçu, rendu ; `MENTION_NON_CERTIFIE` si `avec_mention`).
- Produces (`textes.rs`) : `ESPECES_TITRE = "Paiement en espèces"`, `A_PAYER = "À payer"`, `CHAMP_RECU = "Montant reçu"` (existe), `QUICK_EXACT = "Exact"`, `MONNAIE_A_RENDRE = "Monnaie à rendre"`, `IL_MANQUE = "Il manque"`, `BOUTON_RETOUR = "Retour"`, `BOUTON_VALIDER = "Valider"`, `CARTE_TITRE = "Paiement par carte"`, `AUTRE_TITRE = "Autre moyen de paiement"`, `CARTE_ATTENTE = "Présentez la carte sur le terminal"`, `CARTE_AIDE = "Sans contact, puce ou piste. Le montant est à saisir sur le terminal."`, `AUTRE_ATTENTE = "Le client règle autrement (mobile money, chèque, virement…)."`, `BOUTON_ACCEPTE = "Paiement accepté"`, `BOUTON_REFUSE = "Paiement refusé"`, `BOUTON_RECU = "Paiement reçu"`, `CARTE_REFUSEE = "Carte refusée"`, `CARTE_REFUSEE_AIDE = "Aucun montant n'a été débité. Proposez un autre moyen de paiement."`, `BOUTON_REESSAYER = "Réessayer"`, `BOUTON_PAYER_ESPECES = "Payer en espèces"`, `VENTE_TERMINEE = "Vente terminée"`, `fn vente_terminee_detail(moyen: &str, total: &str)` (`"Carte · 3,60 € · ticket enregistré"`), `BOUTON_VOIR_TICKET = "Voir le ticket"`, `BOUTON_CACHER_TICKET = "Cacher le ticket"` ; `BOUTON_NOUVELLE_VENTE` (existe).

- [ ] **Step 1 : tests qui échouent.**
  - `paiement.rs` (unités) :

```rust
#[test]
fn les_montants_rapides_sont_exact_puis_trois_arrondis_distincts() {
    assert_eq!(montants_rapides(Devise::Xof, Montant(17_025)),
               [17_025, 18_000, 20_000, 50_000].map(Montant));
    assert_eq!(montants_rapides(Devise::Eur, Montant(490)),
               [490, 500, 1_000, 2_000].map(Montant));
    // un total déjà rond n'est pas répété :
    assert_eq!(montants_rapides(Devise::Xof, Montant(1_000))[0], Montant(1_000));
    assert!(montants_rapides(Devise::Xof, Montant(1_000)).windows(2).all(|w| w[0] < w[1]));
}
```

  - `etat.rs` : `detail_ticket` d'une vente Pain × 3 + Savon × 1 (carte) → `ticket == "01-0001"`, `quand == "14/11/2023 23:13"`, `articles == 4`, `taxes == ["TVA 5,5 % incluse : 0,19 €", "TVA 20 % incluse : 0,42 €"]` (nbsp), `total == "6,10 €"`, `recu == None` ; espèces : `recu == Some("10,00 €")`, `rendu == Some("3,90 €")` ; numéro inconnu → `None`.
  - UI (`paiement.rs`, `mod tests`) :
    - `payer_en_especes_montre_le_rendu_puis_la_vente_terminee` : Pain × 2 + Savon × 1 (total 4,90 €) ; clic « Espèces » → fenêtre `ESPECES_TITRE` ; `saisir(CHAMP_RECU, "10")` → `"Monnaie à rendre"` et `"5,10 €"` ; clic « Valider » → 1 pièce, `Modale::VenteTerminee`, panier et reçu vides, `"5,10 €"` en grand ; « Nouvelle vente » → `modale == None`, `etat.derniere_piece().is_none()`, `nombre_de_pieces == 1`.
    - `valider_est_grise_si_le_recu_ne_couvre_pas_le_total` (port de `le_bouton_encaisser_est_grise_…`) : reçu `"2"` → `"Il manque"` + `"2,90 €"`, « Valider » grisé ; aucune pièce.
    - **Review Focus 3** `un_montant_absurde_est_signale_sans_rien_ecrire` : pour `"abc"`, `"1e5"`, `"-5"`, `"12,345"`, `"99999999999999999999"` → `RECU_ILLISIBLE` affiché (ou le message d'erreur de la bibliothèque pour le trop grand), « Valider » grisé, 0 pièce, pas de panique.
    - `un_double_clic_sur_valider_n_enregistre_qu_une_vente` (port de `un_double_clic_sur_encaisser_…`).
    - `un_montant_rapide_remplit_le_recu` : clic « Exact » → `saisie_recu == "4,90"`.
    - `par_carte_le_caissier_confirme_a_la_main` : clic « Carte » → `CARTE_TITRE`, `CARTE_ATTENTE`, le total, **aucun champ de montant reçu** ; clic « Paiement accepté » → 1 pièce `Moyen::Carte`, `VenteTerminee`.
    - `une_carte_refusee_ne_vend_rien_et_propose_de_reessayer_ou_d_encaisser_en_especes` : « Paiement refusé » → `CARTE_REFUSEE`, `CARTE_REFUSEE_AIDE`, 0 pièce ; « Réessayer » → retour à `CARTE_ATTENTE` ; « Payer en espèces » → `Modale::Especes`.
    - `autre_moyen_se_confirme_par_paiement_recu` : clic « Autre » → `AUTRE_TITRE` ; « Paiement reçu » → 1 pièce `Moyen::Autre`.
    - `echap_ferme_la_fenetre_sans_rien_enregistrer` et `retour_ferme_la_fenetre`.
    - `une_erreur_de_la_bibliotheque_s_affiche_dans_la_fenetre` : stock modifié entre-temps (sortie du produit depuis l'état) puis « Valider » → bandeau « Erreur : … » dans la fenêtre, ticket conservé, 0 pièce.
    - `le_journal_altere_bloque_le_paiement_et_le_dit` (port de `un_journal_altere_affiche_un_bandeau_…`, partie vente, avec `app_journal_altere` déplacé dans `essais`) : bandeau « Le journal de caisse a été modifié », boutons de paiement grisés.
    - `voir_le_ticket_detaille_la_tva_par_taux_et_la_mention_non_fiscal` (ports de `le_ticket_detaille_la_tva_par_taux_en_europe`, `…_la_taxe_aux_etats_unis`, `le_ticket_affiche_la_mention_non_certifie`) : après une vente, « Voir le ticket » affiche les lignes de taxes, `titre_ticket`, la date `14/11/2023 23:13` et `MENTION_NON_CERTIFIE`.
    - `tous_les_textes_peints_des_fenetres_de_paiement_atteignent_4_5` : espèces (reçu suffisant), carte en attente, carte refusée, vente terminée avec ticket visible (port du test de contraste du ticket).
- [ ] **Step 2 :** `cargo test` — attendu : échecs.
- [ ] **Step 3 :** Implémenter `montants_rapides` (pas par devise de la section « Décisions » ; premier élément = le total ; ajoute `ceil(total / pas) × pas` s'il dépasse le total, n'est pas déjà présent et tant qu'il y a moins de 4 éléments), `Etat::detail_ticket` (lignes via `Montant::multiplier`, taxes via `detail_taxes` et `libelle_taxe`, `StatutPiece` : `Remboursement` si la pièce est une `Annulation`, `Remboursee` si une `Annulation { de: numero }` existe, sinon `Payee` ; un helper privé `statut_de(&self, piece) -> StatutPiece` que la tâche 9 réutilise), les trois fenêtres (largeurs 520, 480, 480 ; champ « Montant reçu » 64 de haut, contour 2 accent, texte 32/600 à droite ; boutons rapides 52 de haut en grille de 4 ; bandeau de monnaie : fond `SUCCES_FOND`/`ALERTE_FOND`, libellé 17/600 et montant 32 ; Retour / Valider en 1fr 2fr, 60 de haut) et `corps_du_ticket`. `Valider`, « Paiement accepté » et « Paiement reçu » appellent `etat.encaisser(...)` **une seule fois** (la fenêtre se ferme à l'image suivante) puis, en cas de succès, vident le panier et le reçu, **effacent le retour de succès**, ouvrent `VenteTerminee` ; en cas d'échec la fenêtre reste ouverte et affiche l'erreur. Le champ `moyen` de `Application` sert d'état de la fenêtre carte/autre : le supprimer s'il n'est plus utilisé.
- [ ] **Step 4 :** `cargo test`, clippy, fmt — attendu : vert.
- [ ] **Step 5 : contrôle visuel.** Ajouter `apercu_paiement_especes` (reçu 5 pour un total de 3,60, Pain × 3), `apercu_paiement_carte` et `apercu_vente_terminee` à `tests/apercu.rs` ; comparer à `maquette/2-especes.png` et `3-carte.png` ; corriger les écarts.
- [ ] **Step 6 :** Commit :

```bash
git add gestion-stock-gui
git commit -m "feat(gui): fenêtres de paiement (espèces, carte, autre), vente terminée et ticket"
```

---

### Tâche 8 : Écran Recherche

**Files :**
- Create : `src/ui/recherche.rs`
- Modify : `src/ui.rs` (`Application` : `recherche_articles: String`, `ajoute: Option<String>`), `src/textes.rs`, `src/ui/vente.rs` (le scan « plusieurs » remplit `recherche_articles`), `tests/apercu.rs`

**Interfaces :**
- Consumes : `Etat::recherche_articles`, `vente::ajouter_au_ticket`, `composants::{repere, bouton}`.
- Produces (`recherche.rs`) : `pub(super) fn ecran_recherche(ui: &mut Ui, app: &mut Application)` ; `pub(crate) fn etat_du_stock(quantite: u32, a_un_prix: bool) -> (&'static str, Rgb)` (texte et couleur).
- Produces (`textes.rs`) : `RECHERCHE_TITRE = "Rechercher un article"`, `CHAMP_RECHERCHE_ARTICLE = "Nom de l'article"`, `AUCUN_ARTICLE = "Aucun article trouvé. Vérifiez l'orthographe."`, `EN_STOCK = "En stock"`, `RUPTURE = "Rupture de stock"`, `fn plus_que(n: u32)` (`"Plus que 3 en stock"`), `BOUTON_AJOUTER_ARTICLE = "Ajouter"`, `BOUTON_AJOUTE = "Ajouté ✓"`, `BOUTON_INDISPONIBLE = "Indisponible"`, `fn ajouter_article(nom: &str)` (nom accessible `"Ajouter Pain au ticket"`).

- [ ] **Step 1 : tests qui échouent** (`recherche.rs`) :
  - `etat_du_stock` : `(0, true)` → `"Rupture de stock"` rouge ; `(1..=5, true)` → `"Plus que N en stock"` ambre ; `(6, true)` → `"En stock"` vert ; `(any, false)` → `PRIX_MANQUANT` rouge.
  - `la_recherche_ignore_la_casse_et_les_accents_et_compte_les_resultats` : « the » trouve « Thé vert » ; compteur `"1 article"` ; vide → tous, `"N articles"` ; `"zzz"` → `AUCUN_ARTICLE` et `"0 article"`.
  - `ajouter_met_une_unite_au_ticket_sans_quitter_la_recherche` : clic `ajouter_article("Pain")` → panier `("Pain", 1)`, bouton devenu `BOUTON_AJOUTE`, `ecran == Recherche` ; changer le texte retire « Ajouté ✓ ».
  - `un_produit_epuise_ou_sans_prix_a_un_bouton_grise` : `est_grise` pour `"Sel"` (épuisé) et `"Thé"` (sans prix) ; le bouton porte `BOUTON_INDISPONIBLE`.
  - `on_ne_depasse_pas_le_stock_depuis_la_recherche` : tout le stock déjà au ticket → clic → toast `stock_limite`.
  - `un_nom_tres_long_passe_a_la_ligne_sans_pousser_le_prix` ; `le_retour_a_la_vente_est_dans_l_en_tete` ; `le_texte_tape_au_scan_est_repris` (le scan « lai » ouvre la recherche avec « lai » dans le champ).
  - `tous_les_textes_peints_de_la_recherche_atteignent_4_5` (liste avec un produit bas, un épuisé, un sans prix).
- [ ] **Step 2 :** `cargo test` — attendu : échecs.
- [ ] **Step 3 :** Implémenter : marges 24 / 120 sur grande fenêtre (16 en dessous de 1100 points), titre 24/600, champ blanc de 64 de haut à contour accent de 2 et icône loupe dessinée, compteur 15 discret, carte de résultats (lignes de 84 minimum : repère 64, nom 18/500 et état du stock 14 colorés, prix 19/600, bouton 150 × 48 : « Ajouter » bleu, « Ajouté ✓ » `SUCCES_BOUTON`, « Indisponible » grisé) ; `ScrollArea` verticale.
- [ ] **Step 4 :** `cargo test`, clippy, fmt.
- [ ] **Step 5 : contrôle visuel.** `apercu_recherche` (texte « lait » si le jeu d'exemple a du lait ; sinon « riz ») vs `maquette/7-recherche.png`.
- [ ] **Step 6 :** Commit :

```bash
git add gestion-stock-gui
git commit -m "feat(gui): écran de recherche d'articles"
```

---

### Tâche 9 : Historique et remboursement

**Files :**
- Create : `src/ui/historique.rs`
- Modify : `src/etat.rs`, `src/ui.rs` (`Modale::Rembourser(u64)` ; `Application` : `historique: ReglagesHistorique { filtre: String, jour: Option<String>, choisi: Option<u64> }`), `src/textes.rs`, `tests/apercu.rs`
- Supprimer de `ui.rs` : l'ancien `ecran_ventes`, `liste_des_ventes`, `ligne_de_piece`, `ActionVente` et leurs tests

**Interfaces :**
- Produces (`etat.rs`) :

```rust
pub struct LigneHistorique {
    pub numero: u64, pub ticket: String, pub jour: String /* iso */, pub heure: String /* "23:13" */,
    pub moyen: Moyen, pub total: String, pub statut: StatutPiece, pub articles: u32,
}
pub fn historique_ventes(&self) -> Vec<LigneHistorique>   // toutes les pièces, la plus récente d'abord
pub fn jours_avec_pieces(&self) -> Vec<String>            // jours iso distincts, croissants
```

  `Etat::annuler` change de message : `"Ticket {numero_de_ticket(numero)} remboursé : stock remis."` (tests d'`etat.rs` mis à jour). `StatutPiece` est défini à la tâche 7 (même helper `statut_de`).
- Produces (`historique.rs`) : `pub(super) fn ecran_historique(ui: &mut Ui, app: &mut Application)` ; `pub(crate) fn filtrer<'a>(lignes: &'a [LigneHistorique], jour: &str, recherche: &str) -> Vec<&'a LigneHistorique>` (recherche vide → pièces du `jour` ; sinon pièces de **tous** les jours dont `ticket` contient la recherche rognée) ; `pub(crate) fn jour_voisin(jours: &[String], courant: &str, avant: bool) -> Option<String>` (jour le plus proche avec des pièces, strictement avant ou après).
- Produces (`textes.rs`) : `HISTORIQUE_TITRE = "Ventes du jour"`, `fn historique_titre_jour(iso: &str)` (`"Ventes du 14/11/2023"`), `CHAMP_TICKET = "N° de ticket"`, `AUCUNE_VENTE_JOUR = "Aucune vente ce jour."`, `JOUR_PRECEDENT = "Jour précédent"`, `JOUR_SUIVANT = "Jour suivant"`, `STATUT_PAYEE = "Payée"`, `STATUT_REMBOURSEE = "Remboursée"`, `STATUT_REMBOURSEMENT = "Remboursement"`, `BOUTON_REMBOURSER = "Rembourser"`, `fn rembourser_titre(ticket: &str)` (`"Rembourser le ticket 01-0001 ?"`), `fn rembourser_texte(total: &str)` (`"Montant remboursé : 3,60 €, par le même moyen de paiement."`), `SELECTIONNER_TICKET` (nom accessible `"Ticket 01-0001, 23:13, Carte, 3,60 €, Payée"` via `fn nom_de_ligne(...)`).

- [ ] **Step 1 : tests qui échouent.**
  - `etat.rs` : `historique_ventes` d'une base avec deux ventes puis le remboursement de la première → trois lignes, la plus récente d'abord, statuts `[Remboursement, Payee, Remboursee]`, `ticket == "01-0003"` pour la première, total de l'annulation `"-3,60 €"` (signe moins ASCII, comme `Devise::formater`), `articles` = somme des quantités ; `jours_avec_pieces` sur deux horloges (14/11 et 15/11) → deux jours croissants (horloge pilotée par un `Rc<Cell<i64>>` : voir `essais::etat_sur_deux_jours`) ; message de `annuler`.
  - `historique.rs` (unités) : `filtrer` — jour sans pièce → vide ; recherche `"0002"` renvoie la pièce n° 2 même d'un autre jour ; recherche d'espaces = vide ; `jour_voisin(["2023-11-14","2023-11-16"], "2023-11-15", true) == Some("2023-11-14")`, `(…, false) == Some("2023-11-16")`, aucun voisin → `None`.
  - UI (`historique.rs`, `mod tests`, `essais::app_avec_ventes` adapté à `Ecran::Historique`) :
    - `la_liste_montre_les_tickets_du_jour_avec_leur_statut` : lignes `01-0002` (espèces, `2,50 €`) puis `01-0001` (carte, `3,60 €`), statut `Payée` ; la première est choisie ; le détail à droite : `"Ticket 01-0002"`, ses lignes, le total en grand.
    - `rembourser_demande_confirmation_puis_ajoute_une_ecriture_inverse` (port de `la_liste_des_ventes_montre_les_pieces_et_permet_d_annuler_avec_confirmation`) : clic sur la ligne 1 puis « Rembourser » → fenêtre `rembourser_titre("01-0001")` + `rembourser_texte("3,60 €")` ; « Annuler » ne change rien (2 pièces, stock Pain 7) ; « Rembourser » (dans la fenêtre) → 3 pièces, stock Pain 10, ligne `01-0003` « Remboursement » en ambre avec `"-3,60 €"`, la ligne 1 passe à « Remboursée » ; « Rembourser » grisé pour la ligne 1 et pour la ligne 3 ; toast `"Ticket 01-0001 remboursé : stock remis."`.
    - **Review Focus 4** `un_double_clic_sur_rembourser_ne_cree_qu_une_ecriture` et `un_journal_altere_grise_rembourser_et_le_dit` (port : `app_journal_altere` ; bandeau « Le journal de caisse a été modifié » + `JOURNAL_ALTERE_SUITE` dans la carte de liste ; « Rembourser » grisé).
    - **Review Focus 5** `un_jour_sans_vente_le_dit_et_reste_navigable` : horloge au 15/11, ventes le 14/11 → jour courant = le 15 → titre `HISTORIQUE_TITRE`, `AUCUNE_VENTE_JOUR`, « Jour précédent » actif, « Jour suivant » grisé ; clic précédent → `"Ventes du 14/11/2023"` et les tickets du 14.
    - `un_ticket_ancien_reste_remboursable_par_son_numero` : vente le 14, horloge au 15 ; saisir `"0001"` dans `CHAMP_TICKET` → la ligne 1 est affichée malgré le jour ; la rembourser réussit.
    - `sans_aucune_vente_la_liste_le_dit` ; `le_detail_d_un_remboursement_ne_propose_pas_de_rembourser` ; `tous_les_textes_peints_de_l_historique_atteignent_4_5` (avec la fenêtre de remboursement ouverte aussi).
- [ ] **Step 2 :** `cargo test` — attendu : échecs.
- [ ] **Step 3 :** Implémenter. `historique_ventes` : parcourt `stock.pieces()` en sens inverse, `jour = horodatage.local().jour_iso()`, heure `HH:MM` ; `Remboursee` si une pièce `Annulation { de: numero }` existe. Écran : carte de gauche (titre 22/600, sélecteur de jour à droite du titre, champ « N° de ticket » 240 × 48, lignes de 56 : grille 120 / 80 / flexible / 140 / 150 comme `maquette/5-historique.html`, ligne choisie sur `ACCENT_DOUX` annoncée « sélectionnée » ; total barré jamais utilisé) ; panneau de droite 420 (titre `"Ticket 01-0001"`, méta `"23:13 · Carte · 4 articles · Payée"`, lignes avec repère 40, total 28, bouton « Rembourser » `Style::Confirmation`-rouge pleine largeur) ; fenêtre de remboursement (largeur 460) → `etat.annuler(numero)` une seule fois ; le choix par défaut est la pièce la plus récente affichée.
- [ ] **Step 4 :** `cargo test`, clippy, fmt.
- [ ] **Step 5 : contrôle visuel.** `apercu_historique` (deux ventes, une remboursée) vs `maquette/5-historique.png`.
- [ ] **Step 6 :** Commit :

```bash
git add gestion-stock-gui
git commit -m "feat(gui): historique des tickets et remboursement"
```

---

### Tâche 10 : Stock et Réglages dans le nouveau cadre, nettoyage

**Files :**
- Create : `src/ui/stock.rs`, `src/ui/reglages.rs`
- Modify : `src/ui.rs` (ne garde que `Application`, `Ecran`, `Modale`, `Toast`, `dessiner`, `ecran_erreur_ouverture`), `src/textes.rs`

**Interfaces :**
- Produces : `ui::stock::ecran_stock(ui, app)` (déplacé : `contenu_liste`, `chiffres`, `liste_produits`, `ligne_produit`, `historique` des mouvements, `contenu_commandes`, `suggestions_de_produits`, `formulaire_ajout`, `commandes_du_produit`, `suppression`, `abandonner_la_selection_perimee`) ; `ui::reglages::ecran_reglages(ui, app)` (déplacé : `formulaire_reglages`, `nom_du_profil`, `nom_de_la_devise`, **plus** une carte « Fichier de données » avec `pied_de_page_donnees(chemin_complet(...))`). Aucun changement de comportement : mêmes champs, mêmes boutons, mêmes messages (affichés par le toast).

- [ ] **Step 1 :** Déplacer le code et ses tests (ceux de la section « stock » et « réglages » de `ui.rs`) dans les deux modules ; adapter les tests qui lisaient `etat.retour` après une action (`un_double_clic_sur_sortie_n_enregistre_qu_une_sortie_et_garde_le_message_de_succes`, `renommer_ou_supprimer_un_produit_vendu_affiche_le_message_de_la_bibliotheque`, etc.) pour lire `app.toast` / le bandeau. **Test ajouté** : `reglages_montre_le_chemin_du_fichier_de_donnees` (`pied_de_page_donnees(...)` affiché) et `les_ecrans_stock_et_reglages_ont_l_en_tete_et_le_retour` (clic « Retour à la vente »).
- [ ] **Step 2 :** `cargo test` — le nombre de tests est celui de la base **moins** les tests supprimés des tâches 4, 6 et 9, **plus** les nouveaux : vérifier qu'aucun test portant un garde-fou (liste ci-dessous) n'a disparu sans équivalent : contraste de chaque écran, noms accessibles des champs et boutons, double clic, zoom 200 %, fenêtre basse, journal altéré, réglages figés après une vente, prix illisible.
- [ ] **Step 3 :** Supprimer de `ui.rs` tout ce qui n'est plus utilisé (`LARGEUR_TUILE`, etc.) ; clippy sans avertissement. Les `pub const` de `textes.rs` ne déclenchent aucun avertissement : pour chacun des textes d'onglets, de tuiles et de panier (`ONGLET_*`, `SECTION_PANIER`, `PANIER_VIDE`, `BOUTON_VIDER`, `BOUTON_RETIRER`, `BOUTON_ENCAISSER`, `RECHERCHE_VENTE`, `TOUT_AU_PANIER` s'il n'est plus utilisé…) lancer `grep -rn NOM src tests` et supprimer ceux qui n'ont plus aucune référence.
- [ ] **Step 4 :** Apercus `stock` (liste + commandes, comme `liste_et_commandes`) et `reglages` à 1366 × 768 : vérifier à l'œil que les cartes plates, la police, le bleu, l'en-tête sombre et le bouton « Retour à la vente » sont cohérents avec la maquette.
- [ ] **Step 5 :** Commit :

```bash
git add gestion-stock-gui
git commit -m "refactor(gui): écrans Stock et Réglages dans leurs modules, dans le nouveau cadre"
```

---

### Tâche 11 : Mise en page étroite, zoom et clavier

**Files :**
- Modify : `src/ui.rs` (`TAILLE_FENETRE = [1366.0, 768.0]`), `src/ui/vente.rs`, `historique.rs`, `recherche.rs`, `entete.rs`

**Interfaces :**
- Consumes : tous les écrans. Produces : une constante `LARGEUR_DEUX_COLONNES_CAISSE: f32 = 1100.0` dans `ui.rs`, utilisée par Vente et Historique (Stock et Réglages gardent 800).

- [ ] **Step 1 : tests qui échouent** (dans les modules d'écran) :
  - `a_1366_par_768_l_essentiel_de_la_vente_est_a_l_ecran` (port de `les_actions_principales_de_la_vente_sont_a_l_ecran_a_la_taille_par_defaut`) : ticket de 7 lignes ; les boutons Espèces, Carte, Autre, les 4 cases de la barre et le champ de scan sont **dans** `Rect(0,0,1366,768)`.
  - `dans_une_fenetre_etroite_le_scan_le_ticket_et_les_paiements_restent_accessibles` (port de `dans_une_fenetre_etroite_…`) : 640 × 1400 ; les trois boutons de paiement et le champ de scan sont dans la fenêtre ; le champ de scan est **avant** le ticket dans l'ordre de lecture.
  - `dans_une_fenetre_basse_la_page_defile_d_un_bloc_et_les_paiements_restent_atteignables` (port de `dans_une_fenetre_basse_le_panier_defile_d_un_bloc_…`) : 1100 × 560.
  - `a_200_pour_cent_la_vente_et_l_historique_tiennent_sans_debordement_horizontal` (port de `l_alerte_de_stock_reste_visible_a_200_pour_cent_sur_l_ecran_vente`, `textes_peints` avec `zoom = 2.0`, aucun texte hors de la largeur).
  - `la_barre_de_fonctions_passe_sur_deux_lignes_en_fenetre_etroite` (les quatre cases restent visibles et ≥ 44 de haut).
  - `l_historique_s_empile_en_fenetre_etroite` : la liste puis le détail, « Rembourser » dans la fenêtre.
  - `on_met_un_produit_au_ticket_puis_on_encaisse_sans_la_souris` (port de `on_met_un_produit_au_panier_puis_on_encaisse_sans_la_souris`) : `Tab`/`Espace`/`Entrée` seulement, du scan à « Vente terminée ».
- [ ] **Step 2 :** `cargo test` — attendu : échecs.
- [ ] **Step 3 :** Sous 1100 points : une seule colonne qui défile (champ de scan, ticket à hauteur naturelle, carte du total, boutons de paiement, barre de fonctions en grille de 2 colonnes) ; Historique : liste puis détail ; ordre de `Tab` : le scan d'abord. `TAILLE_FENETRE` passe à 1366 × 768 (`main.rs` inchangé).
- [ ] **Step 4 :** `cargo test`, clippy, fmt — vert.
- [ ] **Step 5 :** Commit :

```bash
git add gestion-stock-gui
git commit -m "feat(gui): mise en page étroite, zoom et clavier pour la caisse"
```

---

### Tâche 12 : Aperçus complets et comparaison avec la maquette

**Files :**
- Modify : `tests/apercu.rs`

- [ ] **Step 1 :** Dans `tests/apercu.rs`, garder **un test par écran** (le moteur de capture veut un rendu par test), tous à 1366 × 768, échelle 1 : `vente_vide`, `vente_panier`, `vente_confirmation`, `paiement_especes`, `paiement_carte`, `vente_terminee`, `recherche`, `historique`, `stock`, `reglages`, `menu`, `vente_etroite` (640 × 900). Supprimer les anciens aperçus devenus faux (`liste_*`, `fenetre_etroite`, `vente_ticket`, `ventes`, `stock_avec_prix` ou les réécrire).
- [ ] **Step 2 :** Générer : `APERCU_DOSSIER=<scratchpad>/apercu-final UPDATE_SNAPSHOTS=true cargo test --test apercu -- --ignored`. Lire chaque PNG et la capture de maquette correspondante (`maquette/1-vente.png` … `7-recherche.png`) ; pour chaque couple, vérifier : en-tête (hauteur, texte), cartes (bord, rayon), titres et graisses, couleurs des boutons principaux / secondaires / danger, alignements des montants, hauteur des lignes, espacement de la barre de fonctions.
- [ ] **Step 3 :** Corriger les écarts qui se voient à l'œil (un correctif = un test seulement s'il touche un comportement) ; relancer jusqu'à ce que les captures soient proches. Lister dans le message de commit les écarts **connus** et acceptés (par exemple : champ « Montant reçu » non reformaté, contour des champs assombri, boutons « Réimprimer » et « Rapport » absents à l'étape A).
- [ ] **Step 4 :** Commit :

```bash
git add gestion-stock-gui/tests/apercu.rs gestion-stock-gui/src
git commit -m "chore(gui): aperçus 1366 × 768 par écran et retouches visuelles"
```

---

### Tâche 13 : Documentation et vérification finale

**Files :**
- Modify : `gestion-stock-gui/README.md`, `gestion-stock-gui/LICENCES-TIERS.txt` (si le test de licences du projet le demande), `docs/superpowers/specs/2026-10-21-kora-marche-pos-design.md` (état d'avancement)

- [ ] **Step 1 :** README : section « Apparence » réécrite (maquette Kora Marché, IBM Plex Sans, écrans, raccourcis F2, Suppr, Échap, taille 1366 × 768 et mise en page sous 1100 points) ; section « Vendre » mise à jour (scan par nom et quantité `3 x`, fenêtres de paiement, carte et autre confirmés à la main : **aucun terminal relié**) ; « Où sont les données ? » : le chemin se voit dans le Menu et dans Réglages ; section « Accessibilité » : lignes 1.4.11 (contours de champs à 3,14:1 ; contours de boutons secondaires décoratifs gardés de la maquette, ces boutons étant identifiés par leur texte), 1.4.1, 4.1.2 mises à jour avec les nouveaux noms ; remplacer les mentions d'**Inter** par **IBM Plex Sans** (licence SIL OFL 1.1, © 2017 IBM Corp., archive `ibm-plex-sans.zip` 1.1.0, SHA-256 `fb365d91…9200`) ; limites connues : champ « Montant reçu » non reformaté, chiffres de Plex (selon le résultat du test de la tâche 2), `Suppr` seulement si le scan est vide.
- [ ] **Step 2 :** Lancer toutes les vérifications : dans `gestion-stock/` et `gestion-stock-gui/` : `cargo test`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check`. Attendu : tout vert ; le nombre de tests du GUI est reporté dans le README et le message de commit.
- [ ] **Step 3 : relecture de la spec.** Reprendre le § 7.1 à 7.4, 7.6 et 7.9 et cocher dans le document ce que l'étape A couvre ; écrire en tête de la spec « Étape A faite le 2026-10-… ; reste B, C, D ». Vérifier que `LICENCES-TIERS.txt` n'a pas besoin de changer (il liste les dépendances Rust, pas les polices).
- [ ] **Step 4 :** Commit :

```bash
git add gestion-stock-gui docs
git commit -m "docs: README de la caisse Kora Marché (étape A) et avancement de la spec"
```

- [ ] **Step 5 :** Utiliser superpowers:verification-before-completion avant de dire que l'étape est terminée, puis annoncer le plan de l'étape B.
