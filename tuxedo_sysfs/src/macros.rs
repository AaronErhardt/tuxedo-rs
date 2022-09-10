#[macro_export]
macro_rules! sys_fs_impls {
    (RW, $name:ident) => {
        impl $name {
            async fn new() -> Result<Self, ::std::io::Error> {
                Ok(Self {
                    file: $crate::fs::OpenOptions::new()
                        .read(true)
                        .write(true)
                        .open(<Self as $crate::SysFsType>::PATH)
                        .await?,
                })
            }
        }

        impl $crate::SysFsRead for $name {}
        impl $crate::SysFsWrite for $name {}
    };
    (RO, $name:ident) => {
        impl $name {
            async fn new() -> Result<Self, ::std::io::Error> {
                Ok(Self {
                    file: $crate::fs::OpenOptions::new()
                        .read(true)
                        .write(false)
                        .open(<Self as $crate::SysFsType>::PATH)
                        .await?,
                })
            }
        }

        impl $crate::SysFsRead for $name {}
    };
    (WO, $name:ident) => {
        impl $name {
            async fn new() -> Result<Self, ::std::io::Error> {
                Ok(Self {
                    file: $crate::fs::OpenOptions::new()
                        .read(false)
                        .write(true)
                        .open(<Self as $crate::SysFsType>::PATH)
                        .await?,
                })
            }
        }

        impl $crate::SysFsWrite for $name {}
    };
}

#[macro_export]
macro_rules! sys_fs_type {
    ($path:literal, $permission:ident, $ty:ty, $name:ident, $subpath:literal) => {
        struct $name {
            file: $crate::fs::File,
        }

        impl $crate::SysFsType for $name {
            type Type = $ty;
            const PATH: &'static str = concat!($path, $subpath);

            fn get_file(&self) -> &$crate::fs::File {
                &self.file
            }
        }

        $crate::sys_fs_impls!($permission, $name);
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
