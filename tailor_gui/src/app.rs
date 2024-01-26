use std::time::Duration;

use gtk::prelude::{
    ApplicationExt, ApplicationWindowExt, GtkWindowExt, ObjectExt, SettingsExt, WidgetExt,
};
use gtk::{gio, glib};
use relm4::actions::{RelmAction, RelmActionGroup};
use relm4::gtk::prelude::{BoxExt, OrientableExt};
use relm4::{
    adw, gtk, main_application, Component, ComponentController, ComponentParts, ComponentSender,
    Controller,
};
use relm4_icons::icon_name;
use tailor_api::ProfileInfo;

use crate::components::fan_list::FanList;
use crate::components::hardware_info::HardwareInfo;
use crate::components::led_list::LedList;
use crate::components::profiles::Profiles;
use crate::config::{APP_ID, PROFILE};
use crate::modals::about::AboutDialog;
use crate::state::{initialize_tailor_state, TailorStateInner, STATE};

const CONNECT_ERROR_MSG: &str = r#"Please make sure <a href="https://github.com/AaronErhardt/tuxedo-rs#tailord">tailord</a> is running correctly on your system. Tailor will connect automatically once tailord becomes available."#;

pub enum ConnectionState {
    Connecting,
    Ok,
    Error,
}

impl ConnectionState {
    fn is_ok(&self) -> bool {
        matches!(self, ConnectionState::Ok)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FullProfileInfo {
    pub name: String,
    pub data: ProfileInfo,
}

pub(super) struct App {
    about_dialog: Controller<AboutDialog>,
    connection_state: ConnectionState,
    error: Option<adw::Toast>,
}

#[derive(Debug)]
pub(super) enum Command {
    SetInitializedState { error: Option<String> },
}

#[derive(Debug)]
pub(super) enum AppMsg {
    AddError(String),
    Quit,
}

relm4::new_action_group!(pub(super) WindowActionGroup, "win");
relm4::new_stateless_action!(PreferencesAction, WindowActionGroup, "preferences");
relm4::new_stateless_action!(pub(super) ShortcutsAction, WindowActionGroup, "show-help-overlay");
relm4::new_stateless_action!(AboutAction, WindowActionGroup, "about");
relm4::new_stateless_action!(HardwareInfoAction, WindowActionGroup, "hw-info");

#[relm4::component(pub)]
impl Component for App {
    type CommandOutput = Command;
    type Init = ();
    type Input = AppMsg;
    type Output = ();

    menu! {
        primary_menu: {
            section! {
                "_Preferences" => PreferencesAction,
                "_Keyboard Shortcuts" => ShortcutsAction,
                "_Hardware information" => HardwareInfoAction,
                "_About Tailor" => AboutAction,
            }
        }
    }

    view! {
        main_window = adw::ApplicationWindow::new(&main_application()) {
            set_visible: true,
            connect_close_request[sender] => move |_| {
                sender.input(AppMsg::Quit);
                gtk::glib::Propagation::Stop
            },

            #[wrap(Some)]
            set_help_overlay: shortcuts = &gtk::Builder::from_resource(
                    "/com/github/aaronerhardt/Tailor/gtk/help-overlay.ui"
                )
                .object::<gtk::ShortcutsWindow>("help_overlay")
                .unwrap() -> gtk::ShortcutsWindow {
                    set_transient_for: Some(&main_window),
                    set_application: Some(&main_application()),
            },

            add_css_class?: if PROFILE == "Devel" {
                    Some("devel")
                } else {
                    None
                },

            adw::ToastOverlay {
                #[watch]
                add_toast?: model.error.clone(),

                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,

                    adw::HeaderBar {
                        set_centering_policy: adw::CenteringPolicy::Strict,

                        #[wrap(Some)]
                        #[transition(SlideDown)]
                        set_title_widget = if model.connection_state.is_ok() {
                            #[name = "view_title"]
                            adw::ViewSwitcherTitle {
                                set_stack: Some(&view_stack),
                                set_title: "Tailor",
                            }
                        } else {
                            gtk::Label {
                                set_label: "Tailor",
                            }
                        },

                        pack_end = &gtk::MenuButton {
                            set_icon_name: icon_name::MENU_LARGE,
                            set_menu_model: Some(&primary_menu),
                        }
                    },
                    #[transition(SlideDown)]
                    match &model.connection_state {
                        ConnectionState::Ok => {
                            gtk::Box {
                                set_orientation: gtk::Orientation::Vertical,
                                set_vexpand: true,

                                #[name = "view_stack"]
                                adw::ViewStack {
                                    set_vexpand: true,
                                    set_margin_start: 12,
                                    set_margin_end: 12,

                                    #[local_ref]
                                    add_titled[Some("profiles"), "Profiles"] = profile_widget -> gtk::ScrolledWindow {} -> {
                                        set_icon_name: Some(icon_name::SETTINGS),
                                    },
                                    #[local_ref]
                                    add_titled[Some("led"), "LED"] = led_list_widget -> gtk::ScrolledWindow {} -> {
                                        set_icon_name: Some(icon_name::COLOR),
                                    },
                                    #[local_ref]
                                    add_titled[Some("fan"), "Fan control"] = fan_list -> gtk::ScrolledWindow {} -> {
                                        set_icon_name: Some(icon_name::DATA_BAR_VERTICAL_ASCENDING_FILLED),
                                    },
                                },
                                #[name = "view_bar"]
                                adw::ViewSwitcherBar {
                                    set_stack: Some(&view_stack),
                                }
                            }
                        },
                        ConnectionState::Connecting => {
                            #[name = "loading_box"]
                            gtk::Box {
                                set_orientation: gtk::Orientation::Vertical,
                                set_spacing: 15,
                                set_valign: gtk::Align::Center,
                                set_vexpand: true,

                                gtk::Label {
                                    set_label: "Waiting for connection...",
                                    add_css_class: "title-header",
                                },
                                #[name = "spinner"]
                                gtk::Spinner {
                                    start: (),
                                }
                            }
                        },
                        ConnectionState::Error => {
                            #[name = "error_box"]
                            gtk::Box {
                                set_orientation: gtk::Orientation::Vertical,
                                set_spacing: 15,
                                set_valign: gtk::Align::Center,
                                set_vexpand: true,

                                gtk::Label {
                                    set_label: "Connection error",
                                    set_wrap: true,
                                    add_css_class: "title-header",
                                },
                                gtk::Label {
                                    set_label: CONNECT_ERROR_MSG,
                                    set_wrap: true,
                                    set_use_markup: true,
                                },
                                #[name = "err_spinner"]
                                gtk::Spinner {
                                    start: (),
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn pre_view() {
        // Update spinner
        let loading = matches!(&model.connection_state, ConnectionState::Connecting);
        spinner.set_spinning(loading);
        loading_box.set_visible(loading);

        let err = matches!(&model.connection_state, ConnectionState::Error);
        err_spinner.set_spinning(err);
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        STATE.subscribe_optional(sender.input_sender(), |state| {
            state.get().and_then(|state| {
                if state.changed(TailorStateInner::error()) {
                    state.error.clone().map(AppMsg::AddError)
                } else {
                    None
                }
            })
        });

        let about_dialog = AboutDialog::builder()
            .transient_for(&root)
            .launch(())
            .detach();

        let mut led_list = LedList::builder().launch(()).detach();
        led_list.detach_runtime();
        let led_list_widget = &**led_list.widget();

        let mut fan_list = FanList::builder().launch(()).detach();
        fan_list.detach_runtime();
        let fan_list = &**fan_list.widget();

        let mut profiles = Profiles::builder().launch(()).detach();
        profiles.detach_runtime();
        let profile_widget = &**profiles.widget();

        let model = Self {
            about_dialog,
            connection_state: ConnectionState::Connecting,
            error: None,
        };

        let widgets = view_output!();

        widgets
            .view_title
            .bind_property("title-visible", &widgets.view_bar, "reveal")
            .build();

        let shortcuts_action = {
            let shortcuts = widgets.shortcuts.clone();
            RelmAction::<ShortcutsAction>::new_stateless(move |_| {
                shortcuts.present();
            })
        };

        let hardware_action = {
            let window = widgets.main_window.clone();
            RelmAction::<HardwareInfoAction>::new_stateless(move |_| {
                HardwareInfo::builder()
                    .transient_for(&window)
                    .launch(())
                    .detach();
            })
        };

        let about_action = {
            let sender = model.about_dialog.sender().clone();
            RelmAction::<AboutAction>::new_stateless(move |_| {
                sender.send(()).unwrap();
            })
        };

        let mut actions = RelmActionGroup::<WindowActionGroup>::new();
        actions.add_action(shortcuts_action);
        actions.add_action(about_action);
        actions.add_action(hardware_action);
        actions.register_for_widget(&widgets.main_window);

        widgets.load_window_size();

        Self::initialize_connection(&sender, None);

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>, _root: &Self::Root) {
        match message {
            AppMsg::AddError(error) => {
                self.error = Some(adw::Toast::new(&error));
            }
            AppMsg::Quit => main_application().quit(),
        }
    }

    fn update_cmd(
        &mut self,
        message: Self::CommandOutput,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            Command::SetInitializedState { error } => {
                if let Some(error) = error {
                    self.connection_state = ConnectionState::Error;
                    self.error = Some(adw::Toast::new(&error));
                    Self::initialize_connection(&sender, Some(Duration::from_secs(5)));
                } else {
                    self.connection_state = ConnectionState::Ok;
                }
            }
        }
    }

    fn shutdown(&mut self, widgets: &mut Self::Widgets, _output: relm4::Sender<Self::Output>) {
        widgets.save_window_size().unwrap();
    }
}

impl AppWidgets {
    fn save_window_size(&self) -> Result<(), glib::BoolError> {
        let settings = gio::Settings::new(APP_ID);
        let (width, height) = self.main_window.default_size();

        settings.set_int("window-width", width)?;
        settings.set_int("window-height", height)?;

        settings.set_boolean("is-maximized", self.main_window.is_maximized())?;

        Ok(())
    }

    fn load_window_size(&self) {
        let settings = gio::Settings::new(APP_ID);

        let width = settings.int("window-width");
        let height = settings.int("window-height");
        let is_maximized = settings.boolean("is-maximized");

        self.main_window.set_default_size(width, height);

        if is_maximized {
            self.main_window.maximize();
        }
    }
}

impl App {
    fn initialize_connection(sender: &ComponentSender<Self>, delay: Option<Duration>) {
        sender.oneshot_command(async move {
            if let Some(delay) = delay {
                tokio::time::sleep(delay).await;
            }
            Command::SetInitializedState {
                error: initialize_tailor_state().await.err(),
            }
        });
    }
}
