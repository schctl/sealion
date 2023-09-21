//! GUI to engine interface.

#[derive(Debug)]
pub enum DebugMode {
    On,
    Off,
}

#[derive(Debug)]
pub enum Position {
    Fen(String),
    StartPos,
}

/// Messages sent by the GUI.
#[derive(Debug)]
pub enum Message {
    /// Toggle the engine's debug mode on or off.
    Debug(DebugMode),
    /// Synchronize the GUI with the engine.
    IsReady,
    /// Set an engine parameter.
    SetOption { name: String, value: String },
    /// Specify that the next position will be from a new game.
    NewGame,
    /// Setup the provided position (as FEN) on the engine, and play the specified moves if any.
    Position {
        position: Position,
        moves: Option<Vec<String>>,
    },
    /// Start calculating the previously setup position.
    Go,
    /// Stop calculating on the setup position.
    Stop,

    /// Quit the engine.
    Quit, // todo: register, ponderhit
}
