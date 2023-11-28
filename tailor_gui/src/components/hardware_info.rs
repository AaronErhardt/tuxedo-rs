use std::fmt::Write;

use relm4::gtk::prelude::{GridExt, OrientableExt, WidgetExt};
use relm4::gtk::traits::{ButtonExt, GtkWindowExt};
use relm4::{gtk, ComponentParts, ComponentSender, RelmWidgetExt, SimpleComponent};

use crate::state::hardware_capabilities;
use crate::templates;

pub struct HardwareInfo;

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
                    set_margin_all: 18,
                    set_row_spacing: 12,
                    set_column_spacing: 12,
                    set_halign: gtk::Align::Center,
                    set_vexpand: true,

                    attach[0, 0, 1, 1] = &gtk::Label {
                        set_halign: gtk::Align::Start,
                        set_label: "Fans",
                    },
                    attach[1, 0, 1, 1] = &gtk::Label {
                        set_halign: gtk::Align::Start,
                        set_label: &info.num_of_fans.to_string(),
                    },
                    attach[0, 1, 1, 1] = &gtk::Label {
                        set_halign: gtk::Align::Start,
                        set_label: "LED devices",
                    },
                    attach[1, 1, 1, 1] = &gtk::Label {
                        set_halign: gtk::Align::Start,
                        set_label: &led_info,
                    },
                    attach[0, 2, 1, 1] = &gtk::Label {
                        set_halign: gtk::Align::Start,
                        set_label: "Performance profiles",
                    },
                    attach[1, 2, 1, 1] = &gtk::Label {
                        set_halign: gtk::Align::Start,
                        set_label: &performance_info,
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
        let model = HardwareInfo;

        let info = hardware_capabilities().unwrap().clone();

        let led_info: String = comma_list(info.led_devices.iter().map(|d| d.device_id()));
        let performance_info = comma_list_optional(info.performance_profiles);
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }
}

fn comma_list<I, S>(iter: I) -> String
where
    I: Iterator<Item = S>,
    S: Into<String>,
{
    let value: String = iter.fold(String::new(), |mut out, string| {
        write!(&mut out, ", {}", string.into()).unwrap();
        out
    });
    value.trim_end_matches(", ").to_owned()
}

fn comma_list_optional<I, S>(iter: Option<I>) -> String
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    if let Some(iter) = iter {
        comma_list(iter.into_iter())
    } else {
        "Device not available".into()
    }
}
