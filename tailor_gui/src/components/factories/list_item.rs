use std::marker::PhantomData;

use adw::prelude::{MessageDialogExt, MessageDialogExtManual};
use gtk::prelude::{BoxExt, ButtonExt, OrientableExt, WidgetExt};
use relm4::factory::{DynamicIndex, FactoryComponent, FactorySender};
use relm4::gtk::prelude::{EntryBufferExtManual, Cast};
use relm4::gtk::traits::{EntryExt, ToggleButtonExt};
use relm4::{adw, factory, gtk, RelmWidgetExt};
use relm4_icons::icon_name;

pub trait ListMsg {
    fn ty() -> &'static str;
    fn rename(index: DynamicIndex, text: String) -> Self;
    fn remove(index: DynamicIndex) -> Self;
}

pub struct ListItem<Msg: ListMsg> {
    pub name: String,
    msg: PhantomData<*const Msg>,
    index: DynamicIndex,
    edit_active: bool,
    name_buffer: gtk::EntryBuffer,
}

#[derive(Debug)]
pub enum ListItemInput {
    SetEditMode(bool),
    Remove(gtk::Widget)
}

#[factory(pub)]
impl<Msg> FactoryComponent for ListItem<Msg>
where
    Msg: ListMsg + std::fmt::Debug + 'static,
{
    type CommandOutput = ();
    type Init = String;
    type Input = ListItemInput;
    type Output = Msg;
    type ParentInput = Msg;
    type ParentWidget = gtk::ListBox;

    view! {
        #[name = "root"]
        gtk::Box {
            set_orientation: gtk::Orientation::Horizontal,
            add_css_class: "header",

            gtk::Box {
                set_hexpand: true,
                set_halign: gtk::Align::Start,
                set_valign: gtk::Align::Center,

                if self.edit_active {
                    #[name(edit_label)]
                    gtk::Entry {
                        set_buffer: &self.name_buffer,
                        connect_activate => ListItemInput::SetEditMode(false),
                        #[track(self.edit_active)]
                        grab_focus: (),
                    }
                } else {
                    gtk::Label {
                        #[watch]
                        set_text: &self.name,
                        set_halign: gtk::Align::Start,
                    }
                }
            },

            gtk::Box {
                set_spacing: 12,
                set_orientation: gtk::Orientation::Horizontal,
                set_valign: gtk::Align::Center,

                gtk::ToggleButton {
                    set_icon_name: icon_name::EDIT,
                    #[watch]
                    set_active: self.edit_active,
                    connect_clicked[sender] => move |btn| {
                        sender.input(ListItemInput::SetEditMode(btn.is_active()))
                    }
                },

                gtk::Button {
                    set_icon_name: icon_name::CROSS_FILLED,
                    connect_clicked[sender, root] => move |_| {
                        sender.input(ListItemInput::Remove(root.clone().upcast()));
                    }
                },

                gtk::Image {
                    set_icon_name: Some(icon_name::RIGHT),
                }
            },
        }
    }

    fn forward_to_parent(output: Self::Output) -> Option<Msg> {
        Some(output)
    }

    fn init_model(name: Self::Init, index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        Self {
            name,
            msg: PhantomData,
            index: index.clone(),
            edit_active: false,
            name_buffer: gtk::EntryBuffer::new(None::<&str>),
        }
    }

    fn update(&mut self, message: Self::Input, sender: FactorySender<Self>) {
        match message {
            ListItemInput::SetEditMode(active) => {
                if active {
                    self.name_buffer.set_text(&self.name);
                } else {
                    let new_name = self.name_buffer.text().to_string();
                    if self.name != new_name {
                        sender.output(Msg::rename(self.index.clone(), new_name))
                    }
                }
                self.edit_active = active;
            }
            ListItemInput::Remove(root) => {
                let window = root.toplevel_window().unwrap();
                let dialog = adw::MessageDialog::builder()
                    .modal(true)
                    .transient_for(&window)
                    .heading(format!("Delete {} profile \"{}\"?", Msg::ty(), self.name))
                    .body("This change is not reversible.")
                    .default_response("cancel")
                    .close_response("cancel")
                    .build();
                dialog.add_responses(&[("cancel", "Cancel"), ("remove", "Remove")]);
                dialog.set_response_appearance("remove", adw::ResponseAppearance::Destructive);

                let sender = sender.clone();
                let index = self.index.clone();
                relm4::spawn_local(async move {
                    let response = dialog.choose_future().await;
                    if response == "remove" {
                        sender.output(Msg::remove(index.clone()));
                    }
                });
            }
        }
    }
}