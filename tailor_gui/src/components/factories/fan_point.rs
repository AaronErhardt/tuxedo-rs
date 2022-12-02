use std::time::{Duration, Instant};

use gtk::glib::{self, timeout_add_local_once, MainContext};
use gtk::{
    glib::{clone, SourceId},
    prelude::{
        BoxExt, ButtonExt, ObjectExt, OrientableExt, PopoverExt, RangeExt, ScaleExt, WidgetExt,
    },
};
use relm4::{
    factory,
    factory::{DynamicIndex, FactoryComponent, FactorySender, FactoryView},
    gtk, tokio, RelmWidgetExt,
};
use tailor_api::{Color, FanProfilePoint};

use crate::components::fan_edit::FanEditInput;
use crate::state::{TailorStateMsg, STATE};

pub struct FanPoint {
    inner: FanProfilePoint,
    last_override_event: Option<SourceId>,
}

#[derive(Debug)]
pub enum FanPointInput {
    Enabled,
    UpdateProfile,
    SetValue(u8),
}

#[factory(pub)]
impl FactoryComponent for FanPoint {
    type ParentWidget = gtk::Box;
    type ParentInput = FanEditInput;
    type CommandOutput = ();
    type Input = FanPointInput;
    type Output = ();
    type Init = FanProfilePoint;
    type Widgets = ProfileWidgets;

    view! {
        main_box = gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 12,

            /*gtk::SpinButton {
                set_adjustment: &gtk::Adjustment::new(self.inner.temp as f64, 20.0, 100.0, 1.0, 1.0, 1.0),
                set_climb_rate: 1.0,
                set_digits: 0,
                set_snap_to_ticks: true,
                set_numeric: true,
            },*/

            #[name = "level_bar"]
            gtk::LevelBar {
                set_orientation: gtk::Orientation::Vertical,
                set_vexpand: true,
                set_max_value: 100.0,
                set_min_value: 0.0,
                set_inverted: true,

                #[watch]
                set_value: self.inner.fan as f64,

                add_offset_value: (*gtk::LEVEL_BAR_OFFSET_FULL, 50.0),
                add_offset_value: (*gtk::LEVEL_BAR_OFFSET_HIGH, 70.0),
                add_offset_value: (*gtk::LEVEL_BAR_OFFSET_LOW, 90.0),
            },

            #[name = "open_button"]
            gtk::Button {
                connect_clicked: clone!(@weak popover => move |_| {
                    popover.popup();
                }),

                gtk::Box {
                    set_halign: gtk::Align::Center,

                    gtk::Image {
                        set_icon_name: Some("profile-settings"),
                    },
                    #[name = "popover"]
                    gtk::Popover {
                        set_vexpand: false,

                        gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,
                            set_spacing: 6,
                            set_width_request: 140,

                            gtk::Scale {
                                set_orientation: gtk::Orientation::Horizontal,
                                set_hexpand: true,
                                set_digits: 0,
                                set_range: (0.0, 100.0),

                                #[watch]
                                #[block_signal(value_changed_handler)]
                                set_value: self.inner.fan as f64,

                                connect_value_changed[sender] => move |scale| {
                                    let value = scale.value();
                                    sender.input(FanPointInput::SetValue(value as u8));
                                } @value_changed_handler,
                            },

                            gtk::Scale {
                                set_orientation: gtk::Orientation::Horizontal,
                                set_hexpand: true,
                                set_digits: 0,
                                set_range: (0.0, 100.0),
                            },

                            gtk::Button {
                                set_icon_name: "remove",
                                add_css_class: "destructive-action",
                                connect_clicked[sender, popover] => move |_| {
                                    popover.popdown();
                                }
                            }
                        }
                    },
                }
            },

            gtk::Label {
                #[watch]
                set_label: &format!("{}°C", self.inner.temp),
            },
        }
    }

    fn output_to_parent_input(output: Self::Output) -> Option<FanEditInput> {
        None
    }

    fn init_model(
        inner: Self::Init,
        _index: &DynamicIndex,
        sender: FactorySender<Self>,
    ) -> Self {
        Self {
            inner,
            last_override_event: None,
        }
    }

    fn init_widgets(
        &mut self,
        index: &DynamicIndex,
        root: &Self::Root,
        _returned_widget: &<Self::ParentWidget as FactoryView>::ReturnedWidget,
        sender: FactorySender<Self>,
    ) -> Self::Widgets {
        let widgets = view_output!();

        widgets
    }

    fn update(&mut self, message: Self::Input, sender: FactorySender<Self>) {
        match message {
            FanPointInput::Enabled => todo!(),
            FanPointInput::UpdateProfile => todo!(),
            FanPointInput::SetValue(value) => {
                // Cancel the previous timeout if a new value has arrived.
                if let Some(source_id) = self.last_override_event.take() {
                    let main_context = MainContext::default();
                    if main_context.find_source_by_id(&source_id).is_some() {
                        source_id.remove();
                    }
                }

                // Don't override the value immediately, but wait a bit for other events to arrive.
                self.last_override_event = Some(timeout_add_local_once(
                    Duration::from_millis(60),
                    move || {
                        STATE.emit(TailorStateMsg::OverwriteFanSpeed(value));
                    },
                ));

                self.inner.fan = value;
            }
        }
    }
}
