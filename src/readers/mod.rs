use crate::readers::Format::{Signed, Unsigned};
use ansi_term::Colour::{Blue, Cyan, Green, Yellow};
use std;
use std::convert::TryInto;
use std::error::Error;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::io::{Read, Seek};
use std::str;

pub const ALL: &[&str] = &["bmp"];

mod bmp;

enum Format {
    Signed,
    Unsigned,
    Chars,
    Enum,
}

struct Field {
    data: Box<[u8]>,
    value: u64,
    def: FieldDef,
}

pub struct FieldDef {
    size: usize,
    name: &'static str,
    format: Format,
    items: &'static [&'static str],
}

impl FieldDef {
    fn new(size: usize, name: &'static str, format: Format) -> Self {
        FieldDef {
            size,
            name,
            format,
            items: &[] as &[&str],
        }
    }

    fn chars(name: &'static str, size: usize) -> Self {
        Self::new(size, name, Format::Chars)
    }

    fn new_enum(name: &'static str, size: usize, items: &'static [&'static str]) -> Self {
        FieldDef {
            size,
            name,
            format: Format::Enum,
            items,
        }
    }

    pub fn u64(name: &'static str) -> Self {
        Self::new(8, name, Unsigned)
    }
    pub fn u32(name: &'static str) -> Self {
        Self::new(4, name, Unsigned)
    }
    pub fn u16(name: &'static str) -> Self {
        Self::new(2, name, Unsigned)
    }
    pub fn u8(name: &'static str) -> Self {
        Self::new(1, name, Unsigned)
    }

    pub fn i64(name: &'static str) -> Self {
        Self::new(8, name, Signed)
    }
    pub fn i32(name: &'static str) -> Self {
        Self::new(4, name, Signed)
    }
    pub fn i16(name: &'static str) -> Self {
        Self::new(2, name, Signed)
    }
    pub fn i8(name: &'static str) -> Self {
        Self::new(1, name, Signed)
    }
}

impl Display for Field {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let value = match self.def.format {
            Format::Chars => Yellow.paint(format!(
                "{:<12}",
                String::from_utf8(self.data.to_vec()).unwrap_or(String::from("?"))
            )),
            Format::Enum => Blue.paint(format!(
                "{:<12}",
                self.def
                    .items
                    .get(usize::from(self.data[0]))
                    .unwrap_or(&"?")
            )),
            _ => Cyan.paint(format!(
                "{:<12}",
                match self.def.format {
                    Format::Unsigned => self.value.to_string(),
                    _ => i64::from_ne_bytes(self.value.to_ne_bytes()).to_string(),
                }
            )),
        };
        let _ = write!(f, "{:<22}", value);
        for b in self.data.iter() {
            let _ = write!(f, " {:02x}", b);
        }

        return Ok(());
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
