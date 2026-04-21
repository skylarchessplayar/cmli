use bytemuck::{Pod, Zeroable};

use super::{
    BinField, BinFieldError,
    context::{BinContext, Endian},
    load_pod_bin_field,
};

macro_rules! primitive_bin_field {
    ($ty:ident, $prim:ident) => {
        #[derive(Clone, Copy, Debug, Pod, Zeroable)]
        #[repr(transparent)]
        pub struct $ty($prim);

        impl<'a> BinField<'a> for &'a $ty {
            type ValidateParams = ();
            type Data = $prim;

            fn load(off: usize, context: &BinContext<'a>) -> Self {
                load_pod_bin_field(off, context)
            }

            fn validate(&self, _: &(), _: &mut BinContext<'_>) -> Result<(), BinFieldError> {
                Ok(())
            }

            fn read(&self, context: &BinContext<'a>) -> $prim {
                if context.endian == Endian::Little {
                    $prim::from_le(self.0)
                } else {
                    $prim::from_be(self.0)
                }
            }
        }
    };
}

primitive_bin_field!(U8, u8);
primitive_bin_field!(U16, u16);
primitive_bin_field!(U32, u32);
primitive_bin_field!(U64, u64);
