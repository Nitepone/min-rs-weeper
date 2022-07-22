// Copyright 2022 nitepone <luna@night.horse>

extern crate gtk;
extern crate rand;

pub mod error;
mod game;

use crate::game::{FlagState, MinrsGame, Position, StdMinrsGame, TileContents, TileState};
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
                    None => {}
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
                            if count > 0 {
                                button.set_label(&format!("{count}"));
                            }
                        }
                    }
                }
            }
        }
    }
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
            #[allow(unused_must_use)]
            button.connect_clicked(move |_| {
                let mut game = d_game.lock().unwrap();
                // TODO: Feedback when move impossible.
                game.uncover_tile(&Position { x, y });
                update_grid(game, d_btns.lock().unwrap());
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
