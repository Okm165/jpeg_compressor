use image::io::Reader as ImageReader;
// use notify::{watcher, DebouncedEvent, RecursiveMode, Watcher};
use image::{ImageBuffer, Rgb};
use std::fs::{self};
use std::io::{Error, ErrorKind};
use std::path::Path;
use turbojpeg::{OwnedBuf};

type CompFn = dyn Fn(&ImageBuffer<Rgb<u8>, Vec<u8>>) -> Result<OwnedBuf, Error>;

fn compress(data: &ImageBuffer<Rgb<u8>, Vec<u8>>, qty: i32) -> Result<OwnedBuf, Error> {
    Ok(
        turbojpeg::compress_image(data, qty, turbojpeg::Subsamp::Sub2x2)
            .map_err(|err| -> Error { Error::new(ErrorKind::InvalidData, err) })?,
    )
}

fn comp_fn_factory(qty: i32) -> Box<CompFn> {
    Box::new(
        move |data: &ImageBuffer<Rgb<u8>, Vec<u8>>| -> Result<OwnedBuf, Error> {
            compress(data, qty)
        },
    )
}

struct Compressor {
    comp_fn: Box<CompFn>,
}

impl Compressor {
    fn new(comp_fn: Box<CompFn>) -> Self {
        Compressor { comp_fn }
    }

    fn comp_image(&self, src_abs_path: &Path, dst_abs_path: &Path) -> Result<(), Error> {
        let img = ImageReader::open(src_abs_path)?
            .decode()
            .map_err(|err| -> Error { Error::new(ErrorKind::InvalidData, err) })?
            .into_rgb8();
        let comp = (self.comp_fn)(&img)
            .map_err(|err| -> Error { Error::new(ErrorKind::InvalidData, err) })?;

        fs::write(dst_abs_path, comp)?;
        println!("{}", src_abs_path.to_str().unwrap());
        Ok(())
    }

    fn comp_dir(& mut self, src_abs_path: &Path, dst_abs_path: &Path) -> Result<(), Error> {
        if !src_abs_path.is_dir() {
            return Err(Error::from(ErrorKind::InvalidData));
        }
        fs::read_dir(src_abs_path)?.try_for_each(|f| -> Result<(), Error> {
            let path = f?.file_name();
            let src_abs_path_joined = src_abs_path.join(&path);
            let dst_abs_path_joined = dst_abs_path.join(&path);
            if src_abs_path_joined.is_file() {
                self.comp_image(&src_abs_path_joined, &dst_abs_path_joined)
                    .unwrap_or_else(|f| {
                        println!("{}", f);
                    });
            } else if src_abs_path_joined.is_dir() {
                fs::create_dir(&dst_abs_path_joined).unwrap_or_else(|f| {
                    println!("{}", f);
                });
                self.comp_dir(&src_abs_path_joined, &dst_abs_path_joined)
                    .unwrap_or_else(|f| {
                        println!("{}", f);
                    });
            }
            Ok(())
        })?;
        Ok(())
    }
}

fn main() {
    Compressor::new(comp_fn_factory(10))
        .comp_dir(Path::new("./images"), Path::new("./compressed"))
        .unwrap();
    
}
