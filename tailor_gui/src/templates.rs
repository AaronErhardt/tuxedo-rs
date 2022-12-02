use gtk::traits::{ButtonExt, OrientableExt, WidgetExt};
use relm4::{RelmWidgetExt, WidgetTemplate};

#[relm4::widget_template(pub)]
impl WidgetTemplate for MsgDialogBox {
    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            // This must be moved in the next libadwaita release due to stylesheet changes
            add_css_class: "response-area",

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
