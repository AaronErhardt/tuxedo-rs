use gettextrs::{gettext, LocaleCategory};
use gtk::{gdk, gio, glib};
use relm4::gtk;

use crate::config::{APP_ID, GETTEXT_PACKAGE, ICON_RESOURCES_FILE, LOCALEDIR, RESOURCES_FILE};

pub fn setup() {
    // Initialize GTK
    gtk::init().unwrap();

    setup_gettext();

    glib::set_application_name(&gettext("Tailor"));

    let res = gio::Resource::load(RESOURCES_FILE).expect("Could not load gresource file");
    gio::resources_register(&res);

    let res = gio::Resource::load(ICON_RESOURCES_FILE).expect("Could not load gresource file");
    gio::resources_register(&res);

    setup_css();

    gtk::Window::set_default_icon_name(APP_ID);
}

fn setup_gettext() {
    // Prepare i18n
    gettextrs::setlocale(LocaleCategory::LcAll, "");
    gettextrs::bindtextdomain(GETTEXT_PACKAGE, LOCALEDIR).expect("Unable to bind the text domain");
    gettextrs::textdomain(GETTEXT_PACKAGE).expect("Unable to switch to the text domain");
}

fn setup_css() {
    let provider = gtk::CssProvider::new();
    provider.load_from_resource("/com/github/aaronerhardt/Tailor/style.css");
    if let Some(display) = gdk::Display::default() {
        gtk::StyleContext::add_provider_for_display(
            &display,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}
