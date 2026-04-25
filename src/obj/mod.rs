use std::borrow::Cow;

use bitflags::bitflags;

use crate::intern::Symbol;

#[cfg(feature = "elf")]
pub mod elf;

pub struct ObjectFile {}

bitflags! {
    pub struct SectionFlags: u16 {
        const WRITE = 0b0000_0001;
        const EXEC = 0b0000_0010;
    }
}

pub struct Section {
    pub name: Symbol,
    pub flags: SectionFlags,
    pub mem_size: u64,
    pub data: Box<[u8]>,
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
