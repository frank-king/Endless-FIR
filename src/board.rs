#![allow(dead_code)]

use amethyst::core::Transform;
use amethyst::ecs::*;
use amethyst::renderer::sprite::SpriteSheetHandle;
use amethyst::renderer::SpriteRender;
use log::info;
use std::collections::hash_map::Entry;
use std::collections::HashMap;

use crate::cursor::Coord;
use crate::{Turn, ARENA_HEIGHT, ARENA_WIDTH, GRID_OFFSET};

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
    pub fn turn(&self) -> Turn {
        match self {
            Piece::Black => Turn::Player,
            Piece::White => Turn::Computer,
        }
    }
}

#[derive(Default)]
pub struct Board {
    half_width: i32,
    width: i32,
    pieces: HashMap<usize, Piece>,
    entity_map: HashMap<usize, Entity>,
    five_in_a_row: Option<[usize; 5]>,
}

impl Board {
    pub fn new(half_width: i32) -> Self {
        let width = half_width * 2 + 1;
        Self {
            half_width,
            width,
            ..Default::default()
        }
    }
    pub fn out_of_bound(&self, x: i32, y: i32) -> bool {
        x < -self.half_width || x > self.half_width || y < -self.half_width || y > self.half_width
    }
    pub fn pos2idx(&self, pos: &Coord) -> usize {
        ((pos.y + self.half_width) * self.width + (pos.x + self.half_width)) as usize
    }
    pub fn idx2pos(&self, idx: usize) -> Coord {
        let x = idx as i32 % self.width - self.half_width;
        let y = idx as i32 / self.width - self.half_width;
        Coord::new_bounded(x, y)
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

    pub fn get_piece(&self, pos: &Coord) -> Option<&Piece> {
        self.pieces.get(&self.pos2idx(pos))
    }
    pub fn set_piece(&mut self, pos: &Coord, piece: Piece) -> bool {
        if let Entry::Vacant(entry) = self.pieces.entry(self.pos2idx(pos)) {
            info!("{:?} piece inserted at {}", piece, pos);
            entry.insert(piece);
            return true;
        }
        false
    }
    pub fn remove_piece(&mut self, pos: &Coord) -> Option<Piece> {
        let idx = self.pos2idx(pos);
        self.pieces.remove(&idx)
    }

    pub fn put_entity(&mut self, pos: &Coord, entity: Entity) {
        self.entity_map.insert(self.pos2idx(pos), entity);
        self.five_in_a_row = self.calc_five_in_a_row(pos);
    }
    pub fn get_entity(&self, pos: &Coord) -> Option<Entity> {
        self.entity_map.get(&self.pos2idx(pos)).cloned()
    }
    pub fn remove_entity(&mut self, pos: &Coord) -> Option<Entity> {
        let idx = self.pos2idx(pos);
        self.entity_map.remove(&idx)
    }

    pub fn take_five_in_a_row(&mut self) -> Option<([Entity; 5], Turn)> {
        if let Some(line) = self.five_in_a_row.take() {
            info!("take away five-in-a-row");
            let turn = self.pieces[&line[0]].turn();
            for idx in line.iter() {
                self.pieces.remove(idx);
            }
            let entities = [
                self.entity_map.remove(&line[0]).unwrap(),
                self.entity_map.remove(&line[1]).unwrap(),
                self.entity_map.remove(&line[2]).unwrap(),
                self.entity_map.remove(&line[3]).unwrap(),
                self.entity_map.remove(&line[4]).unwrap(),
            ];
            return Some((entities, turn));
        }
        None
    }
    fn calc_five_in_a_row(&self, pos: &Coord) -> Option<[usize; 5]> {
        for (dx, dy) in &[(1, 0), (0, 1), (1, 1), (1, -1)] {
            let count0 = self.count_ours(pos, -*dx, -*dy);
            let count1 = self.count_ours(pos, *dx, *dy);
            if count0 + count1 + 1 >= 5 {
                let start = Coord::new_bounded(pos.x - dx * count0, pos.y - dy * count0);
                let mut line = [0; 5];
                let mut pos = start;
                for idx in 0..5 {
                    line[idx] = self.pos2idx(&pos);
                    pos.x += dx;
                    pos.y += dy;
                }
                info!("find five in row: {:?}", line);
                return Some(line);
            }
        }
        None
    }
    fn count_ours(&self, pos: &Coord, dx: i32, dy: i32) -> i32 {
        let mut count = 0;
        if let Some(piece) = self.get_piece(pos) {
            let Coord { mut x, mut y, .. } = *pos;
            loop {
                x += dx;
                y += dy;
                let pos = Coord::new_bounded(x, y);
                if self.out_of_bound(x, y) || self.get_piece(&pos) != Some(piece) {
                    break;
                }
                count += 1;
            }
        }
        count
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
        WriteStorage<'a, WantsToPlacePiece>,
        WriteStorage<'a, Piece>,
        WriteStorage<'a, SpriteRender>,
        WriteStorage<'a, Transform>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            renderer,
            default_trans,
            mut board,
            mut pieces,
            mut piece_storage,
            mut render_storage,
            mut transform_storage,
        ) = data;
        for (piece,) in (&mut pieces,).join() {
            if board.set_piece(&piece.pos, piece.piece) {
                let mut renderer = (*renderer).clone();
                Self::setup_renderer(&mut renderer, piece.piece.idx());
                let transform = Self::setup_transform(&*default_trans, &piece.pos);
                board.put_entity(
                    &piece.pos,
                    entities
                        .build_entity()
                        .with(piece.piece, &mut piece_storage)
                        .with(renderer, &mut render_storage)
                        .with(transform, &mut transform_storage)
                        .build(),
                );
                info!("put a {:?} entity at {}", piece.piece, piece.pos);
            }
        }
        pieces.clear();
    }
}
