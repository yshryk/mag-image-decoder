use std::io::{self, Cursor, Read, Seek, SeekFrom};

use byteorder::{LittleEndian as LE, ReadBytesExt};
use encoding_rs::*;
use log::{debug, info};

use crate::error::*;

mod error;

/// Represents metadata of an image.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ImageInfo {
    pub x: u16,
    pub y: u16,
    /// The width of the image, in pixels
    pub width: u16,
    /// The height of the image, in pixels
    pub height: u16,
    /// The number of colors
    pub num_colors: u16,
    /// The pixel shape
    pub oblong_pixel: bool,
}

/// MAG decoder
pub struct Decoder<R> {
    reader: R,
}

const MAGIC_NUMBER: &[u8; 8] = b"MAKI02  ";
const TEXT_ENCODING: &str = "Shift_JIS";
const HEADER_SIZE: usize = 32;

impl<R: Read> Decoder<R> {
    /// Creates a new `Decoder` using the reader `reader`.
    pub fn new(reader: R) -> Decoder<R> {
        Decoder {
            reader: reader,
        }
    }

    /// Tries to read metadata from the image without decoding it.
    pub fn read_info(&mut self) -> Result<ImageInfo> {
        let encoding = Encoding::for_label(TEXT_ENCODING.as_bytes())
            .ok_or_else(|| other_err(format!("Unknown encoding; {}", TEXT_ENCODING)))?;

        let mut buf = Vec::new();
        self.reader.read_to_end(&mut buf).unwrap();
        if &buf[..8] != MAGIC_NUMBER {
            return Err(Error::InvalidFormat("Magic number mismatch".into()));
        }

        let machine_code = String::from_utf8(buf[8..12].to_owned()).unwrap();
        let (user_name, _, _) = encoding.decode(&buf[12..(12 + 19)]);
        debug!("machine_code: '{}', user_name: '{}'", machine_code, user_name);

        let comment = &buf.iter().skip(31).take_while(|&b| *b != 0x1au8)
            .cloned().collect::<Vec<u8>>();
        let header_offset = 31 + comment.len() + 1;
        dbg!(header_offset);
        let mut header_buf = Cursor::new(buf[header_offset..header_offset + HEADER_SIZE].to_owned());
//        header_buf.set_position((31 + comment.len() + 1) as u64);
//        dbg!(header_buf.position());
        let (comment, _, _) = encoding.decode(&comment);
        debug!("comment: '{}'", comment);


        if header_buf.read_u8()? != 0 {
            return Err(Error::InvalidFormat("header offset 0x00".into()));
        }
        header_buf.seek(SeekFrom::Current(2))?;
        let screen_mode = header_buf.read_u8()?;
        let num_colors = if screen_mode & 0x80 != 0 { 256 } else { 16 };
        dbg!(screen_mode, num_colors);
        let x = header_buf.read_u16::<LE>()?;
        let y = header_buf.read_u16::<LE>()?;
        let end_x = header_buf.read_u16::<LE>()?;
        let end_y = header_buf.read_u16::<LE>()?;
        dbg!(x, y, end_x, end_y);
        let flag_a_offset = header_buf.read_u32::<LE>()?;
        let flag_b_offset = header_buf.read_u32::<LE>()?;
        let flag_b_size = header_buf.read_u32::<LE>()?;
        let flag_a_size = flag_b_offset - flag_a_offset;
        let pixel_offset = header_buf.read_u32::<LE>()?;
        let pixel_size = header_buf.read_u32::<LE>()?;
        dbg!(flag_a_offset, flag_b_offset, flag_a_size, flag_b_size, pixel_offset, pixel_size);
        assert_eq!(header_buf.position() as usize, HEADER_SIZE);

        let start = header_offset + HEADER_SIZE;
        let palette = &buf[start..start + (num_colors as usize * 3)];

        let start = header_offset + flag_a_offset as usize;
        let flag_a = &buf[start..start + flag_a_size as usize];

        let start = header_offset + flag_b_offset as usize;
        let flag_b = &buf[start..start + flag_b_size as usize];

        let start = header_offset + pixel_offset as usize;
        let pixels = &buf[start..start + pixel_size as usize];

//        dbg!(flag_a.len(), flag_b.len(), pixels.len());
        dbg!(buf.len() - (start + pixel_size as usize));


        Ok(ImageInfo {
            x,
            y,
            width: end_x - x + 1,
            height: end_y - y + 1,
            num_colors,
            oblong_pixel: screen_mode & 1 != 0,
        })
    }
}
