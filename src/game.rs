// Copyright 2022 nitepone <luna@night.horse>

use crate::error::{MinrsError, MinrsResult};
use rand::Rng;
use std::collections::HashSet;

const MIN_BOARD_DIMENSION: u8 = 8;

#[derive(Debug, PartialEq, Clone, Copy)]
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
    flag: Option<FlagState>,
}

impl StdTile {
    fn new(mine: bool) -> StdTile {
        StdTile {
            covered: true,
            mine,
            flag: None,
        }
    }

    fn set_mine(&mut self, mine: bool) {
        self.mine = mine;
    }
}

trait Tile {
    fn is_covered(&self) -> bool;
    fn is_mine(&self) -> bool;
    fn get_flag(&self) -> Option<FlagState>;
    fn get_contents(&self, neighbors: Vec<&dyn Tile>) -> TileContents;
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
        if !self.is_covered() {
            return None;
        }
        self.flag
    }

    fn get_contents(&self, neighbors: Vec<&dyn Tile>) -> TileContents {
        // if we are mine, say so!
        if self.mine {
            return TileContents::Mine;
        }

        // finally, we are an empty number tile.
        let count = neighbors.iter().fold(0, |count, n| {
            if n.is_mine() {
                return count + 1;
            }
            count
        });
        TileContents::MineCount(count)
    }

    fn get_state(&self, neighbors: Vec<&dyn Tile>) -> TileState {
        if self.covered {
            return TileState::Covered(self.get_flag());
        } else {
            return TileState::Uncovered(self.get_contents(neighbors));
        }
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
    /// Get the TileState of a tile at a position.
    fn get_tile_state(&self, position: &Position) -> MinrsResult<TileState>;
    /// Get the width of the current game.
    fn get_width(&self) -> u8;
    /// Get the height of the current game.
    fn get_height(&self) -> u8;
    /// Check if the game is won.
    fn victory(&self) -> bool;
}

pub struct StdMinrsGame {
    started: bool,
    game_over: bool,
    board: Vec<Vec<StdTile>>,
    width: u8,
    height: u8,
    mine_count: u16,
}

impl StdMinrsGame {
    pub fn new(width: u8, height: u8, mine_count: u16) -> MinrsResult<StdMinrsGame> {
        let mut new_game = StdMinrsGame {
            started: false,
            game_over: false,
            board: Vec::new(),
            width,
            height,
            mine_count,
        };

        if width < MIN_BOARD_DIMENSION || height < MIN_BOARD_DIMENSION {
            return Err(MinrsError::InvalidArgument);
        }

        let tile_count: u16 = width as u16 * height as u16;
        if mine_count >= tile_count {
            return Err(MinrsError::InvalidArgument);
        }

        // create the board
        for _col in 0..width {
            let mut cur_col: Vec<StdTile> = Vec::new();
            for _row in 0..height {
                cur_col.push(StdTile::new(false));
            }
            new_game.board.push(cur_col);
        }

        new_game.generate_mines(mine_count)?;

        return Ok(new_game);
    }

    fn generate_mines(&mut self, mine_count: u16) -> MinrsResult<()> {
        let mut rng_vec = HashSet::new();
        let tile_count: u16 = self.get_width() as u16 * self.get_height() as u16;
        let mut rng = rand::thread_rng();
        // create a unique set of random numbers indexing the tiles as:
        // col * width + row
        // XXX I feel like there might be a nicer way to do this?
        //     Perhaps we would prefer to modify existing tiles. The rand crate
        //     would allow us to nicely select a subset of elements from our
        //     board if it implements the Collections trait.
        //     Further, it would be more efficient to be able to regenerate a
        //     single mine tile if it is chosen first.
        for _i in 0..mine_count {
            while !rng_vec.insert(rng.gen_range(0..tile_count)) {}
        }

        // create the board
        let mut idx = 0;
        for x in 0..self.get_width() {
            for y in 0..self.get_height() {
                let mine = rng_vec.contains(&idx);
                let pos = Position { x, y };
                self.mod_tile(&pos, |t| t.set_mine(mine))?;
                idx += 1;
            }
        }

        Ok(())
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

    fn get_neighbors_pos(&self, pos: &Position) -> MinrsResult<Vec<Position>> {
        let mut neighbors: Vec<Position> = Vec::new();
        let x_max = self.board.len() as i32;
        for x_mod in -1..=1 {
            let x = (pos.x as i32) + x_mod;
            if x < 0 || x >= x_max {
                continue;
            }
            let y_max = self
                .board
                .get(x as usize)
                .ok_or(MinrsError::InvalidArgument)?
                .len() as i32;
            for y_mod in -1..=1 {
                let y = (pos.y as i32) + y_mod;
                if y < 0 || y >= y_max || (y_mod == 0 && x_mod == 0) {
                    continue;
                }
                neighbors.push(Position {
                    x: x as u8,
                    y: y as u8,
                });
            }
        }
        Ok(neighbors)
    }

    fn get_neighbors(&self, pos: &Position) -> MinrsResult<Vec<&dyn Tile>> {
        Ok(self
            .get_neighbors_pos(pos)?
            .iter()
            .map(|pos| -> &dyn Tile {
                // since positions are validated by self.get_neighbors_pos..
                // we assume we can unwrap (or panic)
                self.get_tile(pos).unwrap()
            })
            .collect())
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

    fn uncover_tile(&mut self, pos: &Position) -> MinrsResult<()> {
        let self_tile = self.get_tile(pos)?;
        if !self_tile.is_covered() {
            return Err(MinrsError::InvalidArgument);
        }

        let neighbors_pos = self.get_neighbors_pos(pos)?;
        match self_tile.get_contents(self.get_neighbors(pos)?) {
            TileContents::MineCount(mine_count) => {
                if mine_count == 0 {
                    self.started = true; // enforce started game
                    self.mod_tile(pos, |tile| tile.uncover())??;
                    for n_pos in neighbors_pos {
                        let n = self.get_tile(&n_pos)?;
                        if !n.is_covered() {
                            continue;
                        }
                        self.uncover_tile(&n_pos)?;
                    }
                    return Ok(());
                }
            }
            TileContents::Mine => {
                // don't game over on first move..
                if self.started == true {
                    self.game_over = true;
                }
            }
        }

        // If we are here on the first move, we didn't open a "sea" above.
        // We want to ensure the player's first move opens a sea of empty
        // tiles.
        // So, regenerate the board and recurse uncovering the desired tile...
        // And the player is none the wiser >:3c
        if !self.started {
            self.generate_mines(self.mine_count)?;
            self.uncover_tile(pos)?;
            return Ok(());
        }

        self.mod_tile(pos, |tile| tile.uncover())??;
        Ok(())
    }

    fn uncover_neighbors(&mut self, pos: &Position) -> MinrsResult<()> {
        let mine_count;
        // only allow uncovered tiles with a minecount
        match self.get_tile_state(pos)? {
            TileState::Uncovered(contents) => match contents {
                TileContents::MineCount(count) => {
                    mine_count = count;
                }
                TileContents::Mine => {
                    return Err(MinrsError::InvalidArgument);
                }
            },
            TileState::Covered(_) => {
                return Err(MinrsError::InvalidArgument);
            }
        }

        // enforce that the user has enough flags placed to make this move
        let neighbors_flag_count =
            self.get_neighbors(pos)?
                .iter()
                .fold(0, |mut flag_count, tile| -> u8 {
                    if tile.get_flag().is_some() {
                        flag_count += 1;
                    }
                    flag_count
                });
        if neighbors_flag_count < mine_count {
            return Err(MinrsError::InvalidArgument);
        }

        // uncover unflagged neighbors
        let neighbors_pos = self.get_neighbors_pos(pos)?;
        for n_pos in neighbors_pos {
            let tile_state = self.get_tile_state(&n_pos)?;
            match tile_state {
                TileState::Covered(flag_state) => {
                    if flag_state.is_none() {
                        self.uncover_tile(&n_pos)?;
                    }
                }
                TileState::Uncovered(_) => {}
            }
        }

        return Ok(());
    }

    fn get_tile_state(&self, position: &Position) -> MinrsResult<TileState> {
        Ok(self
            .get_tile(position)?
            .get_state(self.get_neighbors(position)?))
    }

    fn get_width(&self) -> u8 {
        self.width
    }

    fn get_height(&self) -> u8 {
        self.height
    }

    fn victory(&self) -> bool {
        let in_progress = self.board.iter().all(|row| {
            row.iter()
                .all(|tile| !(tile.is_covered() && !tile.is_mine()))
        });
        return in_progress;
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
        let game = StdMinrsGame::new(w, h, 0).unwrap();
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
