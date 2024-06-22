use std::io;
use std::path::Path;

use tokio_uring::fs;

pub(crate) async fn rw_file<P>(path: P) -> Result<fs::File, io::Error>
where
    P: AsRef<Path>,
{
    fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)
        .await
}

pub(crate) async fn r_file<P>(path: P) -> Result<fs::File, io::Error>
where
    P: AsRef<Path>,
{
    fs::File::open(path).await
}

pub(crate) async fn read_path_to_string<P>(path: P) -> Result<String, io::Error>
where
    P: AsRef<Path>,
{
    let mut file = r_file(path).await?;
    read_to_string(&mut file).await
}

pub(crate) async fn read_to_string(file: &mut fs::File) -> Result<String, io::Error> {
    let buffer = Vec::with_capacity(256);
    let (res, buffer) = file.read_at(buffer, 0).await;
    res?;
    String::from_utf8(buffer).map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))
}

pub(crate) async fn read_path_to_int_list<P>(path: P) -> Result<Vec<u32>, io::Error>
where
    P: AsRef<Path>,
{
    let mut file = r_file(path).await?;
    read_int_list(&mut file).await
}

pub(crate) async fn read_int_list(file: &mut fs::File) -> Result<Vec<u32>, io::Error> {
    let content = read_to_string(file).await?;

    let mut output = Vec::new();
    for value in content.split(' ') {
        let value = value.trim();
        let value: u32 = value
            .parse()
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
        output.push(value);
    }

    if output.is_empty() {
        Err(io::Error::new(io::ErrorKind::InvalidData, "Empty file"))
    } else {
        Ok(output)
    }
}

pub(crate) async fn read_to_string_list(file: &mut fs::File) -> Result<Vec<String>, io::Error> {
    let output = read_to_string(file).await?;
    Ok(output
        .split(' ')
        .map(|s| s.trim().to_owned())
        .collect::<Vec<String>>())
}

async fn write_buffer<V>(file: &mut fs::File, value: V) -> Result<(), io::Error>
where
    V: tokio_uring::buf::IoBuf,
{
    file.write_at(value, 0).await.0?;
    Ok(())
}

pub(crate) async fn write_string(file: &mut fs::File, string: String) -> Result<(), io::Error> {
    write_buffer(file, string.into_bytes()).await
}

pub(crate) async fn write_int(file: &mut fs::File, int: u32) -> Result<(), io::Error> {
    write_string(file, format!("{}", int)).await
}
