use gtk::prelude::GtkWindowExt;
use relm4::gtk;
use relm4::{ComponentParts, ComponentSender, SimpleComponent};

use gettextrs::gettext;

use crate::config::{APP_ID, VERSION};

pub struct AboutDialog {}

impl SimpleComponent for AboutDialog {
    type Init = ();
    type Widgets = gtk::AboutDialog;
    type Input = ();
    type Output = ();
    type Root = gtk::AboutDialog;

    fn init_root() -> Self::Root {
        gtk::AboutDialog::builder()
            .logo_icon_name(APP_ID)
            // Insert your license of choice here
            .license_type(gtk::License::Gpl20)
            // Insert your website here
            // .website("https://gitlab.gnome.org/bilelmoussaoui/tailor_gui/")
            .version(VERSION)
            .translator_credits(&gettext("translator-credits"))
            .modal(true)
            .authors(vec!["Aaron Erhardt".into()])
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
