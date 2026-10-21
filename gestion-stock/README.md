# Gestion de stock

Un petit programme en ligne de commande pour suivre le stock d'une boutique : produits, entrées de marchandise, sorties (ventes, pertes), alertes quand un produit est bientôt épuisé, et historique des mouvements.

Tout est écrit en Rust. Les bibliothèques externes sont `sha2` (les empreintes SHA-256 du journal des ventes, utilisé par l'interface graphique) et `mysql` (la base de données, facultative : voir plus bas). Par défaut, les données sont gardées dans un simple fichier texte et le programme n'ouvre aucune connexion réseau.

> **Cette version ne vend pas** : les prix, les ventes, les tickets et les annulations se font dans l'interface graphique (`../gestion-stock-gui`). Elle en conserve toutefois les données dans le fichier.

## Installation

```bash
cargo build --release
```

Le programme est alors dans `target/release/stock`. Copie-le où tu veux et lance-le depuis un terminal.

## Les commandes

Un nom de produit qui contient des espaces se met entre guillemets. Les majuscules n'ont pas d'importance : `riz 5kg` et `Riz 5KG` désignent le même produit.

| Commande | Ce qu'elle fait | Exemple |
|---|---|---|
| `stock ajouter <nom> [--seuil N]` | Crée un produit (stock à 0). Le seuil d'alerte vaut 0 par défaut. | `stock ajouter "Riz 5kg" --seuil 5` |
| `stock entree <nom> <quantité>` | Enregistre une entrée de marchandise. | `stock entree "Riz 5kg" 12` |
| `stock sortie <nom> <quantité>` | Enregistre une sortie. Refusée si le stock ne suffit pas. | `stock sortie "Riz 5kg" 2` |
| `stock lister` | Affiche tout le stock. | `stock lister` |
| `stock alertes` | Affiche seulement les produits en alerte (quantité inférieure ou égale au seuil). | `stock alertes` |
| `stock historique [N]` | Affiche les N derniers mouvements, du plus récent au plus ancien (10 par défaut). | `stock historique 20` |
| `stock aide` | Affiche la liste des commandes (aussi : `stock` sans rien). | `stock aide` |

Quand une sortie fait passer un produit sous son seuil, le programme affiche une ligne `ATTENTION : stock bas pour « … »`.

En cas d'erreur, le programme affiche `Erreur : …` et ne change rien aux données.

## Où sont les données ?

Dans le fichier `stock.txt`, dans le dossier où tu lances le programme. Pour utiliser un autre fichier, indique son chemin dans la variable d'environnement `STOCK_FICHIER` :

```bash
STOCK_FICHIER=/chemin/vers/mon-stock.txt stock lister
```

**Attention au dossier.** Si le programme ne trouve pas le fichier, il part d'un stock vide et te prévient sur la sortie d'erreur : `Note : fichier de données introuvable (« … »). On part d'un stock vide.` Si tu vois ce message alors que tu as déjà du stock, tu n'es pas dans le bon dossier : n'enregistre rien, sinon tu crées un deuxième fichier. Le plus sûr est de toujours indiquer le chemin complet avec `STOCK_FICHIER`. À la toute première utilisation, le programme annonce aussi la création du fichier : `Nouveau fichier de données créé : « … »`.

Le fichier est du texte lisible. Si tu as fait une faute de frappe dans un nom de produit, tu peux l'ouvrir et la corriger à la main (une ligne `P;nom;quantité;seuil;prix;taux` par produit ; les deux derniers champs sont vides si le produit n'a pas de prix). **Ne touche pas aux lignes `S;` et `L;`** : c'est le journal des ventes, protégé par des empreintes ; l'interface graphique détecterait la modification et bloquerait les ventes.

## La base MySQL (XAMPP)

**Facultative.** Sans elle, rien ne change : le programme travaille sur le fichier `stock.txt`. La base sert à partager les produits, le stock et les ventes entre plusieurs postes (chaque poste est une « caisse » : `01`, `04`…, avec ses propres numéros de ticket). Elle fonctionne avec le MariaDB de XAMPP.

> **État actuel.** Cette étape ajoute la couche de stockage (tables, transactions, import, vérification). Les commandes ci-dessus (`ajouter`, `entree`, `lister`…) et l'interface graphique **travaillent encore sur le fichier** ; l'étape suivante les branchera sur la base. Pour l'instant, la base se prépare, se remplit avec `stock importer` et se vérifie.

1. **Démarrez MySQL** dans le gestionnaire XAMPP.
2. **Sécurisez XAMPP** avant de brancher un deuxième poste : par défaut le compte `root` n'a pas de mot de passe et le serveur écoute sur tout le réseau. Donnez un mot de passe à `root` (par exemple avec `/Applications/XAMPP/xamppfiles/bin/mysqladmin -u root password`) et, si un seul poste sert de serveur, ajoutez `bind-address=127.0.0.1` dans la section `[mysqld]` de `/Applications/XAMPP/xamppfiles/etc/my.cnf`.
3. **Créez la base et son compte**, une seule fois : ouvrez `base/creer_base.sql`, remplacez `CHANGEZ-MOI` par un vrai mot de passe, puis lancez `/Applications/XAMPP/xamppfiles/bin/mysql -u root < base/creer_base.sql`. Cela crée la base `kora_marche` et un compte `kora` qui n'a de droits que sur elle ; les autres bases de XAMPP ne sont pas touchées.
4. **Dites au programme où est la base** : créez `~/.config/kora-marche/base.conf` (ou indiquez un autre chemin avec la variable `KORA_BASE_CONF`) :

   ```text
   hote=127.0.0.1
   port=3306
   base=kora_marche
   utilisateur=kora
   mot_de_passe=le-mot-de-passe-de-l-etape-3
   caisse=01
   ```

   `caisse` est le numéro de ce poste, sur deux chiffres. Ce fichier contient un mot de passe : `chmod 600 ~/.config/kora-marche/base.conf` (le programme vous avertit si d'autres utilisateurs peuvent le lire). Il ne doit jamais être mis dans Git.
5. **Créez les tables** : `stock migrer` (sans danger si vous le lancez deux fois ; refusé si la base est plus récente que le programme).
6. **Importez votre stock actuel** : `stock importer stock.txt`. Les produits, les mouvements, les réglages et le journal des ventes sont copiés **tels quels** (mêmes numéros de ticket, mêmes empreintes) dans la caisse de ce poste. La base doit être vide ; le fichier n'est jamais modifié ; si le journal du fichier a été altéré, rien n'est importé.
7. **Vérifiez le journal** quand vous voulez : `stock verifier` recalcule toute la chaîne d'empreintes de la caisse et dit quelle pièce ne correspond plus.

**Sauvegarde.** La base devient alors l'unique copie des ventes : sauvegardez-la régulièrement, par exemple `/Applications/XAMPP/xamppfiles/bin/mysqldump -u kora -p kora_marche > sauvegarde.sql`.

**À savoir.** La liaison avec le serveur n'est pas chiffrée : à réserver à un réseau local de confiance. Si le serveur ne répond pas, le programme le dit clairement (délai de 3 secondes pour se connecter, 5 pour lire ou écrire) ; une vente n'est jamais enregistrée à moitié.

## Changement de format du fichier (version 2)

Dès qu'une commande enregistre le fichier (même une simple entrée), il est réécrit au **format version 2** : il contient en plus les prix, les réglages et le journal des ventes de l'interface graphique. Les fichiers de l'ancien format se lisent sans perte, mais **l'ancienne version du programme ne sait pas relire le format version 2** : **garde une copie de `stock.txt` avant de passer à la nouvelle version.**

## Sauvegarde conseillée

Copie régulièrement `stock.txt` (sur une clé USB, par exemple) : c'est la seule chose à conserver. Le programme écrit d'abord dans un fichier temporaire puis le renomme, pour que le fichier ne soit pas abîmé si l'ordinateur s'éteint au mauvais moment.

## Pour les développeurs

```bash
cargo test                          # tous les tests (sans serveur MySQL)
KORA_TEST_URL=mysql://root@127.0.0.1:3306 cargo test -- --ignored   # tests de la base (MySQL démarré)
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

Le cœur (`src/stock.rs`, `src/stockage.rs`) n'affiche rien et ne lit pas le terminal : il peut être réutilisé tel quel par une autre interface, par exemple une interface graphique.
