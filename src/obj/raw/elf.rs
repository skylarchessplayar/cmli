use bytemuck::{Pod, Zeroable};

use super::helpers::{
    BinField, EndianFlag, Magic, Pad,
    context::BinContext,
    load_pod_bin_field,
    primitives::{U8, U16, U32, U64},
};

#[derive(Clone, Copy, Debug, Pod, Zeroable)]
#[repr(C)]
pub struct ElfIdent {
    pub magic: Magic<4>, // b"\x7fELF"
    pub class: U8,       // todo: save to context
    pub data: EndianFlag<1, 2>,
    pub version: Magic<1>, // effectively magic; can only be 1
    pub osabi: U8,
    pub abi_version: U8,
    pub pad: Pad<7>,
}

impl<'a> BinField<'a> for &'a ElfIdent {
    type ValidateParams = ();
    type Data = Self;

    fn load(off: usize, context: &BinContext<'a>) -> Self {
        load_pod_bin_field(off, context)
    }

    fn validate(
        &self,
        _: &(),
        context: &mut BinContext<'_>,
    ) -> Result<(), super::helpers::BinFieldError> {
        (&self.magic).validate(&[*b"\x7fELF"], context)?;
        (&self.class).validate(&(), context)?;
        (&self.data).validate(&(), context)?;
        (&self.version).validate(&[[1]], context)?;
        (&self.osabi).validate(&(), context)?;
        (&self.abi_version).validate(&(), context)?;
        (&self.pad).validate(&(), context)?;
        Ok(())
    }

    fn read(&self, _: &BinContext<'a>) -> Self {
        &self
    }
}

#[derive(Clone, Copy, Debug, Pod, Zeroable)]
#[repr(C)]
pub struct Elf32Header {
    pub ty: U16,
    pub machine: U16,
    pub version: U32,
    pub entry: U32,
    pub phoff: U32,
    pub shoff: U32,
    pub flags: U32,
    pub ehsize: U16,
    pub phentsize: U16,
    pub phnum: U16,
    pub shentsize: U16,
    pub shnum: U16,
    pub shstrndx: U16,
}

impl<'a> BinField<'a> for &'a Elf32Header {
    type ValidateParams = ();
    type Data = Self;

    fn load(off: usize, context: &BinContext<'a>) -> Self {
        load_pod_bin_field(off, context)
    }

    fn validate(
        &self,
        _: &(),
        context: &mut BinContext<'_>,
    ) -> Result<(), super::helpers::BinFieldError> {
        // TODO
        Ok(())
    }

    fn read(&self, _: &BinContext<'a>) -> Self {
        &self
    }
}

#[derive(Clone, Copy, Debug, Pod, Zeroable)]
#[repr(C)]
pub struct Elf64Header {
    pub ty: U16,
    pub machine: U16,
    pub version: U32,
    pub entry: U64,
    pub phoff: U64,
    pub shoff: U64,
    pub flags: U32,
    pub ehsize: U16,
    pub phentsize: U16,
    pub phnum: U16,
    pub shentsize: U16,
    pub shnum: U16,
    pub shstrndx: U16,
}

impl<'a> BinField<'a> for &'a Elf64Header {
    type ValidateParams = ();
    type Data = Self;

    fn load(off: usize, context: &BinContext<'a>) -> Self {
        load_pod_bin_field(off, context)
    }

    fn validate(
        &self,
        _: &(),
        context: &mut BinContext<'_>,
    ) -> Result<(), super::helpers::BinFieldError> {
        // TODO
        Ok(())
    }

    fn read(&self, _: &BinContext<'a>) -> Self {
        &self
    }
}

#[derive(Clone, Copy, Debug)]
pub enum ElfVariableHeader<'a> {
    Elf32(&'a Elf32Header),
    Elf64(&'a Elf64Header),
}

#[derive(Clone, Copy, Debug)]
pub struct ElfHeader<'a> {
    pub ident: &'a ElfIdent,
    pub variable_header: ElfVariableHeader<'a>,
}

impl<'a> BinField<'a> for ElfHeader<'a> {
    type ValidateParams = ();
    type Data = Self;

    fn load(off: usize, context: &BinContext<'a>) -> Self {
        let ident = <&'a ElfIdent>::load(off, context);
        let variable_header = if (&ident.class).read(context) == 2 {
            ElfVariableHeader::Elf64(<&Elf64Header>::load(16, context))
        } else {
            ElfVariableHeader::Elf32(<&Elf32Header>::load(16, context))
        };
        Self {
            ident,
            variable_header,
        }
    }

    fn validate(
        &self,
        params: &(),
        context: &mut BinContext<'_>,
    ) -> Result<(), super::helpers::BinFieldError> {
        self.ident.validate(&(), context)
        // TODO: validate the variable header
    }

    fn read(&self, context: &BinContext<'a>) -> Self {
        *self
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ElfFile<'a> {
    header: ElfHeader<'a>,
}

impl<'a> BinField<'a> for ElfFile<'a> {
    type ValidateParams = ();
    type Data = Self;

    fn load(off: usize, context: &BinContext<'a>) -> Self {
        let header = ElfHeader::load(off, context);
        Self { header }
    }

    fn validate(
        &self,
        params: &(),
        context: &mut BinContext<'_>,
    ) -> Result<(), super::helpers::BinFieldError> {
        self.header.validate(&(), context)
    }

    fn read(&self, context: &BinContext<'a>) -> Self {
        *self
    }
}

#[cfg(test)]
mod test {
    use crate::obj::raw::helpers::context::BinContext;

    use super::ElfFile;

    #[test]
    fn load_elf() {
        let test_object =
            include_bytes_align_as!(u128, "../../../tests/x86_64-unknown-linux-gnu/test.o");
        let mut context = BinContext::new(test_object);
        let file: ElfFile = context.load().unwrap();
        todo!("{file:?}");
    }
}
