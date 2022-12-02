use gtk::prelude::{OrientableExt, WidgetExt};
use relm4::{
    factory,
    factory::{DynamicIndex, FactoryComponent, FactorySender, FactoryView},
    gtk, Component, ComponentController, Controller,
};
use tailor_api::{Color, ColorPoint};

use crate::components::{
    color_button::ColorButton, keyboard_edit::KeyboardEditInput, profiles::ProfilesInput,
};

pub struct ColorRow {
    inner: ColorPoint,
    color_button: Controller<ColorButton>,
}

#[derive(Debug)]
pub enum ColorInput {
    ColorSet(Color),
    Enabled,
    UpdateProfile,
}

#[factory(pub)]
impl FactoryComponent for ColorRow {
    type ParentWidget = gtk::Box;
    type ParentInput = KeyboardEditInput;
    type CommandOutput = ();
    type Input = ColorInput;
    type Output = ProfilesInput;
    type Init = ColorPoint;
    type Widgets = ProfileWidgets;

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Horizontal,

            gtk::SpinButton {
                set_adjustment: &gtk::Adjustment::new(1.0, 0.0, 100.0, 0.1, 1.0, 1.0),
                set_climb_rate: 1.0,
                set_digits: 2,
            },

            gtk::Box {
                set_hexpand: true,
            },

            #[local_ref]
            color_button -> gtk::Button,
        }
    }

    fn output_to_parent_input(output: Self::Output) -> Option<KeyboardEditInput> {
        None
    }

    fn init_model(
        inner: Self::Init,
        _index: &DynamicIndex,
        sender: FactorySender<Self>,
    ) -> Self {
        let color_button = ColorButton::builder()
            .launch(Color {
                r: 0,
                g: 255,
                b: 100,
            })
            .forward(sender.input_sender(), |color| ColorInput::ColorSet(color));

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

    fn update(&mut self, message: Self::Input, sender: FactorySender<Self>) {
        match message {
            ColorInput::ColorSet(color) => {}
            ColorInput::Enabled => todo!(),
            ColorInput::UpdateProfile => todo!(),
        }
    }
}
