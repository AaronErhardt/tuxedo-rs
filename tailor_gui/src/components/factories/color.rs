use gtk::prelude::{OrientableExt, WidgetExt};
use relm4::factory::{DynamicIndex, FactoryComponent, FactorySender, FactoryView};
use relm4::gtk::traits::{BoxExt, ButtonExt};
use relm4::{factory, gtk, Component, ComponentController, Controller};
use relm4_icons::icon_name;
use tailor_api::{Color, ColorPoint};

use crate::components::color_button::ColorButton;
use crate::components::led_edit::LedEditInput;

pub struct ColorRow {
    pub inner: ColorPoint,
    color_button: Controller<ColorButton>,
}

#[derive(Debug)]
pub enum ColorInput {
    SetColor(Color),
    SetTime(u32),
}

#[derive(Debug)]
pub enum ColorOutput {
    Up(DynamicIndex),
    Down(DynamicIndex),
    Remove(DynamicIndex),
}

#[factory(pub)]
impl FactoryComponent for ColorRow {
    type CommandOutput = ();
    type Init = ColorPoint;
    type Input = ColorInput;
    type Output = ColorOutput;
    type ParentInput = LedEditInput;
    type ParentWidget = gtk::ListBox;

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Horizontal,
            add_css_class: "header",

            gtk::Box {
                set_spacing: 12,
                set_orientation: gtk::Orientation::Horizontal,
                set_valign: gtk::Align::Center,

                #[local_ref]
                color_button -> gtk::Button,

                gtk::SpinButton {
                    set_adjustment: &gtk::Adjustment::new(1.0, 0.0, 1000.0, 0.1, 1.0, 1.0),
                    set_climb_rate: 1.0,
                    set_digits: 2,
                    set_width_request: 112,
                    set_tooltip_text: Some("The transition time in ms"),
                    set_value: self.inner.transition_time as f64 / 1000.0,

                    connect_value_changed[sender] => move |btn| {
                        let value = btn.value() * 1000.0;
                        sender.input(ColorInput::SetTime(value as u32));
                    }
                },

                gtk::Box {
                    set_hexpand: true,
                },

                gtk::Box {
                    set_spacing: 6,

                    gtk::Button {
                        set_icon_name: icon_name::UP,
                        connect_clicked[sender, index] => move |_| {
                            sender.output(ColorOutput::Up(index.clone()));
                        }
                    },
                    gtk::Button {
                        set_icon_name: icon_name::DOWN,
                        connect_clicked[sender, index] => move |_| {
                            sender.output(ColorOutput::Down(index.clone()));
                        }
                    },
                    gtk::Button {
                        set_icon_name: icon_name::CROSS_FILLED,
                        add_css_class: "destructive-action",
                        connect_clicked[sender, index] => move |_| {
                            sender.output(ColorOutput::Remove(index.clone()));
                        }
                    }
                }
            }
        }
    }

    fn forward_to_parent(output: Self::Output) -> Option<LedEditInput> {
        Some(match output {
            ColorOutput::Up(index) => LedEditInput::Up(index),
            ColorOutput::Down(index) => LedEditInput::Down(index),
            ColorOutput::Remove(index) => LedEditInput::Remove(index),
        })
    }

    fn init_model(inner: Self::Init, _index: &DynamicIndex, sender: FactorySender<Self>) -> Self {
        let color_button = ColorButton::builder()
            .launch(inner.color.clone())
            .forward(sender.input_sender(), ColorInput::SetColor);

        Self {
            color_button,
            inner,
        }
    }

    fn init_widgets(
        &mut self,
        index: &DynamicIndex,
        root: &Self::Root,
        _returned_widget: &<Self::ParentWidget as FactoryView>::ReturnedWidget,
        sender: FactorySender<Self>,
    ) -> Self::Widgets {
        let color_button = self.color_button.widget();

        let widgets = view_output!();

        widgets
    }

    fn update(&mut self, message: Self::Input, _sender: FactorySender<Self>) {
        match message {
            ColorInput::SetColor(color) => {
                self.inner.color = color;
            }
            ColorInput::SetTime(time) => {
                self.inner.transition_time = time;
            }
        }
    }
}
