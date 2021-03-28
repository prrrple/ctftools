use ansi_term::Colour::{Blue, Cyan, Green, Yellow};
use ansi_term::Style;
use std;
use std::convert::TryInto;
use std::error::Error;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::io::{Read, Seek};
use std::str;

pub const ALL: &[&str] = &["bmp"];

mod bmp;

struct Field {
    data: Box<[u8]>,
    value: u64,
    def: FieldDef,
}

pub struct FieldDef {
    size: usize,
    name: &'static str,
    style: &'static ansi_term::Style,
    dump: fn(u64) -> String,
}

fn dump_signed(value: u64) -> String {
    i64::from_ne_bytes(value.to_ne_bytes()).to_string()
}

fn dump_unsigned(value: u64) -> String {
    value.to_string()
}

fn dump_chars(value: u64) -> String {
    String::from_utf8(
        Vec::from(value.to_le_bytes())
            .into_iter()
            .take_while(|b| *b > 0)
            .collect(),
    )
    .unwrap_or(String::from("?"))
}

pub static DEFAULT_STYLE: Style = Style {
    foreground: None,
    background: None,
    is_bold: false,
    is_dimmed: false,
    is_italic: false,
    is_underline: false,
    is_blink: false,
    is_reverse: false,
    is_hidden: false,
    is_strikethrough: false,
};
pub static ENUM_STYLE: Style = Style {
    foreground: Some(Blue),
    ..DEFAULT_STYLE
};
pub static CHARS_STYLE: Style = Style {
    foreground: Some(Yellow),
    ..DEFAULT_STYLE
};
pub static INT_STYLE: Style = Style {
    foreground: Some(Cyan),
    ..DEFAULT_STYLE
};

impl FieldDef {
    fn new(
        name: &'static str,
        size: usize,
        style: &'static Style,
        dump: fn(u64) -> String,
    ) -> Self {
        FieldDef {
            size,
            name,
            style,
            dump,
        }
    }

    fn chars(name: &'static str, size: usize) -> Self {
        Self::new(name, size, &CHARS_STYLE, dump_chars)
    }

    fn signed(name: &'static str, size: usize) -> Self {
        Self::new(name, size, &INT_STYLE, dump_signed)
    }

    fn unsigned(name: &'static str, size: usize) -> Self {
        Self::new(name, size, &INT_STYLE, dump_unsigned)
    }

    pub fn u64(name: &'static str) -> Self {
        Self::unsigned(name, 8)
    }
    pub fn u32(name: &'static str) -> Self {
        Self::unsigned(name, 4)
    }
    pub fn u16(name: &'static str) -> Self {
        Self::unsigned(name, 2)
    }
    pub fn u8(name: &'static str) -> Self {
        Self::unsigned(name, 1)
    }

    pub fn i64(name: &'static str) -> Self {
        Self::signed(name, 8)
    }
    pub fn i32(name: &'static str) -> Self {
        Self::signed(name, 4)
    }
    pub fn i16(name: &'static str) -> Self {
        Self::signed(name, 2)
    }
    pub fn i8(name: &'static str) -> Self {
        Self::signed(name, 1)
    }
}

impl Display for Field {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let value = self
            .def
            .style
            .paint(format!("{:<12}", (self.def.dump)(self.value)));
        let _ = write!(f, "{:<22}", value);

        for b in self.data.iter() {
            let _ = write!(f, " {:02x}", b);
        }

        Ok(())
    }
}

struct Section {
    name: &'static str,
    fields: Vec<Field>,
}

impl Section {
    fn from_bytes<R: Read + Seek>(
        reader: &mut R,
        name: &'static str,
        proto_fields: Vec<FieldDef>,
    ) -> Self {
        let mut fields = Vec::with_capacity(proto_fields.len());
        for def in proto_fields {
            let mut data = vec![0 as u8; def.size].into_boxed_slice();
            let _ = reader.read(&mut data).unwrap();
            let size = data.len();

            let value = match size {
                1 => u64::from(u8::from_le_bytes(data[0..1].try_into().unwrap())),
                2 => u64::from(u16::from_le_bytes(data[0..2].try_into().unwrap())),
                4 => u64::from(u32::from_le_bytes(data[0..4].try_into().unwrap())),
                _ => u64::from_le_bytes(data[0..8].try_into().unwrap()),
            };

            fields.push(Field { def, data, value });
        }

        Section { name, fields }
    }

    fn get(&self, name: &str) -> Option<&Field> {
        for field in &self.fields {
            if field.def.name == name {
                return Some(&field);
            }
        }
        None
    }
}

impl Display for Section {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let _ = writeln!(f, "  {}:", Green.paint(self.name));
        for field in &self.fields {
            let _ = writeln!(f, "{}+ {:<16}: {}", "  ", field.def.name, field);
        }
        Ok(())
    }
}

type Reader = fn(&Vec<u8>) -> Result<(usize, usize), Box<dyn Error>>;

pub fn get_reader(reader: &str) -> Option<Reader> {
    match reader {
        "bmp" => Some(bmp::read),
        _ => None,
    }
}
