use gtk::prelude::{
    ApplicationExt, ApplicationWindowExt, GtkWindowExt, ObjectExt, SettingsExt, WidgetExt,
};
use gtk::{gio, glib};
use relm4::actions::{ActionGroupName, RelmAction, RelmActionGroup};
use relm4::gtk::prelude::{BoxExt, OrientableExt};
use relm4::{
    adw, gtk, main_application, Component, ComponentController, ComponentParts, ComponentSender,
    Controller,
};
use tailor_api::ProfileInfo;

use crate::components::fan_list::FanList;
use crate::components::keyboard_edit::KeyboardEdit;
use crate::components::profiles::Profiles;
use crate::config::{APP_ID, PROFILE};
use crate::modals::about::AboutDialog;
use crate::state::initialize_tailor_state;

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
}

#[derive(Debug)]
pub(super) enum Command {
    SetInitializedState(bool),
}

#[derive(Debug)]
pub(super) enum AppMsg {
    Quit,
}

relm4::new_action_group!(pub(super) WindowActionGroup, "win");
relm4::new_stateless_action!(PreferencesAction, WindowActionGroup, "preferences");
relm4::new_stateless_action!(pub(super) ShortcutsAction, WindowActionGroup, "show-help-overlay");
relm4::new_stateless_action!(AboutAction, WindowActionGroup, "about");

#[relm4::component(pub)]
impl Component for App {
    type CommandOutput = Command;
    type Init = ();
    type Input = AppMsg;
    type Output = ();
    type Widgets = AppWidgets;

    menu! {
        primary_menu: {
            section! {
                "_Preferences" => PreferencesAction,
                "_Keyboard" => ShortcutsAction,
                "_About Tailor" => AboutAction,
            }
        }
    }

    view! {
        main_window = adw::ApplicationWindow::new(&main_application()) {
            connect_close_request[sender] => move |_| {
                sender.input(AppMsg::Quit);
                gtk::Inhibit(true)
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

            gtk::Box{
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
                        set_icon_name: "open-menu-symbolic",
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

                                #[local_ref]
                                add_titled[Some("profiles"), "Profiles"] = profile_widget -> adw::Clamp {
                                } -> {
                                    set_icon_name: Some("profile-settings"),
                                },
                                #[local]
                                add_titled[Some("keyboard"), "Keyboard"] = &keyboard_edit_widget -> adw::Clamp {
                                } -> {
                                    set_icon_name: Some("keyboard-color"),
                                },
                                #[local_ref]
                                add_titled[Some("fan"), "Fan control"] = fan_list -> adw::Clamp {
                                } -> {
                                    set_icon_name: Some("fan-speed"),
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
                                set_label: "Error",
                                add_css_class: "title-header",
                            },
                            gtk::Label {
                                set_label: "ERROR MESSAGE"
                            },
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
    }

    fn init(
        _init: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let about_dialog = AboutDialog::builder()
            .transient_for(root)
            .launch(())
            .detach();

        let mut keyboard_list = KeyboardEdit::builder().launch(()).detach();
        keyboard_list.detach_runtime();
        let keyboard_edit_widget = keyboard_list.widget();

        let keyboard_edit_widget = adw::Clamp::default();

        let mut fan_list = FanList::builder().launch(()).detach();
        fan_list.detach_runtime();
        let fan_list = fan_list.widget();

        let mut profiles = Profiles::builder().launch(()).detach();
        profiles.detach_runtime();
        let profile_widget = profiles.widget();

        let model = Self {
            about_dialog,
            connection_state: ConnectionState::Connecting,
        };

        let widgets = view_output!();

        widgets
            .view_title
            .bind_property("title-visible", &widgets.view_bar, "reveal")
            .build();

        let actions = RelmActionGroup::<WindowActionGroup>::new();

        let shortcuts_action = {
            let shortcuts = widgets.shortcuts.clone();
            RelmAction::<ShortcutsAction>::new_stateless(move |_| {
                shortcuts.present();
            })
        };

        let about_action = {
            let sender = model.about_dialog.sender().clone();
            RelmAction::<AboutAction>::new_stateless(move |_| {
                sender.send(()).unwrap();
            })
        };

        actions.add_action(&shortcuts_action);
        actions.add_action(&about_action);

        widgets
            .main_window
            .insert_action_group(WindowActionGroup::NAME, Some(&actions.into_action_group()));

        widgets.load_window_size();

        Self::initialize_connection(&sender);

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>, root: &Self::Root) {
        match message {
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
            Command::SetInitializedState(initialized) => {
                if initialized {
                    self.connection_state = ConnectionState::Ok;
                } else {
                    self.connection_state = ConnectionState::Error;
                    Self::initialize_connection(&sender);
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
    fn initialize_connection(sender: &ComponentSender<Self>) {
        sender.oneshot_command(async {
            Command::SetInitializedState(initialize_tailor_state().await.is_ok())
        });
    }
}
