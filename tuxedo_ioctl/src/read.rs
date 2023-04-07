use std::{fs::File, os::unix::prelude::AsRawFd};

use nix::{
    errno::Errno,
    libc::{c_ulong, ioctl},
};

use crate::{config::IOCTL_MAGIC, error::IoctlError};

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

// Read clevo
pub mod cl {
    use super::{read_int, read_string};
    use crate::config::{IOCTL_MAGIC, MAGIC_READ_CL};

    ioctl_read_int!(hw_check, IOCTL_MAGIC, 0x05);

    ioctl_read_string!(hw_interface_id, MAGIC_READ_CL, 0x00);
    ioctl_read_int!(fan_info_0, MAGIC_READ_CL, 0x10);
    ioctl_read_int!(fan_info_1, MAGIC_READ_CL, 0x11);
    ioctl_read_int!(fan_info_2, MAGIC_READ_CL, 0x12);

    ioctl_read_int!(webcam_sw, MAGIC_READ_CL, 0x13);
    ioctl_read_int!(flightmode_sw, MAGIC_READ_CL, 0x14);
    ioctl_read_int!(touchpad_sw, MAGIC_READ_CL, 0x15);
}

// Read uniwill
pub mod uw {
    use super::{read_int, read_string};
    use crate::config::{IOCTL_MAGIC, MAGIC_READ_UW};

    ioctl_read_int!(hw_check, IOCTL_MAGIC, 0x06);

    ioctl_read_string!(hw_interface_id, MAGIC_READ_UW, 0x00);
    ioctl_read_int!(model_id, MAGIC_READ_UW, 0x01);
    ioctl_read_int!(fan_speed_0, MAGIC_READ_UW, 0x10);
    ioctl_read_int!(fan_speed_1, MAGIC_READ_UW, 0x11);
    ioctl_read_int!(fan_temp_0, MAGIC_READ_UW, 0x12);
    ioctl_read_int!(fan_temp_1, MAGIC_READ_UW, 0x13);

    ioctl_read_int!(mode, MAGIC_READ_UW, 0x14);
    ioctl_read_int!(mode_enable, MAGIC_READ_UW, 0x15);
    ioctl_read_int!(fans_off_available, MAGIC_READ_UW, 0x16);
    ioctl_read_int!(fans_min_speed, MAGIC_READ_UW, 0x17);

    ioctl_read_int!(tdp_0, MAGIC_READ_UW, 0x18);
    ioctl_read_int!(tdp_1, MAGIC_READ_UW, 0x19);
    ioctl_read_int!(tdp_2, MAGIC_READ_UW, 0x1a);
    ioctl_read_int!(tdp_min_0, MAGIC_READ_UW, 0x1b);
    ioctl_read_int!(tdp_min_1, MAGIC_READ_UW, 0x1c);
    ioctl_read_int!(tdp_min_2, MAGIC_READ_UW, 0x1d);
    ioctl_read_int!(tdp_max_0, MAGIC_READ_UW, 0x1e);
    ioctl_read_int!(tdp_max_1, MAGIC_READ_UW, 0x1f);
    ioctl_read_int!(tdp_max_2, MAGIC_READ_UW, 0x20);

    ioctl_read_int!(profs_available, MAGIC_READ_UW, 0x21);
}

/*
#[cfg(test)]
mod test {
    use super::*;
    use crate::config::open_device_file;

    #[test]
    fn test_cl_read() {
        sudo::escalate_if_needed().unwrap();

        let file = open_device_file().unwrap();
        assert!(mod_version(&file).unwrap().contains("0.2"));

        assert_eq!(cl_hwcheck(&file).unwrap(), 1);
        assert_eq!(uw_hwcheck(&file).unwrap(), 0);

        assert!(cl_hw_interface_id(&file).unwrap().contains("clevo_acpi"));
        cl_faninfo1(&file).unwrap();
        cl_faninfo2(&file).unwrap();
        cl_faninfo3(&file).unwrap();

        cl_webcam_sw(&file).unwrap();
        cl_flightmode_sw(&file).unwrap();
        cl_touchpad_sw(&file).unwrap();
    }
}
 */
