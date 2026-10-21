-- Kora Marché : schéma version 1 (MariaDB 10.4, InnoDB, utf8mb4).
-- Réexécutable : chaque table est créée seulement si elle manque.
-- Les colonnes `*_cle` sont le nom nettoyé en MINUSCULES, calculé par le programme, comparé en
-- binaire : « Riz » et « riz » sont le même produit, « Thé » et « the » sont deux produits.

CREATE TABLE IF NOT EXISTS schema_version (
  version INT UNSIGNED NOT NULL PRIMARY KEY,
  applique_le DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE IF NOT EXISTS reglage (
  cle VARCHAR(40) NOT NULL PRIMARY KEY,
  valeur VARCHAR(100) NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE IF NOT EXISTS caisse (
  code CHAR(2) NOT NULL PRIMARY KEY,
  nom VARCHAR(60) NOT NULL DEFAULT '',
  dernier_numero BIGINT UNSIGNED NOT NULL DEFAULT 0,
  dernier_hash CHAR(64) NOT NULL DEFAULT '0000000000000000000000000000000000000000000000000000000000000000'
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE IF NOT EXISTS produit (
  id INT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
  nom VARCHAR(190) NOT NULL,
  nom_cle VARCHAR(190) CHARACTER SET utf8mb4 COLLATE utf8mb4_bin NOT NULL,
  quantite INT UNSIGNED NOT NULL DEFAULT 0,
  seuil INT UNSIGNED NOT NULL DEFAULT 0,
  prix BIGINT NULL,
  taux_bp INT UNSIGNED NULL,
  UNIQUE KEY produit_nom_cle (nom_cle)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE IF NOT EXISTS mouvement (
  id BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
  produit_nom VARCHAR(190) NOT NULL,
  produit_cle VARCHAR(190) CHARACTER SET utf8mb4 COLLATE utf8mb4_bin NOT NULL,
  genre ENUM('entree', 'sortie') NOT NULL,
  quantite INT UNSIGNED NOT NULL,
  caisse CHAR(2) NOT NULL,
  piece_numero BIGINT UNSIGNED NULL,
  cree_le DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  KEY mouvement_produit (produit_cle),
  CONSTRAINT mouvement_caisse FOREIGN KEY (caisse) REFERENCES caisse (code)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE IF NOT EXISTS piece (
  caisse CHAR(2) NOT NULL,
  numero BIGINT UNSIGNED NOT NULL,
  genre CHAR(1) NOT NULL,
  annule_numero BIGINT UNSIGNED NULL,
  utc BIGINT NOT NULL,
  decalage_minutes INT NOT NULL,
  total BIGINT NOT NULL,
  moyen CHAR(1) NOT NULL,
  recu BIGINT NOT NULL,
  rendu BIGINT NOT NULL,
  hash_precedent CHAR(64) NOT NULL,
  hash CHAR(64) NOT NULL,
  PRIMARY KEY (caisse, numero),
  UNIQUE KEY piece_annulation (caisse, annule_numero),
  CONSTRAINT piece_caisse FOREIGN KEY (caisse) REFERENCES caisse (code),
  CONSTRAINT piece_genre CHECK (genre IN ('V', 'A')),
  CONSTRAINT piece_moyen CHECK (moyen IN ('E', 'C', 'X')),
  CONSTRAINT piece_annulation_nommee CHECK ((genre = 'A') = (annule_numero IS NOT NULL))
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE IF NOT EXISTS piece_ligne (
  caisse CHAR(2) NOT NULL,
  numero BIGINT UNSIGNED NOT NULL,
  rang INT UNSIGNED NOT NULL,
  nom VARCHAR(190) NOT NULL,
  nom_cle VARCHAR(190) CHARACTER SET utf8mb4 COLLATE utf8mb4_bin NOT NULL,
  quantite INT UNSIGNED NOT NULL,
  prix BIGINT NOT NULL,
  taux_bp INT UNSIGNED NOT NULL,
  PRIMARY KEY (caisse, numero, rang),
  KEY piece_ligne_produit (nom_cle),
  CONSTRAINT piece_ligne_piece FOREIGN KEY (caisse, numero) REFERENCES piece (caisse, numero)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;
