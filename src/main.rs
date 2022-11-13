use image::io::Reader as ImageReader;
// use notify::{watcher, DebouncedEvent, RecursiveMode, Watcher};
use std::fs::{self};
use std::io::{Error, ErrorKind, Result};
use std::path::{Path, PathBuf};

const TARGET_DIR: &str = "./compressed";

type CompFn = dyn Fn(&Path) -> Result<Vec<u8>>;

fn compress(path: &Path, qty: i32) -> Result<Vec<u8>> {
    let img = ImageReader::open(path)?
        .decode()
        .map_err(|err| -> Error { Error::new(ErrorKind::InvalidData, err) })?
        .into_rgb8();
    let jpeg_data = turbojpeg::compress_image(&img, qty, turbojpeg::Subsamp::Sub2x2)
        .map_err(|err| -> Error { Error::new(ErrorKind::InvalidData, err) })?;
    Ok(jpeg_data.to_vec())
}

fn comp_fn(qty: i32) -> Box<CompFn> {
    Box::new(move |path: &Path| -> Result<Vec<u8>> { compress(path, qty) })
}

fn target_path(target_rel_path: &str) -> PathBuf {
    Path::new(format!("{}/{}", TARGET_DIR, target_rel_path).as_str()).to_owned()
}

fn target_rel_path_conv(src_rel_path: &Path) -> &Path {
    let mut rel = src_rel_path.iter();
    rel.next();
    rel.next();
    rel.as_path()
}

fn comp_image(src_rel_path: &Path, comp_fn: &CompFn) -> Result<()> {
    let comp = comp_fn(src_rel_path)
        .map_err(|err| -> Error { Error::new(ErrorKind::InvalidData, err) })?;
    let target = target_path(
        target_rel_path_conv(src_rel_path)
            .to_str()
            .ok_or_else(|| Error::from(ErrorKind::InvalidData))?,
    );
    fs::write(target.as_path(), comp)?;
    Ok(())
}

fn comp_dir(src_rel_path: &Path, comp_fn: &CompFn) {
    if !src_rel_path.is_dir() {
        return;
    }
    fs::read_dir(src_rel_path).unwrap().for_each(|f| {
        let path = f.unwrap().path();
        println!("{}", path.as_path().to_str().unwrap());
        if path.is_file() {
            comp_image(path.as_path(), comp_fn).unwrap_or_else(|f| {
                println!("{}", f);
            });
        } else if path.is_dir() {
            fs::create_dir(target_path(
                target_rel_path_conv(path.as_path()).to_str().unwrap(),
            ))
            .unwrap_or_else(|f| {
                println!("{}", f);
            });
            comp_dir(path.as_path(), comp_fn);
        }
    })
}

fn rm_file(target_rel_path: &Path) {
    fs::remove_file(target_rel_path).unwrap();
}

fn rm_dir(target_rel_path: &Path) {
    if !target_rel_path.is_dir() {
        return;
    }
    fs::read_dir(target_rel_path).unwrap().for_each(|f| {
        let path = f.unwrap().path();
        if path.is_file() {
            rm_file(target_path(target_rel_path_conv(path.as_path()).to_str().unwrap()).as_path());
        } else if path.is_dir() {
            fs::remove_dir_all(
                target_path(target_rel_path_conv(path.as_path()).to_str().unwrap()).as_path(),
            )
            .unwrap_or_else(|f| {
                println!("{}", f);
            });
        }
    })
}

fn main() {
    comp_dir(Path::new("./images"), &comp_fn(40));
}
