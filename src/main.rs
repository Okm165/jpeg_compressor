use std::path::Path;
use notify::{RecomendedWatcher, RecursoveMode};

fn event_fn(res: Result<notify::Event>) {
    match res {
       Ok(event) => println!("event: {:?}", event),
       Err(e) => println!("watch error: {:?}", e),
    }
}

fn main() {
    let mut watcher = notify::recommended_watcher(event_fn)?;
    watcher.watch(Path::new("./zdjecia"), RecursiveMode::Recursive).unwrap();
}
