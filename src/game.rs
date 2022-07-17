// Copyright 2022 nitepone <luna@night.horse>

use crate::error::{MinrsError, MinrsResult};
use rand::Rng;
use std::collections::HashSet;

const MIN_BOARD_DIMENSION: u8 = 8;

pub struct Position {
    pub x: u8,
    pub y: u8,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum FlagState {
    Questionable,
    RedFlag,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum TileContents {
    MineCount(u8),
    Mine,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum TileState {
    Covered(Option<FlagState>),
    Uncovered(TileContents),
}

struct StdTile {
    covered: bool,
    mine: bool,
    cached_mine_count: Option<u8>, // Hmm.. Rethink this if we make evil tile types.
    flag: Option<FlagState>,
}

impl StdTile {
    fn new(mine: bool) -> StdTile {
        StdTile {
            covered: true,
            mine,
            cached_mine_count: None,
            flag: None,
        }
    }
}

trait Tile {
    fn is_covered(&self) -> bool;
    fn is_mine(&self) -> bool;
    fn get_flag(&self) -> Option<FlagState>;
    fn get_state(&self, neighbors: Vec<&dyn Tile>) -> TileState;
    fn toggle_flag(&mut self);
    fn uncover(&mut self) -> MinrsResult<bool>;
}

impl Tile for StdTile {
    fn is_covered(&self) -> bool {
        self.covered
    }

    fn is_mine(&self) -> bool {
        self.mine
    }

    fn get_flag(&self) -> Option<FlagState> {
        self.flag
    }

    fn get_state(&self, neighbors: Vec<&dyn Tile>) -> TileState {
        // covered tiles all just return with flag state
        if self.covered {
            return TileState::Covered(self.get_flag());
        }

        // if we are mine, say so!
        if self.mine {
            return TileState::Uncovered(TileContents::Mine);
        }

        // finally, we are an empty number tile.
        if let Some(count) = self.cached_mine_count {
            return TileState::Uncovered(TileContents::MineCount(count));
        }
        let count = neighbors.iter().fold(0, |count, n| {
            if n.is_mine() {
                return count + 1;
            }
            count
        });
        //self.cached_mine_count = Some(count);
        return TileState::Uncovered(TileContents::MineCount(count));
    }

    fn toggle_flag(&mut self) {
        if let Some(flag_state) = self.flag {
            match flag_state {
                FlagState::RedFlag => self.flag = Some(FlagState::Questionable),
                FlagState::Questionable => self.flag = None,
            }
        } else {
            self.flag = Some(FlagState::RedFlag);
        }
    }

    fn uncover(&mut self) -> MinrsResult<bool> {
        if self.covered == false {
            return Err(MinrsError::InvalidPosition);
        }

        self.covered = false;
        Ok(self.is_mine())
    }
}

pub trait MinrsGame {
    /// Check if the current game is in progress.
    fn game_over(&self) -> bool;
    /// Cycles the flag of a covered tile.
    ///
    /// throws InvalidPosition on uncovered tiles.
    fn cycle_flag(&mut self, position: &Position) -> MinrsResult<()>;
    /// Uncovers a tile.
    ///
    /// throws InvalidPosition on uncovered tiles.
    /// throws BlockedByFlag on red_flagged tiles.
    fn uncover_tile(&mut self, position: &Position) -> MinrsResult<()>;
    /// Uncovers all neighbors from an uncovered tile.
    ///
    /// throws InvalidPosition on covered tiles.
    /// throws BlockedByFlag iff there is not an equal flags to mine ratio for
    ///        the mines counted by the target tile. (Else, this move is self
    ///        destructive)
    fn uncover_neighbors(&mut self, position: &Position) -> MinrsResult<()>;
    fn get_tile_state(&mut self, position: &Position) -> MinrsResult<TileState>;
}

pub struct StdMinrsGame {
    game_over: bool,
    board: Vec<Vec<StdTile>>,
}

impl StdMinrsGame {
    pub fn new(width: u8, height: u8, mine_count: u16) -> MinrsResult<StdMinrsGame> {
        let mut new_game = StdMinrsGame {
            game_over: false,
            board: Vec::new(),
        };

        if width < MIN_BOARD_DIMENSION || height < MIN_BOARD_DIMENSION {
            return Err(MinrsError::InvalidArgument);
        }

        let tile_count: u16 = width as u16 * height as u16;
        if mine_count >= tile_count {
            return Err(MinrsError::InvalidArgument);
        }

        let mut rng = rand::thread_rng();

        // create a unique set of random numbers indexing the tiles as:
        // col * width + row
        // XXX I feel like there might be a nicer way to do this?
        //     Perhaps we would prefer to modify existing tiles. The rand crate
        //     would allow us to nicely select a subset of elements from our
        //     board if it implements the Collections trait.
        //     Further, it would be more efficient to be able to regenerate a
        //     single mine tile if it is chosen first.
        let mut rng_vec = HashSet::new();
        for _i in 0..mine_count {
            while !rng_vec.insert(rng.gen_range(0..tile_count)) {}
        }

        // create the board
        let mut idx = 0;
        for _col in 0..width {
            let mut cur_col: Vec<StdTile> = Vec::new();
            for _row in 0..height {
                let mine = rng_vec.contains(&idx);
                cur_col.push(StdTile::new(mine));
                idx += 1;
            }
            new_game.board.push(cur_col);
        }

        return Ok(new_game);
    }

    fn mod_tile<B, F>(&mut self, pos: &Position, mut f: F) -> MinrsResult<B>
    where
        F: FnMut(&mut StdTile) -> B,
    {
        Ok(f(self
            .board
            .get_mut(pos.x as usize)
            .ok_or(MinrsError::OobPosition)?
            .get_mut(pos.y as usize)
            .ok_or(MinrsError::OobPosition)?))
    }

    fn get_tile(&self, pos: &Position) -> MinrsResult<&dyn Tile> {
        Ok(self
            .board
            .get(pos.x as usize)
            .ok_or(MinrsError::OobPosition)?
            .get(pos.y as usize)
            .ok_or(MinrsError::OobPosition)?)
    }
}

impl MinrsGame for StdMinrsGame {
    fn game_over(&self) -> bool {
        self.game_over
    }

    fn cycle_flag(&mut self, position: &Position) -> MinrsResult<()> {
        if self.game_over {
            return Err(MinrsError::GameOver);
        }

        self.mod_tile(position, |tile| tile.toggle_flag())?;
        Ok(())
    }

    fn uncover_tile(&mut self, position: &Position) -> MinrsResult<()> {
        if self.game_over {
            return Err(MinrsError::GameOver);
        }

        self.mod_tile(position, |tile| tile.uncover())??;
        Ok(())
    }

    fn uncover_neighbors(&mut self, _position: &Position) -> MinrsResult<()> {
        // TODO Unimplemented!!
        return Err(MinrsError::InvalidArgument);
    }

    fn get_tile_state(&mut self, position: &Position) -> MinrsResult<TileState> {
        let mut neighbors = Vec::new();
        let x_max = self.board.len() as i32;
        for x_mod in -1..=1 {
            let x = (position.x as i32) + x_mod;
            if x < 0 || x >= x_max {
                continue;
            }
            let y_max = self
                .board
                .get(x as usize)
                .ok_or(MinrsError::InvalidArgument)?
                .len() as i32;
            for y_mod in -1..=1 {
                let y = (position.y as i32) + y_mod;
                if y < 0 || y >= y_max || (y_mod == 0 && x_mod == 0) {
                    continue;
                }
                let tile = self.get_tile(&Position {
                    x: x as u8,
                    y: y as u8,
                })?;
                neighbors.push(tile);
            }
        }

        Ok(self.get_tile(position)?.get_state(neighbors))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_std_game() {
        let h = 8;
        let w = 10;
        let game = StdMinrsGame::new(w, h, 10).unwrap();
        assert_eq!(game.board.len(), w.into());
        game.board.iter().for_each(|col| {
            assert_eq!(col.len(), h.into());
        });
    }

    #[test]
    fn test_error_size_init_std_game() {
        let h = MIN_BOARD_DIMENSION - 1;
        let w = 10;
        let game = StdMinrsGame::new(w, h, 10);
        assert!(game.is_err());
    }

    #[test]
    fn test_error_mine_cnt_init_std_game() {
        let h = 9;
        let w = 9;
        let game = StdMinrsGame::new(w, h, 81);
        assert!(game.is_err());
    }

    /// Tests that the board spawns in all covered.
    #[test]
    fn test_tile_state_covered() {
        let h = 8;
        let w = 8;
        let ts_covered = TileState::Covered(None);
        let mut game = StdMinrsGame::new(w, h, 0).unwrap();
        for x in 0..h {
            for y in 0..w {
                assert_eq!(game.get_tile_state(&Position { x, y }).unwrap(), ts_covered);
            }
        }
    }

    #[test]
    fn test_tile_mine_count() {
        let h = 8;
        let w = 8;
        let mines = 10;
        let mut count = 0;
        let game = StdMinrsGame::new(w, h, mines).unwrap();
        for x in 0..h {
            for y in 0..w {
                if game.get_tile(&Position { x, y }).unwrap().is_mine() {
                    count += 1;
                }
            }
        }
        assert_eq!(count, mines);
    }

    #[test]
    fn test_tile_simple_uncover() {
        let h = 8;
        let w = 8;
        let test_pos = Position { x: 3, y: 4 };
        let ts_uncovered = TileState::Uncovered(TileContents::MineCount(0));
        let mut game = StdMinrsGame::new(w, h, 0).unwrap();
        game.mod_tile(&test_pos, |tile| tile.uncover())
            .unwrap()
            .unwrap();
        assert_eq!(game.get_tile_state(&test_pos).unwrap(), ts_uncovered);
    }

    #[test]
    fn test_tile_simple_uncover_mine_counting() {
        let h = 8;
        let w = 8;
        let test_pos = Position { x: 3, y: 4 };
        // following positions to all set as mines
        let test_pos1 = Position { x: 4, y: 4 };
        let test_pos2 = Position { x: 5, y: 4 };
        let test_pos3 = Position { x: 2, y: 3 };
        let ts_uncovered = TileState::Uncovered(TileContents::MineCount(2));
        let mut game = StdMinrsGame::new(w, h, 0).unwrap();
        game.mod_tile(&test_pos1, |tile| tile.mine = true).unwrap();
        game.mod_tile(&test_pos2, |tile| tile.mine = true).unwrap();
        game.mod_tile(&test_pos3, |tile| tile.mine = true).unwrap();
        game.mod_tile(&test_pos, |tile| tile.uncover())
            .unwrap()
            .unwrap();
        assert_eq!(game.get_tile_state(&test_pos).unwrap(), ts_uncovered);
    }
}