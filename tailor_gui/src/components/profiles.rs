use crate::{
    app::FullProfileInfo,
    state::{TailorStateMsg, STATE},
};
use adw::traits::PreferencesGroupExt;
use futures::StreamExt;
use gtk::prelude::{ButtonExt, WidgetExt};
use relm4::{
    adw, component, factory::FactoryVecDeque, gtk, prelude::DynamicIndex, Component,
    ComponentParts, ComponentSender,
};

use super::{
    factories::profile::{Profile, ProfileInit},
    new_profile::{NewProfileDialog, NewProfileInit},
};

pub struct Profiles {
    profiles: FactoryVecDeque<Profile>,
    keyboard: Vec<String>,
    fan: Vec<String>,
}

#[derive(Debug)]
pub enum ProfilesInput {
    UpdateProfiles {
        profiles: Vec<FullProfileInfo>,
        active_profile: String,
        fan_profiles: Vec<String>,
        keyboard_profiles: Vec<String>,
    },
    Enabled(DynamicIndex),
    Remove(DynamicIndex),
    Add,
}

#[component(pub)]
impl Component for Profiles {
    type CommandOutput = ();
    type Input = ProfilesInput;
    type Output = ();
    type Init = ();
    type Widgets = ProfilesWidgets;

    view! {
        adw::Clamp {
            set_margin_top: 10,
            set_margin_bottom: 10,

            #[local]
            profile_box -> adw::PreferencesGroup {
                set_title: "Profiles",
                #[wrap(Some)]
                set_header_suffix = &gtk::Button {
                    set_icon_name: "plus-symbolic",
                    connect_clicked => ProfilesInput::Add,
                }
            },
        }
    }

    fn init(
        _: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        STATE.subscribe(sender.input_sender(), move |state| {
            let state = state.unwrap();
            ProfilesInput::UpdateProfiles {
                profiles: state.profiles.clone(),
                active_profile: state.active_profile_name.clone(),
                fan_profiles: state.fan_profiles.clone(),
                keyboard_profiles: state.keyboard_profiles.clone(),
            }
        });

        let profile_box = adw::PreferencesGroup::default();
        let profiles = FactoryVecDeque::new(profile_box.clone(), sender.input_sender());

        let model = Self {
            profiles,
            keyboard: Vec::new(),
            fan: Vec::new(),
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, input: Self::Input, _sender: ComponentSender<Self>, root: &Self::Root) {
        match input {
            ProfilesInput::UpdateProfiles {
                profiles,
                active_profile,
                keyboard_profiles,
                fan_profiles,
            } => {
                // Repopulate the profiles
                let mut guard = self.profiles.guard();
                guard.clear();
                for profile in profiles {
                    let active = active_profile == profile.name;
                    guard.push_back(ProfileInit {
                        name: profile.name,
                        info: profile.data,
                        keyboard_profiles: keyboard_profiles.clone(),
                        fan_profiles: fan_profiles.clone(),
                        active,
                    });
                }
                self.keyboard = keyboard_profiles;
                self.fan = fan_profiles;
            }
            ProfilesInput::Enabled(index) => {
                let index = index.current_index();
                for (idx, profile) in self.profiles.guard().iter_mut().enumerate() {
                    profile.active = idx == index;
                }
            }
            ProfilesInput::Remove(index) => {
                let index = index.current_index();
                if let Some(profile) = self.profiles.get(index) {
                    STATE.emit(TailorStateMsg::DeleteProfile(profile.name.clone()));
                }
            }
            ProfilesInput::Add => {
                let profiles = self.profiles.iter().map(|i| i.name.to_string()).collect();
                let fan = self.fan.clone();
                let keyboard = self.keyboard.clone();
                let mut new_profile = NewProfileDialog::builder()
                    .transient_for(root)
                    .launch(NewProfileInit {
                        profiles,
                        keyboard,
                        fan,
                    })
                    .into_stream();
                relm4::spawn_local(async move {
                    if let Some(info) = new_profile.next().await.unwrap() {
                        STATE.emit(TailorStateMsg::AddProfile(info.name, info.data));
                    }
                });
            }
        }
    }

    fn id(&self) -> String {
        "Profiles".to_string()
    }
}
