use std::{
    fs::File,
    io::{self, Read, Seek, Write},
    str::FromStr,
};

mod cpu;
pub mod keyboard;
mod macros;

trait SysFsType: Sized {
    type Type;
    const PATH: &'static str;
    fn open_file() -> Result<Self, io::Error>;
    fn get_mut_file(&mut self) -> &mut File;
}

trait SysFsRead: SysFsType {}
trait SysFsWrite: SysFsType {}

fn sys_fs_read<S>(ty: &mut S) -> Result<S::Type, io::Error>
where
    S: SysFsType + SysFsRead,
    S::Type: FromStr,
    <S::Type as FromStr>::Err: ToString,
{
    let file = ty.get_mut_file();
    file.rewind()?;

    let mut string = String::new();
    file.read_to_string(&mut string)?;

    string
        .trim()
        .parse()
        .map_err(|err: <S::Type as FromStr>::Err| {
            io::Error::new(io::ErrorKind::InvalidData, err.to_string())
        })
}

fn sys_fs_read_separated<S>(ty: &mut S) -> Result<Vec<u8>, io::Error>
where
    S: SysFsType<Type = Vec<u8>> + SysFsRead,
{
    let file = ty.get_mut_file();
    file.rewind()?;

    let mut string = String::new();
    file.read_to_string(&mut string)?;

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

fn sys_fs_write<S>(ty: &mut S, value: &S::Type) -> Result<(), io::Error>
where
    S: SysFsType + SysFsRead,
    S::Type: ToString,
{
    let file = ty.get_mut_file();

    let string = value.to_string();
    file.write_all(string.as_bytes())?;
    Ok(())
}
