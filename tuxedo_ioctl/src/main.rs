mod error;
pub mod high_level;
pub mod read;
pub mod write;

const TUXEDO_IO_DEVICE_FILE: &str = "/dev/tuxedo_io";

const IOCTL_MAGIC: u8 = 0xEC;

// Clevo interface
const MAGIC_READ_CL: u8 = IOCTL_MAGIC + 1;
const MAGIC_WRITE_CL: u8 = IOCTL_MAGIC + 2;

// Uniwill interface
const MAGIC_READ_UW: u8 = IOCTL_MAGIC + 3;
const MAGIC_WRITE_UW: u8 = IOCTL_MAGIC + 4;

pub fn open_device_file() -> Result<std::fs::File, std::io::Error> {
    std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(TUXEDO_IO_DEVICE_FILE)
}

fn main() {}
