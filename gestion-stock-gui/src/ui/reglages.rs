//! L'écran Réglages : profil (Europe ou États-Unis), devise, taux par défaut, et le fichier de
//! données utilisé. Rien ne compte avant « Enregistrer » ; une fois des pièces au journal, la
//! devise et le mode de prix ne changent plus.

use super::composants::*;
use super::{Application, LARGEUR_DEUX_COLONNES};
use crate::etat::chemin_complet;
use crate::palette::{FOND, couleur};
use crate::textes::*;
use eframe::egui::{CentralPanel, Frame, Margin, ScrollArea, Ui};
use gestion_stock::monnaie::Devise;
use gestion_stock::reglages::Profil;

/// La page d'un écran « de lecture » (Ventes, Réglages) : une colonne centrée, qui défile.
fn page(ui: &mut Ui, largeur_max: f32, contenu: impl FnOnce(&mut Ui)) {
    let marge = if ui.available_width() >= LARGEUR_DEUX_COLONNES {
        Margin::symmetric(24, 16)
    } else {
        Margin::symmetric(16, 14)
    };
    CentralPanel::default()
        .frame(Frame::new().fill(couleur(FOND)).inner_margin(marge))
        .show(ui, |ui| {
            ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
                ui.set_max_width(largeur_max);
                contenu(ui);
            });
        });
}

// ---- Les réglages --------------------------------------------------------------------------------

fn nom_du_profil(profil: Profil) -> &'static str {
    match profil {
        Profil::Europe => PROFIL_EUROPE,
        Profil::EtatsUnis => PROFIL_ETATS_UNIS,
    }
}

fn nom_de_la_devise(devise: Devise) -> &'static str {
    match devise {
        Devise::Eur => DEVISE_EUR,
        Devise::Usd => DEVISE_USD,
        Devise::Gbp => DEVISE_GBP,
        Devise::Xof => DEVISE_XOF,
    }
}

pub(super) fn ecran_reglages(ui: &mut Ui, app: &mut Application) {
    page(ui, 640.0, |ui| {
        carte(ui, |ui| formulaire_reglages(ui, app));
        carte(ui, |ui| {
            titre_de_carte(ui, SECTION_FICHIER);
            if let Some(etat) = &app.etat {
                discret(ui, &pied_de_page_donnees(&chemin_complet(etat.chemin())));
            }
        });
    });
}

/// Les choix se font sur une copie (le brouillon) : rien ne compte avant « Enregistrer ». Une
/// fois des ventes enregistrées, la devise et le mode de prix sont grisés (voir `Etat`).
fn formulaire_reglages(ui: &mut Ui, app: &mut Application) {
    titre_de_carte(ui, SECTION_REGLAGES);
    ui.label(RAPPEL_REGLAGES);
    let Some(etat) = &app.etat else { return };
    let enregistres = etat.reglages();
    let verrouilles = etat.reglages_verrouilles();
    let depart = app
        .reglages_saisis
        .clone()
        .unwrap_or_else(|| super::BrouillonReglages {
            profil: enregistres.profil,
            devise: enregistres.devise,
            taux: enregistres
                .devise
                .formater_taux_brut(enregistres.taux_defaut_bp),
        });
    let mut brouillon = depart.clone();

    ui.add_space(4.0);
    discret(ui, LIBELLE_PROFIL);
    ui.add_enabled_ui(!verrouilles, |ui| {
        ui.horizontal_wrapped(|ui| {
            for profil in [Profil::Europe, Profil::EtatsUnis] {
                let choisi = brouillon.profil == profil;
                if bouton_choix(ui, nom_du_profil(profil), choisi, 200.0).clicked() {
                    brouillon.profil = profil;
                }
            }
        });
    });
    ui.label(match brouillon.profil {
        Profil::Europe => EXPLICATION_EUROPE,
        Profil::EtatsUnis => EXPLICATION_ETATS_UNIS,
    });

    ui.add_space(4.0);
    discret(ui, LIBELLE_DEVISE);
    ui.add_enabled_ui(!verrouilles, |ui| {
        ui.horizontal_wrapped(|ui| {
            for devise in [Devise::Eur, Devise::Usd, Devise::Gbp, Devise::Xof] {
                let choisie = brouillon.devise == devise;
                if bouton_choix(ui, nom_de_la_devise(devise), choisie, 120.0).clicked() {
                    brouillon.devise = devise;
                }
            }
        });
    });

    ui.add_space(4.0);
    champ(ui, CHAMP_TAUX_DEFAUT, &mut brouillon.taux);
    if verrouilles {
        discret(ui, REGLAGES_VERROUILLES);
    }
    ui.add_space(4.0);
    let enregistrer = bouton(ui, BOUTON_ENREGISTRER_REGLAGES, Style::Principal, true).clicked();
    discret(ui, MENTION_CAISSE_NON_CERTIFIEE);

    if enregistrer
        && let Some(etat) = app.etat.as_mut()
        && etat.definir_reglages(brouillon.profil, brouillon.devise, &brouillon.taux)
    {
        app.reglages_saisis = None;
        return;
    }
    // Un refus garde les choix faits : on ne retape pas tout pour corriger un taux.
    if brouillon != depart {
        app.reglages_saisis = Some(brouillon);
    }
}

#[cfg(test)]
mod tests {
    use super::super::essais::*;
    use super::super::*;
    use eframe::egui::accesskit::Role;
    use eframe::egui::{Pos2, Rect};
    use egui_kittest::kittest::Queryable;
    use gestion_stock::monnaie::Devise;
    use gestion_stock::reglages::Profil;

    #[test]
    fn les_reglages_se_changent_et_se_conservent() {
        let chemin = chemin_temp("reglages-ui");
        let etat = Etat::ouvrir(chemin.clone()).unwrap().avec_horloge(|| FIXE);
        let mut app = Application::nouvelle(etat);
        app.ecran = Ecran::Reglages;
        let mut h = harness_vente(app, FENETRE);
        // Les réglages enregistrés sont montrés d'abord (Europe, euro, 20 %).
        assert_eq!(est_selectionne(&h, PROFIL_EUROPE), Some(true));
        assert_eq!(est_selectionne(&h, DEVISE_EUR), Some(true));
        h.get_by_label_contains(RAPPEL_REGLAGES);
        h.get_by_label_contains("n'est pas une caisse certifiée");

        cliquer(&mut h, PROFIL_ETATS_UNIS);
        cliquer(&mut h, DEVISE_USD);
        assert_eq!(est_selectionne(&h, PROFIL_ETATS_UNIS), Some(true));
        assert_eq!(est_selectionne(&h, PROFIL_EUROPE), Some(false));
        h.get_by_label(EXPLICATION_ETATS_UNIS);
        // Rien n'est enregistré tant qu'on n'a pas validé.
        assert_eq!(
            h.state().etat.as_ref().unwrap().reglages().devise,
            Devise::Eur
        );
        h.state_mut().reglages_saisis.as_mut().unwrap().taux.clear();
        h.run();
        saisir(&mut h, CHAMP_TAUX_DEFAUT, "8,25");
        cliquer(&mut h, BOUTON_ENREGISTRER_REGLAGES);
        let reglages = h.state().etat.as_ref().unwrap().reglages();
        assert_eq!(
            (reglages.profil, reglages.devise, reglages.taux_defaut_bp),
            (Profil::EtatsUnis, Devise::Usd, 825)
        );
        h.get_by_label_contains("Réglages enregistrés");

        // L'écran de vente passe en dollars et la taxe s'ajoute au prix hors taxe.
        {
            let etat = h.state_mut().etat.as_mut().unwrap();
            assert!(etat.ajouter_avec_prix("Pain", "0", "10", ""));
            assert!(etat.entree("Pain", "5"));
        }
        cliquer(&mut h, BOUTON_RETOUR_VENTE);
        scanner(&mut h, "pain");
        for ligne in [LIBELLE_SOUS_TOTAL, "Taxe 8.25\u{a0}%", "$0.83", "$10.83"] {
            h.get_by_label(ligne);
        }

        // Les réglages se conservent : on rouvre le fichier.
        let rouvert = Etat::ouvrir(chemin).unwrap().reglages();
        assert_eq!(
            (rouvert.profil, rouvert.devise, rouvert.taux_defaut_bp),
            (Profil::EtatsUnis, Devise::Usd, 825)
        );
    }

    #[test]
    fn un_taux_illisible_est_refuse_et_garde_la_saisie() {
        let mut app = app_vendeuse("taux-illisible");
        app.ecran = Ecran::Reglages;
        let mut h = harness_vente(app, FENETRE);
        cliquer(&mut h, DEVISE_GBP);
        h.state_mut().reglages_saisis.as_mut().unwrap().taux = "beaucoup".into();
        h.run();
        cliquer(&mut h, BOUTON_ENREGISTRER_REGLAGES);
        h.get_by_label_contains("Erreur : ");
        let etat = h.state().etat.as_ref().unwrap();
        assert_eq!(etat.reglages().devise, Devise::Eur);
        // Les choix faits ne sont pas perdus.
        let brouillon = h.state().reglages_saisis.as_ref().unwrap();
        assert_eq!(
            (brouillon.devise, brouillon.taux.as_str()),
            (Devise::Gbp, "beaucoup")
        );
    }

    #[test]
    fn apres_une_vente_la_devise_et_le_profil_sont_figes_mais_pas_le_taux() {
        let mut app = app_avec_ventes("reglages-figes");
        app.ecran = Ecran::Reglages;
        let mut h = harness_vente(app, FENETRE);
        h.get_by_label(REGLAGES_VERROUILLES);
        for nom in [
            PROFIL_EUROPE,
            PROFIL_ETATS_UNIS,
            DEVISE_EUR,
            DEVISE_USD,
            DEVISE_GBP,
            DEVISE_XOF,
        ] {
            assert!(est_grise(&h, nom), "« {nom} » devrait être grisé");
        }
        // Le choix actuel reste annoncé comme sélectionné même grisé.
        assert_eq!(est_selectionne(&h, DEVISE_EUR), Some(true));
        // Le taux par défaut, lui, change encore.
        cliquer(&mut h, BOUTON_ENREGISTRER_REGLAGES); // sans rien modifier : réenregistre à l'identique
        h.get_by_label_contains("Réglages enregistrés");
        h.state_mut().reglages_saisis = Some(BrouillonReglages {
            profil: Profil::Europe,
            devise: Devise::Eur,
            taux: "10".into(),
        });
        h.run();
        cliquer(&mut h, BOUTON_ENREGISTRER_REGLAGES);
        assert_eq!(
            h.state().etat.as_ref().unwrap().reglages().taux_defaut_bp,
            1000
        );
    }

    #[test]
    fn le_bouton_enregistrer_les_reglages_est_a_l_ecran_a_la_taille_par_defaut() {
        let mut app = app_vendeuse("reglages-ecran");
        app.ecran = Ecran::Reglages;
        let h = harness_vente(app, FENETRE);
        let ecran = Rect::from_min_size(Pos2::ZERO, FENETRE);
        for nom in [BOUTON_ENREGISTRER_REGLAGES, DEVISE_XOF, PROFIL_ETATS_UNIS] {
            let rect = h.get_by_role_and_label(Role::Button, nom).rect();
            assert!(
                ecran.contains_rect(rect),
                "« {nom} » : {rect:?} hors de {ecran:?}"
            );
        }
    }

    #[test]
    fn les_noms_de_champs_et_boutons_des_reglages_ont_un_role_et_un_nom() {
        let mut app = app_vendeuse("roles-reglages");
        app.ecran = Ecran::Reglages;
        let h = harness_vente(app, FENETRE);
        h.get_by_role_and_label(Role::TextInput, CHAMP_TAUX_DEFAUT);
        for nom in [
            PROFIL_EUROPE,
            PROFIL_ETATS_UNIS,
            DEVISE_EUR,
            DEVISE_USD,
            DEVISE_GBP,
            DEVISE_XOF,
            BOUTON_ENREGISTRER_REGLAGES,
        ] {
            h.get_by_role_and_label(Role::Button, nom);
        }
    }

    #[test]
    fn les_reglages_montrent_le_fichier_de_donnees_utilise() {
        let mut app = app_vendeuse("reglages-fichier");
        app.ecran = Ecran::Reglages;
        let chemin = app.etat.as_ref().unwrap().chemin().to_path_buf();
        let h = harness_vente(app, FENETRE);
        h.get_by_label(SECTION_FICHIER);
        h.get_by_label(&pied_de_page_donnees(&crate::etat::chemin_complet(&chemin)));
    }

    #[test]
    fn tous_les_textes_peints_de_l_ecran_reglages_atteignent_4_5() {
        // Les contrôles grisés sont dispensés de contraste : on les écarte par leur nom.
        let sauf = |textes: Vec<TextePeint>, grises: &[&str]| -> Vec<TextePeint> {
            textes
                .into_iter()
                .filter(|t| !grises.contains(&t.texte.as_str()))
                .collect()
        };

        let mut app = app_vendeuse("contraste-reglages");
        app.ecran = Ecran::Reglages;
        let textes = textes_peints(&mut app, FENETRE, 1.0);
        assert!(
            textes
                .iter()
                .any(|t| t.texte == BOUTON_ENREGISTRER_REGLAGES)
        );
        verifier_contrastes(&textes, "réglages");

        let mut app = app_avec_ventes("contraste-reglages-figes");
        app.ecran = Ecran::Reglages;
        let textes = textes_peints(&mut app, FENETRE, 1.0);
        assert!(textes.iter().any(|t| t.texte == REGLAGES_VERROUILLES));
        let grises = [
            PROFIL_EUROPE,
            PROFIL_ETATS_UNIS,
            DEVISE_EUR,
            DEVISE_USD,
            DEVISE_GBP,
            DEVISE_XOF,
        ];
        verifier_contrastes(&sauf(textes, &grises), "réglages figés");
    }
}
