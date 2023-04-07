use gtk::prelude::{ButtonExt, WidgetExt};
use relm4::factory::FactoryVecDeque;
use relm4::gtk::traits::OrientableExt;
use relm4::prelude::DynamicIndex;
use relm4::{
    adw, component, gtk, Component, ComponentController, ComponentParts, ComponentSender,
    Controller, RelmWidgetExt,
};
use relm4_components::simple_combo_box::{SimpleComboBox, SimpleComboBoxMsg};
use tailor_api::{Color, ColorPoint, ColorProfile, ColorTransition};

use super::color_button::{ColorButton, ColorButtonInput};
use super::factories::color::ColorRow;
use crate::state::{tailor_connection, TailorStateMsg, STATE};
use crate::templates;

#[derive(Debug)]
pub enum ColorProfileType {
    Loading,
    None,
    Single,
    Multiple,
}

impl std::fmt::Display for ColorProfileType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Loading => "Loading",
            Self::None => "None",
            Self::Single => "Single",
            Self::Multiple => "Multiple",
        })
    }
}

pub struct KeyboardEdit {
    profile_name: Option<String>,
    color_profile_type: ColorProfileType,
    colors: FactoryVecDeque<ColorRow>,
    color_button: Controller<ColorButton>,
    type_selector: Controller<SimpleComboBox<ColorProfileType>>,
    visible: bool,
}

#[derive(Debug)]
pub enum KeyboardEditInput {
    Load(String),
    SetType(ColorProfileType),
    Up(DynamicIndex),
    Down(DynamicIndex),
    Remove(DynamicIndex),
    Add,
    Apply,
    Cancel,
}

#[component(pub)]
impl Component for KeyboardEdit {
    type CommandOutput = ColorProfile;
    type Init = ();
    type Input = KeyboardEditInput;
    type Output = ();

    view! {
        #[template]
        templates::DialogWindow {
            #[watch]
            set_visible: model.visible,

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,

                gtk::WindowHandle {
                    gtk::CenterBox {
                        #[wrap(Some)]
                        set_center_widget = &gtk::Label {
                            add_css_class: "title-4",
                            set_margin_all: 12,
                            #[watch]
                            set_label: &format!("Edit keyboard profile '{}'", model.profile_name.as_deref().unwrap_or_default()),
                        },

                        #[local_ref]
                        #[wrap(Some)]
                        set_start_widget = type_selector_widget -> gtk::ComboBoxText {
                            set_margin_start: 12,
                            set_valign: gtk::Align::Center,
                        },
                    },
                },

                gtk::ScrolledWindow {
                    add_css_class: "background",

                    adw::Clamp {
                        set_margin_all: 12,
                        set_vexpand: true,

                        match model.color_profile_type {
                            ColorProfileType::Loading => {
                                #[name(spinner)]
                                gtk::Spinner {
                                }
                            },
                            ColorProfileType::None => {
                                gtk::Label {
                                    set_label: "Disable the keyboard lights",
                                }
                            },
                            ColorProfileType::Single => {
                                gtk::Box {
                                    set_halign: gtk::Align::Center,
                                    set_valign: gtk::Align::Center,

                                    #[local_ref]
                                    color_button -> gtk::Button,
                                }
                            },
                            ColorProfileType::Multiple => {
                                gtk::Box {
                                    set_orientation: gtk::Orientation::Vertical,

                                    gtk::Box {
                                        set_margin_bottom: 6,

                                        gtk::Label {
                                            set_label: "Color pattern",
                                        },
                                        gtk::Box {
                                            set_hexpand: true,
                                        },
                                        gtk::Button {
                                            set_icon_name: "plus",
                                            set_halign: gtk::Align::End,
                                            connect_clicked => KeyboardEditInput::Add,
                                        }
                                    },

                                    #[local]
                                    color_points -> gtk::ListBox {
                                        set_valign: gtk::Align::Start,
                                        add_css_class: "boxed-list",
                                    }
                                }
                            }
                        }
                    }
                },

                gtk::Separator {},

                #[template]
                templates::MsgDialogButtons {
                    #[template_child]
                    cancel_button -> gtk::Button {
                        connect_clicked => KeyboardEditInput::Cancel,
                    },
                    #[template_child]
                    save_button -> gtk::Button {
                        connect_clicked => KeyboardEditInput::Apply,
                    },
                }
            }
        }
    }

    fn post_view() {
        spinner.set_spinning(matches!(
            model.color_profile_type,
            ColorProfileType::Loading
        ));
    }

    fn init(
        _: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let color_points = gtk::ListBox::default();
        let colors = FactoryVecDeque::new(color_points.clone(), sender.input_sender());

        let color_button = ColorButton::builder()
            .launch(Color {
                r: 255,
                g: 255,
                b: 255,
            })
            .detach();

        let type_selector = SimpleComboBox::builder()
            .launch(SimpleComboBox {
                active_index: None,
                variants: vec![
                    ColorProfileType::None,
                    ColorProfileType::Single,
                    ColorProfileType::Multiple,
                ],
            })
            .forward(sender.input_sender(), |idx| {
                KeyboardEditInput::SetType(match idx {
                    0 => ColorProfileType::None,
                    1 => ColorProfileType::Single,
                    _ => ColorProfileType::Multiple,
                })
            });

        let model = Self {
            profile_name: None,
            color_profile_type: ColorProfileType::Loading,
            colors,
            color_button,
            type_selector,
            visible: false,
        };

        let type_selector_widget = model.type_selector.widget();
        let color_button = model.color_button.widget();
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update_cmd(
        &mut self,
        color_profile: Self::CommandOutput,
        _sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        self.visible = true;

        match color_profile {
            ColorProfile::None => {
                self.type_selector.emit(SimpleComboBoxMsg::SetActiveIdx(0));
            }
            ColorProfile::Single(color) => {
                self.type_selector.emit(SimpleComboBoxMsg::SetActiveIdx(1));
                self.color_button.emit(ColorButtonInput::UpdateColor(color));
            }
            ColorProfile::Multiple(color_profile) => {
                self.color_profile_type = ColorProfileType::Multiple;
                self.type_selector.emit(SimpleComboBoxMsg::SetActiveIdx(2));
                let mut guard = self.colors.guard();
                guard.clear();
                for color_point in color_profile {
                    guard.push_back(color_point);
                }
            }
        }
    }

    fn update(&mut self, input: Self::Input, sender: ComponentSender<Self>, _root: &Self::Root) {
        match input {
            KeyboardEditInput::Load(name) => {
                self.profile_name = Some(name.clone());

                let connection = tailor_connection().unwrap();
                sender.oneshot_command(async move {
                    if let Ok(color_profile) = connection.get_keyboard_profile(&name).await {
                        color_profile
                    } else {
                        tracing::error!("Couldn't load keyboard profile");
                        ColorProfile::None
                    }
                });
            }
            KeyboardEditInput::SetType(ty) => {
                self.color_profile_type = ty;
            }
            KeyboardEditInput::Apply => {
                STATE.emit(TailorStateMsg::AddKeyboardProfile(
                    self.profile_name.clone().unwrap(),
                    self.compile(),
                ));
                self.visible = false;
            }
            KeyboardEditInput::Cancel => {
                self.visible = false;
            }
            KeyboardEditInput::Add => {
                let last_elem = self
                    .colors
                    .back()
                    .map(|row| row.inner.clone())
                    .unwrap_or_else(|| ColorPoint {
                        transition: ColorTransition::Linear,
                        color: Color {
                            r: 255,
                            g: 255,
                            b: 255,
                        },
                        transition_time: 1000,
                    });
                self.colors.guard().push_back(last_elem);
            }
            KeyboardEditInput::Up(index) => {
                let index = index.current_index();
                if index != 0 {
                    self.colors.guard().move_to(index, index.saturating_sub(1));
                }
            }
            KeyboardEditInput::Down(index) => {
                let index = index.current_index();
                let last_idx = self.colors.len().saturating_sub(1);
                if index != last_idx {
                    self.colors
                        .guard()
                        .move_to(index, (index + 1).min(last_idx));
                }
            }
            KeyboardEditInput::Remove(index) => {
                let index = index.current_index();
                self.colors.guard().remove(index);
            }
        }
    }
}

impl KeyboardEdit {
    fn compile(&self) -> ColorProfile {
        match self.color_profile_type {
            ColorProfileType::Loading | ColorProfileType::None => ColorProfile::None,
            ColorProfileType::Single => {
                ColorProfile::Single(self.color_button.model().color.clone())
            }
            ColorProfileType::Multiple => {
                ColorProfile::Multiple(self.colors.iter().map(|row| row.inner.clone()).collect())
            }
        }
    }
}
