use std::env;
use std::error::Error;
use std::fs::File;

use m8_files::remapper::Remapper;
use m8_files::*;

fn main() {
    match run() {
        Ok(_) => (),
        Err(err) => println!("Error: {}", err),
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    let mut ffrom = File::open("./examples/songs/TRACKEQ.m8s")?;
    let from = Song::read(&mut ffrom)?;

    let mut fto = File::open("./examples/songs/V6EMPTY.m8s")?;
    let mut to = Song::read(&mut fto)?;


    let mut remapper =
        Remapper::create(&from, &to, vec![0].iter());

    remapper.unwrap().apply(&from, &mut to);
    // dbg!(&song.eqs);

    Ok(())
}
