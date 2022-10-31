use gtk::{
    prelude::{ButtonExt, WidgetExt},
    traits::ListBoxRowExt,
};
use relm4::{
    adw, component, factory::FactoryVecDeque, gtk, prelude::DynamicIndex, Component,
    ComponentController, ComponentParts, ComponentSender, Controller, RelmWidgetExt
};

use crate::tailor_state::{FAN_PROFILES, TAILOR_STATE};

use super::{
    factories::list_item::ListItem,
    fan_edit::{FanEdit, FanEditInput},
};

pub struct FanList {
    profiles: FactoryVecDeque<ListItem>,
    fan_edit: Controller<FanEdit>,
    root: adw::Clamp,
}

#[derive(Debug)]
pub enum ListInput {
    UpdateProfiles(Vec<String>),
    Edit(usize),
    Remove(DynamicIndex),
}

#[component(pub)]
impl Component for FanList {
    type CommandOutput = ();
    type Input = ListInput;
    type Output = ();
    type Init = ();
    type Widgets = ProfilesWidgets;

    view! {
        adw::Clamp {
            set_margin_top: 10,
            set_margin_bottom: 10,

            #[local]
            profile_box -> gtk::ListBox {
                set_valign: gtk::Align::Start,
                add_css_class: "boxed-list",

                connect_row_activated[sender] => move |_, row| {
                    let index = row.index();
                    sender.input(ListInput::Edit(index as usize));
                }
            },
        }
    }

    fn init(
        _: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let profile_box = gtk::ListBox::default();
        let profiles = FactoryVecDeque::new(profile_box.clone(), sender.input_sender());

        let fan_edit = FanEdit::builder().transient_for(root).launch(()).detach();

        FAN_PROFILES.subscribe(sender.input_sender(), move |state| {
            ListInput::UpdateProfiles(state.clone())
        });

        let model = Self {
            profiles,
            root: root.clone(),
            fan_edit,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, input: Self::Input, sender: ComponentSender<Self>) {
        match input {
            ListInput::UpdateProfiles(list) => {
                // Repopulate the profiles
                let mut guard = self.profiles.guard();
                guard.clear();
                for list_item in list {
                    guard.push_back(list_item);
                }
            }
            ListInput::Edit(index) => {
                if let Some(item) = self.profiles.get(index) {
                    let name = item.name.clone();
                    self.fan_edit.emit(FanEditInput::Load(name));
                }
            }
            ListInput::Remove(index) => {
                let index = index.current_index();
                let element = self.profiles.guard().remove(index).unwrap();

                let connection = TAILOR_STATE.read().as_ref().unwrap().connection.clone();
                sender.oneshot_command(async move {
                    dbg!(connection.remove_fan_profile(&element.name).await).ok();
                })
            }
        }
    }
}
