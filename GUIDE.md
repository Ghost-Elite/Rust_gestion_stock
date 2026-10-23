# Apprendre Rust en tapant du code

Un guide pratique en 10 chapitres : **tu tapes, tu lances, tu casses, tu comprends.**

Tout le code de ce guide a été compilé et exécuté avec Rust 1.93 (édition 2024). Les résultats que tu vois ici sont les vrais.

---

## Comment utiliser ce guide

**La règle d'or : tape le code à la main, ne le copie-colle pas.**
Taper oblige ton cerveau à lire chaque mot, et surtout tu vas faire des fautes de frappe : c'est en lisant les messages du compilateur pour les corriger que tu apprends le plus vite.

Chaque chapitre suit le même déroulé :

| Étape | Ce que tu fais |
|-------|----------------|
| **L'idée** | Tu lis 2 minutes de théorie |
| **Tape ce code** | Tu tapes le programme dans le fichier indiqué |
| **Résultat attendu** | Tu lances et tu compares avec ce qui est annoncé |
| **Ce qu'il faut comprendre** | Tu relis le code ligne par ligne avec les explications |
| **Casse-le exprès** | Tu provoques volontairement une erreur de compilation et tu lis le message |
| **À toi de jouer** | Un petit défi à résoudre seul(e) |
| **À retenir** | Le résumé en quelques lignes |

Prévois **20 à 30 minutes par chapitre**. Les chapitres 1 à 3 sont rapides : survole-les si tu connais déjà ces notions. Les chapitres 4 et 5 (ownership et emprunts) sont le cœur de Rust : prends ton temps.

---

## Organisation du projet

```text
rust-apprentissage/
├── Cargo.toml          configuration du projet
├── GUIDE.md            ce guide
├── src/bin/            TES fichiers : tu y tapes le code de chaque chapitre
│   ├── ch01_variables.rs
│   ├── ch02_types.rs
│   └── ...             (un fichier par chapitre, vide au départ)
└── solutions/          les solutions des défis (à regarder APRÈS avoir essayé)
    ├── ch01_defi.rs
    └── ...
```

Chaque fichier de `src/bin/` est un petit programme indépendant, avec son propre `main`. Pour l'instant ils ne font rien : ils contiennent juste `fn main() {}`.

## Les commandes à connaître

Place-toi d'abord dans le dossier du projet :

```bash
cd ~/rust-apprentissage
```

| Commande | Rôle |
|----------|------|
| `cargo run --bin ch01_variables` | Compile et lance le chapitre 1 |
| `cargo check --bin ch01_variables` | Vérifie seulement que ça compile (plus rapide) |
| `cargo test --bin ch10_taches` | Lance les tests du chapitre 10 |
| `cargo fmt` | Remet ton code en forme automatiquement |
| `cargo clippy` | Donne des conseils pour améliorer ton code |
| `cargo run --example ch01_defi` | Lance la solution du défi du chapitre 1 |

## Lire un message d'erreur de Rust

Le compilateur Rust est réputé pour la qualité de ses messages. Apprends à les lire, ils sont tes meilleurs professeurs :

```text
error[E0384]: cannot assign twice to immutable variable `score`
  --> src/bin/ch01_variables.rs:13:5
   |
13 |     score += 10;
   |     ^^^^^^^^^^^ cannot assign twice to immutable variable
   |
help: consider making this binding mutable
   |
11 |     let mut score = 0;
   |         +++
```

- `error[E0384]` : le type d'erreur. Tape `rustc --explain E0384` dans le terminal pour une explication détaillée avec exemples.
- `-->` : le fichier, la ligne et la colonne du problème.
- Les flèches `^^^` pointent le code fautif, et les lignes `help:` proposent souvent la correction.

Lis toujours **le premier message d'erreur** en premier : les suivants en sont parfois la conséquence.

Dans ce guide, les messages d'erreur sont les vrais (Rust 1.93), mais **raccourcis** : j'ai retiré les lignes secondaires. Les numéros de ligne dépendent de ton fichier, ils seront donc légèrement différents chez toi.

---

## Programme

1. Hello Cargo et variables
2. Types et fonctions
3. Contrôle de flux (`if`, `loop`, `match`)
4. Ownership
5. Emprunts et références
6. Structs et enums
7. `Option` et `Result`
8. Collections (`Vec`, `String`, `HashMap`)
9. Traits et génériques
10. Tests et mini-projet : un gestionnaire de tâches

---

## Chapitre 1 — Hello Cargo et variables

**Fichier :** `src/bin/ch01_variables.rs` · **Lancer :** `cargo run --bin ch01_variables`

### L'idée

En Rust, une variable est **immuable par défaut** : une fois qu'elle a une valeur, tu ne peux plus la changer, sauf si tu l'as déclarée avec `mut`. Cela évite des bugs : quand tu lis `let age = 30;`, tu sais que `age` vaudra 30 jusqu'à la fin.

### Tape ce code

<!-- fichier: src/bin/ch01_variables.rs -->
```rust
const MAX_POINTS: u32 = 100;

fn main() {
    println!("Bonjour, Rust !");

    // Une variable est immuable par défaut
    let age = 30;
    println!("J'ai {age} ans");

    // `mut` rend la variable modifiable
    let mut score = 0;
    println!("Score de départ : {score}");
    score += 10;
    println!("Score après un bonus : {score}");

    // Le shadowing : on redéclare une variable avec le même nom
    let espaces = "   ";
    let espaces = espaces.len();
    println!("Il y a {espaces} espaces");

    // Une constante : type obligatoire, nom en MAJUSCULES
    println!("Maximum : {MAX_POINTS}");
}
```

### Résultat attendu

<!-- sortie: ch01_variables -->
```text
Bonjour, Rust !
J'ai 30 ans
Score de départ : 0
Score après un bonus : 10
Il y a 3 espaces
Maximum : 100
```

### Ce qu'il faut comprendre

- `fn main()` est le **point d'entrée** : c'est ce que Rust exécute au lancement.
- `println!` se termine par un `!` : c'est une **macro**, pas une fonction normale. Dans le texte, `{age}` est remplacé par la valeur de la variable `age`.
- `let age = 30;` déclare une variable **immuable**. Rust devine tout seul son type (ici un entier `i32`).
- `let mut score = 0;` déclare une variable **modifiable**. Seul `mut` permet `score += 10;` (qui veut dire `score = score + 10;`).
- Le **shadowing** (`let espaces` deux fois) crée une *nouvelle* variable qui masque l'ancienne. Contrairement à `mut`, on peut même changer de type : `espaces` passe d'un texte à un nombre.
- `const` déclare une constante : le type est **obligatoire** (`u32` = entier positif sur 32 bits), elle peut être déclarée en dehors de `main`, et on l'écrit en `MAJUSCULES_AVEC_TIRETS_BAS`.
- Chaque instruction se termine par un `;`.

### Casse-le exprès

1. Dans `main`, remplace `let mut score = 0;` par `let score = 0;`.
2. Lance `cargo run --bin ch01_variables`.

Le compilateur refuse de compiler :

```text
error[E0384]: cannot assign twice to immutable variable `score`
  --> src/bin/ch01_variables.rs:13:5
   |
11 |     let score = 0;
   |         ----- first assignment to `score`
12 |     println!("Score de départ : {score}");
13 |     score += 10;
   |     ^^^^^^^^^^^ cannot assign twice to immutable variable
   |
help: consider making this binding mutable
   |
11 |     let mut score = 0;
   |         +++
```

Le message te dit exactement quoi faire : ajouter `mut`. Remets `let mut score = 0;` avant de continuer.

### À toi de jouer

Dans un nouveau `main` (efface le contenu de ton fichier ou ajoute à la suite) :

1. Déclare `let mut compteur = 0;`, ajoute 1 trois fois avec `compteur += 1;` et affiche sa valeur avant puis après chaque ajout.
2. **Bonus :** avec le shadowing, transforme ensuite `compteur` en texte avec `compteur.to_string()` et affiche le nombre de caractères avec `.len()`.

Solution (après avoir essayé !) : `cargo run --example ch01_defi`

### À retenir

- Variables **immuables par défaut** ; `mut` pour les rendre modifiables.
- Le **shadowing** redéclare une variable (et peut changer son type).
- `const` : type obligatoire, valeur connue à la compilation.

---

## Chapitre 2 — Types et fonctions

**Fichier :** `src/bin/ch02_types.rs` · **Lancer :** `cargo run --bin ch02_types`

### L'idée

Rust est **typé statiquement** : le type de chaque valeur est connu à la compilation. Le plus souvent le compilateur le devine, mais pour les fonctions tu dois écrire les types des paramètres et de la valeur renvoyée. Une fonction renvoie la valeur de sa **dernière expression**, celle qui n'a pas de `;`.

### Tape ce code

<!-- fichier: src/bin/ch02_types.rs -->
```rust
fn main() {
    // Entiers, flottants, booléens, caractères
    let entier: i32 = -42;
    let petit: u8 = 255;
    let prix: f64 = 19.99;
    let vrai: bool = true;
    let lettre: char = 'R';
    println!("{entier} {petit} {prix} {vrai} {lettre}");

    // Un tuple regroupe des valeurs de types différents
    let personne = ("Awa", 28, 1.68);
    let (nom, age, taille) = personne;
    println!("{nom} a {age} ans et mesure {taille} m");
    println!("Premier élément du tuple : {}", personne.0);

    // Un tableau : taille fixe, même type
    let jours = ["lundi", "mardi", "mercredi"];
    println!("Premier jour : {}", jours[0]);
    println!("Nombre de jours : {}", jours.len());

    // Appel de fonctions
    let somme = additionner(2, 3);
    println!("2 + 3 = {somme}");
    println!("Le carré de 7 est {}", carre(7));

    // Conversion explicite avec `as`
    let temperature_c = 25;
    let temperature_f = celsius_vers_fahrenheit(temperature_c as f64);
    println!("{temperature_c} °C = {temperature_f} °F");
}

fn additionner(a: i32, b: i32) -> i32 {
    a + b
}

fn carre(x: i32) -> i32 {
    x * x
}

fn celsius_vers_fahrenheit(c: f64) -> f64 {
    c * 9.0 / 5.0 + 32.0
}
```

### Résultat attendu

<!-- sortie: ch02_types -->
```text
-42 255 19.99 true R
Awa a 28 ans et mesure 1.68 m
Premier élément du tuple : Awa
Premier jour : lundi
Nombre de jours : 3
2 + 3 = 5
Le carré de 7 est 49
25 °C = 77 °F
```

### Ce qu'il faut comprendre

- **Entiers :** `i32` (avec signe, le type par défaut), `u8` (de 0 à 255), et aussi `i64`, `u32`, `usize`, etc. Le chiffre est le nombre de bits.
- **Flottants :** `f64` est le type par défaut pour les nombres à virgule (`f32` existe aussi).
- **`char`** s'écrit avec des **apostrophes** (`'R'`) et représente un caractère Unicode. Un texte s'écrit avec des **guillemets** (`"Awa"`).
- **Tuple :** `("Awa", 28, 1.68)` mélange des types. On l'ouvre par *déstructuration* (`let (nom, age, taille) = personne;`) ou par position (`personne.0`).
- **Tableau :** `[T; N]` a une taille fixe connue à la compilation. Les indices commencent à 0.
- **Fonctions :** `fn additionner(a: i32, b: i32) -> i32`. Les types sont obligatoires, et `a + b` **sans point-virgule** est la valeur renvoyée.
- **`as`** convertit explicitement un type en un autre. Rust ne convertit jamais seul un entier en flottant : `celsius_vers_fahrenheit(temperature_c)` ne compilerait pas.

### Casse-le exprès

1. Dans la fonction `additionner`, ajoute un point-virgule : remplace `a + b` par `a + b;`.
2. Lance `cargo run --bin ch02_types`.

```text
error[E0308]: mismatched types
  --> src/bin/ch02_types.rs:32:35
   |
32 | fn additionner(a: i32, b: i32) -> i32 {
   |    -----------                    ^^^ expected `i32`, found `()`
   |    |
   |    implicitly returns `()` as its body has no tail or `return` expression
33 |     a + b;
   |          - help: remove this semicolon to return this value
```

Un seul `;` change tout : `a + b` est une **expression** (elle produit une valeur), `a + b;` est une **instruction** (elle ne produit rien, donc la fonction renvoie `()`, le type « vide »). Rust te propose même de supprimer le point-virgule. Remets `a + b`.

### À toi de jouer

1. Écris `fn est_pair(n: i32) -> bool` et `fn moyenne(a: f64, b: f64) -> f64`, puis appelle-les depuis `main`.
2. **Bonus :** écris `fn min_max(a: i32, b: i32) -> (i32, i32)` qui renvoie un tuple `(le_plus_petit, le_plus_grand)`, et récupère-le avec `let (min, max) = ...`.

Solution : `cargo run --example ch02_defi`

### À retenir

- Les types des paramètres et du retour d'une fonction sont **obligatoires**.
- La dernière expression **sans `;`** est la valeur renvoyée.
- Les conversions de types sont **toujours explicites** (`as`).

---

## Chapitre 3 — Contrôle de flux

**Fichier :** `src/bin/ch03_flux.rs` · **Lancer :** `cargo run --bin ch03_flux`

### L'idée

Rust a les structures classiques (`if`, `while`, `for`) plus deux particularités : presque tout est une **expression** qui renvoie une valeur (même `if` et `loop`), et `match` oblige à **traiter tous les cas possibles**.

### Tape ce code

<!-- fichier: src/bin/ch03_flux.rs -->
```rust
fn main() {
    // if est une expression : il renvoie une valeur
    let nombre = 7;
    let parite = if nombre % 2 == 0 { "pair" } else { "impair" };
    println!("{nombre} est {parite}");

    // loop : boucle infinie, on en sort avec break
    let mut essais = 0;
    let resultat = loop {
        essais += 1;
        if essais == 5 {
            break essais * 10;
        }
    };
    println!("loop terminée avec {resultat}");

    // while
    let mut compte_a_rebours = 3;
    while compte_a_rebours > 0 {
        println!("{compte_a_rebours}...");
        compte_a_rebours -= 1;
    }
    println!("Décollage !");

    // for sur un intervalle
    for i in 1..=5 {
        println!("{i} x 3 = {}", i * 3);
    }

    // match : le compilateur vérifie que tous les cas sont couverts
    for note in [4, 10, 14, 18] {
        println!("{note}/20 -> {}", mention(note));
    }
}

fn mention(note: u32) -> &'static str {
    match note {
        0..=9 => "insuffisant",
        10..=13 => "passable",
        14..=16 => "bien",
        17..=20 => "très bien",
        _ => "note invalide",
    }
}
```

### Résultat attendu

<!-- sortie: ch03_flux -->
```text
7 est impair
loop terminée avec 50
3...
2...
1...
Décollage !
1 x 3 = 3
2 x 3 = 6
3 x 3 = 9
4 x 3 = 12
5 x 3 = 15
4/20 -> insuffisant
10/20 -> passable
14/20 -> bien
18/20 -> très bien
```

### Ce qu'il faut comprendre

- **`if` est une expression :** `let parite = if ... { "pair" } else { "impair" };`. Les deux branches doivent renvoyer le même type. La condition doit être un vrai `bool` : `if 1 {...}` ne compile pas.
- **`loop`** tourne indéfiniment jusqu'à un `break`. `break essais * 10;` sort de la boucle **en renvoyant une valeur**.
- **`while`** répète tant que la condition est vraie.
- **`for i in 1..=5`** parcourt un intervalle. `1..=5` inclut le 5, `1..5` l'exclurait. `for` est la boucle que tu utiliseras le plus.
- **`match`** compare une valeur à des motifs. `0..=9` est un intervalle, `_` signifie « tous les autres cas ». Rust **vérifie que tous les cas sont couverts**.
- `&'static str` est le type d'un texte écrit directement dans le programme. Retiens-le tel quel pour l'instant : on reviendra sur le `'static` plus tard.

### Casse-le exprès

1. Dans la fonction `mention`, supprime la ligne `_ => "note invalide",`.
2. Lance `cargo run --bin ch03_flux`.

```text
error[E0004]: non-exhaustive patterns: `21_u32..=u32::MAX` not covered
  --> src/bin/ch03_flux.rs:37:11
   |
37 |     match note {
   |           ^^^^ pattern `21_u32..=u32::MAX` not covered
   |
   = note: the matched value is of type `u32`
help: ensure that all possible cases are being handled by adding a match arm with a wildcard pattern or an explicit pattern as shown
   |
41 ~         17..=20 => "très bien",
42 ~         21_u32..=u32::MAX => todo!(),
```

Un `u32` peut valoir bien plus que 20 : le compilateur te dit précisément quelles valeurs tu as oubliées. C'est ce qui rend `match` si sûr. Remets la ligne.

### À toi de jouer

1. Écris un **FizzBuzz** de 1 à 20 : affiche `Fizz` pour les multiples de 3, `Buzz` pour les multiples de 5, `FizzBuzz` pour les deux, sinon le nombre. *Indice : `match (i % 3, i % 5)` avec des motifs comme `(0, 0)`.*
2. **Bonus :** avec une `loop` qui renvoie une valeur, trouve le premier multiple de 7 supérieur à 50.

Solution : `cargo run --example ch03_defi`

### À retenir

- `if` et `loop` sont des expressions qui peuvent renvoyer une valeur.
- `for x in début..fin` (ou `début..=fin`) est la boucle de référence.
- `match` doit être **exhaustif** : le compilateur vérifie tous les cas.

---

## Chapitre 4 — Ownership

**Fichier :** `src/bin/ch04_ownership.rs` · **Lancer :** `cargo run --bin ch04_ownership`

### L'idée

C'est **LA** particularité de Rust. Les autres langages libèrent la mémoire soit à la main (C : on oublie un `free`, c'est un bug), soit avec un ramasse-miettes (Java, Python : c'est plus lent et moins prévisible). Rust choisit une troisième voie : des règles vérifiées **à la compilation**.

Les trois règles de l'ownership :

1. Chaque valeur a **un propriétaire** : une variable.
2. Il ne peut y avoir **qu'un seul propriétaire à la fois**.
3. Quand le propriétaire **sort de sa portée** (la fin de son bloc `{ }`), la valeur est **libérée**.

Quand tu écris `let s2 = s1;` avec une `String`, la propriété est **déplacée** (*move*) vers `s2` : `s1` n'est plus utilisable. Les types simples comme les entiers font exception : ils sont si petits qu'ils sont simplement **copiés**.

### Tape ce code

<!-- fichier: src/bin/ch04_ownership.rs -->
```rust
fn main() {
    // Les entiers sont copiés : les deux variables restent utilisables
    let a = 5;
    let b = a;
    println!("a = {a}, b = {b}");

    // Une String vit sur le tas. L'affectation DÉPLACE la valeur
    let s1 = String::from("bonjour");
    let s2 = s1;
    println!("s2 = {s2}");
    // println!("{s1}"); // <- ne compile plus : s1 n'est plus propriétaire

    // clone() fait une vraie copie (coûteuse)
    let original = String::from("rust");
    let copie = original.clone();
    println!("original = {original}, copie = {copie}");

    // Passer une String à une fonction lui donne la propriété
    let message = String::from("coucou");
    prend_la_propriete(message);
    // `message` n'est plus utilisable ici

    // Une fonction peut rendre la propriété
    let texte = String::from("ownership");
    let texte = ajoute_exclamation(texte);
    println!("{texte}");

    // Quand une variable sort de sa portée, sa valeur est libérée
    {
        let temporaire = String::from("je vais disparaître");
        println!("{temporaire}");
    } // <- `temporaire` est libérée ici (drop)
    println!("fin du programme");
}

fn prend_la_propriete(texte: String) {
    println!("J'ai reçu : {texte}");
} // `texte` est libérée ici

fn ajoute_exclamation(mut texte: String) -> String {
    texte.push('!');
    texte
}
```

### Résultat attendu

<!-- sortie: ch04_ownership -->
```text
a = 5, b = 5
s2 = bonjour
original = rust, copie = rust
J'ai reçu : coucou
ownership!
je vais disparaître
fin du programme
```

### Ce qu'il faut comprendre

- `let b = a;` avec des entiers : une **copie**. `a` et `b` valent chacun 5.
- `String::from("bonjour")` crée un texte modifiable dont le contenu est stocké sur le **tas** (la mémoire dynamique). `let s2 = s1;` ne copie pas le texte : elle **transfère la propriété**. Rust invalide `s1` pour qu'il n'y ait jamais deux propriétaires qui libèrent la même mémoire (le fameux bug de double libération).
- `.clone()` fait une vraie copie profonde. C'est explicite parce que c'est potentiellement coûteux.
- `prend_la_propriete(message)` **déplace** `message` dans la fonction. À la fin de la fonction, `texte` sort de sa portée et la mémoire est libérée. Après l'appel, `message` n'existe plus pour `main`.
- `ajoute_exclamation` reçoit la propriété, modifie le texte (le paramètre est déclaré `mut`) et **la rend** en dernière expression. `main` la récupère avec `let texte = ...`.
- Les accolades `{ }` créent une **portée** : `temporaire` est libérée au `}`.

### Casse-le exprès

1. Décommente la ligne `// println!("{s1}");` (enlève les deux barres `//` au début).
2. Lance `cargo run --bin ch04_ownership`.

```text
error[E0382]: borrow of moved value: `s1`
  --> src/bin/ch04_ownership.rs:11:16
   |
 8 |     let s1 = String::from("bonjour");
   |         -- move occurs because `s1` has type `String`, which does not implement the `Copy` trait
 9 |     let s2 = s1;
   |              -- value moved here
10 |     println!("s2 = {s2}");
11 |     println!("{s1}"); // <- ne compile plus : s1 n'est plus propriétaire
   |                ^^ value borrowed here after move
   |
help: consider cloning the value if the performance cost is acceptable
   |
 9 |     let s2 = s1.clone();
   |                ++++++++
```

Rust te dit tout : la valeur a été **déplacée** (`value moved here`), pourquoi (`String` n'implémente pas `Copy`), et propose même un `.clone()` si tu as vraiment besoin des deux. Remets la ligne en commentaire.

### À toi de jouer

1. Écris `fn longueur_et_texte(s: String) -> (String, usize)` qui renvoie la `String` **et** sa longueur (`s.len()`). Appelle-la dans `main` et affiche les deux.
2. Que se passe-t-il si tu écris `(s, s.len())` ? Essaie, et lis le message : pourquoi `s` est-elle déjà déplacée au moment du `s.len()` ?

Pénible, non ? Devoir « rendre » la valeur juste pour continuer à s'en servir, c'est exactement le problème que résout le chapitre suivant.

Solution : `cargo run --example ch04_defi`

### À retenir

- Une valeur = un propriétaire. Affecter ou passer à une fonction **déplace** la propriété.
- Les types simples (entiers, booléens, `char`...) sont **copiés** ; pour une `String`, utilise `.clone()` si tu veux une copie.
- Quand le propriétaire sort de sa portée, la valeur est libérée automatiquement.

---

## Chapitre 5 — Emprunts et références

**Fichier :** `src/bin/ch05_emprunts.rs` · **Lancer :** `cargo run --bin ch05_emprunts`

### L'idée

Au lieu de donner la propriété, on peut **prêter** une valeur avec une **référence**. Prêter ne transfère rien : le propriétaire reste le même, et quand l'emprunt est fini il retrouve sa valeur.

Il existe deux sortes d'emprunts, avec une règle stricte :

- `&valeur` : emprunt **immuable** (lecture seule). Tu peux en avoir **autant que tu veux** en même temps.
- `&mut valeur` : emprunt **mutable** (lecture et écriture). Tu ne peux en avoir **qu'un seul**, et aucun emprunt immuable en même temps.

Autrement dit : **plusieurs lecteurs OU un seul écrivain**, jamais les deux. C'est ce qui élimine toute une catégorie de bugs (les accès concurrents aux mêmes données).

### Tape ce code

<!-- fichier: src/bin/ch05_emprunts.rs -->
```rust
fn main() {
    // Emprunt immuable : on lit sans prendre la propriété
    let texte = String::from("les emprunts en Rust");
    let longueur = compte_octets(&texte);
    println!("« {texte} » contient {longueur} octets");

    // Plusieurs emprunts immuables en même temps : OK
    let r1 = &texte;
    let r2 = &texte;
    println!("{r1} / {r2}");

    // Emprunt mutable : un seul à la fois
    let mut panier = String::from("pommes");
    ajoute_article(&mut panier, "poires");
    println!("{panier}");

    // Les slices : une vue sur une partie d'une donnée
    let phrase = String::from("bonjour tout le monde");
    let premier = premier_mot(&phrase);
    println!("premier mot : {premier}");

    let nombres = [10, 20, 30, 40, 50];
    let milieu = &nombres[1..4];
    println!("milieu : {milieu:?}");
    println!("somme du milieu : {}", somme(milieu));
}

fn compte_octets(s: &str) -> usize {
    s.len()
}

fn ajoute_article(panier: &mut String, article: &str) {
    panier.push_str(" et ");
    panier.push_str(article);
}

fn premier_mot(s: &str) -> &str {
    match s.find(' ') {
        Some(position) => &s[..position],
        None => s,
    }
}

fn somme(valeurs: &[i32]) -> i32 {
    let mut total = 0;
    for v in valeurs {
        total += v;
    }
    total
}
```

### Résultat attendu

<!-- sortie: ch05_emprunts -->
```text
« les emprunts en Rust » contient 20 octets
les emprunts en Rust / les emprunts en Rust
pommes et poires
premier mot : bonjour
milieu : [20, 30, 40]
somme du milieu : 90
```

### Ce qu'il faut comprendre

- `compte_octets(&texte)` : le `&` **prête** `texte` à la fonction. Après l'appel, `texte` est toujours utilisable (contrairement au chapitre 4).
- Le paramètre `s: &str` accepte une référence vers du texte. On écrit `&str` plutôt que `&String` car c'est plus général : une `&String` est automatiquement convertie en `&str`. Prends cette habitude.
- `len()` compte des **octets**, pas des lettres. Ici le texte est en ASCII donc c'est pareil ; un `é` pèse 2 octets.
- `r1` et `r2` : deux emprunts immuables simultanés, aucun problème.
- `ajoute_article(&mut panier, ...)` : pour qu'une fonction puisse modifier ta valeur, il faut `&mut` dans le type du paramètre (`&mut String`) **et** à l'appel (`&mut panier`). La variable `panier` doit elle-même être déclarée `mut`.
- Une **slice** (`&s[..position]`, `&nombres[1..4]`) est une référence vers un morceau de donnée, sans copie. `..position` signifie « du début jusqu'à `position` exclu », `1..4` « de l'indice 1 à 3 ».
- `premier_mot` utilise `Option` (`Some` / `None`) : on le voit en détail au chapitre 7. Pour l'instant, lis-le comme « s'il y a un espace, prends ce qui est avant, sinon prends tout ».
- `{milieu:?}` affiche avec le format de **débogage** (`[20, 30, 40]`).
- `for v in valeurs` parcourt une slice : `v` est une *référence* vers chaque nombre.

### Casse-le exprès

1. À la fin de `main`, avant l'accolade fermante, ajoute :

   ```rust
   let mut s = String::from("salut");
   let lecture = &s;
   let ecriture = &mut s;
   println!("{lecture} {ecriture}");
   ```

2. Lance `cargo run --bin ch05_emprunts`.

```text
error[E0502]: cannot borrow `s` as mutable because it is also borrowed as immutable
  --> src/bin/ch05_emprunts.rs:28:20
   |
27 |     let lecture = &s;
   |                   -- immutable borrow occurs here
28 |     let ecriture = &mut s;
   |                    ^^^^^^ mutable borrow occurs here
29 |     println!("{lecture} {ecriture}");
   |                ------- immutable borrow later used here
```

Tu demandes un emprunt **mutable** pendant qu'un emprunt **immuable** (`lecture`) est encore utilisé après. Le compilateur refuse : si `ecriture` modifiait `s`, `lecture` pourrait pointer vers des données qui ont changé ou disparu. Supprime ces 4 lignes.

### À toi de jouer

1. Écris `fn plus_grand(valeurs: &[i32]) -> i32` qui renvoie le plus grand nombre d'une slice. Teste avec `&[12, 17, 9, 14]`.
2. Écris `fn met_en_majuscules(texte: &mut String)` qui remplace le texte par sa version en majuscules. *Indice : `*texte = texte.to_uppercase();`. L'étoile `*` signifie « la valeur vers laquelle pointe la référence ».*

Solution : `cargo run --example ch05_defi`

### À retenir

- `&x` prête en lecture (autant que tu veux), `&mut x` prête en écriture (un seul à la fois).
- **Plusieurs lecteurs OU un seul écrivain**, jamais les deux en même temps.
- Pour les paramètres de fonction, préfère `&str` à `&String` et `&[T]` à `&Vec<T>`.

---

## Chapitre 6 — Structs et enums

**Fichier :** `src/bin/ch06_structs_enums.rs` · **Lancer :** `cargo run --bin ch06_structs_enums`

### L'idée

Pour modéliser tes propres données, Rust te donne deux outils :

- **`struct`** : regroupe plusieurs champs nommés (une chose *ET* une autre).
- **`enum`** : une valeur qui est **l'une** de plusieurs variantes possibles (une chose *OU* une autre), et chaque variante peut transporter des données.

Les comportements s'écrivent dans un bloc **`impl`** : ce sont des *méthodes*.

### Tape ce code

<!-- fichier: src/bin/ch06_structs_enums.rs -->
```rust
#[derive(Debug)]
struct Rectangle {
    largeur: f64,
    hauteur: f64,
}

impl Rectangle {
    // Fonction associée (comme un constructeur)
    fn new(largeur: f64, hauteur: f64) -> Self {
        Self { largeur, hauteur }
    }

    // Méthode : le premier paramètre est &self
    fn aire(&self) -> f64 {
        self.largeur * self.hauteur
    }

    fn est_carre(&self) -> bool {
        self.largeur == self.hauteur
    }

    // Méthode qui modifie : &mut self
    fn agrandir(&mut self, facteur: f64) {
        self.largeur *= facteur;
        self.hauteur *= facteur;
    }
}

enum Forme {
    Cercle(f64),
    Rectangle { largeur: f64, hauteur: f64 },
    Point,
}

impl Forme {
    fn aire(&self) -> f64 {
        match self {
            Forme::Cercle(rayon) => std::f64::consts::PI * rayon * rayon,
            Forme::Rectangle { largeur, hauteur } => largeur * hauteur,
            Forme::Point => 0.0,
        }
    }
}

fn main() {
    let mut r = Rectangle::new(3.0, 4.0);
    println!("{r:?}");
    println!("aire = {}", r.aire());
    println!("carré ? {}", r.est_carre());
    r.agrandir(2.0);
    println!("après agrandissement : {r:?}, aire = {}", r.aire());

    let formes = [
        Forme::Cercle(1.0),
        Forme::Rectangle {
            largeur: 2.0,
            hauteur: 5.0,
        },
        Forme::Point,
    ];
    for forme in &formes {
        println!("aire = {:.2}", forme.aire());
    }
}
```

### Résultat attendu

<!-- sortie: ch06_structs_enums -->
```text
Rectangle { largeur: 3.0, hauteur: 4.0 }
aire = 12
carré ? false
après agrandissement : Rectangle { largeur: 6.0, hauteur: 8.0 }, aire = 48
aire = 3.14
aire = 10.00
aire = 0.00
```

### Ce qu'il faut comprendre

- `#[derive(Debug)]` demande à Rust de générer automatiquement l'affichage de débogage : c'est ce qui permet `{r:?}`.
- `impl Rectangle { ... }` regroupe les fonctions liées au type. `Self` désigne le type lui-même (ici `Rectangle`).
- **`new` n'est pas un mot-clé** : c'est une simple fonction *associée* (sans `self`), appelée avec `::` (`Rectangle::new(...)`). C'est la convention pour créer une valeur.
- `Self { largeur, hauteur }` : quand le nom du paramètre est le même que celui du champ, on peut écrire le nom une seule fois.
- **Les méthodes** ont `self` en premier paramètre, et on les appelle avec un point (`r.aire()`). Le choix de l'emprunt compte, comme au chapitre 5 : `&self` pour lire, `&mut self` pour modifier.
- Chaque variante d'un enum peut transporter des données différentes : `Cercle(f64)` (un rayon), `Rectangle { largeur, hauteur }` (deux champs nommés), `Point` (rien).
- Le `match` **déstructure** la variante pour récupérer les données qu'elle contient (`Forme::Cercle(rayon)`). Et comme au chapitre 3, il doit couvrir **toutes** les variantes.
- `{:.2}` affiche un flottant avec 2 décimales.
- Le `struct Rectangle` et la variante `Forme::Rectangle` portent le même nom sans conflit : la variante est toujours préfixée par `Forme::`.

### Casse-le exprès

1. Dans `main`, remplace `let mut r = Rectangle::new(3.0, 4.0);` par `let r = Rectangle::new(3.0, 4.0);`.
2. Lance `cargo run --bin ch06_structs_enums`.

```text
error[E0596]: cannot borrow `r` as mutable, as it is not declared as mutable
  --> src/bin/ch06_structs_enums.rs:50:5
   |
50 |     r.agrandir(2.0);
   |     ^ cannot borrow as mutable
   |
help: consider changing this to be mutable
   |
46 |     let mut r = Rectangle::new(3.0, 4.0);
   |         +++
```

`agrandir` demande `&mut self`, donc la variable doit être déclarée `mut`. C'est la règle du chapitre 1, appliquée aux méthodes. Remets `mut`.

### À toi de jouer

1. Ajoute à `Forme` une variante `Triangle { base: f64, hauteur: f64 }` (aire = base × hauteur / 2) **sans toucher au reste d'abord** : lance, et laisse le compilateur te montrer exactement où il manque quelque chose. Puis corrige et ajoute un triangle dans le tableau `formes`.
2. Ajoute à `Rectangle` une méthode `perimetre(&self) -> f64` et affiche-la dans `main`.

Solution : `cargo run --example ch06_defi`

### À retenir

- `struct` = plusieurs champs ensemble ; `enum` = une variante parmi plusieurs, avec des données possibles.
- Les méthodes vivent dans un bloc `impl` ; `&self` lit, `&mut self` modifie.
- Ajouter une variante à un enum fait échouer tous les `match` incomplets : le compilateur te guide dans la modification.

---

## Chapitre 7 — `Option` et `Result`

**Fichier :** `src/bin/ch07_option_result.rs` · **Lancer :** `cargo run --bin ch07_option_result`

### L'idée

Rust n'a **ni `null`, ni exceptions**. Deux enums de la bibliothèque standard les remplacent, et le compilateur t'oblige à les traiter :

- **`Option<T>`** : une valeur qui peut être absente. `Some(valeur)` ou `None`.
- **`Result<T, E>`** : une opération qui peut échouer. `Ok(valeur)` ou `Err(erreur)`.

Impossible d'oublier un cas : si une fonction peut échouer, son type le dit, et tu dois décider quoi faire.

### Tape ce code

<!-- fichier: src/bin/ch07_option_result.rs -->
```rust
use std::num::ParseIntError;

fn main() {
    // Option : une valeur peut être absente
    let fruits = ["pomme", "banane", "cerise"];
    match fruits.get(1) {
        Some(fruit) => println!("fruit n°1 : {fruit}"),
        None => println!("pas de fruit à cet indice"),
    }
    let absent = fruits.get(10);
    println!("fruit n°10 : {absent:?}");

    // if let : pratique quand on ne s'intéresse qu'à un cas
    if let Some(dernier) = fruits.last() {
        println!("dernier fruit : {dernier}");
    }

    // unwrap_or : une valeur par défaut
    let nom = trouve_nom(0).unwrap_or("inconnu");
    let autre = trouve_nom(9).unwrap_or("inconnu");
    println!("{nom} / {autre}");

    // Result : une opération peut échouer
    match "42".parse::<i32>() {
        Ok(n) => println!("nombre : {n}"),
        Err(e) => println!("erreur : {e}"),
    }
    match "abc".parse::<i32>() {
        Ok(n) => println!("nombre : {n}"),
        Err(e) => println!("erreur : {e}"),
    }

    // L'opérateur ? propage l'erreur à l'appelant
    println!("{:?}", additionne_textes("10", "32"));
    println!("{:?}", additionne_textes("10", "oups"));
}

fn trouve_nom(id: u32) -> Option<&'static str> {
    match id {
        0 => Some("Awa"),
        1 => Some("Moussa"),
        _ => None,
    }
}

fn additionne_textes(a: &str, b: &str) -> Result<i32, ParseIntError> {
    let x: i32 = a.parse()?;
    let y: i32 = b.parse()?;
    Ok(x + y)
}
```

### Résultat attendu

<!-- sortie: ch07_option_result -->
```text
fruit n°1 : banane
fruit n°10 : None
dernier fruit : cerise
Awa / inconnu
nombre : 42
erreur : invalid digit found in string
Ok(42)
Err(ParseIntError { kind: InvalidDigit })
```

### Ce qu'il faut comprendre

- `fruits.get(1)` ne plante pas si l'indice n'existe pas (contrairement à l'accès direct `fruits[i]`, qui **fait paniquer** le programme quand `i` est trop grand) : elle renvoie `Some(&valeur)` ou `None`. Le `match` t'oblige à écrire les deux cas.
- `if let Some(x) = ... { }` est un raccourci quand un seul cas t'intéresse et que tu ignores l'autre.
- `unwrap_or("inconnu")` déballe le `Some`, ou donne une valeur par défaut si c'est `None`. Il existe aussi `.unwrap()`, qui **fait paniquer le programme** si c'est `None` ou `Err` : réserve-le aux prototypes et aux tests.
- `"42".parse::<i32>()` essaie de convertir un texte en nombre et renvoie un `Result`. Le `::<i32>` (qu'on appelle *turbofish*) précise le type voulu.
- **L'opérateur `?`** est le plus pratique : sur un `Result`, si c'est `Ok(v)` il te donne `v`, si c'est `Err(e)` il **sort immédiatement de la fonction** en renvoyant cette erreur. C'est pour ça que `additionne_textes` renvoie elle-même un `Result`.
- `{:?}` affiche `Ok(42)` ou `Err(...)` : le format de débogage montre la variante.

### Casse-le exprès

1. Au début de `main`, juste après `fn main() {`, ajoute :

   ```rust
   let x: i32 = "42".parse::<i32>();
   ```

2. Lance `cargo run --bin ch07_option_result`.

```text
error[E0308]: mismatched types
 --> src/bin/ch07_option_result.rs:4:18
  |
4 |     let x: i32 = "42".parse::<i32>();
  |            ---   ^^^^^^^^^^^^^^^^^^^ expected `i32`, found `Result<i32, ParseIntError>`
  |            |
  |            expected due to this
  |
  = note: expected type `i32`
             found enum `Result<i32, ParseIntError>`
help: consider using `Result::expect` to unwrap the `Result<i32, ParseIntError>` value, panicking if the value is a `Result::Err`
  |
4 |     let x: i32 = "42".parse::<i32>().expect("REASON");
  |                                     +++++++++++++++++
```

Tu as oublié que la conversion **peut échouer** : tu as un `Result`, pas un `i32`. Rust t'interdit de faire comme si l'erreur n'existait pas. Supprime la ligne.

### À toi de jouer

1. Écris `fn division(a: f64, b: f64) -> Result<f64, String>` qui renvoie `Err("division par zéro")` quand `b` vaut 0.0, sinon `Ok(a / b)`.
2. Écris `fn premier_pair(valeurs: &[i32]) -> Option<i32>` qui renvoie le premier nombre pair d'une slice, ou `None`.

Affiche les résultats avec `{:?}`.

Solution : `cargo run --example ch07_defi`

### À retenir

- `Option` pour « peut être absent », `Result` pour « peut échouer » : pas de `null`, pas d'exception.
- `match` et `if let` pour les traiter ; `unwrap_or` pour une valeur par défaut ; `?` pour propager une erreur.
- `.unwrap()` fait paniquer : à éviter dans du vrai code.

---

## Chapitre 8 — Collections

**Fichier :** `src/bin/ch08_collections.rs` · **Lancer :** `cargo run --bin ch08_collections`

### L'idée

La bibliothèque standard fournit des collections qui grandissent dynamiquement (sur le tas) :

- **`Vec<T>`** : une liste d'éléments du même type.
- **`String`** : un texte modifiable (en UTF-8).
- **`HashMap<K, V>`** : un dictionnaire (clé → valeur).

Avec les **itérateurs** (`map`, `filter`, `sum`, `collect`...), tu transformes ces collections sans écrire de boucle à la main.

### Tape ce code

<!-- fichier: src/bin/ch08_collections.rs -->
```rust
use std::collections::HashMap;

fn main() {
    // Vec : tableau dynamique
    let mut nombres = vec![3, 1, 4];
    nombres.push(1);
    nombres.push(5);
    println!("{nombres:?} (longueur {})", nombres.len());
    nombres.sort();
    println!("trié : {nombres:?}");

    // Parcours avec for (emprunt immuable)
    let mut total = 0;
    for n in &nombres {
        total += n;
    }
    println!("total = {total}");

    // Les itérateurs : map, filter, collect
    let doubles: Vec<i32> = nombres.iter().map(|n| n * 2).collect();
    let impairs: Vec<i32> = nombres.iter().filter(|n| *n % 2 == 1).copied().collect();
    println!("doubles : {doubles:?}");
    println!("impairs : {impairs:?}");
    let somme: i32 = nombres.iter().sum();
    println!("somme : {somme}");

    // String : texte modifiable en UTF-8
    let mut salut = String::from("Bonjour");
    salut.push_str(", monde");
    salut.push('!');
    println!("{salut} ({} octets)", salut.len());
    println!("majuscules : {}", salut.to_uppercase());
    for mot in "un deux trois".split(' ') {
        println!("mot : {mot}");
    }

    // HashMap : associe des clés à des valeurs
    let mut ages = HashMap::new();
    ages.insert("Awa", 28);
    ages.insert("Moussa", 34);
    println!("âge d'Awa : {:?}", ages.get("Awa"));
    println!("âge de Zoé : {:?}", ages.get("Zoé"));

    // Compter les occurrences des mots d'une phrase
    let phrase = "le chat et le chien et le poisson";
    let mut compteur: HashMap<&str, i32> = HashMap::new();
    for mot in phrase.split_whitespace() {
        *compteur.entry(mot).or_insert(0) += 1;
    }
    let mut paires: Vec<_> = compteur.iter().collect();
    paires.sort();
    println!("{paires:?}");
}
```

### Résultat attendu

<!-- sortie: ch08_collections -->
```text
[3, 1, 4, 1, 5] (longueur 5)
trié : [1, 1, 3, 4, 5]
total = 14
doubles : [2, 2, 6, 8, 10]
impairs : [1, 1, 3, 5]
somme : 14
Bonjour, monde! (15 octets)
majuscules : BONJOUR, MONDE!
mot : un
mot : deux
mot : trois
âge d'Awa : Some(28)
âge de Zoé : None
[("chat", 1), ("chien", 1), ("et", 2), ("le", 3), ("poisson", 1)]
```

### Ce qu'il faut comprendre

- `vec![3, 1, 4]` est une macro qui crée un `Vec`. `push` ajoute à la fin, `sort` trie sur place : les deux demandent un `Vec` déclaré `mut`.
- `for n in &nombres` **emprunte** le vecteur (chapitre 5). Sans le `&`, le `for` prendrait la propriété et tu ne pourrais plus utiliser `nombres` ensuite.
- `.iter()` crée un itérateur qui prête chaque élément. `.map(|n| n * 2)` applique une **closure** (une petite fonction anonyme, `|paramètre| résultat`) à chaque élément. `.filter(...)` en garde certains. `.copied()` transforme les références en valeurs. `.collect()` rassemble le résultat dans un nouveau `Vec`.
- Les itérateurs sont **paresseux** : rien ne se calcule tant qu'un `collect()` ou un `sum()` ne le demande.
- `String` est un texte qu'on peut agrandir (`push_str`, `push`). `len()` compte les **octets**. `split(' ')` découpe, `split_whitespace()` découpe sur tous les espaces.
- `HashMap::get` renvoie une `Option` (la clé peut ne pas exister), comme au chapitre 7.
- `compteur.entry(mot).or_insert(0)` renvoie une référence modifiable vers la valeur de la clé (créée à 0 si elle n'existe pas). Le `*` suit la référence pour faire le `+= 1`. C'est **le** motif classique pour compter.
- L'ordre d'un `HashMap` n'est **pas garanti** (il peut changer d'une exécution à l'autre) : on le met dans un `Vec` et on le trie avant d'afficher, pour avoir toujours le même résultat.

### Casse-le exprès

1. À la fin de `main`, avant l'accolade fermante, ajoute :

   ```rust
   let mut v = vec![1, 2, 3];
   let premier = &v[0];
   v.push(4);
   println!("premier = {premier}");
   ```

2. Lance `cargo run --bin ch08_collections`.

```text
error[E0502]: cannot borrow `v` as mutable because it is also borrowed as immutable
  --> src/bin/ch08_collections.rs:55:5
   |
54 |     let premier = &v[0];
   |                    - immutable borrow occurs here
55 |     v.push(4);
   |     ^^^^^^^^^ mutable borrow occurs here
56 |     println!("premier = {premier}");
   |                          ------- immutable borrow later used here
```

Voilà pourquoi la règle du chapitre 5 existe : `push` peut obliger le `Vec` à **déménager** en mémoire pour grandir, et `premier` pointerait alors vers un emplacement qui n'existe plus. En C ou C++ ce programme compile et plante de manière imprévisible ; en Rust il est refusé.

**Bonus : un plantage à l'exécution.** Remplace ces 4 lignes par :

```rust
let v = vec![1, 2, 3];
println!("{}", v[10]);
```

Là, ça compile, mais ça **panique** quand tu lances :

```text
thread 'main' (174762) panicked at src/bin/ch08_collections.rs:54:21:
index out of bounds: the len is 3 but the index is 10
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

Un indice hors limites ne fait pas planter la machine : Rust arrête proprement le programme avec un message clair (c'est une *panique*). Pour éviter la panique, utilise `v.get(10)`, qui renvoie une `Option` (chapitre 7). Supprime ces lignes.

### À toi de jouer

1. Écris `fn mots_longs(phrase: &str) -> Vec<String>` qui renvoie, **en majuscules**, les mots de plus de 4 lettres. Utilise des itérateurs (`split_whitespace`, `filter`, `map`, `collect`). *Indice : `mot.chars().count()` compte les lettres, alors que `mot.len()` compte les octets.*
2. Écris `fn frequence_lettres(texte: &str) -> HashMap<char, usize>` qui compte combien de fois chaque lettre apparaît (ignore les espaces avec `c.is_alphabetic()`).

Solution : `cargo run --example ch08_defi`

### À retenir

- `Vec` (liste), `String` (texte modifiable), `HashMap` (dictionnaire).
- Les itérateurs (`iter`, `map`, `filter`, `sum`, `collect`) évitent les boucles manuelles.
- Les règles d'emprunt s'appliquent aux collections : on ne peut pas modifier un `Vec` tant qu'on en emprunte un élément.

---

## Chapitre 9 — Traits et génériques

**Fichier :** `src/bin/ch09_traits.rs` · **Lancer :** `cargo run --bin ch09_traits`

### L'idée

- Un **trait** décrit un **comportement partagé** : « tout type qui sait faire ceci ». C'est l'équivalent des interfaces d'autres langages.
- Un **générique** (`<T>`) permet d'écrire **une seule fonction** qui marche pour plusieurs types.
- Une **contrainte de trait** (`T: Animal`) dit quels types sont acceptés : seulement ceux qui implémentent ce trait.

### Tape ce code

<!-- fichier: src/bin/ch09_traits.rs -->
```rust
use std::fmt;

trait Animal {
    fn nom(&self) -> String;
    fn cri(&self) -> String;

    // Méthode avec implémentation par défaut
    fn presentation(&self) -> String {
        format!("{} fait « {} »", self.nom(), self.cri())
    }
}

struct Chien;
struct Chat {
    nom: String,
}

impl Animal for Chien {
    fn nom(&self) -> String {
        String::from("Le chien")
    }

    fn cri(&self) -> String {
        String::from("ouaf")
    }
}

impl Animal for Chat {
    fn nom(&self) -> String {
        self.nom.clone()
    }

    fn cri(&self) -> String {
        String::from("miaou")
    }

    // On peut aussi remplacer l'implémentation par défaut
    fn presentation(&self) -> String {
        format!("{} ronronne puis dit « {} »", self.nom, self.cri())
    }
}

// Générique avec contrainte de trait : T doit implémenter Animal
fn fais_parler<T: Animal>(animal: &T) {
    println!("{}", animal.presentation());
}

// Générique : fonctionne pour tout type comparable et copiable
fn plus_grand<T: PartialOrd + Copy>(valeurs: &[T]) -> T {
    let mut max = valeurs[0];
    for &v in valeurs {
        if v > max {
            max = v;
        }
    }
    max
}

// Un struct générique
struct Paire<T> {
    a: T,
    b: T,
}

impl<T: fmt::Display> fmt::Display for Paire<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.a, self.b)
    }
}

fn main() {
    let chien = Chien;
    let chat = Chat {
        nom: String::from("Mimi"),
    };
    fais_parler(&chien);
    fais_parler(&chat);

    // dyn Trait : une collection d'animaux de types différents
    let zoo: Vec<Box<dyn Animal>> = vec![Box::new(Chien), Box::new(chat)];
    for animal in &zoo {
        println!("- {}", animal.presentation());
    }

    println!("{}", plus_grand(&[3, 9, 2]));
    println!("{}", plus_grand(&[1.5, 0.2, 7.25]));
    println!("{}", plus_grand(&['x', 'b', 'z']));

    let p = Paire { a: 1, b: 2 };
    println!("{p}");
    let q = Paire {
        a: "gauche",
        b: "droite",
    };
    println!("{q}");
}
```

### Résultat attendu

<!-- sortie: ch09_traits -->
```text
Le chien fait « ouaf »
Mimi ronronne puis dit « miaou »
- Le chien fait « ouaf »
- Mimi ronronne puis dit « miaou »
9
7.25
z
(1, 2)
(gauche, droite)
```

### Ce qu'il faut comprendre

- `trait Animal { ... }` déclare **ce qu'un animal sait faire** : deux méthodes à fournir (`nom`, `cri`) et une troisième (`presentation`) qui a déjà un **corps par défaut**, construit à partir des deux premières.
- `impl Animal for Chien` dit « `Chien` sait faire tout ça » et fournit le code. `Chat` fait pareil, mais **remplace** `presentation` par sa propre version.
- `fn fais_parler<T: Animal>(animal: &T)` : une seule fonction pour tous les types qui implémentent `Animal`. Le compilateur génère le code spécialisé pour chaque type utilisé (c'est rapide : aucun coût à l'exécution).
- `Vec<Box<dyn Animal>>` : quand tu veux **mélanger des types différents** dans une même collection, tu utilises `dyn Animal` (un objet-trait). `Box` est un pointeur vers le tas : il faut le passer parce que `Chien` et `Chat` n'ont pas la même taille en mémoire.
- `plus_grand<T: PartialOrd + Copy>` : `PartialOrd` autorise la comparaison `>`, et `Copy` permet de copier une valeur hors de la slice. Tu peux cumuler les contraintes avec `+`. Cette même fonction marche avec des entiers, des flottants et des caractères.
- `struct Paire<T>` est un **struct générique** : `Paire<i32>` ou `Paire<&str>`, au choix.
- `impl<T: fmt::Display> fmt::Display for Paire<T>` implémente le trait standard `Display`, celui qui permet d'écrire `{}` dans `println!`. On le fait **seulement** quand `T` sait déjà s'afficher.

### Casse-le exprès

1. À la fin de `main`, avant l'accolade fermante, ajoute :

   ```rust
   fais_parler(&42);
   ```

2. Lance `cargo run --bin ch09_traits`.

```text
error[E0277]: the trait bound `{integer}: Animal` is not satisfied
  --> src/bin/ch09_traits.rs:96:17
   |
96 |     fais_parler(&42);
   |     ----------- ^^^ the trait `Animal` is not implemented for `{integer}`
   |     |
   |     required by a bound introduced by this call
   |
help: the following other types implement trait `Animal`
  --> src/bin/ch09_traits.rs:18:1
   |
18 | impl Animal for Chien {
   | ^^^^^^^^^^^^^^^^^^^^^ `Chien`
...
28 | impl Animal for Chat {
   | ^^^^^^^^^^^^^^^^^^^^ `Chat`
```

Un nombre n'est pas un animal : le compilateur le sait et te le dit, en listant même les types qui, eux, implémentent `Animal`. Supprime la ligne.

### À toi de jouer

1. Ajoute un `struct Vache` qui implémente `Animal` (cri « meuh ») et ajoute-en une au `zoo`.
2. Écris `fn affiche_tout<T: fmt::Display>(elements: &[T])` qui affiche chaque élément sur sa propre ligne. Teste-la avec des `i32`, des `&str`, et un tableau de `Paire<i32>`.

Solution : `cargo run --example ch09_defi`

### À retenir

- Un **trait** définit un comportement ; `impl Trait for Type` l'implémente.
- Un **générique** `<T: Trait>` écrit le code une fois pour tous les types qui respectent la contrainte.
- `Box<dyn Trait>` pour des types différents dans une même collection.

---

## Chapitre 10 — Tests et mini-projet

**Fichier :** `src/bin/ch10_taches.rs` · **Lancer :** `cargo run --bin ch10_taches` · **Tester :** `cargo test --bin ch10_taches`

### L'idée

Dernier chapitre : on met tout en pratique dans un vrai petit programme, un **gestionnaire de tâches** en ligne de commande. Il réutilise presque tout ce que tu as vu : `struct`, `enum`, `impl`, `Vec`, `Result`, `?`, `match`, emprunts, itérateurs.

Et tu découvres les **tests**, intégrés à Rust : une fonction marquée `#[test]` est exécutée par `cargo test`. Le module de tests se met dans le même fichier, derrière `#[cfg(test)]` (ce code n'est compilé que pour les tests).

### Tape ce code (partie 1 : le programme)

<!-- fichier: src/bin/ch10_taches.rs -->
```rust
use std::io::{self, Write};

#[derive(Debug)]
struct Tache {
    titre: String,
    faite: bool,
}

#[derive(Default)]
struct Liste {
    taches: Vec<Tache>,
}

impl Liste {
    fn ajouter(&mut self, titre: &str) {
        self.taches.push(Tache {
            titre: titre.to_string(),
            faite: false,
        });
    }

    // Vérifie un numéro (à partir de 1, comme à l'affichage) et renvoie l'indice dans le Vec
    fn indice(&self, numero: usize) -> Result<usize, String> {
        if numero == 0 || numero > self.taches.len() {
            Err(format!("la tâche {numero} n'existe pas"))
        } else {
            Ok(numero - 1)
        }
    }

    fn terminer(&mut self, numero: usize) -> Result<(), String> {
        let i = self.indice(numero)?;
        self.taches[i].faite = true;
        Ok(())
    }

    fn supprimer(&mut self, numero: usize) -> Result<Tache, String> {
        let i = self.indice(numero)?;
        Ok(self.taches.remove(i))
    }

    fn afficher(&self) {
        if self.taches.is_empty() {
            println!("(aucune tâche)");
        }
        for (i, tache) in self.taches.iter().enumerate() {
            let case = if tache.faite { "x" } else { " " };
            println!("{}. [{case}] {}", i + 1, tache.titre);
        }
    }
}

enum Suite {
    Continuer,
    Quitter,
}

fn lire_numero(texte: &str) -> Result<usize, String> {
    texte
        .parse::<usize>()
        .map_err(|_| format!("« {texte} » n'est pas un numéro valide"))
}

fn executer(liste: &mut Liste, ligne: &str) -> Result<Suite, String> {
    let ligne = ligne.trim();
    let (commande, reste) = match ligne.split_once(' ') {
        Some((c, r)) => (c, r.trim()),
        None => (ligne, ""),
    };

    match commande {
        "ajouter" => {
            if reste.is_empty() {
                return Err(String::from("usage : ajouter <texte>"));
            }
            liste.ajouter(reste);
            println!("ajoutée : {reste}");
        }
        "liste" => liste.afficher(),
        "fait" => {
            let n = lire_numero(reste)?;
            liste.terminer(n)?;
            println!("tâche {n} terminée");
        }
        "supprimer" => {
            let n = lire_numero(reste)?;
            let tache = liste.supprimer(n)?;
            println!("supprimée : {}", tache.titre);
        }
        "quitter" => return Ok(Suite::Quitter),
        "" => {}
        autre => return Err(format!("commande inconnue : {autre}")),
    }
    Ok(Suite::Continuer)
}

fn main() {
    let mut liste = Liste::default();
    println!("Commandes : ajouter <texte>, liste, fait <n>, supprimer <n>, quitter");

    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let mut ligne = String::new();
        let lu = io::stdin().read_line(&mut ligne).unwrap();
        if lu == 0 {
            break; // fin de l'entrée (Ctrl-D)
        }

        match executer(&mut liste, &ligne) {
            Ok(Suite::Continuer) => {}
            Ok(Suite::Quitter) => break,
            Err(message) => println!("erreur : {message}"),
        }
    }
    println!("À bientôt !");
}
```

### Lance-le et essaie

Ce programme est **interactif** : il attend que tu tapes des commandes. Voici une session type (les lignes qui commencent par `> ` sont ce que **tu** tapes) :

<!-- session: ch10_taches -->
```text
Commandes : ajouter <texte>, liste, fait <n>, supprimer <n>, quitter
> ajouter apprendre Rust
ajoutée : apprendre Rust
> ajouter faire les courses
ajoutée : faire les courses
> liste
1. [ ] apprendre Rust
2. [ ] faire les courses
> fait 1
tâche 1 terminée
> liste
1. [x] apprendre Rust
2. [ ] faire les courses
> fait 9
erreur : la tâche 9 n'existe pas
> fait abc
erreur : « abc » n'est pas un numéro valide
> supprimer 2
supprimée : faire les courses
> machin
erreur : commande inconnue : machin
> quitter
À bientôt !
```

### Tape ce code (partie 2 : les tests)

Ajoute ceci **à la fin du fichier**, après `main` :

<!-- ajout: src/bin/ch10_taches.rs -->
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ajouter_une_tache() {
        let mut liste = Liste::default();
        liste.ajouter("apprendre Rust");
        assert_eq!(liste.taches.len(), 1);
        assert_eq!(liste.taches[0].titre, "apprendre Rust");
        assert!(!liste.taches[0].faite);
    }

    #[test]
    fn terminer_une_tache() {
        let mut liste = Liste::default();
        liste.ajouter("faire les courses");
        assert_eq!(liste.terminer(1), Ok(()));
        assert!(liste.taches[0].faite);
    }

    #[test]
    fn numero_invalide() {
        let mut liste = Liste::default();
        assert!(liste.terminer(1).is_err());
        liste.ajouter("une tâche");
        assert!(liste.terminer(0).is_err());
        assert!(liste.terminer(2).is_err());
    }

    #[test]
    fn supprimer_renvoie_la_tache() {
        let mut liste = Liste::default();
        liste.ajouter("a");
        liste.ajouter("b");
        let tache = liste.supprimer(1).unwrap();
        assert_eq!(tache.titre, "a");
        assert_eq!(liste.taches.len(), 1);
    }
}
```

Lance les tests :

```bash
cargo test --bin ch10_taches
```

<!-- tests: ch10_taches -->
```text
running 4 tests
test tests::ajouter_une_tache ... ok
test tests::numero_invalide ... ok
test tests::terminer_une_tache ... ok
test tests::supprimer_renvoie_la_tache ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

(L'ordre des lignes peut varier d'une exécution à l'autre : les tests tournent en parallèle.)

### Ce qu'il faut comprendre

- **Les structures :** `Tache` regroupe un titre et un état ; `Liste` contient un `Vec<Tache>`. `#[derive(Default)]` fournit `Liste::default()` : une liste vide.
- **`indice`** centralise la vérification d'un numéro. Les utilisateurs comptent à partir de 1, le `Vec` à partir de 0 : on traduit à un seul endroit. Elle renvoie un `Result`, et `terminer` et `supprimer` la propagent avec `?` (chapitre 7).
- **`afficher`** utilise `iter().enumerate()`, qui donne `(position, élément)` à chaque tour.
- **`enum Suite`** dit à `main` s'il faut continuer ou quitter. `executer` renvoie `Result<Suite, String>` : soit la suite à donner, soit un message d'erreur à afficher.
- **`ligne.split_once(' ')`** coupe au premier espace et renvoie une `Option` de deux morceaux. `trim()` retire les espaces et le retour à la ligne.
- **`map_err(|_| ...)`** transforme l'erreur de `parse` (un `ParseIntError`) en notre propre message. `|_|` est une closure qui ignore son argument.
- **Dans le `match`**, `autre =>` capte « tous les autres cas » en gardant la valeur (contrairement à `_`).
- **Entrées/sorties :** `print!` n'affiche pas tout de suite (il attend un retour à la ligne), d'où `io::stdout().flush()` (qui exige `use std::io::Write`). `read_line` ajoute la ligne tapée à la `String` et renvoie le nombre d'octets lus : **0 signifie que l'entrée est terminée** (Ctrl-D). Le `.unwrap()` est acceptable ici : si le terminal lui-même est cassé, on n'a rien de mieux à faire que s'arrêter.
- **Les tests :** `#[cfg(test)]` n'est compilé que pour `cargo test`. `use super::*;` importe tout le fichier parent. `assert!` vérifie qu'une condition est vraie, `assert_eq!` que deux valeurs sont égales (en affichant les deux si ce n'est pas le cas).

### Casse-le exprès

Un test qui n'a jamais échoué ne prouve rien : vérifie que les tiens savent échouer.

1. Dans le test `ajouter_une_tache`, remplace `"apprendre Rust"` par `"apprendre Python"` **dans la ligne `assert_eq!(liste.taches[0].titre, ...)`**.
2. Lance `cargo test --bin ch10_taches`.

```text
running 4 tests
test tests::terminer_une_tache ... ok
test tests::supprimer_renvoie_la_tache ... ok
test tests::numero_invalide ... ok
test tests::ajouter_une_tache ... FAILED

failures:

---- tests::ajouter_une_tache stdout ----

thread 'tests::ajouter_une_tache' (174972) panicked at src/bin/ch10_taches.rs:129:9:
assertion `left == right` failed
  left: "apprendre Rust"
 right: "apprendre Python"

failures:
    tests::ajouter_une_tache

test result: FAILED. 3 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out
```

Le test échoue, et `assert_eq!` t'affiche les deux valeurs : `left` est la valeur réelle, `right` celle que tu attendais. Remets `"apprendre Rust"`.

### À toi de jouer

Ajoute une commande `reste` qui affiche combien de tâches ne sont pas encore terminées. Fais-le **en écrivant les tests d'abord**, comme les développeurs Rust :

1. **Rouge :** ajoute dans `mod tests` un test `restantes_compte_les_taches_non_terminees` qui appelle `liste.restantes()` (0 pour une liste vide, 2 après deux `ajouter`, 1 après un `terminer(1)`). Lance `cargo test` : ça ne compile même pas, car la méthode n'existe pas. C'est normal : c'est le « rouge ».
2. **Vert :** écris `fn restantes(&self) -> usize` dans `impl Liste` *(indice : `self.taches.iter().filter(...).count()`)*. Relance les tests jusqu'à ce qu'ils passent tous.
3. **Branche-la :** ajoute `"reste" => println!(...)` dans `executer`, et lance le programme pour l'essayer.

**Bonus :** sauvegarde les tâches dans un fichier avec `std::fs::write` et recharge-les au démarrage avec `std::fs::read_to_string`.

Solution : `cargo run --example ch10_defi` et `cargo test --example ch10_defi`

### À retenir

- `#[test]` + `cargo test` : les tests sont un citoyen de première classe dans Rust.
- `assert!` et `assert_eq!` vérifient ; un test qui n'a jamais été vu en échec ne prouve rien.
- Un vrai programme, c'est des `struct` et des `enum` pour modéliser, des `Result` pour les erreurs, et des emprunts bien choisis.

---

## Bravo, et maintenant ?

Tu as vu les fondations : variables, types, flux de contrôle, ownership, emprunts, structs et enums, `Option` et `Result`, collections, traits, génériques et tests. Ce sont **80 % de ce qu'il faut pour lire n'importe quel code Rust**.

### Les prochaines étapes

| Sujet | Pourquoi |
|-------|----------|
| **Les durées de vie** (`'a`) | Expliquer au compilateur combien de temps une référence reste valide |
| **Les modules** (`mod`, `use`, `pub`) | Organiser un projet en plusieurs fichiers |
| **Les closures et itérateurs en profondeur** | Écrire du Rust idiomatique, sans boucles manuelles |
| **Les pointeurs intelligents** (`Box`, `Rc`, `RefCell`) | Partager et modifier des données de façon flexible |
| **Les erreurs personnalisées** (`enum` d'erreurs, crates `thiserror` / `anyhow`) | Gérer proprement les erreurs d'un vrai programme |
| **La concurrence** (`thread`, `Arc`, `Mutex`, canaux) | Programmer en parallèle sans data race |
| **Les crates** (`serde`, `clap`, `tokio`) | Utiliser l'écosystème : JSON, arguments de ligne de commande, asynchrone |

### Des projets pour s'entraîner

- Un petit **`grep`** : cherche un mot dans un fichier (lecture de fichier, `String`, itérateurs).
- Le jeu du **pendu** ou du **plus ou moins** (entrées/sorties, boucles, `match`).
- Un **convertisseur d'unités** en ligne de commande, avec `clap` pour les arguments.
- Étends ton **gestionnaire de tâches** : sauvegarde en fichier, priorités (un `enum`), dates limites.

### Pour aller plus loin

- *The Rust Programming Language* (« le Book »), le livre officiel : <https://doc.rust-lang.org/book/>
- *Rust by Example*, des exemples commentés : <https://doc.rust-lang.org/rust-by-example/>
- *Rustlings*, de petits exercices à corriger dans ton terminal : <https://github.com/rust-lang/rustlings>
- `rustc --explain E0382` : explique n'importe quel code d'erreur du compilateur.

Un dernier conseil : quand le compilateur te refuse quelque chose, ne le vis pas comme un obstacle. **Chaque erreur de compilation est un bug que tu n'auras pas à chercher plus tard.**

