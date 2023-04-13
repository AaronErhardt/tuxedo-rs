use gtk::prelude::{ButtonExt, OrientableExt, WidgetExt};
use relm4::gtk::traits::{GtkWindowExt, ListBoxRowExt};
use relm4::{adw, gtk, RelmWidgetExt, WidgetTemplate};

#[relm4::widget_template(pub)]
impl WidgetTemplate for CustomClamp {
    view! {
        gtk::ScrolledWindow {
            #[name = "clamp"]
            adw::Clamp {
                set_margin_top: 10,
                set_margin_bottom: 10,
            }
        }
    }
}

#[relm4::widget_template(pub)]
impl WidgetTemplate for MsgDialogBox {
    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,

            gtk::WindowHandle {
                #[name(title)]
                gtk::Label {
                    add_css_class: "title-2",
                    set_margin_all: 12,
                },
            }
        }
    }
}

#[relm4::widget_template(pub)]
impl WidgetTemplate for MsgDialogButtons {
    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Horizontal,
            add_css_class: "response-area",

            #[name(cancel_button)]
            gtk::Button {
                set_label: "Cancel",
                set_hexpand: true,
                #[iterate]
                add_css_class: &["flat", "destructive"],
            },
            gtk::Separator,
            #[name(save_button)]
            gtk::Button {
                set_label: "Save",
                set_hexpand: true,
                #[iterate]
                add_css_class: &["flat", "suggested"],
            },
        }
    }
}

#[relm4::widget_template(pub)]
impl WidgetTemplate for DialogWindow {
    view! {
        window = adw::Window {
            set_default_size: (600, 350),
            add_css_class: "messagedialog",
            set_modal: true,
            connect_close_request => |_| gtk::Inhibit(true),
        }
    }
}

impl AsRef<gtk::Window> for DialogWindow {
    fn as_ref(&self) -> &gtk::Window {
        self.window.as_ref()
    }
}

#[relm4::widget_template(pub)]
impl WidgetTemplate for ProfileListItem {
    view! {
        gtk::ListBoxRow {
            set_activatable: false,

            #[name = "row"]
            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_margin_all: 6,

                #[name = "image"]
                gtk::Image {
                    set_margin_all: 6,
                },

                #[name = "label"]
                gtk::Label {
                    set_margin_all: 6,
                },
                gtk::Box {
                    set_hexpand: true,
                },
            }
        }
    }
}

impl AsRef<gtk::Widget> for ProfileListItem {
    fn as_ref(&self) -> &gtk::Widget {
        (**self).as_ref()
    }
}
