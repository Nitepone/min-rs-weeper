# Minesweeper! In Rust! With a GTK GUI!

![image of minrsweeper](./docs/img/minrsweeper.png)

Just a cute little project to play around with. It's a fully featured
Minesweeper clone.

## Controls

- Left click to Uncover a tile
- Right click a covered tile to place a flag
- Right click an uncovered number tile to uncover unflagged neighbors

## Building

### Dependencies

The `gtk3-rs` crate depends on GTK3 development libraries.

- `libgtk-3-dev` on Debian based distros.
- `gtk3-devel` on Fedora based distros.

### Actually Building

Just, `cargo make`
