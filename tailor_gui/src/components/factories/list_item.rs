use std::marker::PhantomData;

use adw::prelude::{MessageDialogExt, MessageDialogExtManual};
use gtk::glib;
use gtk::prelude::{BoxExt, ButtonExt, EditableExt, ObjectExt, OrientableExt, WidgetExt};
use relm4::factory::{DynamicIndex, FactoryComponent, FactorySender};
use relm4::{adw, factory, gtk, RelmWidgetExt};

pub trait ListMsg {
    fn ty() -> &'static str;
    fn rename(index: DynamicIndex, text: String) -> Self;
    fn remove(index: DynamicIndex) -> Self;
}

pub struct ListItem<Msg: ListMsg> {
    pub name: String,
    msg: PhantomData<*const Msg>,
}

#[factory(pub)]
impl<Msg> FactoryComponent for ListItem<Msg>
where
    Msg: ListMsg + std::fmt::Debug + 'static,
{
    type CommandOutput = ();
    type Init = String;
    type Input = ();
    type Output = Msg;
    type ParentInput = Msg;
    type ParentWidget = gtk::ListBox;

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Horizontal,
            add_css_class: "header",

            gtk::Box {
                set_hexpand: true,
                set_halign: gtk::Align::Start,
                set_valign: gtk::Align::Center,

                #[name(edit_label)]
                gtk::EditableLabel {
                    add_css_class: "padded",
                    #[watch]
                    set_text: &self.name,

                    connect_editing_notify[sender, index] => move |e| {
                        if !e.is_editing() {
                            sender.output(Msg::rename(index.clone(), e.text().into()))
                        }
                    }
                },
            },

            gtk::Box {
                set_spacing: 12,
                set_orientation: gtk::Orientation::Horizontal,
                set_valign: gtk::Align::Center,

                gtk::Button {
                    add_css_class: "destructive-action",
                    set_icon_name: "remove",
                    connect_clicked[sender, index, name = self.name.clone()] => move |btn| {
                        let window = btn.toplevel_window().unwrap();
                        let dialog = adw::MessageDialog::builder()
                            .modal(true)
                            .transient_for(&window)
                            .heading(&format!("Delete {} profile \"{name}\"?", Msg::ty()))
                            .body("This change is not reversible.")
                            .default_response("cancel")
                            .close_response("cancel")
                            .build();
                        dialog.add_responses(&[("cancel", "Cancel"), ("remove", "Remove")]);
                        dialog.set_response_appearance("remove", adw::ResponseAppearance::Destructive);

                        let sender = sender.clone();
                        let index = index.clone();
                        relm4::spawn_local(async move {
                            let response = dialog.choose_future().await;
                            if response == "remove" {
                                sender.output(Msg::remove(index.clone()));
                            }
                        });
                    }
                },

                gtk::Image {
                    set_icon_name: Some("go-next"),
                }
            },
        }
    }

    fn output_to_parent_input(output: Self::Output) -> Option<Msg> {
        Some(output)
    }

    fn init_model(name: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        Self {
            name,
            msg: PhantomData,
        }
    }

    fn init_widgets(
        &mut self,
        index: &DynamicIndex,
        root: &Self::Root,
        returned_widget: &gtk::ListBoxRow,
        sender: FactorySender<Self>,
    ) -> Self::Widgets {
        let widgets = view_output!();

        widgets
            .edit_label
            .bind_property("editing", returned_widget, "activatable")
            .flags(glib::BindingFlags::INVERT_BOOLEAN)
            .build();

        widgets
    }

    fn update(&mut self, _message: Self::Input, _sender: FactorySender<Self>) {}
}
