use mag_image_decoder::Decoder;
use std::fs::File;
use std::io::BufReader;
use simple_logger;

fn main() {
    simple_logger::init().expect("logger init error");

    let path = "FGALS.MAG";
//    let path = "CPU&WAKA.MAG";
//    let path = "OENM0001.MAG";
    let file = File::open(path).expect("failed to open file");
    let decoder = Decoder::new(BufReader::new(file)).expect("failed to read header");
    let header = decoder.info();
    dbg!(header);

    let img = decoder.decode().expect("failed to decode");
    img.save("test.png").expect("failed to save");

    println!("ok");
}
