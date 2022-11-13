use notify::{watcher, DebouncedEvent, RecursiveMode, Watcher};
use std::fs::{self, read};
use std::io::Result;
use std::path::{Path, PathBuf};
use std::sync::mpsc::channel;
use std::time::Duration;

const TARGET_DIR: &str = "./compressed";

fn comp_indentity(path: &Path) -> Result<Vec<u8>> {
    fs::read(path)
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

fn comp_image(src_rel_path: &Path, comp_fn: fn(&Path) -> Result<Vec<u8>>) {
    let comp = comp_fn(src_rel_path).unwrap();
    let target = target_path(target_rel_path_conv(src_rel_path).to_str().unwrap());
    fs::write(target.as_path(), comp).unwrap();
}

fn comp_dir(src_rel_path: &Path, comp_fn: fn(&Path) -> Result<Vec<u8>>) {
    if !src_rel_path.is_dir() {
        return;
    }
    fs::read_dir(src_rel_path).unwrap().for_each(|f| -> () {
        let path = f.unwrap().path();
        println!("{}", path.as_path().to_str().unwrap());
        if path.is_file() {
            comp_image(path.as_path(), comp_fn);
        } else if path.is_dir() {
            fs::create_dir(target_path(
                target_rel_path_conv(path.as_path()).to_str().unwrap(),
            ))
            .unwrap();
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
    fs::read_dir(target_rel_path).unwrap().for_each(|f| -> () {
        let path = f.unwrap().path();
        if path.is_file() {
            rm_file(
                target_path(target_rel_path_conv(path.as_path()).to_str().unwrap()).as_path(),
            );
        } else if path.is_dir() {
            fs::remove_dir_all(
                target_path(target_rel_path_conv(path.as_path()).to_str().unwrap()).as_path(),
            )
            .unwrap();
        }
    })
}

fn main() {
    comp_dir(Path::new("./temp"), comp_indentity);
}
