#[macro_export]
macro_rules! sys_fs_impls {
    (RW, $name:ident) => {
        impl $name {
            fn new() -> Result<Self, ::std::io::Error> {
                Ok(Self {
                    file: ::std::fs::OpenOptions::new()
                        .read(true)
                        .write(true)
                        .open(<Self as crate::SysFsType>::PATH)?,
                })
            }
        }

        impl crate::SysFsRead for $name {}
        impl crate::SysFsWrite for $name {}
    };
    (RO, $name:ident) => {
        impl $name {
            fn new() -> Result<Self, ::std::io::Error> {
                Ok(Self {
                    file: ::std::fs::OpenOptions::new()
                        .read(true)
                        .write(false)
                        .open(<Self as crate::SysFsType>::PATH)?,
                })
            }
        }

        impl crate::SysFsRead for $name {}
    };
    (WO, $name:ident) => {
        impl $name {
            fn new() -> Result<Self, ::std::io::Error> {
                Ok(Self {
                    file: ::std::fs::OpenOptions::new()
                        .read(false)
                        .write(true)
                        .open(<Self as crate::SysFsType>::PATH)?,
                })
            }
        }

        impl crate::SysFsWrite for $name {}
    };
}

#[macro_export]
macro_rules! sys_fs_type {
    ($path:literal, $permission:ident, $ty:ty, $name:ident, $subpath:literal) => {
        struct $name {
            file: ::std::fs::File,
        }

        impl crate::SysFsType for $name {
            type Type = $ty;
            const PATH: &'static str = concat!($path, $subpath);

            fn open_file() -> Result<Self, ::std::io::Error> {
                Self::new()
            }

            fn get_mut_file(&mut self) -> &mut ::std::fs::File {
                &mut self.file
            }
        }

        crate::sys_fs_impls!($permission, $name);
    };
    (CPU, $permission:ident, $ty:ty, $name:ident, $subpath:literal) => {
        sys_fs_type!(
            "/sys/devices/system/cpu/",
            $permission,
            $ty,
            $name,
            $subpath
        );
    };
    (KB, $permission:ident, $ty:ty, $name:ident, $subpath:literal) => {
        sys_fs_type!(
            "/sys/devices/platform/tuxedo_keyboard/",
            $permission,
            $ty,
            $name,
            $subpath
        );
    };
}
