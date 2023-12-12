//! [UCI] framework for chess engines.
//!
//! [UCI]: https://www.wbec-ridderkerk.nl/html/UCIProtocol.html

use std::io::{BufRead, Write};

pub mod engine;
pub mod gui;

struct Core<Stdin: BufRead, Stdout: Write> {
    stdin: Stdin,
    stdout: Stdout,
}

impl<I: BufRead, O: Write> Core<I, O> {}
