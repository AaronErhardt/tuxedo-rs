mod collection;
mod controller;

#[derive(Debug)]
pub struct Collection {
    controllers: Vec<Controller>,
}

/// A type that manages all sysfs files related to
/// led color management.
#[derive(Debug)]
pub struct Controller {
    pub device_name: String,
    pub function: String,
    max_brightness: u32,
    brightness_file: tokio_uring::fs::File,
    intensities_file: Option<tokio_uring::fs::File>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ControllerMode {
    Rgb,
    Monochrome,
}
