use adw::traits::{ExpanderRowExt, PreferencesRowExt};
use gtk::prelude::{WidgetExt, CheckButtonExt};
use relm4::{
    adw, factory,
    factory::{DynamicIndex, FactoryComponent, FactoryComponentSender, FactoryView},
    gtk, Component, ComponentController, Controller, WidgetPlus,
};
use relm4_components::simple_combo_box::SimpleComboBox;
use tailor_api::ProfileInfo;

use crate::{
    components::profiles::ProfilesInput,
    tailor_state::{FAN_PROFILES, KEYBOARD_PROFILES, TAILOR_STATE},
};

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
}

#[derive(Debug)]
pub enum ProfileInput {
    Enabled,
    UpdateProfile,
}

//relm4::new_stateful_action!()

#[factory(pub)]
impl FactoryComponent for Profile {
    type ParentWidget = adw::PreferencesGroup;
    type ParentInput = ProfilesInput;
    type CommandOutput = ();
    type Input = ProfileInput;
    type Output = ProfilesInput;
    type Init = ProfileInit;
    type Widgets = ProfileWidgets;

    view! {
        adw::ExpanderRow {
            set_title: &self.name,

            add_prefix = &gtk::Box {
                set_valign: gtk::Align::Center,
                gtk::CheckButton {
                    #[watch]
                    set_active: self.active,

                    connect_toggled[index] => move |btn| {
                        if btn.is_active() {
                            sender.input(ProfileInput::Enabled);
                            sender.output(ProfilesInput::Enabled(index.clone()));
                        }
                    }
                }
            },

            add_row = &gtk::Box {
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
            },

            add_row = &gtk::Box {
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

    fn output_to_parent_input(output: Self::Output) -> Option<ProfilesInput> {
        Some(output)
    }

    fn init_widgets(
        &mut self,
        index: &DynamicIndex,
        root: &Self::Root,
        _returned_widget: &<Self::ParentWidget as FactoryView>::ReturnedWidget,
        sender: FactoryComponentSender<Self>,
    ) -> Self::Widgets {
        let keyboard_box = self.keyboard_combo_box.widget();
        let fan_box = self.fan_combo_box.widget();

        let widgets = view_output!();

        widgets
    }

    fn init_model(
        init: Self::Init,
        _index: &DynamicIndex,
        sender: FactoryComponentSender<Self>,
    ) -> Self {
        let ProfileInit { name, info, active } = init;

        let variants = KEYBOARD_PROFILES.read().clone();
        let active_index = variants.iter().position(|var| var == &info.keyboard);
        let keyboard_combo_box = SimpleComboBox::builder()
            .launch(SimpleComboBox {
                variants,
                active_index,
            })
            .forward(sender.input_sender(), |_| ProfileInput::UpdateProfile);

        let variants = FAN_PROFILES.read().clone();
        let active_index = variants.iter().position(|var| var == &info.fan);
        let fan_combo_box = SimpleComboBox::builder()
            .launch(SimpleComboBox {
                variants,
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

    fn update(&mut self, message: Self::Input, sender: FactoryComponentSender<Self>) {
        let name = self.name.clone();

        let guard = TAILOR_STATE.read();
        let connection = guard.as_ref().unwrap().connection.clone();

        match message {
            ProfileInput::Enabled => {
                sender.oneshot_command(async move {
                    connection.set_active_global_profile_name(&name).await.ok();
                    connection.reload().await.ok();
                });
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
                    connection.add_global_profile(&name, &profile).await.ok();
                    connection.reload().await.ok();
                });
            }
        }
    }
}
