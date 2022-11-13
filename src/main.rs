use image::io::Reader as ImageReader;
// use notify::{watcher, DebouncedEvent, RecursiveMode, Watcher};
use image::{ImageBuffer, Rgb, DynamicImage, GenericImage};
use turbojpeg::OwnedBuf;
use std::fs::{self};
use std::io::{Error, ErrorKind};
use std::path::Path;

type CompFn = dyn Fn(&ImageBuffer<Rgb<u8>, Vec<u8>>) -> Result<OwnedBuf, Error>;

fn compress(data: &ImageBuffer<Rgb<u8>, Vec<u8>>, qty: i32) -> Result<OwnedBuf, Error> {
    Ok(
        turbojpeg::compress_image(data, qty, turbojpeg::Subsamp::Sub2x2)
            .map_err(|err| -> Error { Error::new(ErrorKind::InvalidData, err) })?
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
            .map_err(|err| -> Error { Error::new(ErrorKind::InvalidData, err) })?.into_rgb8();
        let width = img.width();
        let height = img.height();
        let vec = img.to_vec();
        let comp = (self.comp_fn)(&img)
            .map_err(|err| -> Error { Error::new(ErrorKind::InvalidData, err) })?;
        println!("{} {}", vec.len(), comp.len());
        println!("{}", (width as f64)/(height as f64));
        std::fs::write(dst_abs_path, comp)?;
        Ok(())
    }

    fn comp_dir(&self, src_abs_path: &Path, dst_abs_path: &Path) -> Result<(), Error> {
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
    // Compressor::new(comp_fn_factory(1)).comp_image(Path::new("./images/20210813_112050.jpg"), Path::new("./compressed/20210813_112050.jpg"));
    Compressor::new(comp_fn_factory(1))
        .comp_dir(Path::new("./images"), Path::new("./compressed"))
        .unwrap();
}
