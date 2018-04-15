
extern crate rand;
extern crate bear_lib_terminal;

use bear_lib_terminal::{terminal, Color, terminal::{Event, KeyCode}};
use rand::{Rng, thread_rng};
use std::slice::Iter;

const WIDTH: i32 = 80;
const HEIGHT: i32 = 25;
const BOARD_WIDTH: i32 = WIDTH;
const BOARD_HEIGHT: i32 = HEIGHT - 1;

const GOLD_COLORS: [[u8; 3]; 9] = [
    [31, 127, 255],
    [31, 127, 127],
    [63, 63, 0],
    [95, 95, 0],
    [127, 127, 0],
    [159, 159, 0],
    [191, 191, 0],
    [223, 223, 0],
    [255, 255, 0],
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Direction {
    North,
    South,
    East,
    West,
    Northwest,
    Northeast,
    Southwest,
    Southeast
}
use self::Direction::*;

impl Direction {
    fn dx(self) -> i32 {
        match self {
            West | Northwest | Southwest => -1,
            East | Northeast | Southeast => 1,
            _ => 0
        }
    }

    fn dy(self) -> i32 {
        match self {
            North | Northwest | Northeast => -1,
            South | Southwest | Southeast => 1,
            _ => 0
        }
    }

    fn iter() -> Iter<'static, Direction> {
        static DIRECTIONS: [Direction;  8] = [
            North, South, East, West,
            Northwest, Northeast, Southwest, Southeast
        ];
        DIRECTIONS.into_iter()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Tile {
    Empty,
    Rogue,
    Gold(i32)
}

struct GreedyContext {
    cleared: i32,
    rogue_x: i32,
    rogue_y: i32,
    tiles: [Tile; (BOARD_HEIGHT * BOARD_WIDTH) as usize]
}

impl GreedyContext {
    fn new() -> GreedyContext {
        let rogue_x = BOARD_WIDTH / 2;
        let rogue_y = BOARD_HEIGHT / 2;
        let mut tiles = [Tile::Empty; (BOARD_HEIGHT * BOARD_WIDTH) as usize];
        tiles[(rogue_y * BOARD_WIDTH + rogue_x) as usize] = Tile::Rogue;
        GreedyContext { cleared: 0, rogue_x, rogue_y, tiles }
    }

    fn in_bounds(&self, x: i32, y: i32) -> bool {
        return x >= 0 && y >= 0 && x < BOARD_WIDTH && y < BOARD_HEIGHT;
    }

    fn tile(&self, x: i32, y: i32) -> Tile {
        if !self.in_bounds(x, y) {
            return Tile::Empty;
        }
        let t = self.tiles[(y * BOARD_WIDTH + x) as usize];
        return t;
    }

    fn set_tile(&mut self, x: i32, y: i32, t: Tile) {
        self.tiles[(y * BOARD_WIDTH + x) as usize] = t;
    }
}

fn valid_moves(gc: &GreedyContext) -> Vec<Direction> {
    let mut dirs = vec![];
    let rx = gc.rogue_x;
    let ry = gc.rogue_y;
    for &d in Direction::iter() {
        let mut x = rx + d.dx();
        let mut y = ry + d.dy();
        if let Tile::Gold(n) = gc.tile(x, y) {
            let mut tiles = vec![];
            for _ in 1..n {
                x += d.dx();
                y += d.dy();
                tiles.push(gc.tile(x, y));
            }
            if !tiles.iter().any(|&t| t == Tile::Empty) {
                dirs.push(d)
            }
        }
    }
    dirs
}

fn move_rogue(gc: &mut GreedyContext, dir: Direction) {
    let valid = valid_moves(gc);
    if valid.iter().all(|&d| d != dir) {
        return
    }
    let rx = gc.rogue_x;
    let ry = gc.rogue_y;
    gc.set_tile(rx, ry, Tile::Empty);
    let mut x = rx + dir.dx();
    let mut y = ry + dir.dy();
    if let Tile::Gold(n) = gc.tile(x, y) {
        gc.cleared += n;
        gc.set_tile(x, y, Tile::Empty);
        for _ in 1..n {
            x += dir.dx();
            y += dir.dy();
            gc.set_tile(x, y, Tile::Empty);
        }
    }
    gc.rogue_x = x;
    gc.rogue_y = y;
    gc.set_tile(x, y, Tile::Rogue);
}

fn print_tile(tile: Tile, x: i32, y: i32, highlight: bool) {
        match tile {
            Tile::Rogue => {
                terminal::set_foreground(Color::from_rgb(255, 0, 0));
                terminal::set_background(Color::from_rgb(0, 0, 0));
                terminal::put_xy(x, y, '@');
            }
            Tile::Gold(n) => {
                let c = GOLD_COLORS[(n - 1) as usize];
                let r = c[0];
                let g = c[1];
                let b = c[2];
                terminal::set_foreground(Color::from_rgb(r, g, b));
                if highlight {
                    terminal::set_background(Color::from_rgb(196, 196, 196));
                } else {
                    terminal::set_background(Color::from_rgb(0, 0, 0));
                }
                terminal::put_xy(x, y, n.to_string().chars().nth(0).unwrap());
            }
            _ => {}
        }
}

fn print_greedy(gc: &GreedyContext, xo: i32, yo: i32) {
    for x in 0..BOARD_WIDTH {
        for y in 0..BOARD_HEIGHT {
            print_tile(gc.tile(x, y), x + xo, y + yo, false);
        }
    }
    terminal::set_background(Color::from_rgb(0, 0, 0));
    terminal::print_xy(0, BOARD_HEIGHT + yo,
        &*format!("[color=white]Cleared: [color=yellow]{:.4}%",
            gc.cleared as f64 / (BOARD_WIDTH * BOARD_HEIGHT * 100) as f64));
}

fn print_moves(gc: &GreedyContext, xo: i32, yo: i32) {
    let moves = valid_moves(gc);
    let rx = gc.rogue_x;
    let ry = gc.rogue_y;
    for dir in moves {
        let mut x = rx + dir.dx();
        let mut y = ry + dir.dy();
        if let Tile::Gold(n) = gc.tile(x, y) {
            print_tile(gc.tile(x, y), x + xo, y + yo, true);
            for _ in 1..n {
                x += dir.dx();
                y += dir.dy();
                print_tile(gc.tile(x, y), x + xo, y + yo, true);
            }
        }
    }
}

fn roll(dice: i32, sides: i32) -> i32 {
    (0..dice).fold(0, |sum, _| sum + thread_rng().gen_range(1, sides + 1))
}

fn setup_game(gc: &mut GreedyContext) {
    for x in 0..BOARD_WIDTH {
        for y in 0..BOARD_HEIGHT {
            gc.set_tile(x, y, Tile::Gold(roll(1, 9)));
        }
    }
    // The rogue.
    gc.cleared = 1;
    let rx = BOARD_WIDTH / 2;
    let ry = BOARD_HEIGHT / 2;
    gc.rogue_x = rx;
    gc.rogue_y = ry;
    gc.set_tile(rx, ry, Tile::Rogue);
}

fn end_game(gc: &GreedyContext, show_moves: bool) {
    terminal::set_background(Color::from_rgb(0, 0, 0));
    terminal::clear(None);
    print_greedy(&gc, 0, 0);
    if show_moves {
        print_moves(&gc, 0, 0);
    }
    terminal::set_background(Color::from_rgb(0, 0, 0));
    terminal::print_xy(BOARD_WIDTH / 2, BOARD_HEIGHT,
        &*format!("[color=red]No more moves. Press any key to quit."));
    terminal::refresh();
    let _ = terminal::wait_event();
}

fn main() {
    let mut gc = GreedyContext::new();
    let mut show_moves = false;
    setup_game(&mut gc);
    terminal::open("Greedy", WIDTH as u32, HEIGHT as u32);
    
    loop {
        terminal::set_background(Color::from_rgb(0, 0, 0));
        terminal::clear(None);
        print_greedy(&gc, 0, 0);
        if show_moves {
            print_moves(&gc, 0, 0);
        }
        terminal::refresh();
        let ev = terminal::wait_event().unwrap();
        match ev {
            Event::KeyPressed {key, ..} => {
                match key {
                    KeyCode::Q => break,
                    KeyCode::Left | KeyCode::H => {
                        move_rogue(&mut gc, West);
                    }
                    KeyCode::Right | KeyCode::L => {
                        move_rogue(&mut gc, East);
                    }
                    KeyCode::Up | KeyCode::K => {
                        move_rogue(&mut gc, North);
                    }
                    KeyCode::Down | KeyCode::J => {
                        move_rogue(&mut gc, South);
                    }
                    KeyCode::Y => {
                        move_rogue(&mut gc, Northwest);
                    }
                    KeyCode::U => {
                        move_rogue(&mut gc, Northeast);
                    }
                    KeyCode::B => {
                        move_rogue(&mut gc, Southwest);
                    }
                    KeyCode::N => {
                        move_rogue(&mut gc, Southeast);
                    }
                    KeyCode::V => {
                        show_moves = !show_moves;
                    }
                    _ => {}
                }
            }
            Event::Close => break,
            _ => {}
        }

        if valid_moves(&gc).len() == 0 {
            end_game(&gc, show_moves);
            break;
        }
    }

    terminal::close();
}
