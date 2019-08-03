use mag_image_decoder::Decoder;
use std::fs::File;
use std::io::BufReader;
use simple_logger;

fn main() {
    simple_logger::init().expect("logger init error");

    let file = File::open("SAMPLE.MAG").expect("failed to open file");
    let decoder = Decoder::new(BufReader::new(file)).expect("failed to read header");
    let header = decoder.info();
    dbg!(header);

    decoder.decode().expect("failed to decode");

    println!("ok");
}
