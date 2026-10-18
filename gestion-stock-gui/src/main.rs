use eframe::egui;
use gestion_stock_gui::chemin::chemin_par_defaut;
use gestion_stock_gui::etat::Etat;
use gestion_stock_gui::palette::visuals;
use gestion_stock_gui::textes::TITRE_FENETRE;
use gestion_stock_gui::ui::Application;

fn main() -> eframe::Result {
    // Un fichier illisible n'est jamais écrasé : on ouvre alors un écran d'erreur lisible
    // (un double-clic n'a pas de terminal où afficher le message).
    let application = match Etat::ouvrir(chemin_par_defaut()) {
        Ok(etat) => Application::nouvelle(etat),
        Err(e) => Application::en_erreur(e.to_string()),
    };
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(TITRE_FENETRE)
            .with_inner_size([960.0, 640.0])
            .with_min_inner_size([640.0, 420.0]),
        ..Default::default()
    };
    eframe::run_native(
        TITRE_FENETRE,
        options,
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(visuals());
            Ok(Box::new(application))
        }),
    )
}
