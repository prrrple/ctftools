use crate::readers::{FieldDef, Section};
use std::error::Error;
use std::io::Cursor;

const COMP_ITEMS: &[&str] = &[
    "RGB",
    "RLE8",
    "RLE4",
    "BITFIELDS",
    "JPEG",
    "PNG",
    "ALPHABITFIELDS",
    "?7",
    "?8",
    "?9",
    "?10",
    "CMYK",
    "CMYKRLE8",
    "CMYKRLE4",
];

pub fn read(bytes: &Vec<u8>) -> Result<(usize, usize), Box<dyn Error>> {
    let mut cursor = Cursor::new(bytes);

    let file_section = Section::from_bytes(
        &mut cursor,
        "BITMAPFILEHEADER",
        vec![
            FieldDef::chars("Magic", 2),
            FieldDef::u32("Size"),
            FieldDef::u16("Reserved1"),
            FieldDef::u16("Reserved2"),
            FieldDef::u32("OffsetToBits"),
        ],
    );

    println!("{}", file_section);

    let info_section = Section::from_bytes(
        &mut cursor,
        "BITMAPINFOHEADER",
        vec![
            FieldDef::u32("Size"),
            FieldDef::i32("Width"),
            FieldDef::i32("Height"),
            FieldDef::u16("Planes"),
            FieldDef::u16("BitCount"),
            FieldDef::new_enum("Compression", 4, COMP_ITEMS),
            FieldDef::u32("SizeImage"),
            FieldDef::i32("XPelsPerMeter"),
            FieldDef::i32("YPelsPerMeter"),
            FieldDef::u32("ClrUsed"),
            FieldDef::u32("ClrImportant"),
        ],
    );

    println!("{}", info_section);

    return Ok((0, bytes.len()));
}
