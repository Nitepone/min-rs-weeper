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

const NO_WINDOW_PARENT: Option<&gtk::Window> = None;

#[derive(Copy, Clone)]
enum GameDifficulty {
    Easy,
    Medium,
    Hard,
}

impl GameDifficulty {
    fn get_width(&self) -> u8 {
        match self {
            GameDifficulty::Easy => 8,
            GameDifficulty::Medium => 15,
            GameDifficulty::Hard => 30,
        }
    }

    fn get_height(&self) -> u8 {
        match self {
            GameDifficulty::Easy => 8,
            GameDifficulty::Medium => 15,
            GameDifficulty::Hard => 30,
        }
    }

    fn get_mines(&self) -> u16 {
        match self {
            GameDifficulty::Easy => 10,
            GameDifficulty::Medium => 40,
            GameDifficulty::Hard => 99,
        }
    }
}

struct GuiPriv {
    difficulty: GameDifficulty,
    buttons: Vec<Vec<gtk::Button>>,
    window: gtk::ApplicationWindow,
    grid: gtk::Grid,
    game: StdMinrsGame,
    gp_arc: Option<Arc<Mutex<GuiPriv>>>,
    menu_bar: gtk::MenuBar,
    v_box: gtk::Box,
}

fn main() {
    let application = gtk::Application::new(
        Some("com.github.nitepone.min-rs-weeper"),
        Default::default(),
    );

    application.connect_activate(build_ui);

    application.run();
}

fn update_buttons(gp: &mut MutexGuard<GuiPriv>) {
    for x in 0..gp.game.get_width() {
        for y in 0..gp.game.get_height() {
            let button = gp.buttons.get(x as usize).unwrap().get(y as usize).unwrap();

            match gp.game.get_tile_state(&Position { x, y }).unwrap() {
                TileState::Covered(flag_opt) => {
                    button.set_relief(gtk::ReliefStyle::Normal);
                    match flag_opt {
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
                    }
                }
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

fn draw_gameover_dialog() -> bool {
    let mtype = gtk::MessageType::Warning;
    let dialog = gtk::MessageDialog::new(
        NO_WINDOW_PARENT,
        gtk::DialogFlags::MODAL,
        mtype,
        gtk::ButtonsType::YesNo,
        "Gameover!\nStart a new game?",
    );
    dialog.set_title("min-rs-weeper - gameover");
    dialog.set_position(gtk::WindowPosition::Center);
    let resp = dialog.run();
    dialog.close();
    match resp {
        gtk::ResponseType::Yes => true,
        gtk::ResponseType::No => false,
        _ => false,
    }
}

fn draw_victory_dialog() -> bool {
    let mtype = gtk::MessageType::Warning;
    let dialog = gtk::MessageDialog::new(
        NO_WINDOW_PARENT,
        gtk::DialogFlags::MODAL,
        mtype,
        gtk::ButtonsType::YesNo,
        "Victory!\nStart a new game?",
    );
    dialog.set_title("min-rs-weeper - victory!");
    dialog.set_position(gtk::WindowPosition::Center);
    let resp = dialog.run();
    dialog.close();
    match resp {
        gtk::ResponseType::Yes => true,
        gtk::ResponseType::No => false,
        _ => false,
    }
}

fn restart_game(gp: &mut MutexGuard<GuiPriv>) {
    let diff = gp.difficulty;
    gp.game = StdMinrsGame::new(diff.get_width(), diff.get_height(), diff.get_mines()).unwrap();
    draw_buttons(gp);
}

fn draw_buttons(gp: &mut MutexGuard<GuiPriv>) {
    gp.buttons = Vec::new();
    gp.v_box.remove(&gp.grid);
    gp.grid = gtk::Grid::new();
    for x in 0..gp.game.get_width() {
        let mut buttons_row_arr = Vec::new();
        for y in 0..gp.game.get_height() {
            let button = gtk::Button::new();
            let d_gui_priv = gp.gp_arc.clone().unwrap();
            #[allow(unused_must_use)] // there are a lot of errors to no-op on..
            button.connect_event(move |_btn, e| {
                if e.event_type() == gdk::EventType::ButtonPress {
                    let pos = Position { x, y };
                    let mut gp = d_gui_priv.lock().unwrap();
                    if e.button().unwrap_or(0) == 1 {
                        gp.game.uncover_tile(&pos);
                    } else if e.button().unwrap_or(0) == 3 {
                        // Uh. So we can attempt uncovering neighbors, then cycling the flag.
                        // One of these will always error. But.. Whatever.
                        gp.game.uncover_neighbors(&pos);
                        gp.game.cycle_flag(&pos);
                    }
                    update_buttons(&mut gp);
                    let mut restart = None;
                    if gp.game.game_over() {
                        restart = Some(draw_gameover_dialog());
                    }
                    if gp.game.victory() {
                        restart = Some(draw_victory_dialog());
                    }
                    if let Some(restart) = restart {
                        if !restart {
                            std::process::exit(0);
                        } else {
                            restart_game(&mut gp);
                            update_buttons(&mut gp);
                        }
                    }
                }
                return gtk::Inhibit(false);
            });
            let gbox = gtk::Box::new(gtk::Orientation::Horizontal, 0);
            button.set_size_request(50, 50);
            button.set_expand(false);
            button.set_hexpand(false);
            button.set_margin(0);
            gp.grid.attach(&button, x as i32, y as i32, 1, 1);
            gbox.set_expand(false);
            buttons_row_arr.push(button)
        }
        gp.buttons.push(buttons_row_arr);
    }
    gp.v_box.pack_start(&gp.grid, true, true, 0);
    gp.window.show_all();
}

fn populate_menu_bar(gp: &mut MutexGuard<GuiPriv>) {
    let diff_submenu = gtk::Menu::new();
    let diff = gtk::MenuItem::with_label("Difficulty");
    let easy = gtk::MenuItem::with_label("Easy");
    let medium = gtk::MenuItem::with_label("Medium");
    let hard = gtk::MenuItem::with_label("Hard");
    let easy_gp = gp.gp_arc.clone().unwrap();
    easy.connect_activate(move |_| {
        let mut gp = easy_gp.lock().unwrap();
        gp.difficulty = GameDifficulty::Easy;
        restart_game(&mut gp);
        draw_buttons(&mut gp);
        update_buttons(&mut gp);
    });
    let medium_gp = gp.gp_arc.clone().unwrap();
    medium.connect_activate(move |_| {
        let mut gp = medium_gp.lock().unwrap();
        gp.difficulty = GameDifficulty::Medium;
        restart_game(&mut gp);
        draw_buttons(&mut gp);
        update_buttons(&mut gp);
    });
    let hard_gp = gp.gp_arc.clone().unwrap();
    hard.connect_activate(move |_| {
        let mut gp = hard_gp.lock().unwrap();
        gp.difficulty = GameDifficulty::Hard;
        restart_game(&mut gp);
        draw_buttons(&mut gp);
        update_buttons(&mut gp);
    });
    diff_submenu.append(&easy);
    diff_submenu.append(&medium);
    diff_submenu.append(&hard);
    diff.set_submenu(Some(&diff_submenu));
    gp.menu_bar.append(&diff);
}

fn build_ui(application: &gtk::Application) {
    let gui_priv_arc = Arc::new(Mutex::new(GuiPriv {
        difficulty: GameDifficulty::Easy,
        game: StdMinrsGame::new(8, 8, 10).unwrap(),
        buttons: Vec::new(),
        grid: gtk::Grid::new(),
        window: gtk::ApplicationWindow::new(application),
        gp_arc: None,
        menu_bar: gtk::MenuBar::new(),
        v_box: gtk::Box::new(gtk::Orientation::Vertical, 10),
    }));
    let mut gp = gui_priv_arc.lock().unwrap();
    gp.gp_arc = Some(gui_priv_arc.clone());
    populate_menu_bar(&mut gp);
    gp.v_box.pack_start(&gp.menu_bar, false, false, 0);
    gp.window.add(&gp.v_box);
    restart_game(&mut gp);
    draw_buttons(&mut gp);
    gp.window.set_title("min-rs-weeper");
    gp.window.set_position(gtk::WindowPosition::Center);
    gp.window.show_all();
}
