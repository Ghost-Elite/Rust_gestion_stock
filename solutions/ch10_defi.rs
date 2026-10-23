// Solution du défi du chapitre 10 (le code du chapitre + la commande `reste`).
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

    // DÉFI : le nombre de tâches pas encore terminées
    fn restantes(&self) -> usize {
        self.taches.iter().filter(|t| !t.faite).count()
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
        // DÉFI : la nouvelle commande
        "reste" => println!("{} tâche(s) restante(s)", liste.restantes()),
        "quitter" => return Ok(Suite::Quitter),
        "" => {}
        autre => return Err(format!("commande inconnue : {autre}")),
    }
    Ok(Suite::Continuer)
}

fn main() {
    let mut liste = Liste::default();
    println!("Commandes : ajouter <texte>, liste, fait <n>, supprimer <n>, reste, quitter");

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

    // DÉFI : ce test a été écrit AVANT la méthode `restantes`
    #[test]
    fn restantes_compte_les_taches_non_terminees() {
        let mut liste = Liste::default();
        assert_eq!(liste.restantes(), 0);
        liste.ajouter("a");
        liste.ajouter("b");
        assert_eq!(liste.restantes(), 2);
        liste.terminer(1).unwrap();
        assert_eq!(liste.restantes(), 1);
    }
}
