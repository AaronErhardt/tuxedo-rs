use futures::StreamExt;
use gtk::prelude::{BoxExt, ButtonExt, ListBoxRowExt, OrientableExt, WidgetExt};
use relm4::factory::FactoryVecDeque;
use relm4::prelude::DynamicIndex;
use relm4::{
    adw, component, gtk, Component, ComponentController, ComponentParts, ComponentSender,
    Controller,
};
use relm4_icons::icon_name;

use super::factories::list_item::{ListItem, ListMsg};
use super::led_edit::{LedEdit, LedEditInput};
use super::new_entry::{NewEntryDialog, NewEntryInit, NewEntryOutput};
use crate::state::{TailorStateInner, TailorStateMsg, STATE};
use crate::templates;

#[tracker::track]
pub struct LedList {
    #[do_not_track]
    profiles: FactoryVecDeque<ListItem<LedListInput>>,
    #[do_not_track]
    led_edit: Controller<LedEdit>,
    toast: Option<adw::Toast>,
}

#[derive(Debug)]
pub enum LedListInput {
    UpdateProfiles(Vec<String>),
    Rename(DynamicIndex, String),
    Edit(usize),
    Remove(DynamicIndex),
    Add,
}

impl ListMsg for LedListInput {
    fn ty() -> &'static str {
        "LED"
    }

    fn rename(index: DynamicIndex, text: String) -> Self {
        Self::Rename(index, text)
    }

    fn remove(index: DynamicIndex) -> Self {
        Self::Remove(index)
    }
}

#[component(pub)]
impl Component for LedList {
    type CommandOutput = ();
    type Init = ();
    type Input = LedListInput;
    type Output = ();

    view! {
        #[template]
        templates::CustomClamp {
            #[template_child]
            clamp {
                #[name(toast_overlay)]
                adw::ToastOverlay {
                    #[track(model.changed(LedList::toast()))]
                    add_toast?: model.toast.clone(),

                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_spacing: 6,

                        gtk::Box {
                            set_orientation: gtk::Orientation::Horizontal,

                            gtk::Label {
                                add_css_class: "heading",
                                set_label: "LED profiles",
                            },
                            gtk::Box {
                                set_hexpand: true,
                            },
                            gtk::Button {
                                set_icon_name: icon_name::PLUS,
                                connect_clicked => LedListInput::Add,
                            }
                        },

                        #[local]
                        profile_box -> gtk::ListBox {
                            set_valign: gtk::Align::Start,
                            add_css_class: "boxed-list",

                            connect_row_activated[sender] => move |_, row| {
                                let index = row.index();
                                sender.input(LedListInput::Edit(index as usize));
                            }
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
        STATE.subscribe_optional(sender.input_sender(), move |state| {
            let state = state.unwrap();
            if state.changed(TailorStateInner::led_profiles()) {
                Some(LedListInput::UpdateProfiles(state.led_profiles.clone()))
            } else {
                None
            }
        });

        let profile_box = gtk::ListBox::default();
        let profiles = FactoryVecDeque::new(profile_box.clone(), sender.input_sender());

        let led_edit = LedEdit::builder()
            .transient_for(&**root)
            .launch(())
            .detach();

        let model = Self {
            profiles,
            led_edit,
            toast: None,
            tracker: 0,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, input: Self::Input, _sender: ComponentSender<Self>, root: &Self::Root) {
        self.reset();

        match input {
            LedListInput::UpdateProfiles(list) => {
                // Repopulate the profiles
                let mut guard = self.profiles.guard();
                guard.clear();
                for list_item in list {
                    guard.push_back(list_item);
                }
            }
            LedListInput::Edit(index) => {
                if let Some(item) = self.profiles.get(index) {
                    let name = item.name.clone();
                    self.led_edit.emit(LedEditInput::Load(name));
                }
            }
            LedListInput::Rename(index, name) => {
                let index = index.current_index();
                let current_name = &self.profiles[index].name;
                if current_name != &name {
                    let count = self.profiles.iter().filter(|p| p.name == name).count();
                    if count == 0 {
                        STATE.emit(TailorStateMsg::RenameLedProfile {
                            from: current_name.clone(),
                            to: name,
                        });
                    } else {
                        self.profiles.guard()[index].name = current_name.clone();
                        self.set_toast(Some(adw::Toast::new("Name already exists")));
                    }
                }
            }
            LedListInput::Remove(index) => {
                if self.profiles.len() > 1 {
                    let index = index.current_index();
                    let element = self.profiles.guard().remove(index).unwrap();

                    STATE.emit(TailorStateMsg::DeleteLedProfile(element.name));
                } else {
                    self.set_toast(Some(adw::Toast::new("There must be at least one profile")));
                }
            }
            LedListInput::Add => {
                let profiles = self.profiles.iter().map(|i| i.name.to_string()).collect();
                let mut new_entry = NewEntryDialog::builder()
                    .transient_for(&**root)
                    .launch(NewEntryInit {
                        info: "Add LED profile".into(),
                        profiles,
                    })
                    .into_stream();
                relm4::spawn_local(async move {
                    if let Some(NewEntryOutput { name, based_of }) = new_entry.next().await.unwrap()
                    {
                        STATE.emit(TailorStateMsg::CopyLedProfile {
                            from: based_of,
                            to: name,
                        });
                    }
                });
            }
        }
    }
}
