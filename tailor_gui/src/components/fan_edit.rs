use std::f64::consts::PI;
use std::time::Duration;

use gtk::cairo::Operator;
use gtk::gdk::{self, RGBA};
use gtk::glib::{timeout_add_local_once, MainContext, SourceId};
use gtk::prelude::{
    BoxExt, ButtonExt, DrawingAreaExt, GestureDragExt, OrientableExt, StyleContextExt, WidgetExt,
};
use relm4::drawing::DrawHandler;
use relm4::{adw, component, gtk, Component, ComponentParts, ComponentSender, RelmWidgetExt};
use tailor_api::FanProfilePoint;

use crate::state::{tailor_connection, TailorStateMsg, STATE};
use crate::templates;

struct Colors {
    fill: RGBA,
    stroke: RGBA,
    warn: RGBA,
}

impl Colors {
    fn new(root: &adw::Window) -> Self {
        let style_context = root.style_context();
        let fill = style_context.lookup_color("theme_bg_color").unwrap();

        let stroke = style_context
            .lookup_color("theme_selected_bg_color")
            .unwrap();

        let warn = style_context.lookup_color("warning_color").unwrap();

        Self { fill, stroke, warn }
    }
}

pub struct FanEdit {
    profile_name: Option<String>,
    profile: Vec<FanProfilePoint>,
    drawing_handler: DrawHandler,
    drawn_points: Vec<(f64, f64)>,
    colors: Colors,
    selection: Option<usize>,
    active_drag_info: Option<(usize, f64, f64)>,
    drag_into_danger_zone: bool,
    visible: bool,
    last_override_event: Option<SourceId>,
}

#[derive(Debug)]
pub enum FanEditInput {
    Load(String),
    DragStart((f64, f64)),
    DragUpdate((f64, f64)),
    DragEnd((f64, f64)),
    #[doc(hidden)]
    Cancel,
    #[doc(hidden)]
    Update,
    #[doc(hidden)]
    Apply,
}

#[component(pub)]
impl Component for FanEdit {
    type CommandOutput = Option<Vec<FanProfilePoint>>;
    type Init = ();
    type Input = FanEditInput;
    type Output = ();

    view! {
        #[template]
        dialog = templates::DialogWindow {
            #[watch]
            set_visible: model.visible,

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                // This must be moved in the next libadwaita release due to stylesheet changes
                add_css_class: "response-area",

                gtk::Label {
                    add_css_class: "title-4",
                    set_margin_all: 12,
                    #[watch]
                    set_label: &format!("Edit fan profile '{}'", model.profile_name.as_deref().unwrap_or_default()),
                },

                gtk::Overlay {
                    #[local_ref]
                    drawing_area -> gtk::DrawingArea {
                        set_margin_all: 12,

                        #[watch]
                        set_cursor: if model.active_drag_info.is_some() {
                                gdk::Cursor::from_name("grab", None)
                            } else {
                                gdk::Cursor::from_name("pointer", None)
                            }.as_ref(),
                        set_vexpand: true,
                        set_hexpand: true,
                        add_controller = &gtk::GestureDrag {
                            connect_drag_begin[sender] => move |_, x, y| {
                                sender.input(FanEditInput::DragStart((x, y)));
                            },
                            connect_drag_update[sender] => move |_, x, y| {
                                sender.input(FanEditInput::DragUpdate((x, y)));
                            },
                            connect_drag_end[sender] => move |_, x, y| {
                                sender.input(FanEditInput::DragEnd((x, y)));
                            },
                        },
                        connect_resize[sender] => move |_, _, _| {
                            sender.input(FanEditInput::Update);
                        },
                        connect_realize => FanEditInput::Update,
                    },
                    add_overlay = &gtk::Box {
                        set_halign: gtk::Align::End,
                        set_valign: gtk::Align::End,
                        set_margin_bottom: 30,
                        set_margin_end: 60,

                        if let Some((idx, _, _)) = &model.active_drag_info {
                            gtk::Box {
                                set_orientation: gtk::Orientation::Vertical,
                                set_spacing: 6,

                                gtk::Label {
                                    set_halign: gtk::Align::Start,
                                    #[watch]
                                    set_label: &format!("{}°C", model.profile[*idx].temp),
                                },
                                gtk::Label {
                                    set_halign: gtk::Align::Start,
                                    #[watch]
                                    set_label: &format!("{}%", model.profile[*idx].fan),
                                }
                            }
                        } else {
                            gtk::Box {}
                        }
                    }
                },
                gtk::Separator {},

                #[template]
                templates::MsgDialogButtons {
                    #[template_child]
                    cancel_button -> gtk::Button {
                        connect_clicked => FanEditInput::Cancel,
                    },
                    #[template_child]
                    save_button -> gtk::Button {
                        connect_clicked => FanEditInput::Apply,
                    },
                }
            }
        }
    }

    fn init(
        _: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let colors = Colors::new(root);

        let model = Self {
            profile_name: None,
            profile: Vec::new(),
            drawing_handler: DrawHandler::new(),
            active_drag_info: None,
            colors,
            drag_into_danger_zone: false,
            drawn_points: Vec::new(),
            selection: None,
            visible: false,
            last_override_event: None,
        };

        let drawing_area = model.drawing_handler.drawing_area();
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, input: Self::Input, sender: ComponentSender<Self>, _root: &Self::Root) {
        match input {
            FanEditInput::Load(name) => {
                self.profile_name = Some(name.clone());

                let connection = tailor_connection().unwrap();
                sender.oneshot_command(async move {
                    if let Ok(profile_points) = connection.get_fan_profile(&name).await {
                        Some(profile_points)
                    } else {
                        tracing::error!("Couldn't load fan profile");
                        None
                    }
                });
            }
            FanEditInput::Apply => {
                self.visible = false;
                if let Some(name) = self.profile_name.clone() {
                    let profile = self.profile.drain(..).collect();
                    STATE.emit(TailorStateMsg::AddFanProfile(name, profile));
                }
            }
            FanEditInput::Cancel => {
                self.visible = false;
            }
            FanEditInput::Update => {
                self.update_drawn_points();
            }
            FanEditInput::DragStart((x, y)) => {
                if let Some(idx) = self.nearest_point(x, y, 15.0) {
                    self.selection = Some(idx);
                    self.active_drag_info = Some((idx, x, y));
                } else {
                    self.selection = self.add_point(x, y);
                    if let Some(idx) = self.selection {
                        self.active_drag_info = Some((idx, x, y));
                        self.update_drawn_points();
                    }
                }
            }
            FanEditInput::DragUpdate((x, y)) => {
                self.move_point(x, y);
                self.update_drawn_points();
            }
            FanEditInput::DragEnd((x, y)) => {
                self.move_point(x, y);
                self.eliminate_duplicates();
                self.update_drawn_points();

                self.drag_into_danger_zone = false;
                self.active_drag_info = None;
            }
        }
        self.draw();
    }

    fn update_cmd(
        &mut self,
        profile: Self::CommandOutput,
        _sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        self.profile = profile.unwrap_or_default();
        self.visible = true;

        self.update_drawn_points();
        self.draw();
    }
}

impl FanEdit {
    fn dimensions(&self) -> (f64, f64) {
        let width = self.drawing_handler.width() as f64;
        let height = (self.drawing_handler.height() - 5) as f64;
        (width, height)
    }

    fn temp_range(&self) -> f64 {
        self.profile.last().map(|p| p.temp as f64).unwrap_or(100.0) - 15.0
    }

    fn eliminate_duplicates(&mut self) {
        self.profile.dedup();
    }

    fn x_to_temp(x: f64, temp_range: f64, width: f64) -> f64 {
        (x * temp_range / width) + 20.0
    }

    fn temp_to_x(temp: f64, temp_range: f64, width: f64) -> f64 {
        (temp - 20.0).max(0.0) * width / temp_range
    }

    fn y_to_fan(y: f64, height: f64) -> f64 {
        (height - y) * 115.0 / height
    }

    fn fan_to_y(fan: f64, height: f64) -> f64 {
        height - fan * height / 115.0
    }

    fn add_point(&mut self, x: f64, y: f64) -> Option<usize> {
        let temp_range = self.temp_range();
        let (width, height) = self.dimensions();

        let x = Self::x_to_temp(x, temp_range, width);
        let y = Self::y_to_fan(y, height);

        let temp = x.clamp(20.0, 100.0) as u8;
        let fan = y.clamp(0.0, 100.0) as u8;

        if fan < temp.saturating_sub(50) * 2 {
            return None;
        }

        let new_profile = FanProfilePoint { temp, fan };

        Some(
            if let Some(idx) = self.profile.iter().position(|p| p.temp > temp) {
                let new_idx = idx.max(1);
                self.profile.insert(new_idx, new_profile);
                new_idx
            } else {
                self.profile.push(new_profile);
                self.profile.len() - 1
            },
        )
    }

    fn update_drawn_points(&mut self) {
        let _ = self.drawing_handler.get_context();
        let temp_range = self.temp_range();
        let (width, height) = self.dimensions();

        self.drawn_points = self
            .profile
            .iter()
            .map(|point| {
                let x = Self::temp_to_x(point.temp as f64, temp_range, width);
                let y = Self::fan_to_y(point.fan as f64, height);
                (x, y)
            })
            .collect();
    }

    fn nearest_point(&mut self, x: f64, y: f64, threshold: f64) -> Option<usize> {
        if self.profile.is_empty() {
            None
        } else {
            let mut min_idx = usize::MAX;
            let mut min_diff = f64::MAX;
            for (idx, (point_x, point_y)) in self.drawn_points.iter().enumerate() {
                let (diff_x, diff_y) = (point_x - x, point_y - y);
                let diff = diff_x.hypot(diff_y);

                if diff < min_diff {
                    min_diff = diff;
                    min_idx = idx;
                }
            }
            if min_diff < threshold {
                Some(min_idx)
            } else {
                None
            }
        }
    }

    fn draw(&mut self) {
        let ctx = self.drawing_handler.get_context();
        let (width, height) = self.dimensions();

        // Clear the image surface
        ctx.set_operator(Operator::Source);
        ctx.set_source_rgba(0.0, 0.0, 0.0, 0.0);
        ctx.rectangle(0.0, 0.0, width, height + 5.0);
        ctx.fill().unwrap();
        ctx.set_operator(Operator::Over);

        set_source_rgb(&ctx, &self.colors.fill);
        ctx.new_path();
        ctx.move_to(
            self.drawn_points.first().map(|p| p.0).unwrap_or_default(),
            height,
        );

        for (x, y) in &self.drawn_points {
            ctx.line_to(*x, *y);
        }
        let path = ctx.copy_path().unwrap();

        ctx.line_to(
            self.drawn_points.last().map(|p| p.0).unwrap_or_default(),
            height,
        );
        ctx.close_path();
        ctx.fill().unwrap();

        if self.drag_into_danger_zone {
            let temp_range = self.temp_range();

            let x = Self::temp_to_x(50.0, temp_range, width);
            let y = Self::fan_to_y(0.0, height);
            ctx.move_to(x, y);

            let last_temp = self.profile.last().map(|p| p.temp).unwrap_or_default();
            let last_fan = last_temp.saturating_sub(50) * 2;
            let x = Self::temp_to_x(last_temp as f64, temp_range, width);
            let y = Self::fan_to_y(last_fan as f64, height);
            ctx.line_to(x, y);

            set_source_rgba(&ctx, &self.colors.warn, 0.3);
            ctx.stroke().unwrap();
        }

        ctx.new_path();
        ctx.append_path(&path);
        ctx.set_line_width(2.0);
        set_source_rgb(&ctx, &self.colors.stroke);
        ctx.stroke().unwrap();

        for (idx, (x, y)) in self.drawn_points.iter().enumerate() {
            set_source_rgb(&ctx, &self.colors.stroke);
            ctx.arc(*x, *y, 5.0, 0.0, PI * 2.0);
            ctx.fill().unwrap();

            if Some(idx) == self.active_drag_info.map(|(idx, _, _)| idx) {
                if self
                    .profile
                    .iter()
                    .filter(|elem| *elem == &self.profile[idx])
                    .count()
                    == 2
                {
                    set_source_rgb(&ctx, &self.colors.warn);
                }

                ctx.arc(*x, *y, 8.0, 0.0, PI * 2.0);
                ctx.stroke().unwrap();
            }
        }
    }

    fn move_point(&mut self, x: f64, y: f64) {
        self.drag_into_danger_zone = false;
        if let Some((idx, offset_x, offset_y)) = self.active_drag_info {
            let temp_range = self.temp_range();
            let (width, height) = self.dimensions();

            // Transform from coordinates to parameter values
            let x = Self::x_to_temp(x + offset_x, temp_range, width);
            let y = Self::y_to_fan(y + offset_y, height);

            let temp = x.clamp(20.0, 100.0) as u8;
            let fan = y.clamp(0.0, 100.0) as u8;

            let mut fan = {
                let min_fan = if idx == 0 {
                    0
                } else {
                    self.profile[idx - 1].fan
                };

                let max_fan = if idx == self.profile.len() - 1 {
                    100
                } else {
                    self.profile[idx + 1].fan
                }
                .max(min_fan);

                fan.clamp(min_fan, max_fan)
            };

            let mut temp = {
                let min_temp = if idx == 0 {
                    20
                } else {
                    self.profile[idx - 1].temp
                };
                let max_temp = if idx == self.profile.len() - 1 {
                    100
                } else {
                    self.profile[idx + 1].temp
                };

                temp.clamp(min_temp, max_temp)
            };

            if let Some(profile) = self.profile.get(idx - 1) {
                if profile.fan != fan && profile.temp == temp {
                    temp += 1;
                }
            }
            if let Some(profile) = self.profile.get(idx + 1) {
                if profile.fan != fan && profile.temp == temp {
                    temp -= 1;
                }
            }

            let safety_fan_speed = temp.saturating_sub(50) * 2;
            if fan < safety_fan_speed {
                self.drag_into_danger_zone = true;
                fan = safety_fan_speed;
            }

            self.profile[idx].temp = temp;
            self.profile[idx].fan = fan;

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
                    STATE.emit(TailorStateMsg::OverwriteFanSpeed(fan));
                },
            ));
        }
    }
}

fn set_source_rgb(ctx: &gtk::cairo::Context, color: &RGBA) {
    ctx.set_source_rgb(
        color.red() as f64,
        color.green() as f64,
        color.blue() as f64,
    );
}

fn set_source_rgba(ctx: &gtk::cairo::Context, color: &RGBA, alpha: f64) {
    ctx.set_source_rgba(
        color.red() as f64,
        color.green() as f64,
        color.blue() as f64,
        alpha,
    );
}
