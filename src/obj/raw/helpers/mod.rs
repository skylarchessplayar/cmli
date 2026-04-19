pub mod context;

use context::BinContext;

use bytemuck::{Pod, Zeroable};

#[derive(Clone, Debug, PartialEq)]
pub struct BinFieldError {
    pub error: String,
}

pub trait BinField<'a> {
    type ValidateParams: ?Sized;

    fn load(off: usize, context: &BinContext<'a>) -> Self;
    fn validate(
        &self,
        params: &Self::ValidateParams,
        context: &mut BinContext<'_>,
    ) -> Result<(), BinFieldError>;
}

pub fn load_pod_bin_field<'a, T: Pod>(off: usize, context: &BinContext<'a>) -> &'a T {
    bytemuck::from_bytes(&context.raw_file[off..(off + size_of::<T>())])
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Pod, Zeroable)]
#[repr(transparent)]
pub struct Magic<const N: usize>(pub [u8; N]);

impl<'a, const N: usize> BinField<'a> for &'a Magic<N> {
    type ValidateParams = [[u8; N]];

    fn load(off: usize, context: &BinContext<'a>) -> Self {
        load_pod_bin_field(off, context)
    }

    fn validate(&self, params: &[[u8; N]], _: &mut BinContext<'_>) -> Result<(), BinFieldError> {
        if !params.contains(&self.0) {
            Err(BinFieldError {
                error: format!("magic {:?} is not recognized", self.0),
            })
        } else {
            Ok(())
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Pod, Zeroable)]
#[repr(transparent)]
pub struct EndianFlag<const LE: u8, const BE: u8>(pub u8);

impl<'a, const LE: u8, const BE: u8> BinField<'a> for &'a EndianFlag<LE, BE> {
    type ValidateParams = ();

    fn load(off: usize, context: &BinContext<'a>) -> Self {
        load_pod_bin_field(off, context)
    }

    fn validate(&self, _: &(), context: &mut BinContext<'_>) -> Result<(), BinFieldError> {
        if self.0 == LE {
            context.set_endian(context::Endian::Little);
            Ok(())
        } else if self.0 == BE {
            context.set_endian(context::Endian::Big);
            Ok(())
        } else {
            Err(BinFieldError {
                error: format!("endian flag has unrecognized value 0x{:2X}", self.0),
            })
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Pod, Zeroable)]
#[repr(transparent)]
pub struct Pad<const N: usize>(pub [u8; N]);

impl<'a, const N: usize> BinField<'a> for &'a Pad<N> {
    type ValidateParams = ();

    fn load(off: usize, context: &BinContext<'a>) -> Self {
        load_pod_bin_field(off, context)
    }

    fn validate(&self, _: &(), _: &mut BinContext<'_>) -> Result<(), BinFieldError> {
        // todo: possibly ensure padding sections are empty? not doing for not because depends on format
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::obj::raw::helpers::{BinFieldError, Magic, context::BinContext};

    #[test]
    fn magic() {
        let valid_magic = b"uwu\0";
        let invalid_magic = b"nooo";

        let valid: Result<&Magic<4>, _> =
            BinContext::new(valid_magic).load_with_params(&[*valid_magic] as _);
        let invalid: Result<&Magic<4>, _> =
            BinContext::new(invalid_magic).load_with_params(&[*valid_magic] as _);

        assert_eq!(valid, Ok(&Magic(*valid_magic)));
        assert_eq!(
            invalid,
            Err(BinFieldError {
                error: "magic [110, 111, 111, 111] is not recognized".into()
            })
        );
    }
}
