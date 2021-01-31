#![allow(dead_code)]

use amethyst::core::Transform;
use amethyst::ecs::*;
use amethyst::renderer::sprite::SpriteSheetHandle;
use amethyst::renderer::SpriteRender;

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
    pub fn next(&self) -> Self {
        match self {
            Piece::Black => Piece::White,
            Piece::White => Piece::Black,
        }
    }
}

pub struct Board {
    half_width: i32,
    width: i32,
    pieces: Vec<Option<Piece>>,
}

impl Board {
    pub fn new(half_width: i32) -> Self {
        let width = half_width * 2 + 1;
        Self {
            half_width,
            width,
            pieces: vec![None; (width * width) as usize],
        }
    }
    pub fn pos2idx(&self, pos: &Coord) -> usize {
        ((pos.y + self.half_width) * self.width + (pos.x + self.half_width)) as usize
    }
    pub fn idx2pos(&self, idx: usize) -> Coord {
        let x = idx as i32 % self.width - self.half_width;
        let y = idx as i32 / self.width - self.half_width;
        Coord::new_bounded(x, y)
    }
    pub fn out_of_bound(&self, x: i32, y: i32) -> bool {
        x < -self.half_width || x > self.half_width || y < -self.half_width || y > self.half_width
    }
    pub fn get_piece(&self, pos: &Coord) -> Option<Piece> {
        self.pieces[self.pos2idx(pos)]
    }
    pub fn set_piece(&mut self, pos: &Coord, piece: Piece) -> bool {
        let idx = self.pos2idx(pos);
        if self.pieces[idx].is_none() {
            self.pieces[idx] = Some(piece);
            return true;
        }
        false
    }
    pub fn logic2pos(&self, x: f32, y: f32) -> Coord {
        let x = (x - 0.5) * ARENA_WIDTH / GRID_OFFSET;
        let y = (y - 0.5) * ARENA_HEIGHT / GRID_OFFSET;
        let x = x.round() as i32;
        let y = y.round() as i32;
        let out_of_bound = self.out_of_bound(x, y);
        let x = x.max(-self.half_width).min(self.half_width);
        let y = y.max(-self.half_width).min(self.half_width);
        Coord::new(x, y, out_of_bound)
    }
}

pub fn initialize_board(world: &mut World, sprite_sheet_handle: SpriteSheetHandle) {
    world.insert(Board::new(BOARD_HALF_WIDTH));

    let mut transform = Transform::default();
    // transform.set_translation_z(-1.0);
    transform.set_translation_xyz(ARENA_WIDTH, ARENA_HEIGHT, -1.0);
    let sprite_render = SpriteRender::new(sprite_sheet_handle, 0);
    world
        .create_entity()
        .with(sprite_render)
        .with(transform)
        .build();
}

#[derive(Component)]
pub struct WantsToPlacePiece {
    pub piece: Piece,
    pub pos: Coord,
}

pub trait PieceRender {
    fn setup_renderer(renderer: &mut SpriteRender, piece_idx: usize) {
        renderer.sprite_number = piece_idx;
    }
    fn setup_transform(default_trans: &Transform, pos: &Coord) -> Transform {
        let x = pos.x as f32 * GRID_OFFSET;
        let y = pos.y as f32 * GRID_OFFSET;
        let mut transform = default_trans.clone();
        transform.append_translation_xyz(x, y, 0.0);
        transform
    }
}

pub struct PieceSystem;

impl PieceRender for PieceSystem {}

impl<'a> System<'a> for PieceSystem {
    type SystemData = (
        Entities<'a>,
        ReadExpect<'a, SpriteRender>,
        ReadExpect<'a, Transform>,
        WriteExpect<'a, Board>,
        Option<WriteExpect<'a, WantsToPlacePiece>>,
        WriteStorage<'a, SpriteRender>,
        WriteStorage<'a, Transform>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            renderer,
            default_trans,
            mut board,
            mut piece,
            mut render_storage,
            mut transform_storage,
        ) = data;
        if let Some(piece) = piece.take() {
            if board.set_piece(&piece.pos, piece.piece) {
                let mut renderer = (*renderer).clone();
                Self::setup_renderer(&mut renderer, piece.piece.idx());
                let transform = Self::setup_transform(&*default_trans, &piece.pos);
                entities
                    .build_entity()
                    .with(renderer, &mut render_storage)
                    .with(transform, &mut transform_storage)
                    .build();
            }
        }
    }
}
