use bytemuck::{Pod, Zeroable};

use crate::obj::raw::helpers::{
    BinField, EndianFlag, Magic, Pad, context::BinContext, load_pod_bin_field,
};

#[derive(Clone, Copy, Debug, Pod, Zeroable)]
#[repr(C)]
pub struct ElfHeader {
    pub magic: Magic<4>, // b"\x7fELF"
    pub class: u8,       // todo: save to context
    pub data: EndianFlag<1, 2>,
    pub version: Magic<1>, // effectively magic; can only be 1
    pub osabi: u8,
    pub abi_version: u8,
    pub ei_pad: Pad<7>,
}

impl<'a> BinField<'a> for &'a ElfHeader {
    type ValidateParams = ();

    fn load(off: usize, context: &BinContext<'a>) -> Self {
        load_pod_bin_field(off, context)
    }

    fn validate(
        &self,
        _: &(),
        context: &mut BinContext<'_>,
    ) -> Result<(), super::helpers::BinFieldError> {
        (&self.magic).validate(&[*b"\x7fELF"], context)?;
        // self.class has no validate step yet
        (&self.data).validate(&(), context)?;
        (&self.version).validate(&[[1]], context)?;
        // self.osabi has no validate step yet
        // self.abi_version has no validate step yet
        (&self.ei_pad).validate(&(), context)?;
        Ok(())
    }
}

pub struct ElfFile<'a> {
    header: &'a ElfHeader,
}

impl<'a> BinField<'a> for ElfFile<'a> {
    type ValidateParams = ();

    fn load(off: usize, context: &BinContext<'a>) -> Self {
        let header = <&'a ElfHeader>::load(off, context);
        Self { header }
    }

    fn validate(
        &self,
        params: &(),
        context: &mut BinContext<'_>,
    ) -> Result<(), super::helpers::BinFieldError> {
        self.header.validate(&(), context)
    }
}

#[cfg(test)]
mod test {
    use crate::obj::raw::helpers::context::BinContext;

    use super::ElfFile;

    #[test]
    fn load_elf() {
        let test_object = include_bytes!("../../../tests/x86_64-unknown-linux-gnu/test.o");
        let file: ElfFile = BinContext::new(test_object).load().unwrap();
    }
}
