use core::fmt;
use std::borrow::Cow;

use bitflags::bitflags;

use crate::intern::Symbol;

#[cfg(feature = "elf")]
pub mod elf;

// todo: move to a util module
struct LargeArray(usize);

impl fmt::Debug for LargeArray {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[... ({} bytes) ...]", self.0)
    }
}

#[derive(Clone, Debug)]
pub struct ObjectFile {
    pub sections: Vec<Section>,
}

impl ObjectFile {
    pub fn from_reader(mut reader: impl ObjectFileReader) -> Self {
        let mut sections = Vec::new();
        let mut section_visitor = reader.sections();
        while let Some(section) = section_visitor.accept_next() {
            sections.push(Section::from_reader(section));
        }
        Self { sections }
    }
}

bitflags! {
    #[derive(Clone, Copy, Debug)]
    pub struct SectionFlags: u16 {
        const WRITE = 0b0000_0001;
        const EXEC = 0b0000_0010;
    }
}

#[derive(Clone)]
pub struct Section {
    pub name: Option<Symbol>,
    pub flags: SectionFlags,
    pub mem_size: u64,
    pub data: Box<[u8]>,
}

impl Section {
    pub fn from_reader(mut reader: impl SectionReader) -> Self {
        let name = reader
            .name()
            .map(|x| Symbol::intern(String::from_utf8(x.into_owned()).unwrap()));
        let flags = reader.flags();
        let mem_size = reader.mem_size();
        let data = reader.data();

        Self {
            name,
            flags,
            mem_size,
            data,
        }
    }
}

impl fmt::Debug for Section {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Section")
            .field("name", &self.name)
            .field("flags", &self.flags)
            .field("mem_size", &self.mem_size)
            .field("data", &LargeArray(self.data.len()))
            .finish()
    }
}

pub trait SectionReader {
    fn name(&mut self) -> Option<Cow<'_, [u8]>>;
    fn flags(&mut self) -> SectionFlags;
    fn mem_size(&mut self) -> u64;
    fn data(&mut self) -> Box<[u8]>;
}

pub trait SectionVisitor {
    fn accept_next<'b>(&'b mut self) -> Option<impl SectionReader + 'b>;
}

pub trait ObjectFileReader {
    fn sections<'a>(&'a mut self) -> impl SectionVisitor + 'a;
}
