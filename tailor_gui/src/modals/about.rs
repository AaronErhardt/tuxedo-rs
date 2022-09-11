use gtk::prelude::GtkWindowExt;
use relm4::{component::EmptyRoot, ComponentParts, ComponentSender, SimpleComponent, RelmWidgetExt};
use relm4::gtk;

use gettextrs::gettext;

use crate::config::{APP_ID, VERSION};

pub struct AboutDialog {}

impl SimpleComponent for AboutDialog {
    type Init = gtk::Widget;
    type Widgets = gtk::Widget;
    type Input = ();
    type Output = ();
    type Root = EmptyRoot;

    fn init_root() -> Self::Root {
        EmptyRoot::default()
    }

    fn init(
        widgets: Self::Init,
        _root: &Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self {};

        ComponentParts { model, widgets }
    }

    fn update_view(&self, widgets: &mut Self::Widgets, _sender: ComponentSender<Self>) {
        let dialog = gtk::AboutDialog::builder()
            .logo_icon_name(APP_ID)
            // Insert your license of choice here
            // .license_type(gtk::License::MitX11)
            // Insert your website here
            // .website("https://gitlab.gnome.org/bilelmoussaoui/tailor_gui/")
            .version(VERSION)
            .translator_credits(&gettext("translator-credits"))
            .modal(true)
            .transient_for(&widgets.toplevel_window().unwrap())
            .authors(vec!["Aaron Erhardt".into()])
            .artists(vec!["Aaron Erhardt".into()])
            .build();
        dialog.present();
    }
}
