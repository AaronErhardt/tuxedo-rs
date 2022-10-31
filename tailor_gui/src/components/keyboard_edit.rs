use crate::tailor_state::TAILOR_STATE;
use gtk::{
    prelude::{BoxExt, WidgetExt},
    traits::GtkWindowExt,
};
use relm4::{
    adw, component, factory::FactoryVecDeque, gtk, Component, ComponentParts, ComponentSender,
};
use tailor_api::ColorProfile;

use super::factories::color::ColorRow;

enum ColorProfileType {
    Loading,
    None,
    Single,
    Multiple,
}

pub struct KeyboardEdit {
    profile_name: String,
    color_profile_type: ColorProfileType,
    colors: FactoryVecDeque<ColorRow>,
}

#[derive(Debug)]
pub enum KeyboardEditInput {}

#[component(pub)]
impl Component for KeyboardEdit {
    type CommandOutput = ColorProfile;
    type Input = KeyboardEditInput;
    type Output = ();
    type Init = ();
    type Widgets = KeyboardEditWidgets;

    view! {
        adw::Window {
            set_modal: true,

            #[wrap(Some)]
            set_titlebar = &adw::HeaderBar {

            },

            adw::Clamp {
                set_margin_top: 10,
                set_margin_bottom: 10,

                match model.color_profile_type {
                    ColorProfileType::Loading => {
                        gtk::Box {}
                    },
                    ColorProfileType::None => {
                        gtk::Box {}
                    },
                    ColorProfileType::Single => {
                        gtk::Box {}
                    },
                    ColorProfileType::Multiple => {
                        #[local]
                        color_points -> gtk::Box {
                            set_spacing: 5,
                            set_valign: gtk::Align::Start,
                        }
                    }
                }
            }
        }
    }

    fn init(
        _: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let color_points = gtk::Box::default();
        let colors = FactoryVecDeque::new(color_points.clone(), sender.input_sender());

        TAILOR_STATE.subscribe_optional(sender.input_sender(), move |state| {
            /*state.as_ref().map(|state| {
                KeyboardEditInput::UpdateProfiles((
                    state.profiles.clone(),
                    state.active_profile_name.clone(),
                ))
                ()
            })*/
            None
        });

        let model = Self {
            profile_name: "".into(),
            color_profile_type: ColorProfileType::Loading,
            colors,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update_cmd(&mut self, color_profile: Self::CommandOutput, sender: ComponentSender<Self>) {
        match color_profile {
            ColorProfile::None => todo!(),
            ColorProfile::Single(_) => todo!(),
            ColorProfile::Multiple(color_profile) => {
                let mut guard = self.colors.guard();
                guard.clear();
                for color_point in color_profile {
                    guard.push_back(color_point);
                }
            }
        }
    }

    fn update(&mut self, input: Self::Input, _sender: ComponentSender<Self>) {
        match input {}
    }
}
