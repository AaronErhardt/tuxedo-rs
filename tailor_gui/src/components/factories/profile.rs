use adw::prelude::{ExpanderRowExt, PreferencesRowExt};
use gtk::prelude::{ButtonExt, CheckButtonExt, ListBoxRowExt, ObjectExt, WidgetExt};
use once_cell::unsync::Lazy;
use relm4::factory::{DynamicIndex, FactoryComponent, FactorySender, FactoryView};
use relm4::{adw, factory, gtk, Component, ComponentController, Controller, RelmWidgetExt};
use relm4_components::simple_combo_box::SimpleComboBox;
use tailor_api::ProfileInfo;

use crate::components::profiles::ProfilesInput;
use crate::state::{TailorStateMsg, STATE};

thread_local! {
    static RADIO_GROUP: Lazy<gtk::CheckButton> = Lazy::new(gtk::CheckButton::default);
}

#[derive(Debug)]
pub struct Profile {
    pub name: String,
    pub info: ProfileInfo,
    pub active: bool,
    pub keyboard_combo_box: Controller<SimpleComboBox<String>>,
    pub fan_combo_box: Controller<SimpleComboBox<String>>,
}

#[derive(Debug)]
pub struct ProfileInit {
    pub name: String,
    pub info: ProfileInfo,
    pub active: bool,
    pub keyboard_profiles: Vec<String>,
    pub fan_profiles: Vec<String>,
}

#[derive(Debug)]
pub enum ProfileInput {
    Enabled,
    UpdateProfile,
}

#[factory(pub)]
impl FactoryComponent for Profile {
    type CommandOutput = ();
    type Init = ProfileInit;
    type Input = ProfileInput;
    type Output = ProfilesInput;
    type ParentInput = ProfilesInput;
    type ParentWidget = adw::PreferencesGroup;

    view! {
        adw::ExpanderRow {
            set_title: &self.name,

            #[chain(build())]
            bind_property: ("expanded", &delete_button, "visible"),

            add_prefix = &gtk::Box {
                set_valign: gtk::Align::Center,

                gtk::CheckButton {
                    #[watch]
                    set_active: self.active,

                    set_group: Some(&RADIO_GROUP.with(|g| (**g).clone())),

                    connect_toggled[sender, index] => move |btn| {
                        if btn.is_active() {
                            sender.input(ProfileInput::Enabled);
                            sender.output(ProfilesInput::Enabled(index.clone()));
                        }
                    }
                },
            },

            add_action = &gtk::Box {
                set_valign: gtk::Align::Center,
                set_margin_end: 2,

                #[name = "delete_button"]
                gtk::Button {
                    add_css_class: "destructive-action",
                    set_icon_name: "remove",
                    set_visible: false,
                    #[watch]
                    set_sensitive: !self.active,
                    connect_clicked[sender, index] => move |_| {
                        sender.output(ProfilesInput::Remove(index.clone()));
                    }
                }
            },

            add_row = &gtk::ListBoxRow {
                set_activatable: false,

                gtk::Box {
                    set_margin_all: 5,

                    #[name = "keyboard_label"]
                    gtk::Label {
                        set_label: "Keyboard profile",
                    },
                    gtk::Box {
                        set_hexpand: true,
                    },
                    #[local_ref]
                    keyboard_box -> gtk::ComboBoxText {},
                }
            },

            add_row = &gtk::ListBoxRow {
                set_activatable: false,

                gtk::Box {
                    set_margin_all: 5,

                    #[name = "fan_label"]
                    gtk::Label {
                        set_label: "Fan profile"
                    },
                    gtk::Box {
                        set_hexpand: true,
                    },
                    #[local_ref]
                    fan_box -> gtk::ComboBoxText {},
                }
            }
        }
    }

    fn output_to_parent_input(output: Self::Output) -> Option<ProfilesInput> {
        Some(output)
    }

    fn init_widgets(
        &mut self,
        index: &DynamicIndex,
        root: &Self::Root,
        _returned_widget: &<Self::ParentWidget as FactoryView>::ReturnedWidget,
        sender: FactorySender<Self>,
    ) -> Self::Widgets {
        let keyboard_box = self.keyboard_combo_box.widget();
        let fan_box = self.fan_combo_box.widget();

        let widgets = view_output!();

        widgets
    }

    fn init_model(init: Self::Init, _index: &DynamicIndex, sender: FactorySender<Self>) -> Self {
        let ProfileInit {
            name,
            info,
            active,
            keyboard_profiles,
            fan_profiles,
        } = init;

        let active_index = keyboard_profiles
            .iter()
            .position(|profile| profile == &info.keyboard);
        let keyboard_combo_box = SimpleComboBox::builder()
            .launch(SimpleComboBox {
                variants: keyboard_profiles,
                active_index,
            })
            .forward(sender.input_sender(), |_| ProfileInput::UpdateProfile);

        let active_index = fan_profiles.iter().position(|profile| profile == &info.fan);
        let fan_combo_box = SimpleComboBox::builder()
            .launch(SimpleComboBox {
                variants: fan_profiles,
                active_index,
            })
            .forward(sender.input_sender(), |_| ProfileInput::UpdateProfile);

        Self {
            name,
            info,
            active,
            keyboard_combo_box,
            fan_combo_box,
        }
    }

    fn update(&mut self, message: Self::Input, sender: FactorySender<Self>) {
        let name = self.name.clone();

        match message {
            ProfileInput::Enabled => {
                if !self.active {
                    sender.oneshot_command(async move {
                        STATE.emit(TailorStateMsg::SetActiveProfile(name));
                    });
                }
            }
            ProfileInput::UpdateProfile => {
                let keyboard = self
                    .keyboard_combo_box
                    .state()
                    .get()
                    .model
                    .get_active_elem()
                    .unwrap()
                    .clone();

                let fan = self
                    .fan_combo_box
                    .state()
                    .get()
                    .model
                    .get_active_elem()
                    .unwrap()
                    .clone();

                self.info = ProfileInfo { keyboard, fan };

                let profile = self.info.clone();

                sender.oneshot_command(async move {
                    STATE.emit(TailorStateMsg::AddProfile(name, profile));
                });
            }
        }
    }
}
