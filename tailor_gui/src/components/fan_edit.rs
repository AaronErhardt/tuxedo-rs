use crate::tailor_state::tailor_connection;
use adw::{prelude::MessageDialogExtManual, traits::MessageDialogExt};
use gtk::{
    prelude::{BoxExt, WidgetExt},
    traits::{ButtonExt, GtkWindowExt, OrientableExt, DrawingAreaExt, StyleContextExt}, glib::ffi::G_REGEX_DOTALL, gdk::{builders::RGBABuilder, RGBA},
};
use relm4::{
    adw, component, factory::FactoryVecDeque, gtk, prelude::DynamicIndex, Component,
    ComponentParts, ComponentSender, RelmWidgetExt, drawing::DrawHandler,
};
use tailor_api::FanProfilePoint;

use super::factories::fan_point::FanPoint;

pub struct FanEdit {
    profile_name: Option<String>,
    old_profile: Vec<FanProfilePoint>,
    profile: FactoryVecDeque<FanPoint>,
    drawing_handler: DrawHandler,
    color: RGBA,
}

#[derive(Debug)]
pub enum FanEditInput {
    Load(String),
    Click,
    #[doc(hidden)]
    Cancel,
    #[doc(hidden)]
    Save,
    #[doc(hidden)]
    Apply,
}

#[component(pub)]
impl Component for FanEdit {
    type CommandOutput = Vec<FanProfilePoint>;
    type Input = FanEditInput;
    type Output = ();
    type Init = ();
    type Widgets = KeyboardEditWidgets;

    view! {
        dialog = adw::MessageDialog {
            set_default_size: (600, 300),
            add_css_class: "fan-dialog",
            set_modal: true,
            connect_close_request => |_| gtk::Inhibit(true),

            #[wrap(Some)]
            #[local_ref]
            set_extra_child = drawing_area -> gtk::DrawingArea {
                set_vexpand: true,
                set_hexpand: true,
                add_controller = &gtk::GestureClick {
                    connect_pressed[sender] => move |_, _, _, _| {
                        sender.input(FanEditInput::Click);
                    }
                }
            },

            // #[wrap(Some)]
            // set_extra_child = &gtk::Box {
                // set_orientation: gtk::Orientation::Vertical,
                // set_spacing: 12,
// 
                // gtk::CenterBox {
                    // set_orientation: gtk::Orientation::Horizontal,
// 
                    // #[wrap(Some)]
                    // set_center_widget = &gtk::Label {
                        // add_css_class: "title-2",
                        // #[watch]
                        // set_label: &format!("Edit profile \"{}\"",
                            // model.profile_name.as_deref().unwrap_or_default()),
                    // },
// 
                    // #[wrap(Some)]
                    // set_end_widget = &gtk::Button {
                        // set_halign: gtk::Align::End,
                        // set_icon_name: "plus-symbolic",
                    // }
                // },
// 
                // gtk::ScrolledWindow {
                    // set_vscrollbar_policy: gtk::PolicyType::Never,
// 
                    // if model.profile.is_empty() {
                        // gtk::Box {
                            // gtk::Spinner,
                            // gtk::Label {
                                // set_label: "Loading"
                            // }
                        // }
                    // } else {
                        // gtk::Box {
                            // set_halign: gtk::Align::Center,
// 
                            // #[local]
                            // fan_points -> gtk::Box {
                                // set_margin_all: 12,
                                // set_spacing: 12,
                            // }
                        // }
                    // }
                // },
            //},

            add_response: ("cancel", "Cancel"),
            set_response_appearance: ("cancel", adw::ResponseAppearance::Destructive),
            add_response: ("save", "Save"),
            set_response_appearance: ("save", adw::ResponseAppearance::Suggested),
            set_close_response: "cancel",
        }
    }

    fn init(
        _: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let fan_points = gtk::Box::default();
        let profile = FactoryVecDeque::new(fan_points.clone(), sender.input_sender());
        let color = root.style_context().lookup_color("theme_selected_bg_color").unwrap();

        let mut model = Self {
            profile_name: None,
            old_profile: Vec::new(),
            profile,
            drawing_handler: DrawHandler::new(),
            color,
        };

        let drawing_area = model.drawing_handler.drawing_area();
        let widgets = view_output!();

        model.draw();

        ComponentParts { model, widgets }
    }

    fn update_cmd_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        profile: Self::CommandOutput,
        sender: ComponentSender<Self>,
    ) {
        self.old_profile = profile.clone();

        {
            let mut guard = self.profile.guard();
            guard.clear();
            for point in profile {
                guard.push_back(point);
            }
        }

        let dialog = widgets.dialog.clone();
        let future_sender = sender.clone();
        relm4::spawn_local(async move {
            let response = dbg!(dialog.run_future().await);
            future_sender.input(if response == "save" {
                FanEditInput::Save
            } else {
                FanEditInput::Cancel
            });
            dialog.hide();
        });

        self.update_view(widgets, sender);
    }

    fn update(&mut self, input: Self::Input, sender: ComponentSender<Self>) {
        self.draw();
        match input {
            FanEditInput::Load(name) => {
                self.profile_name = Some(name.clone());

                let connection = tailor_connection();
                sender.oneshot_command(async move {
                    if let Ok(profile_points) = connection.get_fan_profile(&name).await {
                        profile_points
                    } else {
                        todo!()
                    }
                });
            }
            FanEditInput::Apply => {}
            FanEditInput::Cancel => {}
            FanEditInput::Save => {}
            FanEditInput::Click => {}
        }
    }
}

impl FanEdit {
    fn draw(&mut self) {
        println!("Drawing!");
        let ctx = self.drawing_handler.get_context();
        let color = &self.color;
        ctx.set_source_rgb(color.red() as f64, color.green() as f64, color.blue() as f64);
        ctx.rectangle(0.0, 0.0, 100.0, 100.0);
        ctx.fill().unwrap();
        ctx.move_to(0.0, 0.0);
        ctx.new_path();
        ctx.line_to(50.0, 50.0);
        ctx.line_to(300.0, 0.0);
        ctx.line_to(0.0, 0.0);
        ctx.close_path();
        ctx.fill().unwrap();
        //ctx.clip();
    }
}