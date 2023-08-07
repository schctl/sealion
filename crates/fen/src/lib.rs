//! Fen de/serialization utilities.

use sealion_board::Position;

pub mod de;

/// Parse a position from the given fen string.
#[inline]
pub fn from_str(s: &str) -> Result<Position, nom::Err<nom::error::Error<&str>>> {
    de::parse(s).map(|r| r.1)
}
