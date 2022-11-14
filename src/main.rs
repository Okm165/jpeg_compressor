use image::io::Reader as ImageReader;
use image::{ImageBuffer, Rgb};
use notify::{watcher, DebouncedEvent, RecursiveMode, Watcher};
use pathdiff::diff_paths;
use std::env;
use std::fs::{self};
use std::io::{Error, ErrorKind};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use turbojpeg::OwnedBuf;

type CompFn = dyn Fn(&ImageBuffer<Rgb<u8>, Vec<u8>>) -> Result<OwnedBuf, Error> + Send;

fn compress(data: &ImageBuffer<Rgb<u8>, Vec<u8>>, qty: i32) -> Result<OwnedBuf, Error> {
    turbojpeg::compress_image(data, qty, turbojpeg::Subsamp::Sub2x2)
        .map_err(|err| -> Error { Error::new(ErrorKind::InvalidData, err) })
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
        let img = self.read(src_abs_path)?;
        let comp = (self.comp_fn)(&img)
            .map_err(|err| -> Error { Error::new(ErrorKind::InvalidData, err) })?;
        
        self.save(dst_abs_path, comp)?;
        Ok(())
    }

    fn read(&self, src_abs_path: &Path) -> Result<ImageBuffer<Rgb<u8>, Vec<u8>>, Error> {
        Ok(ImageReader::open(src_abs_path)?
            .decode()
            .map_err(|err| -> Error { Error::new(ErrorKind::InvalidData, err) })?
            .into_rgb8())
    }

    fn save(&self, dst_abs_path: &Path, buf: OwnedBuf) -> Result<(), Error> {
        let prefix = dst_abs_path
            .parent()
            .ok_or_else(|| Error::from(ErrorKind::InvalidData))?;
        std::fs::create_dir_all(prefix)?;
        std::fs::write(dst_abs_path, buf)?;
        Ok(())
    }
}
#[derive(Clone, Debug)]
enum CompressorAction {
    Create(PathBuf),
    Remove(PathBuf),
    Rename(PathBuf, PathBuf),
}

struct CompressorScheduler {
    comp_tasks: Vec<(JoinHandle<()>, mpsc::UnboundedSender<CompressorAction>)>,
}

impl CompressorScheduler {
    fn new() -> Self {
        CompressorScheduler {
            comp_tasks: Vec::new(),
        }
    }

    fn add_compressor(&mut self, comp: Compressor, dest_folder: PathBuf) {
        let (tx, mut rx) = mpsc::unbounded_channel::<CompressorAction>();
        let task = tokio::spawn(async move {
            while let Some(event) = rx.recv().await {
                println!("{} -- {:?}", dest_folder.to_str().unwrap(), event);
                match event {
                    CompressorAction::Create(path) => {
                        comp.comp_image(path.as_path(), dest_folder.join(path.as_path()).as_ref())
                            .unwrap_or_else(|err| println!("{}", err));
                    }
                    CompressorAction::Remove(path) => {
                        if dest_folder.join(path.as_path()).as_path().is_dir() {
                            fs::remove_dir_all::<&Path>(dest_folder.join(path.as_path()).as_ref())
                                .unwrap_or_else(|err| println!("{}", err));
                        } else {
                            fs::remove_file::<&Path>(dest_folder.join(path.as_path()).as_ref())
                                .unwrap_or_else(|err| println!("{}", err));
                        }
                    }
                    CompressorAction::Rename(from, to) => {
                        fs::rename::<&Path, &Path>(
                            dest_folder.join(from.as_path()).as_ref(),
                            dest_folder.join(to.as_path()).as_ref(),
                        )
                        .unwrap_or_else(|err| println!("{}", err));
                    }
                }
            }
        });
        self.comp_tasks.push((task, tx));
    }
    fn remove_compressor(&mut self, index: usize) -> Result<(), Error> {
        self.comp_tasks
            .get(index)
            .ok_or_else(|| Error::from(ErrorKind::InvalidData))?
            .0
            .abort();
        self.comp_tasks.remove(index);
        Ok(())
    }

    fn clear(&mut self) {
        for (handle, _) in self.comp_tasks.iter() {
            handle.abort();
        }
        self.comp_tasks.clear();
    }

    fn notify_all(&self, action: CompressorAction) -> Result<(), Error> {
        for comp in self.comp_tasks.iter() {
            comp.1
                .send(action.clone())
                .map_err(|err| -> Error { Error::new(ErrorKind::BrokenPipe, err) })?;
        }
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let mut scheduler = CompressorScheduler::new();
    scheduler.add_compressor(
        Compressor::new(comp_fn_factory(50)),
        Path::new("compressed60").to_path_buf(),
    );
    scheduler.add_compressor(
        Compressor::new(comp_fn_factory(10)),
        Path::new("compressed20").to_path_buf(),
    );

    let dir = Path::new("./temp");
    let work_dir = env::current_dir()?;

    let (tx, rx) = std::sync::mpsc::channel();
    let mut watcher = watcher(tx, Duration::from_secs(2)).unwrap();
    watcher.watch(dir, RecursiveMode::Recursive).unwrap();

    while let Ok(event) = rx.recv() {
        let action = match event {
            DebouncedEvent::Create(path) => Ok(CompressorAction::Create(
                diff_paths(path, work_dir.clone()).unwrap(),
            )),
            DebouncedEvent::Remove(path) => Ok(CompressorAction::Remove(
                diff_paths(path, work_dir.clone()).unwrap(),
            )),
            DebouncedEvent::Rename(from, to) => Ok(CompressorAction::Rename(
                diff_paths(from, work_dir.clone()).unwrap(),
                diff_paths(to, work_dir.clone()).unwrap(),
            )),
            _ => Err(Error::from(ErrorKind::Unsupported)),
        };

        if action.is_err() {
            continue;
        }
        scheduler.notify_all(action.unwrap()).unwrap();
    }

    scheduler.clear();

    Ok(())
}
