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

## CLI Tool

```shell
% cargo build --release
% ./target/release/magdecode --help
% ./target/release/magdecode --outdir out *.MAG
```

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
