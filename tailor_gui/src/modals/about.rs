use gtk::prelude::GtkWindowExt;
use relm4::{gtk, ComponentParts, ComponentSender, SimpleComponent, adw};

// use gettextrs::gettext;
use crate::config::{APP_ID, VERSION};

pub struct AboutDialog {}

impl SimpleComponent for AboutDialog {
    type Init = ();
    type Input = ();
    type Output = ();
    type Root = adw::AboutWindow;
    type Widgets = adw::AboutWindow;

    fn init_root() -> Self::Root {
        adw::AboutWindow::builder()
            .application_icon(APP_ID)
            .license_type(gtk::License::Gpl20)
            .website("https://github.com/AaronErhardt/tuxedo-rs/")
            .version(VERSION)
            //.translator_credits(&gettext("translator-credits"))
            .modal(true)
            .developers(vec!["Aaron Erhardt".into()])
            .artists(vec!["Aaron Erhardt".into()])
            .build()
    }

    fn init(
        _: Self::Init,
        root: &Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self {};

        let widgets = root.clone();

        ComponentParts { model, widgets }
    }

    fn update_view(&self, dialog: &mut Self::Widgets, _sender: ComponentSender<Self>) {
        dialog.present();
    }
}
