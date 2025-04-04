macro_rules! impl_tuple {
    ($($ty:ident)*) => {
        #[allow(non_snake_case)]
        impl<$($ty: $crate::Encode,)*> $crate::Encode for ($($ty,)*) {
            fn encode(&self, mut _w: impl ::std::io::Write) -> ::std::result::Result<(), $crate::ProtocolError> {
                let ($($ty,)*) = self;
                $(
                    $ty.encode(&mut _w)?;
                )*
                ::std::result::Result::Ok(())
            }
        }

        impl<'a, $($ty: $crate::Decode<'a>,)*> $crate::Decode<'a> for ($($ty,)*) {
            fn decode(_r: &mut &'a [u8]) -> ::std::result::Result<Self, $crate::ProtocolError> {
                ::std::result::Result::Ok(($($ty::decode(_r)?,)*))
            }
        }
    }
}

impl_tuple!();
impl_tuple!(A);
impl_tuple!(A B);
impl_tuple!(A B C);
impl_tuple!(A B C D);
impl_tuple!(A B C D E);
impl_tuple!(A B C D E F);
impl_tuple!(A B C D E F G);
impl_tuple!(A B C D E F G H);
impl_tuple!(A B C D E F G H I);
impl_tuple!(A B C D E F G H I J);
impl_tuple!(A B C D E F G H I J K);
impl_tuple!(A B C D E F G H I J K L);
