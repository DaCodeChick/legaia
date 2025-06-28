use psxutils::{CDXAError, SECTOR_SIZE, SECTOR_SYNC, Sector};

use clap::Parser;
use rayon::prelude::*;
use std::fs;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    input: String,
    #[arg(short, long, default_value_t = false)]
    dump: bool,
    #[arg(short, long, default_value_t = false)]
    quick_scan: bool,
}

fn main() -> Result<(), CDXAError<'static>> {
    let args = Args::parse();
    if args.quick_scan {
        let file_size = fs::metadata(&args.input).unwrap().len() as usize;
        let sector_count = file_size / SECTOR_SIZE;
        println!("Found {} sectors in file {}", sector_count, &args.input);
        return Ok(());
    }

    if args.dump {
        let data = fs::read(&args.input).unwrap();
        let sectors: Vec<Sector> = data
            .par_chunks(SECTOR_SIZE)
            .map(|chunk| Sector::parse(chunk).unwrap())
            .collect();
        sectors.iter().enumerate().for_each(|(i, sector)| {
            fs::write(format!("sector_{:08X}.bin", i), sector.get_data()).unwrap();
        });
        return Ok(());
    }
    Ok(())
}
