// heavily inspired by elf crate at https://github.com/cole14/rust-elf/
// written from scratch for use with LCCC's CMLI

// overall structure:
// * program headers (segments: actual loaded data for a compiled/linked program)
// * section headers (sections: more relevant data for an unlinked program)
// * symbols (symbols are obvious enough :3)
// * string table, used by symbols and header names

use core::fmt;
use std::{
    borrow::Cow,
    io::{self, BorrowedBuf},
};

use bytemuck::{Pod, Zeroable};

use crate::obj::{ObjectFileReader, SectionFlags, SectionReader, SectionVisitor};

// TODO: find a new home for this :3
trait IntoEndian {
    fn endian(self, endian: Endian) -> Self;
}

impl IntoEndian for u16 {
    fn endian(self, endian: Endian) -> Self {
        match endian {
            Endian::Little => self.to_le(),
            Endian::Big => self.to_be(),
        }
    }
}

impl IntoEndian for u32 {
    fn endian(self, endian: Endian) -> Self {
        match endian {
            Endian::Little => self.to_le(),
            Endian::Big => self.to_be(),
        }
    }
}

impl IntoEndian for u64 {
    fn endian(self, endian: Endian) -> Self {
        match endian {
            Endian::Little => self.to_le(),
            Endian::Big => self.to_be(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Class {
    Elf32,
    Elf64,
}

impl Class {
    fn either<T>(self, if32: T, if64: T) -> T {
        match self {
            Class::Elf32 => if32,
            Class::Elf64 => if64,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Endian {
    Little,
    Big,
}

#[derive(Clone, Copy, Debug, Pod, Zeroable)]
#[repr(C)]
struct RawElfIdent {
    magic: [u8; 4],
    class: u8,   // 32/64-bit
    data: u8,    // little/big-endian
    version: u8, // should be 1
    osabi: u8,
    abiversion: u8,
    pad: [u8; 7],
}

impl RawElfIdent {
    fn verify(&self) -> io::Result<()> {
        if self.magic != *b"\x7FELF" {
            Err(io::Error::new(io::ErrorKind::InvalidData, "invalid magic"))
        } else if self.class != 1 && self.class != 2 {
            Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid ELF class",
            ))
        } else if self.data != 1 && self.data != 2 {
            Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid ELF data (endian)",
            ))
        } else if self.version != 1 {
            Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "unknown ELF version (only know version 1)",
            ))
        } else {
            // TODO: check for recognized osabi? or maybe not care; it doesn't affect the parse
            Ok(())
        }
    }

    fn endian(&self) -> Endian {
        if self.data == 2 {
            Endian::Big
        } else {
            Endian::Little
        }
    }

    fn class(&self) -> Class {
        if self.class == 2 {
            Class::Elf64
        } else {
            Class::Elf32
        }
    }
}

#[derive(Clone, Copy, Debug, Pod, Zeroable)]
#[repr(C)]
struct RawElf32Header {
    ty: u16,
    machine: u16,
    version: u32,
    entry: u32,
    phoff: u32,
    shoff: u32,
    flags: u32,
    ehsize: u16,
    phentsize: u16,
    phnum: u16,
    shentsize: u16,
    shnum: u16,
    shstrndx: u16,
}

#[derive(Clone, Copy, Debug, Pod, Zeroable)]
#[repr(C)]
struct RawElf64Header {
    ty: u16,
    machine: u16,
    version: u32,
    entry: u64,
    phoff: u64,
    shoff: u64,
    flags: u32,
    ehsize: u16,
    phentsize: u16,
    phnum: u16,
    shentsize: u16,
    shnum: u16,
    shstrndx: u16,
}

pub struct ElfReader<R> {
    reader: R,
    phoff: u64,
    shoff: u64,
    phentsize: u16,
    shentsize: u16,
    phnum: u16,
    shnum: u16,
    endian: Endian,
    class: Class,
    raw_mode: bool,
    strtab: Option<Box<[u8]>>,
}

impl<R> fmt::Debug for ElfReader<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ElfReader")
            .field("phoff", &self.phoff)
            .field("shoff", &self.shoff)
            .field("phentsize", &self.phentsize)
            .field("shentsize", &self.shentsize)
            .field("phnum", &self.phnum)
            .field("shnum", &self.shnum)
            .field("endian", &self.endian)
            .field("class", &self.class)
            .finish()
    }
}

impl<R: io::Read + io::Seek> ElfReader<R> {
    pub fn new(mut reader: R) -> io::Result<Self> {
        let buf: [u8; 16] = reader.read_array()?;
        let ident: &RawElfIdent = bytemuck::from_bytes(&buf);
        ident.verify()?;
        let endian = ident.endian();
        let class = ident.class();
        let (phoff, shoff, phentsize, shentsize, phnum, shnum, shstrndx) = match class {
            Class::Elf32 => {
                let buf: [u8; 36] = reader.read_array()?;
                let header: &RawElf32Header = bytemuck::from_bytes(&buf);
                (
                    header.phoff.endian(endian) as u64,
                    header.shoff.endian(endian) as u64,
                    header.phentsize.endian(endian),
                    header.shentsize.endian(endian),
                    header.phnum.endian(endian),
                    header.shnum.endian(endian),
                    header.shstrndx.endian(endian),
                )
            }
            Class::Elf64 => {
                let buf: [u8; 48] = reader.read_array()?;
                let header: &RawElf64Header = bytemuck::from_bytes(&buf);
                (
                    header.phoff.endian(endian),
                    header.shoff.endian(endian),
                    header.phentsize.endian(endian),
                    header.shentsize.endian(endian),
                    header.phnum.endian(endian),
                    header.shnum.endian(endian),
                    header.shstrndx.endian(endian),
                )
            }
        };

        let mut result = Self {
            reader,
            phoff,
            shoff,
            phentsize,
            shentsize,
            phnum,
            shnum,
            endian,
            class,
            raw_mode: true,
            strtab: None,
        };

        result.strtab = (|| {
            // find the string table and read its contents
            let mut sections = result.sections();
            let mut sections_to_go = shstrndx;
            while let Some(mut section) = sections.accept_next() {
                if sections_to_go == 0 {
                    let data = section.data();
                    drop(section);
                    return Some(data);
                } else {
                    sections_to_go -= 1;
                    drop(section);
                }
            }
            None
        })();

        result.raw_mode = false;

        Ok(result)
    }
}

fn read_word<R: io::Read>(reader: &mut R, endian: Endian) -> io::Result<u32> {
    let buf: [u8; 4] = reader.read_array()?;
    Ok(u32::from_ne_bytes(buf).endian(endian))
}

fn read_xword<R: io::Read>(reader: &mut R, endian: Endian) -> io::Result<u64> {
    let buf: [u8; 8] = reader.read_array()?;
    Ok(u64::from_ne_bytes(buf).endian(endian))
}

// used for all fields that are 32-bit on elf32 and 64-bit on elf64
// todo: better name?
fn read_addr<R: io::Read>(reader: &mut R, class: Class, endian: Endian) -> io::Result<u64> {
    match class {
        Class::Elf32 => read_word(reader, endian).map(|x| x.into()),
        Class::Elf64 => read_xword(reader, endian),
    }
}

pub struct ElfSectionReader<'a, R> {
    reader: &'a mut R,
    off: u64,
    class: Class,
    endian: Endian,
    strtab: Option<&'a [u8]>,
}

impl<'a, R: io::Read + io::Seek> SectionReader for ElfSectionReader<'a, R> {
    fn name(&mut self) -> Option<Cow<'_, [u8]>> {
        self.strtab.and_then(|strtab| {
            self.reader.seek(io::SeekFrom::Start(self.off)).ok()?; // todo: report io error
            let name_off = read_word(self.reader, self.endian).ok()?;
            let str = &strtab[(name_off as usize)..];
            let end = memchr::memchr(0, str)?;
            Some(Cow::Borrowed(&str[..end]))
        })
    }

    fn flags(&mut self) -> SectionFlags {
        todo!()
    }

    fn mem_size(&mut self) -> u64 {
        // todo: return 0 for non-alloc sections
        self.reader
            .seek(io::SeekFrom::Start(self.off + self.class.either(20, 32)))
            .unwrap();
        read_addr(self.reader, self.class, self.endian).unwrap()
    }

    fn data(&mut self) -> Box<[u8]> {
        self.reader
            .seek(io::SeekFrom::Start(self.off + self.class.either(16, 24)))
            .unwrap(); // todo: report io error
        let off = read_addr(self.reader, self.class, self.endian).unwrap();
        let size = read_addr(self.reader, self.class, self.endian).unwrap(); // todo: verify section isn't SHT_NOBITS

        let mut buf = Box::new_uninit_slice(size.try_into().unwrap());
        let mut borrowed_buf = BorrowedBuf::from(&mut *buf);
        self.reader.seek(io::SeekFrom::Start(off)).unwrap();
        self.reader.read_buf_exact(borrowed_buf.unfilled()).unwrap();

        // SAFETY: the read_buf_exact call has filled all bytes by function contract
        unsafe { buf.assume_init() }
    }
}

struct ElfSectionVisitor<'a, R> {
    reader: &'a mut R,
    remaining_sections: u16,
    cur_section_off: u64,
    step: u16,
    class: Class,
    endian: Endian,
    strtab: Option<&'a [u8]>,
    raw_mode: bool,
}

impl<'a, R> ElfSectionVisitor<'a, R> {
    pub fn new(elf_reader: &'a mut ElfReader<R>) -> Self {
        Self {
            reader: &mut elf_reader.reader,
            remaining_sections: elf_reader.shnum,
            cur_section_off: elf_reader.shoff,
            step: elf_reader.shentsize,
            class: elf_reader.class,
            endian: elf_reader.endian,
            strtab: elf_reader.strtab.as_deref(),
            raw_mode: elf_reader.raw_mode,
        }
    }
}

impl<'a, R: io::Read + io::Seek> SectionVisitor for ElfSectionVisitor<'a, R> {
    fn accept_next<'b>(&'b mut self) -> Option<impl SectionReader + 'b> {
        loop {
            if self.remaining_sections == 0 {
                return None;
            }
            let off = self.cur_section_off;
            self.remaining_sections -= 1;
            self.cur_section_off += self.step as u64;

            self.reader.seek(io::SeekFrom::Start(off + 4)).unwrap(); // todo: io error
            let sh_type = read_word(self.reader, self.endian).unwrap();
            if self.raw_mode || sh_type == 1 {
                // sh_type == 1 is PROGBITS; print all sections if in raw mode
                return Some(ElfSectionReader {
                    reader: self.reader,
                    off,
                    class: self.class,
                    endian: self.endian,
                    strtab: self.strtab,
                });
            }
        }
    }
}

impl<R: io::Read + io::Seek> ObjectFileReader for ElfReader<R> {
    fn sections<'a>(&'a mut self) -> impl SectionVisitor + 'a {
        ElfSectionVisitor::new(self)
    }
}

#[cfg(test)]
mod test {
    use std::io::Cursor;

    use crate::obj::{ObjectFileReader, SectionReader, SectionVisitor, elf::ElfReader};

    #[test]
    fn read_test() {
        let data = Cursor::new(include_bytes_align_as!(
            u128,
            "../../tests/x86_64-unknown-linux-gnu/test.o"
        ));
        let mut elf_reader = ElfReader::new(data).unwrap();
        {
            let mut sections = elf_reader.sections();
            while let Some(mut section) = sections.accept_next() {
                println!("{:?}", section.name());
            }
        }
        todo!("{elf_reader:?}");
    }
}
