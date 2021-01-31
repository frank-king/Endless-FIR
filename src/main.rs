use amethyst::assets::{AssetStorage, Handle, Loader};
use amethyst::core::ecs::Entity;
use amethyst::core::{Transform, TransformBundle};
use amethyst::input::{is_close_requested, InputEvent};
use amethyst::prelude::*;
use amethyst::renderer::sprite::SpriteSheetHandle;
use amethyst::renderer::types::DefaultBackend;
use amethyst::renderer::{
    Camera, ImageFormat, RenderFlat2D, RenderToWindow, RenderingBundle, SpriteSheet,
    SpriteSheetFormat, Texture,
};
use amethyst::window::ScreenDimensions;
use amethyst::winit::{ElementState, Event, MouseButton, WindowEvent};
use log::{debug, info, log_enabled, trace, Level};

mod board;
mod cursor;

use board::{initialize_board, Board, ChessBoard, Piece};
use cursor::{initialize_cursor, Coord, Cursor};

pub const ARENA_HEIGHT: f32 = 800.0;
pub const ARENA_WIDTH: f32 = 800.0;
pub const GRID_OFFSET: f32 = 40.0;
const BOARD_TEXTURE: &str = "texture/board.png";
const BOARD_SPRITE_SHEET: &str = "texture/board.ron";
const PIECE_TEXTURE: &str = "texture/piece.png";
const PIECE_SPRITE_SHEET: &str = "texture/piece.ron";

pub enum RunState {
    Player,
    Computer,
}

struct LogicalSize {
    width: f64,
    height: f64,
}

pub struct State;

impl SimpleState for State {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;
        let board_handle = load_sprite_sheet(world, BOARD_TEXTURE, BOARD_SPRITE_SHEET);
        let piece_handle = load_sprite_sheet(world, PIECE_TEXTURE, PIECE_SPRITE_SHEET);
        world.register::<Piece>();
        world.register::<Board>();
        world.register::<Coord>();
        world.register::<Cursor>();
        initialize_board(world, board_handle);
        initialize_cursor(world, piece_handle);
        initialize_camara(world);
        world.insert(RunState::Player);
        world.insert(LogicalSize {
            width: ARENA_WIDTH as f64,
            height: ARENA_HEIGHT as f64,
        });
    }

    fn handle_event(
        &mut self,
        data: StateData<'_, GameData<'_, '_>>,
        event: StateEvent,
    ) -> SimpleTrans {
        if let StateEvent::Window(e) = event {
            if let Event::WindowEvent { event, .. } = e {
                let world = data.world;
                let cursor_entity = world.fetch::<Entity>();
                let board = world.fetch::<ChessBoard>();
                let mut window_size = world.fetch_mut::<LogicalSize>();
                let mut pos = world.write_storage::<Coord>();
                let mut cursor = world.write_storage::<Cursor>();

                // info!("window event {:?}", event);
                match event {
                    WindowEvent::Resized(size) => {
                        window_size.width = size.width;
                        window_size.height = size.height;
                    }
                    WindowEvent::CloseRequested => return Trans::Quit,
                    WindowEvent::CursorMoved { position, .. } => {
                        let x = position.x / window_size.width;
                        let y = 1.0 - position.y / window_size.height;
                        let coord = board.logic2pnt(x as f32, y as f32);
                        let old_coord = pos.get_mut(*cursor_entity).unwrap();
                        if coord != *old_coord {
                            cursor.get_mut(*cursor_entity).unwrap().dirty();
                        }
                        *old_coord = coord;
                    }
                    WindowEvent::CursorEntered { .. } => {
                        cursor.get_mut(*cursor_entity).unwrap().show()
                    }
                    WindowEvent::CursorLeft { .. } => {
                        cursor.get_mut(*cursor_entity).unwrap().hide();
                    }
                    WindowEvent::MouseInput { state, button, .. } => {
                        if !pos.get(*cursor_entity).unwrap().out_of_bound
                            && matches!(state, ElementState::Released)
                            && matches!(button, MouseButton::Left)
                        {}
                    }
                    _ => {}
                }
            }
        }
        Trans::None
    }
}

fn load_sprite_sheet(world: &mut World, texture: &str, sprite_sheet: &str) -> SpriteSheetHandle {
    let texture_handle = {
        let texture_storage = world.read_resource::<AssetStorage<Texture>>();
        world
            .read_resource::<Loader>()
            .load(texture, ImageFormat::default(), (), &texture_storage)
    };

    let board_store = world.read_resource::<AssetStorage<SpriteSheet>>();
    world.read_resource::<Loader>().load(
        sprite_sheet,
        SpriteSheetFormat(texture_handle),
        (),
        &board_store,
    )
}

fn initialize_camara(world: &mut World) {
    let mut transform = Transform::default();
    // transform.set_translation_z(1.0);
    transform.set_translation_xyz(ARENA_WIDTH, ARENA_HEIGHT, 1.0);
    world
        .create_entity()
        .with(Camera::standard_2d(ARENA_WIDTH, ARENA_HEIGHT))
        .with(transform)
        .build();
}

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    let app_root = amethyst::utils::application_root_dir()?;
    let display_config_path = app_root.join("config").join("display.ron");

    let game_data = GameDataBuilder::default()
        .with_bundle(
            RenderingBundle::<DefaultBackend>::new()
                .with_plugin(
                    RenderToWindow::from_config_path(display_config_path)?
                        .with_clear([0.0, 0.0, 0.0, 1.0]),
                )
                .with_plugin(RenderFlat2D::default()),
        )?
        .with_bundle(TransformBundle::new())?
        .with(cursor::CursorSystem, "cursor system", &[]);

    let assets_dir = app_root.join("assets");
    let mut game = Application::new(assets_dir, State, game_data)?;
    game.run();

    Ok(())
}
