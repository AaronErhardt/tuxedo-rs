use crate::{
    app::FullProfileInfo,
    templates::{MsgDialogBox, MsgDialogButtons},
};
use gtk::prelude::{
    ButtonExt, EditableExt, EntryBufferExtManual, EntryExt, GridExt, GtkWindowExt, WidgetExt,
};
use relm4::{
    Component, ComponentController, ComponentParts, ComponentSender, Controller, RelmWidgetExt,
    SimpleComponent,
};
use relm4_components::simple_combo_box::SimpleComboBox;
use tailor_api::ProfileInfo;

pub struct NewProfileDialog {
    profiles: Vec<String>,
    buffer: gtk::EntryBuffer,
    keyboard: Controller<SimpleComboBox<String>>,
    fan: Controller<SimpleComboBox<String>>,
}

pub struct NewProfileInit {
    pub profiles: Vec<String>,
    pub keyboard: Vec<String>,
    pub fan: Vec<String>,
}

#[derive(Debug)]
pub enum NewProfileInput {
    Save,
    Cancel,
    Noop,
}

#[relm4::component(pub)]
impl SimpleComponent for NewProfileDialog {
    type Input = NewProfileInput;
    type Output = Option<FullProfileInfo>;
    type Init = NewProfileInit;

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
                    set_label: "Create new fan profile"
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
                        connect_changed => NewProfileInput::Noop,
                    },

                    attach[0, 1, 1, 1] = &gtk::Label {
                        set_halign: gtk::Align::Start,
                        set_label: "Keyboard profile",
                    },

                    #[local_ref]
                    attach[1, 1, 1, 1] = keyboard -> gtk::ComboBoxText {},

                    attach[0, 2, 1, 1] = &gtk::Label {
                        set_halign: gtk::Align::Start,
                        set_label: "Fan profile",
                    },

                    #[local_ref]
                    attach[1, 2, 1, 1] = fan -> gtk::ComboBoxText {},
                },

                gtk::Separator {},

                #[template]
                MsgDialogButtons {
                    #[template_child]
                    save_button -> gtk::Button {
                        #[watch]
                        set_sensitive: model.valid_name(),
                        connect_clicked => NewProfileInput::Save,
                    },
                    #[template_child]
                    cancel_button -> gtk::Button {
                        connect_clicked => NewProfileInput::Cancel,
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
        let NewProfileInit {
            profiles,
            keyboard,
            fan,
        } = init;

        let keyboard = SimpleComboBox::builder()
            .launch(SimpleComboBox {
                active_index: Some(0),
                variants: keyboard,
            })
            .forward(sender.input_sender(), |_| NewProfileInput::Noop);

        let fan = SimpleComboBox::builder()
            .launch(SimpleComboBox {
                active_index: Some(0),
                variants: fan,
            })
            .forward(sender.input_sender(), |_| NewProfileInput::Noop);

        let model = Self {
            profiles,
            buffer: gtk::EntryBuffer::default(),
            keyboard,
            fan,
        };

        let keyboard = model.keyboard.widget();
        let fan = model.fan.widget();
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        match message {
            NewProfileInput::Save if self.valid_name() => sender
                .output(Some(FullProfileInfo {
                    name: self.buffer.text(),
                    data: ProfileInfo {
                        keyboard: self.keyboard.model().get_active_elem().unwrap().to_string(),
                        fan: self.fan.model().get_active_elem().unwrap().to_string(),
                    },
                }))
                .unwrap(),
            NewProfileInput::Noop => (),
            _ => {
                sender.output(None).unwrap();
            }
        }
    }

    fn shutdown(&mut self, widgets: &mut Self::Widgets, _output: relm4::Sender<Self::Output>) {
        widgets.window.close();
    }
}

impl NewProfileDialog {
    fn valid_name(&self) -> bool {
        let name = self.buffer.text().trim().to_string();
        !name.is_empty() && !self.profiles.contains(&name)
    }
}
