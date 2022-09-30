mod config;
mod app;
pub mod components;
mod modals;
mod setup;
pub mod tailor_state;

macro_rules! global_widget {
    ($name:ident, $ty:ty) => {
        mod __private {
            use super::*;
            use $ty as __Type;
            thread_local!(static GLOBAL_WIDGET: __Type = __Type::default());

            pub fn $name() -> $ty {
                relm4::gtk::init().unwrap();
                GLOBAL_WIDGET.with(|w| w.clone())
            }
        }

        pub use __private::$name;
    }
}

global_widget!(my_box, gtk::Box);

use gtk::prelude::ApplicationExt;
use relm4::{
    actions::{AccelsPlus, RelmAction, RelmActionGroup},
    gtk, main_application, RelmApp,
};

use app::App;
use setup::setup;

use crate::config::APP_ID;

relm4::new_action_group!(AppActionGroup, "app");
relm4::new_stateless_action!(QuitAction, AppActionGroup, "quit");

fn main() {
    let my_box = my_box();

    // Enable logging
    tracing_subscriber::fmt()
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::FULL)
        .with_max_level(tracing::Level::INFO)
        .init();

    setup();

    let app = main_application();
    app.set_application_id(Some(APP_ID));
    app.set_resource_base_path(Some("/com/github/aaronerhardt/Tailor/"));

    let actions = RelmActionGroup::<AppActionGroup>::new();

    let quit_action = {
        let app = app.clone();
        RelmAction::<QuitAction>::new_stateless(move |_| {
            app.quit();
        })
    };
    actions.add_action(quit_action);

    app.set_accelerators_for_action::<QuitAction>(&["<Control>q"]);

    app.set_action_group(Some(&actions.into_action_group()));

    let app = RelmApp::with_app(app);

    app.run::<App>(());
}
