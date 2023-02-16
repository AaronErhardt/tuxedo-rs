use gtk::prelude::{
    ButtonExt, EditableExt, EntryBufferExtManual, EntryExt, GridExt, GtkWindowExt, WidgetExt,
};
use relm4::{
    adw, gtk, Component, ComponentController, ComponentParts, ComponentSender, Controller,
    RelmWidgetExt, SimpleComponent,
};
use relm4_components::simple_combo_box::SimpleComboBox;

use crate::templates::{MsgDialogBox, MsgDialogButtons};

pub struct NewEntryDialog {
    items: Controller<SimpleComboBox<String>>,
    buffer: gtk::EntryBuffer,
}

pub struct NewEntryInit {
    pub info: String,
    pub profiles: Vec<String>,
}

#[derive(Debug)]
pub enum NewEntryInput {
    Save,
    Cancel,
    Noop,
}

#[derive(Debug)]
pub struct NewEntryOutput {
    pub name: String,
    pub based_of: String,
}

#[relm4::component(pub)]
impl SimpleComponent for NewEntryDialog {
    type Init = NewEntryInit;
    type Input = NewEntryInput;
    type Output = Option<NewEntryOutput>;

    view! {
        window = adw::Window {
            set_default_width: 400,
            add_css_class: "messagedialog",
            set_modal: true,
            present: (),

            #[template]
            MsgDialogBox {
                #[template_child]
                title -> gtk::Label {
                    set_label: &info,
                },

                gtk::Grid {
                    set_margin_all: 12,
                    set_row_spacing: 12,
                    set_column_spacing: 12,
                    set_halign: gtk::Align::Center,

                    attach[0, 0, 1, 1] = &gtk::Label {
                        set_halign: gtk::Align::Start,
                        set_label: "Name",
                    },

                    attach[1, 0, 1, 1] = &gtk::Entry {
                        set_buffer: &model.buffer,
                        connect_changed => NewEntryInput::Noop,
                    },

                    attach[0, 1, 1, 1] = &gtk::Label {
                        set_halign: gtk::Align::Start,
                        set_label: "Based on",
                    },

                    #[local_ref]
                    attach[1, 1, 1, 1] = items -> gtk::ComboBoxText {},
                },

                gtk::Separator {},

                #[template]
                MsgDialogButtons {
                    #[template_child]
                    save_button -> gtk::Button {
                        #[watch]
                        set_sensitive: model.valid_name(),
                        connect_clicked => NewEntryInput::Save,
                    },
                    #[template_child]
                    cancel_button -> gtk::Button {
                        connect_clicked => NewEntryInput::Cancel,
                    }
                },
            }
        }
    }

    fn init(
        init: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let NewEntryInit { info, profiles } = init;

        let items = SimpleComboBox::builder()
            .launch(SimpleComboBox {
                active_index: Some(0),
                variants: profiles,
            })
            .forward(sender.input_sender(), |_| NewEntryInput::Noop);

        let model = Self {
            items,
            buffer: gtk::EntryBuffer::default(),
        };

        let items = model.items.widget();
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        match message {
            NewEntryInput::Save if self.valid_name() => sender
                .output(Some(NewEntryOutput {
                    name: self.buffer.text(),
                    based_of: self.items.model().get_active_elem().unwrap().to_string(),
                }))
                .unwrap(),
            NewEntryInput::Noop => (),
            _ => {
                sender.output(None).unwrap();
            }
        }
    }

    fn shutdown(&mut self, widgets: &mut Self::Widgets, _output: relm4::Sender<Self::Output>) {
        widgets.window.close();
    }
}

impl NewEntryDialog {
    fn valid_name(&self) -> bool {
        let name = self.buffer.text().trim().to_string();
        !name.is_empty() && !self.items.model().variants.contains(&name)
    }
}
