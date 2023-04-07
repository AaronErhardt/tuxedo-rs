use std::{fs::File, os::unix::prelude::AsRawFd};

use nix::{
    errno::Errno,
    libc::{c_ulong, ioctl},
};

use crate::error::IoctlError;

fn write_int(file: &File, data: i32, request_code: c_ulong) -> Result<(), IoctlError> {
    let fd = file.as_raw_fd();

    let data_ptr: *const i32 = &data;
    let res = unsafe { ioctl(fd, request_code, data_ptr) };
    let _ = Errno::result(res)?;
    Ok(())
}

macro_rules! ioctl_write_int {
    ($name:ident, $id:expr, $seq:expr) => {
        pub fn $name(
            file: &::std::fs::File,
            data: i32,
        ) -> ::std::result::Result<(), crate::error::IoctlError> {
            let request_code =
                ::nix::request_code_write!($id, $seq, ::std::mem::size_of::<*mut i32>());

            write_int(file, data, request_code)
        }
    };
}

// Write clevo
pub mod cl {
    use super::write_int;
    use crate::config::MAGIC_WRITE_CL;

    ioctl_write_int!(fan_speed, MAGIC_WRITE_CL, 0x10);
    ioctl_write_int!(fan_auto, MAGIC_WRITE_CL, 0x11);

    ioctl_write_int!(webcam_sw, MAGIC_WRITE_CL, 0x12);
    // ioctl_write_int!(flightmode_sw, MAGIC_WRITE_CL, 0x13);
    // ioctl_write_int!(touchpad_sw, MAGIC_WRITE_CL, 0x14);
    ioctl_write_int!(perf_profile, MAGIC_WRITE_CL, 0x15);
}

// Write uniwill
pub mod uw {
    use super::write_int;
    use crate::config::MAGIC_WRITE_UW;

    ioctl_write_int!(fan_speed_0, MAGIC_WRITE_UW, 0x10);
    ioctl_write_int!(fan_speed_1, MAGIC_WRITE_UW, 0x11);
    // ioctl_write_int!(mode, MAGIC_WRITE_UW, 0x12);
    ioctl_write_int!(mode_enable, MAGIC_WRITE_UW, 0x13);
    ioctl_write_int!(fan_auto, MAGIC_WRITE_UW, 0x14);

    ioctl_write_int!(tdp_0, MAGIC_WRITE_UW, 0x15);
    ioctl_write_int!(tdp_1, MAGIC_WRITE_UW, 0x16);
    ioctl_write_int!(tdp_2, MAGIC_WRITE_UW, 0x17);

    ioctl_write_int!(perf_profile, MAGIC_WRITE_UW, 0x18);
}
