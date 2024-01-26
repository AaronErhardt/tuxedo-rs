#![allow(deprecated)]

mod app;
pub mod components;
mod config;
mod modals;
mod setup;
pub mod state;
pub mod templates;
pub mod util;

use app::App;
use gtk::prelude::ApplicationExt;
use relm4::actions::{AccelsPlus, RelmAction, RelmActionGroup};
use relm4::{gtk, main_application, RelmApp};
use setup::setup;

use crate::config::APP_ID;

relm4::new_action_group!(AppActionGroup, "app");
relm4::new_stateless_action!(QuitAction, AppActionGroup, "quit");

fn main() {
    // Enable logging
    tracing_subscriber::fmt()
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::FULL)
        .with_max_level(tracing::Level::INFO)
        .init();

    setup();

    let app = main_application();
    app.set_application_id(Some(APP_ID));
    app.set_resource_base_path(Some("/com/github/aaronerhardt/Tailor/"));

    let quit_action = {
        let app = app.clone();
        RelmAction::<QuitAction>::new_stateless(move |_| {
            app.quit();
        })
    };

    let mut actions = RelmActionGroup::<AppActionGroup>::new();
    actions.add_action(quit_action);
    actions.register_for_main_application();

    app.set_accelerators_for_action::<QuitAction>(&["<Control>q"]);

    relm4_icons::initialize_icons();

    let app = RelmApp::from_app(app).visible_on_activate(false);
    app.run::<App>(());
}
