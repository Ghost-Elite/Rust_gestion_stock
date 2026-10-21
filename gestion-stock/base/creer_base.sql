-- À lancer UNE SEULE FOIS, par vous, avec le compte root de XAMPP :
--
--     /Applications/XAMPP/xamppfiles/bin/mysql -u root < creer_base.sql
--
-- Avant de le lancer, REMPLACEZ « CHANGEZ-MOI » par un vrai mot de passe (et recopiez-le dans
-- base.conf). Ce fichier ne touche à aucune autre base de XAMPP : il crée la base « kora_marche »
-- et un compte « kora » qui n'a de droits que sur elle.

CREATE DATABASE IF NOT EXISTS kora_marche CHARACTER SET utf8mb4;

CREATE USER IF NOT EXISTS 'kora'@'localhost' IDENTIFIED BY 'CHANGEZ-MOI';
-- Pour un autre poste du réseau, ajoutez une ligne semblable avec son adresse, par exemple :
-- CREATE USER IF NOT EXISTS 'kora'@'192.168.1.%' IDENTIFIED BY 'CHANGEZ-MOI';

GRANT ALL PRIVILEGES ON kora_marche.* TO 'kora'@'localhost';
FLUSH PRIVILEGES;
