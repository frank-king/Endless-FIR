use amethyst::core::math::Vector3;
use amethyst::core::{Hidden, Transform};
use amethyst::ecs::*;
use amethyst::renderer::palette::Srgba;
use amethyst::renderer::resources::Tint;
use amethyst::renderer::sprite::SpriteSheetHandle;
use amethyst::renderer::{SpriteRender, Transparent};

use crate::board::{Piece, PieceRender};
use crate::{ARENA_HEIGHT, ARENA_WIDTH};

#[derive(Component, PartialEq, Copy, Clone)]
pub struct Coord {
    pub x: i32,
    pub y: i32,
    pub out_of_bound: bool,
}

impl Coord {
    pub fn new(x: i32, y: i32, out_of_bound: bool) -> Self {
        Self { x, y, out_of_bound }
    }
    pub fn new_bounded(x: i32, y: i32) -> Self {
        Self::new(x, y, false)
    }
}

#[derive(Default, Component)]
pub struct Cursor {
    pub show: bool,
    pub dirty: bool,
}

impl Cursor {
    pub fn show(&mut self) {
        self.show = true;
        self.dirty();
    }
    pub fn hide(&mut self) {
        self.show = false;
        self.dirty();
    }
    pub fn dirty(&mut self) {
        self.dirty = true;
    }
}

pub struct CursorSystem;

impl PieceRender for CursorSystem {}

impl CursorSystem {
    fn toggle_hidden(hidden: &mut WriteStorage<Hidden>, show: bool, entity: Entity) {
        if show && hidden.get(entity).is_some() {
            hidden.remove(entity);
        }
        if !show && hidden.get(entity).is_none() {
            hidden
                .insert(entity, Hidden)
                .expect("failed to insert Hidden");
        }
    }
}

impl<'a> System<'a> for CursorSystem {
    type SystemData = (
        Entities<'a>,
        ReadExpect<'a, Transform>,
        ReadStorage<'a, Coord>,
        ReadStorage<'a, Piece>,
        WriteStorage<'a, Cursor>,
        WriteStorage<'a, SpriteRender>,
        WriteStorage<'a, Transform>,
        WriteStorage<'a, Hidden>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, default_trans, pos, piece, mut cursor, mut renderer, mut trans, mut hidden) =
            data;
        let joined = (
            &entities,
            &pos,
            &piece,
            &mut cursor,
            &mut renderer,
            &mut trans,
        )
            .join();
        for (entity, pos, piece, cursor, renderer, transform) in joined {
            if cursor.dirty {
                cursor.dirty = false;
                Self::toggle_hidden(&mut hidden, cursor.show, entity);
                Self::setup_renderer(renderer, piece.idx());
                *transform = Self::setup_transform(&*default_trans, pos);
            }
        }
    }
}

pub fn initialize_cursor(world: &mut World, sprite_sheet_handle: SpriteSheetHandle) {
    let mut transform = Transform::default();
    transform.set_translation_xyz(ARENA_WIDTH, ARENA_HEIGHT, 0.0);
    transform.set_scale(Vector3::from_element(0.125));
    world.insert(transform.clone());

    let sprite_render = SpriteRender::new(sprite_sheet_handle, 0);
    world.insert(sprite_render.clone());

    let cursor_entity = world
        .create_entity()
        .with(Piece::Black)
        .with(Coord::new_bounded(0, 0))
        .with(Cursor::default())
        .with(sprite_render)
        .with(transform)
        .with(Hidden)
        .with(Transparent)
        .with(Tint(Srgba::from_components((0.8, 0.8, 0.8, 0.5))))
        .build();
    world.insert(cursor_entity);
}
