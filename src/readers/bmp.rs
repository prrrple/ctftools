use crate::readers::{FieldDef, Section, DEFAULT_STYLE, ENUM_STYLE}; //
use ansi_term::Color::{Blue, Green, Red, White};
use ansi_term::Style;
use num_enum::TryFromPrimitive;
use std::convert::TryFrom;
use std::error::Error;
use std::fmt::Write;
use std::io::Cursor;

#[derive(Debug, TryFromPrimitive)]
#[repr(u64)]
enum Compression {
    RGB = 0,
    RLE8 = 1,
    RLE4 = 2,
    BITFIELDS = 3,
    JPEG = 4,
    PNG = 5,
    ALPHABITFIELDS = 6,
    CMYK = 11,
    CMYKRLE8 = 12,
    CMYKRLE4 = 13,
}

fn dump_mask(value: u64) -> String {
    let mut mask = String::with_capacity(8);
    for b in &value.to_le_bytes()[0..4] {
        let _ = match b {
            0 => Ok(mask.push_str("--")),
            _ => write!(mask, "{:>2x}", b),
        };
    }
    mask
}

static MSKR_STYLE: Style = Style {
    foreground: Some(Red),
    ..DEFAULT_STYLE
};

static MSKG_STYLE: Style = Style {
    foreground: Some(Green),
    ..DEFAULT_STYLE
};

static MSKB_STYLE: Style = Style {
    foreground: Some(Blue),
    ..DEFAULT_STYLE
};

static MSKA_STYLE: Style = Style {
    foreground: Some(White),
    is_dimmed: true,
    ..DEFAULT_STYLE
};

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
            FieldDef::new("Compression", 4, &ENUM_STYLE, |v| {
                format!("{:?}", Compression::try_from(v).unwrap())
            }),
            FieldDef::u32("SizeImage"),
            FieldDef::i32("XPelsPerMeter"),
            FieldDef::i32("YPelsPerMeter"),
            FieldDef::u32("ClrUsed"),
            FieldDef::u32("ClrImportant"),
        ],
    );

    println!("{}", info_section);

    let comp = info_section.get("Compression").unwrap();
    if comp.value == Compression::BITFIELDS as u64 {
        let bitfields_section = Section::from_bytes(
            &mut cursor,
            "BITFIELDS",
            vec![
                FieldDef::new("RedMask", 4, &MSKR_STYLE, dump_mask),
                FieldDef::new("GreenMask", 4, &MSKG_STYLE, dump_mask),
                FieldDef::new("BlueMask", 4, &MSKB_STYLE, dump_mask),
                FieldDef::new("AlphaMask", 4, &MSKA_STYLE, dump_mask),
            ],
        );

        println!("{}", bitfields_section);
    } else {
        println!("Skipping bitfields since the BITFIELDS flag wasn't set");
    }

    return Ok((0, bytes.len()));
}
