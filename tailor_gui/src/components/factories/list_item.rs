use adw::prelude::{MessageDialogExt, MessageDialogExtManual};
use gtk::glib;
use gtk::prelude::{BoxExt, ButtonExt, ObjectExt, OrientableExt, WidgetExt};
use gtk::traits::EditableExt;
use relm4::{
    adw, factory,
    factory::{DynamicIndex, FactoryComponent, FactorySender},
    gtk, RelmWidgetExt,
};

use crate::components::fan_list::ListInput;

pub struct ListItem {
    pub name: String,
}

#[factory(pub)]
impl FactoryComponent for ListItem {
    type ParentWidget = gtk::ListBox;
    type ParentInput = ListInput;
    type CommandOutput = ();
    type Input = ();
    type Output = ListInput;
    type Init = String;
    type Widgets = ProfileWidgets;

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
                            sender.output(ListInput::Rename(index.clone(), e.text().into()))
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
                    set_icon_name: "remove-symbolic",
                    connect_clicked[sender, index, name = self.name.clone()] => move |btn| {
                        let window = btn.toplevel_window().unwrap();
                        let dialog = adw::MessageDialog::builder()
                            .modal(true)
                            .transient_for(&window)
                            .heading(&format!("Delete fan-profile \"{name}\"?"))
                            .body("This change is not reversible.")
                            .default_response("cancel")
                            .close_response("cancel")
                            .build();
                        dialog.add_responses(&[("cancel", "Cancel"), ("remove", "Remove")]);
                        dialog.set_response_appearance("remove", adw::ResponseAppearance::Destructive);

                        let sender = sender.clone();
                        let index = index.clone();
                        relm4::spawn_local(async move {
                            let response = dialog.run_future().await;
                            if response == "remove" {
                                sender.output(ListInput::Remove(index.clone()));
                            }
                        });
                    }
                },

                gtk::Image {
                    set_icon_name: Some("go-next-symbolic"),
                }
            },
        }
    }

    fn output_to_parent_input(output: Self::Output) -> Option<ListInput> {
        Some(output)
    }

    fn init_model(name: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        Self { name }
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
