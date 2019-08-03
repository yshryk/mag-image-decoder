use mag_image_decoder::Decoder;
use std::fs::File;
use std::io::BufReader;
use simple_logger;
use log::info;

fn main() {
    simple_logger::init().expect("logger init error");

    let path = "FGALS.MAG";
//    let path = "CPU&WAKA.MAG";
//    let path = "OENM0001.MAG";
    let file = File::open(path).expect("failed to open file");
    let decoder = Decoder::new(BufReader::new(file)).expect("failed to read header");
    let header = decoder.info();
    info!("'{}', {:?}", path, header);

    let output_path = "test.png";
    info!("output_path: '{}'", output_path);
    let img = decoder.decode().expect("failed to decode");
    img.save(output_path).expect("failed to save");
    info!("ok");
}
