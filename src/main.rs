// Copyright 2022 nitepone <luna@night.horse>
//
// Mostly gui code in here.. Mostly not cute..

extern crate gtk;
extern crate rand;

pub mod error;
mod game;

use crate::game::{FlagState, MinrsGame, Position, StdMinrsGame, TileContents, TileState};
use gtk::gdk;
use gtk::prelude::*;
use std::sync::{Arc, Mutex, MutexGuard};

fn main() {
    let application = gtk::Application::new(
        Some("com.github.nitepone.min-rs-weeper"),
        Default::default(),
    );

    application.connect_activate(build_ui);

    application.run();
}

fn update_grid(mut game: MutexGuard<StdMinrsGame>, btn_mtx: MutexGuard<Vec<Vec<gtk::Button>>>) {
    for x in 0..game.get_width() {
        for y in 0..game.get_height() {
            let button = btn_mtx.get(x as usize).unwrap().get(y as usize).unwrap();

            match game.get_tile_state(&Position { x, y }).unwrap() {
                TileState::Covered(flag_opt) => match flag_opt {
                    None => {
                        button.set_label(" ");
                    }
                    Some(flag) => match flag {
                        FlagState::Questionable => {
                            button.set_label("?");
                        }
                        FlagState::RedFlag => {
                            button.set_label("!");
                        }
                    },
                },
                TileState::Uncovered(con) => {
                    button.set_relief(gtk::ReliefStyle::None);
                    match con {
                        TileContents::Mine => {
                            button.set_label("*");
                        }
                        TileContents::MineCount(count) => {
                            if count == 0 {
                                button.set_label(" ");
                            } else {
                                button.set_label(&format!("{count}"));
                            }
                        }
                    }
                }
            }
        }
    }
    // *looks up* triple nested decision struture in a double nested loop? O.o
}

fn build_ui(application: &gtk::Application) {
    let arc_game = Arc::new(Mutex::new(StdMinrsGame::new(8, 8, 10).unwrap()));
    let game = arc_game.lock().unwrap();
    let arc_buttons: Arc<Mutex<Vec<Vec<gtk::Button>>>> = Arc::new(Mutex::new(Vec::new()));
    let mut buttons = arc_buttons.lock().unwrap();

    let window = gtk::ApplicationWindow::new(application);

    window.set_title("min-rs-weeper");
    window.set_position(gtk::WindowPosition::Center);
    let grid = gtk::Grid::new();
    for x in 0..game.get_width() {
        let mut buttons_row_arr = Vec::new();
        for y in 0..game.get_height() {
            let button = gtk::Button::new();
            let d_game = arc_game.clone();
            let d_btns = arc_buttons.clone();
            #[allow(unused_must_use)] // there are a lot of errors to no-op on..
            button.connect_event(move |_btn, e| {
                if e.event_type() == gdk::EventType::ButtonPress {
                    let pos = Position { x, y };
                    let mut game = d_game.lock().unwrap();
                    if e.button().unwrap_or(0) == 1 {
                        game.uncover_tile(&pos);
                    } else if e.button().unwrap_or(0) == 3 {
                        // Uh. So we can attempt uncovering neighbors, then cycling the flag.
                        // One of these will always error. But.. Whatever.
                        game.uncover_neighbors(&pos);
                        game.cycle_flag(&pos);
                    }
                    update_grid(game, d_btns.lock().unwrap());
                }
                return gtk::Inhibit(false);
            });
            let gbox = gtk::Box::new(gtk::Orientation::Horizontal, 0);
            button.set_size_request(50, 50);
            button.set_expand(false);
            button.set_hexpand(false);
            button.set_margin(0);
            //            gbox.add(&button);
            //            gbox.set_resize_mode(gtk::ResizeMode::Parent);
            grid.attach(&button, x as i32, y as i32, 1, 1);
            gbox.set_expand(false);
            buttons_row_arr.push(button)
        }
        buttons.push(buttons_row_arr);
    }
    window.add(&grid);
    window.show_all();
}
