use amethyst::assets::Handle;
use amethyst::core::Transform;
use amethyst::ecs::*;
use amethyst::renderer::sprite::SpriteSheetHandle;
use amethyst::renderer::{SpriteRender, SpriteSheet};
use log::{debug, info, log_enabled, trace, Level};

use crate::cursor::Coord;
use crate::{ARENA_HEIGHT, ARENA_WIDTH, GRID_OFFSET};

pub const BOARD_HALF_WIDTH: i32 = 7; // -7..=7

#[derive(Debug, Copy, Clone, PartialEq, Component)]
pub enum Piece {
    Black,
    White,
}

impl Piece {
    pub fn idx(&self) -> usize {
        match self {
            Piece::Black => 0,
            Piece::White => 1,
        }
    }
}

#[derive(Component)]
pub struct Board;

pub struct ChessBoard {
    half_width: i32,
    width: i32,
    pieces: Vec<Option<Piece>>,
}

impl ChessBoard {
    pub fn new(half_width: i32) -> Self {
        let width = half_width * 2 + 1;
        Self {
            half_width,
            width,
            pieces: vec![None; (width * width) as usize],
        }
    }
    pub fn pnt2idx(&self, pnt: Coord) -> usize {
        ((pnt.y + self.half_width) * self.width + (pnt.x + self.half_width)) as usize
    }
    pub fn idx2pnt(&self, idx: usize) -> Coord {
        let x = idx as i32 % self.width - self.half_width;
        let y = idx as i32 / self.width - self.half_width;
        Coord::new_bounded(x, y)
    }
    pub fn out_of_bound(&self, x: i32, y: i32) -> bool {
        x < -self.half_width || x > self.half_width || y < -self.half_width || y > self.half_width
    }
    pub fn get_piece(&self, pnt: Coord) -> Option<Piece> {
        self.pieces[self.pnt2idx(pnt)]
    }
    pub fn set_piece(&mut self, pnt: Coord, piece: Piece) -> bool {
        let idx = self.pnt2idx(pnt);
        if self.pieces[idx].is_none() {
            self.pieces[idx] = Some(piece);
            return true;
        }
        false
    }
    pub fn logic2pnt(&self, x: f32, y: f32) -> Coord {
        let x = (x - 0.5) * ARENA_WIDTH / GRID_OFFSET;
        let y = (y - 0.5) * ARENA_HEIGHT / GRID_OFFSET;
        // info!("(x, y) = ({}, {})", x, y);
        let x = x.round() as i32;
        let y = y.round() as i32;
        let out_of_bound = self.out_of_bound(x, y);
        let x = x.max(-self.half_width).min(self.half_width);
        let y = y.max(-self.half_width).min(self.half_width);
        Coord::new(x, y, out_of_bound)
    }
}

pub fn initialize_board(world: &mut World, sprite_sheet_handle: SpriteSheetHandle) {
    world.insert(ChessBoard::new(BOARD_HALF_WIDTH));

    let mut transform = Transform::default();
    // transform.set_translation_z(-1.0);
    transform.set_translation_xyz(ARENA_WIDTH, ARENA_HEIGHT, -1.0);
    let sprite_render = SpriteRender::new(sprite_sheet_handle, 0);
    world
        .create_entity()
        .with(Board)
        .with(sprite_render)
        .with(transform)
        .build();
}
