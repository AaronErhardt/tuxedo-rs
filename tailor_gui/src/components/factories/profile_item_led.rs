use relm4::factory::FactoryView;
use relm4::prelude::{DynamicIndex, FactoryComponent};
use relm4::{adw, gtk, Component, ComponentController, Controller, FactorySender};
use relm4_components::simple_combo_box::SimpleComboBox;
use relm4_icons::icon_name;
use tailor_api::{LedDeviceInfo, LedProfile};

use super::profile::ProfileInput;
use crate::templates;

#[derive(Debug)]
pub struct ProfileItemLed {
    device_info: LedDeviceInfo,
    combo_box: Controller<SimpleComboBox<String>>,
}

pub struct ProfileItemLedInit {
    pub device_info: LedDeviceInfo,
    pub led_profiles: Vec<String>,
    pub index: usize,
}

#[relm4::factory(pub)]
impl FactoryComponent for ProfileItemLed {
    type CommandOutput = ();
    type Init = ProfileItemLedInit;
    type Input = ();
    type Output = u8;
    type ParentInput = ProfileInput;
    type ParentWidget = adw::ExpanderRow;

    view! {
        #[root]
        #[template]
        templates::ProfileListItem {
            #[template_child]
            image -> gtk::Image {
                set_icon_name: Some(icon_name::COLOR),
            },

            #[template_child]
            label -> gtk::Label {
                set_label: &format!("{}: {}", self.device_info.device_name, self.device_info.function),
            },

            #[template_child]
            row -> gtk::Box {
                #[local_ref]
                led_box -> gtk::ComboBoxText {},
            }
        }
    }

    fn init_model(init: Self::Init, _index: &DynamicIndex, sender: FactorySender<Self>) -> Self {
        let ProfileItemLedInit {
            device_info,
            led_profiles,
            index,
        } = init;
        let combo_box = SimpleComboBox::builder()
            .launch(SimpleComboBox {
                variants: led_profiles,
                active_index: Some(index),
            })
            .forward(sender.output_sender(), |output| output as u8);
        Self {
            device_info,
            combo_box,
        }
    }

    fn init_widgets(
        &mut self,
        _index: &Self::Index,
        root: &Self::Root,
        _returned_widget: &<Self::ParentWidget as FactoryView>::ReturnedWidget,
        _sender: FactorySender<Self>,
    ) -> Self::Widgets {
        let led_box = self.combo_box.widget();
        let widgets = view_output!();
        widgets
    }

    fn forward_to_parent(_output: Self::Output) -> Option<Self::ParentInput> {
        Some(ProfileInput::UpdateProfile)
    }
}

impl ProfileItemLed {
    pub fn get_profile(&self) -> LedProfile {
        let profile = self
            .combo_box
            .state()
            .get()
            .model
            .get_active_elem()
            .unwrap()
            .clone();
        let LedDeviceInfo {
            device_name,
            function,
        } = self.device_info.clone();
        LedProfile {
            device_name,
            function,
            profile,
        }
    }
}
