use mag_image_decoder::Decoder;
use std::fs::File;
use std::io::BufReader;
use simple_logger;

fn main() {
    simple_logger::init().expect("logger init error");

    let file = File::open("SAMPLE.MAG").expect("failed to open file");
    let mut decoder = Decoder::new(BufReader::new(file));
    let header = decoder.read_info().expect("failed to read header");
    dbg!(header);

    println!("ok");
}
