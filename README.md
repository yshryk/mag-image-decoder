# MAG image decoder

[MAG format](https://ja.wikipedia.org/?curid=115972) is also known as MAKI02, Maki-chan Graphics.

[Documentation](https://docs.rs/mag-image-decoder)

## Supported Features
* 16-color mode
* 256-color mode
* 200-line mode, non-square (rectangular) pixel aspect ratio

## Decoding

Cargo.toml:
```toml
[dependencies]
mag-image-decoder = "0.1"
```

main.rs:
```rust
use std::fs::File;
use std::io::BufReader;
use mag_image_decoder::Decoder;

let file = File::open("SAMPLE.MAG").expect("failed to open file");
let decoder = Decoder::new(BufReader::new(file)).expect("failed to decode header");
let header = decoder.info();
println!("{:?}", header);
let img = decoder.decode().expect("failed to decode image");
img.save("SAMPLE.png").expect("failed to save image");
```
