use nom::error::Error as NomError;
use nom::Finish;
use paste::paste;

#[derive(Debug, thiserror::Error)]
pub enum Error<'i> {
    /// An invalid type was found while parsing an input field.
    #[error("invalid type provided at {field}, expected {expected}")]
    InvalidType {
        field: &'static str,
        expected: &'static str,
    },
    /// Raw nom error.
    #[error("nom parsing error: {0}")]
    Nom(NomError<&'i str>),
}

impl<'i> From<NomError<&'i str>> for Error<'i> {
    #[inline]
    fn from(value: NomError<&'i str>) -> Self {
        Self::Nom(value)
    }
}

/// Trait describing a value that can be deserialized from the UCI input.
pub trait Deserialize: Sized {
    /// Deserialize some segment of the UCI input.
    ///
    /// The `Ok` is in the form of `(O, I)`.
    fn deserialize(input: &str) -> Result<(Self, &str), Error>;
}

/// Base Type Implementations //////////////////////////////////

impl Deserialize for String {
    fn deserialize(input: &str) -> Result<(Self, &str), Error> {
        Ok((input.to_owned(), ""))
    }
}

macro_rules! impl_int_types {
    ($($ty:ident),*) => {
        $(
            impl Deserialize for $ty {
                fn deserialize(input: &str) -> Result<(Self, &str), Error> {
                    let (inp, out) = (nom::character::complete::$ty::<&str, NomError<&str>>)(input).finish()?;
                    Ok((out, inp))
                }
            }
        )*
    };
}

impl_int_types! {
    u8, u16, u32, u64, u128,
    i8, i16, i32, i64, i128
}
