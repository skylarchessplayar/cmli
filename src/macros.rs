#[doc(hidden)]
pub use cmli_proc_macro::rand_u64;

#[doc(hidden)]
pub use core::num::NonZero as __NonZero;

#[macro_export]
macro_rules! as_id_array {
    ($base:expr => $id_ty:ty) => {
        const {
            const fn array_as_id<T: const $crate::traits::AsId<$id_ty> + Copy, const N: usize>(
                arr: [T; N],
            ) -> [$id_ty; N] {
                let mut x = [const {
                    <$id_ty as $crate::traits::IdType>::from_raw_parts(
                        <T as $crate::traits::AsRawId>::TYPE,
                        0,
                    )
                }; N];

                let mut i = 0;
                while i < N {
                    x[i] = <$id_ty as $crate::traits::IdType>::new(arr[i]);
                    i += 1;
                }

                x
            }

            &array_as_id($base)
        }
    };
}

#[macro_export]
macro_rules! nzlit {
    ($lit:expr) => {
        const {
            let __val = $lit;

            if __val == 0 {
                panic!("Zero constant produced when non-zero value is expected");
            }

            unsafe { $crate::macros::__NonZero::new_unchecked(__val)}
        }
    }
}

#[repr(C)] // guarantee 'bytes' comes after '_align'
pub struct AlignedAs<Align, Bytes: ?Sized> {
    pub _align: [Align; 0],
    pub bytes: Bytes,
}

macro_rules! include_bytes_align_as {
    ($align_ty:ty, $path:literal) => {{
        // const block expression to encapsulate the static
        use $crate::macros::AlignedAs;

        // this assignment is made possible by CoerceUnsized
        static ALIGNED: &AlignedAs<$align_ty, [u8]> = &AlignedAs {
            _align: [],
            bytes: *include_bytes!($path),
        };

        &ALIGNED.bytes
    }};
}
