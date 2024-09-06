use std::io::Write;

use bincode::config;
use flate2::{write::GzEncoder, Compression};
use hocg_fan_library::setup_library;

fn main() {
    let library = setup_library();

    // generate the library file
    let config = config::standard();
    let bin = bincode::encode_to_vec(library, config).unwrap();
    std::fs::write("hocg-fan-lib.bin", &bin).unwrap();

    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&bin).unwrap();
    std::fs::write("hocg-fan-lib.bin.gz", encoder.finish().unwrap()).unwrap();

    println!("done");
}
