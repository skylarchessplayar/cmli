use crate::obj::raw::helpers::{BinField, BinFieldError};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Endian {
    Little,
    Big,
}

#[derive(Clone, Copy, Debug)]
pub struct BinContext<'a> {
    pub raw_file: &'a [u8],
    pub endian: Endian,
}

impl<'a> BinContext<'a> {
    pub fn new(file: &'a [u8]) -> Self {
        Self {
            raw_file: file,
            endian: Endian::Little,
        }
    }

    pub fn load<T: BinField<'a, ValidateParams = ()>>(&mut self) -> Result<T, BinFieldError> {
        self.load_with_params(&())
    }

    pub fn load_with_params<P: ?Sized, T: BinField<'a, ValidateParams = P>>(&mut self, params: &P) -> Result<T, BinFieldError> {
        let result = T::load(0, self);
        result.validate(params, self)?;
        Ok(result)
    }

    pub fn set_endian(&mut self, endian: Endian) {
        self.endian = endian;
    }
}
