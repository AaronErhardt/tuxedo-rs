use adw::prelude::PreferencesGroupExt;
use futures::StreamExt;
use gtk::prelude::ButtonExt;
use relm4::factory::FactoryVecDeque;
use relm4::prelude::DynamicIndex;
use relm4::{adw, component, gtk, Component, ComponentParts, ComponentSender, WidgetRef};
use relm4_icons::icon_names;

use super::factories::profile::{Profile, ProfileInit};
use super::new_entry::{NewEntryDialog, NewEntryInit, NewEntryOutput};
use crate::app::FullProfileInfo;
use crate::state::{TailorStateMsg, STATE};
use crate::templates;

pub struct Profiles {
    profiles: FactoryVecDeque<Profile>,
    led: Vec<String>,
    fan: Vec<String>,
}

#[derive(Debug)]
pub enum ProfilesInput {
    UpdateProfiles {
        profiles: Vec<FullProfileInfo>,
        active_profile: String,
        fan_profiles: Vec<String>,
        led_profiles: Vec<String>,
    },
    Enabled(DynamicIndex),
    Remove(DynamicIndex),
    Add,
}

#[component(pub)]
impl Component for Profiles {
    type CommandOutput = ();
    type Init = ();
    type Input = ProfilesInput;
    type Output = ();

    view! {
        #[template]
        templates::CustomClamp {
            #[template_child]
            clamp {
                #[local]
                profile_box -> adw::PreferencesGroup {
                    set_title: "Profiles",
                    #[wrap(Some)]
                    set_header_suffix = &gtk::Button {
                        set_icon_name: icon_names::PLUS,
                        connect_clicked => ProfilesInput::Add,
                    }
                },
            }
        }
    }

    fn init(
        _: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        STATE.subscribe(sender.input_sender(), move |state| {
            let state = state.unwrap();
            ProfilesInput::UpdateProfiles {
                profiles: state.profiles.clone(),
                active_profile: state.active_profile_name.clone(),
                fan_profiles: state.fan_profiles.clone(),
                led_profiles: state.led_profiles.clone(),
            }
        });

        let profile_box = adw::PreferencesGroup::default();
        let profiles = FactoryVecDeque::builder()
            .launch(profile_box.clone())
            .forward(sender.input_sender(), |msg| msg);

        let model = Self {
            profiles,
            led: Vec::new(),
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
                led_profiles,
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
                        led_profiles: led_profiles.clone(),
                        fan_profiles: fan_profiles.clone(),
                        active,
                    });
                }
                self.led = led_profiles;
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
                let mut new_profile = NewEntryDialog::builder()
                    .transient_for(root.widget_ref())
                    .launch(NewEntryInit {
                        profiles,
                        info: "Add profile".to_string(),
                    })
                    .into_stream();
                relm4::spawn_local(async move {
                    if let Some(NewEntryOutput { name, based_of }) =
                        new_profile.next().await.unwrap()
                    {
                        STATE.emit(TailorStateMsg::CopyProfile {
                            from: based_of,
                            to: name,
                        });
                    }
                });
            }
        }
    }

    fn id(&self) -> String {
        "Profiles".to_string()
    }
}
