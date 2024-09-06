use std::sync::{Arc, OnceLock};

use crate::cards::*;
use bincode::config;
use flate2::read::GzDecoder;

pub fn test_library() -> &'static Arc<GlobalLibrary> {
    static TEST_LIBRARY: OnceLock<Arc<GlobalLibrary>> = OnceLock::new();
    TEST_LIBRARY.get_or_init(|| {
        // load from file
        let mut decoder = GzDecoder::new(&include_bytes!("../../hocg-fan-lib.bin.gz")[..]);
        let config = config::standard();
        let library = bincode::decode_from_std_read(&mut decoder, config).unwrap();

        Arc::new(library)
    })
}
