use crate::{app::FullProfileInfo, tailor_state::TAILOR_STATE};
use adw::traits::PreferencesGroupExt;
use gtk::prelude::{ButtonExt, WidgetExt};
use relm4::{
    adw, component, factory::FactoryVecDeque, gtk, prelude::DynamicIndex, Component,
    ComponentParts, ComponentSender,
};

use super::factories::profile::{Profile, ProfileInit};

pub struct Profiles {
    profiles: FactoryVecDeque<Profile>,
}

#[derive(Debug)]
pub enum ProfilesInput {
    UpdateProfiles((Vec<FullProfileInfo>, String)),
    Enabled(DynamicIndex),
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
                }
            },
        }
    }

    fn init(
        _: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let profile_box = adw::PreferencesGroup::default();
        let profiles = FactoryVecDeque::new(profile_box.clone(), sender.input_sender());

        TAILOR_STATE.subscribe_optional(sender.input_sender(), move |state| {
            state.as_ref().map(|state| {
                ProfilesInput::UpdateProfiles((
                    state.profiles.clone(),
                    state.active_profile_name.clone(),
                ))
            })
        });

        let model = Self { profiles };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, input: Self::Input, _sender: ComponentSender<Self>) {
        match input {
            ProfilesInput::UpdateProfiles((profiles, active_profile)) => {
                // Repopulate the profiles
                let mut guard = self.profiles.guard();
                guard.clear();
                for profile in profiles {
                    let active = active_profile == profile.name;
                    guard.push_back(ProfileInit {
                        name: profile.name,
                        info: profile.data,
                        active,
                    });
                }
            }
            ProfilesInput::Enabled(index) => {
                let index = index.current_index();
                for (idx, profile) in self.profiles.guard().iter_mut().enumerate() {
                    profile.active = idx == index;
                }
            }
        }
    }
}
