//! MAG image decoder
//!
//! [MAG format](https://ja.wikipedia.org/?curid=115972) is also known as MAKI02, Maki-chan Graphics.
//!
//! # Examples
//! ```
//! use std::fs::File;
//! use std::io::BufReader;
//! use mag_image_decoder::Decoder;
//!
//! let file = File::open("SAMPLE.MAG").unwrap();
//! let decoder = Decoder::new(BufReader::new(file)).unwrap();
//! let header = decoder.info();
//! println!("{:?}", header);
//! let img = decoder.decode().unwrap();
//! img.save("SAMPLE.png").unwrap();
//! ```

use std::io::{Cursor, Read, Seek, SeekFrom};

use byteorder::{LittleEndian as LE, ReadBytesExt};
use encoding_rs::*;
use log::debug;

pub use crate::error::*;
use std::ops::Range;
use image::{ImageBuffer, RgbImage, Rgb, imageops, FilterType};
use bit_vec::BitVec;

pub mod error;

/// Represents metadata of an image.
#[derive(Clone, Debug, PartialEq)]
pub struct ImageInfo {
    /// The machine name (max 4 characters).
    /// e.g. PC98, PC88, ESEQ, X68K, MSX2
    pub machine_code: String,
    /// The author's name
    pub user_name: String,
    /// The author's memo
    pub memo: String,
    /// The x position
    pub x: u16,
    /// The y position
    pub y: u16,
    /// The width of the image, in pixels
    pub width: u16,
    /// The height of the image, in pixels
    pub height: u16,
    /// The number of colors, 16 or 256
    pub num_colors: u16,
    /// The rectangular pixel aspect ratio flag
    pub is_200_line_mode: bool,
}

#[derive(Copy, Clone, Debug)]
enum ColorMode { Palette16, Palette256 }

/// MAG decoder
pub struct Decoder {
    info: ImageInfo,
    header_offset: u32,
    color_mode: ColorMode,
    buf: Vec<u8>,
}

// TODO: 最初に並べ替えておく
struct Palette {
    grb_colors: Vec<u8>,
}

impl Palette {
    pub fn new(grb_colors: &[u8]) -> Palette {
        Palette { grb_colors: grb_colors.to_owned() }
    }

    pub fn rgb(&self, index: u8) -> Rgb<u8> {
        let index = index as usize * 3;
        let g = self.grb_colors[index];
        let r = self.grb_colors[index + 1];
        let b = self.grb_colors[index + 2];
        Rgb([r, g, b])
    }
}

const MAGIC_NUMBER: &[u8; 8] = b"MAKI02  ";
const TEXT_ENCODING: &str = "Shift_JIS";
const HEADER_SIZE: u32 = 32;


fn range(start: u32, size: u32) -> Range<usize> {
    start as usize..(start + size) as usize
}

fn pixel_unit(c: ColorMode) -> u16 {
    match c {
        ColorMode::Palette16 => 8,
        ColorMode::Palette256 => 4,
    }
}

fn nibble_high(b: u8) -> u8 {
    b >> 4
}

fn nibble_low(b: u8) -> u8 {
    b & 0xf
}

impl Decoder {
    /// Creates a new `Decoder` using the reader `reader`.
    pub fn new<R: Read>(mut reader: R) -> Result<Decoder> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf).unwrap();

        let encoding = Encoding::for_label(TEXT_ENCODING.as_bytes())
            .ok_or_else(|| other_err(format!("Unknown encoding; {}", TEXT_ENCODING)))?;

        if &buf[..8] != MAGIC_NUMBER {
            return Err(Error::InvalidFormat("Magic number mismatch".into()));
        }

        let machine_code = String::from_utf8(buf[8..12].to_owned()).unwrap();
        let (user_name, _, _) = encoding.decode(&buf[range(12, 19)]);
        debug!("machine_code: '{}', user_name: '{}'", machine_code, user_name);

        let memo = &buf.iter().skip(31).take_while(|&b| *b != 0x1au8)
            .cloned().collect::<Vec<u8>>();
        let header_offset = 31 + memo.len() as u32 + 1;
        debug!("header_offset: {}", header_offset);
        let mut header_buf = Cursor::new(buf[range(header_offset, HEADER_SIZE)].to_owned());
        let (memo, _, _) = encoding.decode(&memo);
        debug!("memo: '{}'", memo);

        if header_buf.read_u8()? != 0 {
            return Err(Error::InvalidFormat("header offset 0x00".into()));
        }
        header_buf.seek(SeekFrom::Current(2))?;
        let screen_mode = header_buf.read_u8()?;
        let color_mode =
            if screen_mode & 0x80 != 0 { ColorMode::Palette256 } else { ColorMode::Palette16 };
        debug!("screen_mode: {}, color_mode: {:?}", screen_mode, color_mode);

        let x = header_buf.read_u16::<LE>()?;
        let y = header_buf.read_u16::<LE>()?;
        let end_x = header_buf.read_u16::<LE>()?;
        let end_y = header_buf.read_u16::<LE>()?;
        debug!("x: {}, y: {}, end_x: {}, end_y: {}", x, y, end_x, end_y);
        let pixel_unit = pixel_unit(color_mode);

        Ok(Decoder {
            info: ImageInfo {
                machine_code,
                user_name: user_name.to_string(),
                memo: memo.to_string(),
                x,
                y,
                width: (((end_x / pixel_unit) - (x / pixel_unit)) + 1) * pixel_unit,
                height: end_y - y + 1,
                num_colors: match color_mode {
                    ColorMode::Palette16 => 16,
                    ColorMode::Palette256 => 256,
                },
                is_200_line_mode: screen_mode & 1 != 0,
            },
            header_offset,
            color_mode,
            buf,
        })
    }

    /// Gets metadata
    pub fn info(&self) -> &ImageInfo {
        &self.info
    }

    /// Decodes to RGB image buffer
    pub fn decode(&self) -> Result<RgbImage> {
        let buf = &self.buf;
        let mut header_buf = Cursor::new(buf[range(self.header_offset, HEADER_SIZE)].to_owned());
        header_buf.seek(SeekFrom::Start(12))?;

        let flag_a_offset = header_buf.read_u32::<LE>()?;
        let flag_b_offset = header_buf.read_u32::<LE>()?;
        let flag_b_size = header_buf.read_u32::<LE>()?;
        let flag_a_size = flag_b_offset - flag_a_offset;
        let pixel_offset = header_buf.read_u32::<LE>()?;
        let pixel_size = header_buf.read_u32::<LE>()?;
        debug!("flag_a_offset: {}, flag_b_offset: {}, flag_a_size: {}, flag_b_size: {}, pixel_offset: {}, pixel_size: {}",
               flag_a_offset, flag_b_offset, flag_a_size, flag_b_size, pixel_offset, pixel_size);
        assert_eq!(header_buf.position() as u32, HEADER_SIZE);

        let palette = &buf[range(self.header_offset + HEADER_SIZE, u32::from(self.info.num_colors * 3))];
        let flag_a = &buf[range(self.header_offset + flag_a_offset, flag_a_size)];
        let flag_b = &buf[range(self.header_offset + flag_b_offset, flag_b_size)];
        let pixels = &buf[range(self.header_offset + pixel_offset, pixel_size)];
//        dbg!(flag_a.len(), flag_b.len(), pixels.len());
//        dbg!(buf.len() - (self.header_offset + pixel_offset + pixel_size) as usize);

        let mut img: RgbImage = ImageBuffer::new(u32::from(self.info.width), u32::from(self.info.height));
        let pixel_unit = pixel_unit(self.color_mode);
        let num_x_units = self.info.width / pixel_unit;

        let mut flag_a_bits = BitVec::from_bytes(flag_a).into_iter();
        let mut flag_b = Cursor::new(flag_b);
        let mut pixels = Cursor::new(pixels);
        let palette = Palette::new(palette);
        let mut line_flags = vec![0u8; num_x_units as usize];
        let copy_vec = self.init_copy_vec();

        for y in 0..u32::from(self.info.height) {
            for x in 0..num_x_units as usize {
                if let Some(true) = flag_a_bits.next() {
                    line_flags[x] ^= flag_b.read_u8()?;
                }
            }

            let mut dst_x = 0;
            let mut decode_nibble = |flag: u8| {
                if flag == 0 {
                    match self.color_mode {
                        ColorMode::Palette16 => {
                            for _ in 0..2 {
                                let pixel_byte = pixels.read_u8().unwrap();
                                img.put_pixel(dst_x, y, palette.rgb(nibble_high(pixel_byte)));
                                dst_x += 1;
                                img.put_pixel(dst_x, y, palette.rgb(nibble_low(pixel_byte)));
                                dst_x += 1;
                            }
                        }
                        ColorMode::Palette256 => {
                            for _ in 0..2 {
                                let pixel_byte = pixels.read_u8().unwrap();
                                img.put_pixel(dst_x, y, palette.rgb(pixel_byte));
                                dst_x += 1;
                            }
                        }
                    }
                } else {
                    dst_x += self.copy_pixel_unit(&mut img, dst_x, y, copy_vec[flag as usize]);
                }
            };

            for x in 0..num_x_units as usize {
                let flag = line_flags[x];
                decode_nibble(nibble_high(flag));
                decode_nibble(nibble_low(flag));
            }
        }

        if self.info.is_200_line_mode {
            Ok(imageops::resize(&img, u32::from(self.info.width), u32::from(self.info.height) * 2,
                                FilterType::Nearest))
        } else {
            Ok(img)
        }
    }

    fn init_copy_vec(&self) -> Vec<(u32, u32)> {
        let x = [0, 1, 2, 4, 0, 1, 0, 1, 2, 0, 1, 2, 0, 1, 2, 0];
        let y = [0, 0, 0, 0, 1, 1, 2, 2, 2, 4, 4, 4, 8, 8, 8, 16];
        x.iter().zip(y.iter()).map(|(a, b)| (*a, *b)).collect()
    }

    fn copy_pixel_unit(&self, img: &mut RgbImage, x: u32, y: u32, copy_from: (u32, u32)) -> u32 {
        let copy_pixels = match self.color_mode {
            ColorMode::Palette16 => 4,
            ColorMode::Palette256 => 2,
        };
        let src_x = x - (copy_from.0 * copy_pixels);
        let src_y = y - copy_from.1;

        for i in 0..copy_pixels {
            img.put_pixel(x + i, y, img[(src_x + i, src_y)]);
        }

        copy_pixels
    }
}
