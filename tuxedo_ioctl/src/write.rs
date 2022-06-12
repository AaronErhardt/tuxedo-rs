use std::{fs::File, os::unix::prelude::AsRawFd};

use nix::{
    errno::Errno,
    libc::{c_ulong, ioctl},
};

use crate::{error::IoctlError, MAGIC_WRITE_CL, MAGIC_WRITE_UW};

fn write_int(file: &File, data: u32, request_code: c_ulong) -> Result<(), IoctlError> {
    let fd = file.as_raw_fd();

    let data_ptr: *const u32 = &data;
    let res = unsafe { ioctl(fd, request_code, data_ptr) };
    let _ = Errno::result(res)?;
    Ok(())
}

macro_rules! ioctl_write_int {
    ($name:ident, $id:expr, $seq:expr) => {
        pub fn $name(
            file: &::std::fs::File,
            data: u32,
        ) -> ::std::result::Result<(), crate::error::IoctlError> {
            let request_code =
                ::nix::request_code_write!($id, $seq, ::std::mem::size_of::<*mut u32>());

            write_int(file, data, request_code)
        }
    };
}

// Write clevo
ioctl_write_int!(cl_fanspeed, MAGIC_WRITE_CL, 0x10);
ioctl_write_int!(cl_fanauto, MAGIC_WRITE_CL, 0x11);

ioctl_write_int!(cl_webcam_sw, MAGIC_WRITE_CL, 0x12);
ioctl_write_int!(cl_flightmode_sw, MAGIC_WRITE_CL, 0x13);
ioctl_write_int!(cl_touchpad_sw, MAGIC_WRITE_CL, 0x14);
ioctl_write_int!(cl_perf_profile, MAGIC_WRITE_CL, 0x15);

// Write uniwill
ioctl_write_int!(uw_fanspeed, MAGIC_WRITE_UW, 0x10);
ioctl_write_int!(uw_fanspeed2, MAGIC_WRITE_UW, 0x11);
ioctl_write_int!(uw_mode, MAGIC_WRITE_UW, 0x12);
ioctl_write_int!(uw_mode_enable, MAGIC_WRITE_UW, 0x13);
