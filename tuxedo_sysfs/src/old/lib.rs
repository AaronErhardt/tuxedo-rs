use std::{io, str::FromStr};

//mod cpu;
pub mod keyboard;
mod macros;

pub(crate) use tokio_uring::fs;

trait SysFsType: Sized {
    type Type;
    const PATH: &'static str;
    fn get_file(&self) -> &tokio_uring::fs::File;
}

trait SysFsRead: SysFsType {}
trait SysFsWrite: SysFsType {}

async fn sys_fs_read_string<S>(ty: &S) -> Result<String, io::Error>
where
    S: SysFsType + SysFsRead,
{
    let file = ty.get_file();

    let buffer = Vec::new();
    let (res, buffer) = file.read_at(buffer, 0).await;
    res?;
    String::from_utf8(buffer).map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))
}

async fn sys_fs_read<S>(ty: &S) -> Result<S::Type, io::Error>
where
    S: SysFsType + SysFsRead,
    S::Type: FromStr,
    <S::Type as FromStr>::Err: ToString,
{
    let string = sys_fs_read_string(ty).await?;

    string
        .trim()
        .parse()
        .map_err(|err: <S::Type as FromStr>::Err| {
            io::Error::new(io::ErrorKind::InvalidData, err.to_string())
        })
}

async fn sys_fs_read_separated<S>(ty: &S) -> Result<Vec<u8>, io::Error>
where
    S: SysFsType<Type = Vec<u8>> + SysFsRead,
{
    let string = sys_fs_read_string(ty).await?;

    let mut return_indexes = Vec::new();
    for value in string.trim().split(',') {
        let mut values = value.split('-');
        let first: u8 = values.next().unwrap().parse().unwrap();

        if let Some(second_value) = values.next() {
            assert_eq!(values.next(), None);
            let second: u8 = second_value.parse().unwrap();
            for value in first..=second {
                return_indexes.push(value);
            }
        } else {
            return_indexes.push(first);
        }
    }
    Ok(return_indexes)
}

async fn sys_fs_write<S>(ty: &S, value: &S::Type) -> Result<(), io::Error>
where
    S: SysFsType + SysFsWrite,
    S::Type: ToString,
{
    let file = ty.get_file();

    let string = value.to_string();
    file.write_at(string.into_bytes(), 0).await.0?;
    Ok(())
}

#[cfg(test)]
mod test {
    use std::time::Duration;

    #[test]
    fn test_io_uring() {
        sudo::escalate_if_needed().unwrap();

        tokio_uring::start(async {
            let file = tokio_uring::fs::OpenOptions::new()
                .read(true)
                .write(true)
                .open("/sys/devices/platform/tuxedo_keyboard/color_left")
                .await
                .unwrap();

            let colors = ["0x00AA00", "0x00FF00"];
            for i in 0..200 {
                file.write_at(colors[i % 2].as_bytes(), 0).await.0.unwrap();
                tokio::time::sleep(Duration::from_millis(40)).await;
            }
        });
    }
}
