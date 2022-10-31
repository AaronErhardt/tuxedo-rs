use gtk::{
    gdk::RGBA,
    gdk_pixbuf::{Colorspace, Pixbuf},
};
use tailor_api::Color;

pub fn rgba_to_color(rgba: RGBA) -> Color {
    let r = (rgba.red() * 255.0).round() as u8;
    let g = (rgba.green() * 255.0).round() as u8;
    let b = (rgba.blue() * 255.0).round() as u8;

    Color { r, g, b }
}

pub fn color_to_rgba(color: Color) -> RGBA {
    let Color { r, g, b } = color;
    let red = r as f32 / 255.0;
    let green = g as f32 / 255.0;
    let blue = b as f32 / 255.0;

    RGBA::builder().red(red).green(green).blue(blue).build()
}

pub fn new_pixbuf(color: &Color) -> Pixbuf {
    let pixbuf = Pixbuf::new(Colorspace::Rgb, false, 8, 7, 4).unwrap();
    fill_pixbuf(&pixbuf, color);
    pixbuf
}

pub fn fill_pixbuf(pixbuf: &Pixbuf, color: &Color) {
    let color: u32 = u32::from_be_bytes([color.r, color.g, color.b, 0]);
    pixbuf.fill(color);
}
