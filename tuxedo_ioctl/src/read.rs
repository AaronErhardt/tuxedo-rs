use std::{fs::File, os::unix::prelude::AsRawFd};

use nix::{
    errno::Errno,
    libc::{c_ulong, ioctl},
};

use crate::{config::IOCTL_MAGIC, config::MAGIC_READ_CL, config::MAGIC_READ_UW, error::IoctlError};

fn read_string(file: &File, request_code: c_ulong) -> Result<String, IoctlError> {
    let fd = file.as_raw_fd();
    let mut data: Vec<u8> = vec![0; 30];

    // Primitive buffer overflow detection.
    // We don't know how many bytes will be copied by the driver through ioctl,
    // so to prevent worse we want to panic and stop the program if the limits
    // of the buffer were exceeded.
    const OVERFLOW_DETECTION: [u8; 4] = [0b10011001, 0b10101010, 0b11111111, 0b10010010];
    for code in OVERFLOW_DETECTION {
        data.push(code);
    }

    let res = unsafe { ioctl(fd, request_code, data.as_mut_ptr()) };
    let _ = Errno::result(res)?;

    for code in OVERFLOW_DETECTION.iter().rev() {
        if data.pop().unwrap() != *code {
            panic!("Buffer overflow detected!");
        }
    }

    Ok(String::from_utf8(data)?)
}

fn read_int(file: &File, request_code: c_ulong) -> Result<u32, IoctlError> {
    let fd = file.as_raw_fd();

    let mut data = 0_u32;
    let data_ptr: *mut u32 = &mut data;

    // This is safe as long as the kernel driver copies the right
    // amount of bytes. We just assume it does that correctly...
    let res = unsafe { ioctl(fd, request_code, data_ptr) };
    let _ = Errno::result(res)?;
    Ok(data)
}

macro_rules! ioctl_read_string {
    ($name:ident, $id:expr, $seq:expr) => {
        pub fn $name(
            file: &::std::fs::File,
        ) -> ::std::result::Result<::std::string::String, crate::error::IoctlError> {
            let request_code = ::nix::request_code_read!(
                $id,
                $seq,
                ::std::mem::size_of::<*mut ::nix::libc::c_char>()
            );
            read_string(file, request_code)
        }
    };
}

macro_rules! ioctl_read_int {
    ($name:ident, $id:expr, $seq:expr) => {
        pub fn $name(
            file: &::std::fs::File,
        ) -> ::std::result::Result<u32, crate::error::IoctlError> {
            let request_code: ::nix::libc::c_ulong =
                ::nix::request_code_read!($id, $seq, ::std::mem::size_of::<*mut u32>());
            read_int(file, request_code)
        }
    };
}

// General
ioctl_read_string!(mod_version, IOCTL_MAGIC, 0x00);
ioctl_read_int!(hwcheck_cl, IOCTL_MAGIC, 0x05);
ioctl_read_int!(hwcheck_uw, IOCTL_MAGIC, 0x06);

// Read clevo
ioctl_read_string!(cl_hw_interface_id, MAGIC_READ_CL, 0x00);
ioctl_read_int!(cl_faninfo1, MAGIC_READ_CL, 0x10);
ioctl_read_int!(cl_faninfo2, MAGIC_READ_CL, 0x11);
ioctl_read_int!(cl_faninfo3, MAGIC_READ_CL, 0x12);

ioctl_read_int!(cl_webcam_sw, MAGIC_READ_CL, 0x13);
ioctl_read_int!(cl_flightmode_sw, MAGIC_READ_CL, 0x14);
ioctl_read_int!(cl_touchpad_sw, MAGIC_READ_CL, 0x15);

// Read uniwill
ioctl_read_int!(uw_fanspeed, MAGIC_READ_UW, 0x10);
ioctl_read_int!(uw_fanspeed2, MAGIC_READ_UW, 0x11);
ioctl_read_int!(uw_fan_temp, MAGIC_READ_UW, 0x12);
ioctl_read_int!(uw_fan_temp2, MAGIC_READ_UW, 0x13);

ioctl_read_int!(uw_mode, MAGIC_READ_UW, 0x14);
ioctl_read_int!(uw_mode_enable, MAGIC_READ_UW, 0x15);

#[cfg(test)]
mod test {
    use super::*;
    use crate::config::open_device_file;

    #[test]
    fn test_cl_read() {
        sudo::escalate_if_needed().unwrap();

        let file = open_device_file().unwrap();
        assert!(mod_version(&file).unwrap().contains("0.2"));

        assert_eq!(hwcheck_cl(&file).unwrap(), 1);
        assert_eq!(hwcheck_uw(&file).unwrap(), 0);

        assert!(cl_hw_interface_id(&file).unwrap().contains("clevo_acpi"));
        cl_faninfo1(&file).unwrap();
        cl_faninfo2(&file).unwrap();
        cl_faninfo3(&file).unwrap();

        cl_webcam_sw(&file).unwrap();
        cl_flightmode_sw(&file).unwrap();
        cl_touchpad_sw(&file).unwrap();
    }
}
