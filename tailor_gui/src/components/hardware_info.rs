use relm4::gtk::prelude::{GridExt, OrientableExt, WidgetExt};
use relm4::gtk::traits::{ButtonExt, GtkWindowExt};
use relm4::{gtk, ComponentParts, ComponentSender, RelmWidgetExt, SimpleComponent};

use crate::state::{hardware_capabilities, HardwareCapabilities};
use crate::templates;

pub struct HardwareInfo {
    info: HardwareCapabilities,
}

#[relm4::component(pub)]
impl SimpleComponent for HardwareInfo {
    type Init = ();
    type Input = ();
    type Output = ();

    view! {
        #[template]
        #[name = "window"]
        templates::DialogWindow {
            set_visible: true,
            set_default_size: (0, 0),

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,

                gtk::WindowHandle {
                    gtk::CenterBox {
                        #[wrap(Some)]
                        set_center_widget = &gtk::Label {
                            add_css_class: "title-4",
                            set_margin_all: 12,
                            set_label: "Hardware information"
                        },
                    },
                },

                gtk::Grid {
                    set_margin_all: 12,
                    set_row_spacing: 6,
                    set_column_spacing: 6,
                    set_halign: gtk::Align::Center,
                    set_vexpand: true,

                    attach[0, 0, 1, 1] = &gtk::Label {
                        set_halign: gtk::Align::Start,
                        set_label: "Fans",
                    },
                    attach[0, 1, 1, 1] = &gtk::Label {
                        set_halign: gtk::Align::Start,
                        set_label: "LED devices",
                    },
                    attach[1, 0, 1, 1] = &gtk::Label {
                        set_halign: gtk::Align::Start,
                        set_label: &model.info.num_of_fans.to_string(),
                    },
                    attach[1, 1, 1, 1] = &gtk::Label {
                        set_halign: gtk::Align::Start,
                        set_label: led_info.trim_end_matches(", "),
                    },
                },

                gtk::Separator,

                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    add_css_class: "response-area",

                    gtk::Button {
                        set_label: "Close",
                        set_hexpand: true,
                        #[iterate]
                        add_css_class: &["flat", "suggested"],
                        connect_clicked: move |btn| {
                            let window = btn.toplevel_window().unwrap();
                            window.destroy();
                        },
                    },
                }
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: &Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = HardwareInfo {
            info: hardware_capabilities().unwrap().clone(),
        };
        let led_info: String = model
            .info
            .led_devices
            .iter()
            .map(|d| format!("{}, ", d.device_id()))
            .collect();
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }
}
