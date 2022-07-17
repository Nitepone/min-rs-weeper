// Copyright 2022 nitepone <luna@night.horse>

pub type MinrsResult<T> = std::result::Result<T, MinrsError>;

#[derive(Debug)]
pub enum MinrsError {
    /// The current state of tile flags prevents this move.
    BlockedByFlag,
    /// Invalid position argument.
    InvalidPosition,
    /// Out Of Bounds position argument.
    OobPosition,
    /// Game is over and action can not be completed.
    GameOver,
    /// Invalid argument. (likely bad controller code?)
    InvalidArgument,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {}
}
