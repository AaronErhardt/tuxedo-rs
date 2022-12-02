use gtk::gdk::RGBA;
use gtk::gdk_pixbuf::Pixbuf;
use gtk::prelude::{
    ButtonExt, Cast, ColorChooserExt, DialogExt, DialogExtManual, GridExt, GtkWindowExt, ObjectExt,
    WidgetExt,
};
use gtk::ResponseType;
use relm4::{gtk, Component, ComponentParts, ComponentSender, RelmWidgetExt};
use tailor_api::Color;

use crate::state::{TailorStateMsg, STATE};
use crate::util::{self, rgba_to_color};

pub struct ColorButton {
    pixbuf: Pixbuf,
}

#[derive(Debug)]
pub enum ColorButtonInput {
    OpenDialog,
    UpdateColor(Color),
}

#[relm4::component(pub)]
impl Component for ColorButton {
    type CommandOutput = ();
    type Init = Color;
    type Input = ColorButtonInput;
    type Output = Color;
    type Widgets = ColorButtonWidgets;

    view! {
        button = gtk::Button {
            add_css_class: "color",
            connect_clicked => ColorButtonInput::OpenDialog,

            #[name = "image"]
            gtk::Picture::for_pixbuf(&model.pixbuf) {
                inline_css: "border-radius: 2px",
                #[watch]
                set_pixbuf: Some(&model.pixbuf)
            }
        }
    }

    fn init(
        color: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let pixbuf = util::new_pixbuf(&color);

        let model = Self { pixbuf };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>, root: &Self::Root) {
        match message {
            ColorButtonInput::OpenDialog => {
                let parent_window = root.toplevel_window().unwrap();
                let dialog = gtk::ColorChooserDialog::builder()
                    .transient_for(&parent_window)
                    .modal(true)
                    .use_alpha(false)
                    .build();

                let main_box = dialog.content_area();
                let color_chooser = main_box.first_child().unwrap();
                let color_editor = color_chooser.last_child().unwrap();
                let color_overlay = color_editor.first_child().unwrap();
                let color_swatch = color_overlay
                    .first_child()
                    .unwrap()
                    .downcast_ref::<gtk::Grid>()
                    .unwrap()
                    .child_at(1, 0)
                    .unwrap();

                dialog.connect_rgba_notify(|dialog| {
                    let rgba: RGBA = dialog.rgba();
                    let color = util::rgba_to_color(rgba);
                    STATE.emit(TailorStateMsg::OverwriteColor(color));
                });

                color_swatch.connect_notify_local(Some("rgba"), |obj, _| {
                    let rgba: RGBA = obj.property("rgba");
                    let color = util::rgba_to_color(rgba);
                    STATE.emit(TailorStateMsg::OverwriteColor(color));
                });

                relm4::spawn_local(async move {
                    let response = dialog.run_future().await;
                    if let ResponseType::Ok = response {
                        sender.input(ColorButtonInput::UpdateColor(rgba_to_color(dialog.rgba())))
                    }
                    dialog.close();
                });
            }
            ColorButtonInput::UpdateColor(color) => {
                util::fill_pixbuf(&self.pixbuf, &color);
                sender.output(color).unwrap();
            }
        }
    }
}
