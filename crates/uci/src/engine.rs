//! Engine to GUI interface.

use std::time::Duration;

#[derive(Debug)]
pub struct Id {
    name: String,
    author: String,
}

#[derive(Debug)]
pub enum Score {
    CentiPawn(isize),
    Mate(isize),
    LowerBound,
    UpperBound,
}

#[derive(Debug)]
pub enum Info {
    Depth(usize),
    SelectiveDepth(usize),
    Time(Duration),
    Nodes(usize),
    PV(Vec<String>),
    Score(Score),
    // TODO
}

/// Messages sent by the engine.
#[derive(Debug)]
pub enum Message {
    /// Sent after the IsReady command to sync the GUI with the engine.
    ReadyOk,
    /// Update some data to the UI.
    Info(Info),
    /// Best move found after a search in the currently setup position.
    BestMove {
        p_move: String,
        ponder: Option<String>,
    }, // todo: copyprotection, registration, option
}
