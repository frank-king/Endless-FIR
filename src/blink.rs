use amethyst::core::{Hidden, Time};
use amethyst::ecs::*;
use amethyst::{GameData, SimpleState, SimpleTrans, StateData, Trans};
use log::info;
use std::time::Duration;

use crate::{BonusTurn, Turn};

pub struct PiecesBlinkState {
    pub fir: [Entity; 5],
    pub time: Duration,
    pub turn: Turn,
}

impl PiecesBlinkState {
    pub fn new(fir: [Entity; 5], time: Duration, turn: Turn) -> Self {
        Self { fir, time, turn }
    }
}

impl SimpleState for PiecesBlinkState {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        info!("in PiecesBlinkState ");
        let world = data.world;
        let mut blink_storage = world.write_storage::<Blink>();
        for entity in self.fir.iter() {
            let blink = Blink {
                delay: 0.6,
                timer: 0.0,
            };
            blink_storage
                .insert(*entity, blink)
                .expect("unable to insert blink");
        }
    }

    fn on_stop(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;
        world.fetch_mut::<BonusTurn>().0 = true;
        *world.fetch_mut::<Turn>() = self.turn;
        let entities = world.entities_mut();
        self.fir
            .iter()
            .inspect(|entity| info!("entity {:?} removed", entity))
            .for_each(|entity| entities.delete(*entity).expect("unable to delete entity"));
    }

    fn fixed_update(&mut self, data: StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        let world = data.world;
        let delta = world.fetch::<Time>().delta_time();
        if self.time < delta {
            return Trans::Pop;
        } else {
            self.time -= delta;
        }
        Trans::None
    }
}

#[derive(Component)]
pub struct Blink {
    pub delay: f32,
    pub timer: f32,
}

pub trait ToggleHidden {
    fn toggle_hidden(hiddens: &mut WriteStorage<Hidden>, show: bool, entity: Entity) {
        match (show, hiddens.contains(entity)) {
            (false, false) => hiddens
                .insert(entity, Hidden)
                .expect("unable to insert entity"),
            (true, true) => hiddens.remove(entity),
            _ => None,
        };
    }
}

pub struct BlinkSystem;

impl ToggleHidden for BlinkSystem {}

impl<'a> System<'a> for BlinkSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, Hidden>,
        WriteStorage<'a, Blink>,
        Read<'a, Time>,
    );

    fn run(&mut self, (entities, mut hiddens, mut blinks, time): Self::SystemData) {
        let abs_sec = time.delta_seconds();

        for (entity, blink) in (&*entities, &mut blinks).join() {
            blink.timer += abs_sec;

            if blink.timer > blink.delay {
                blink.timer -= blink.delay;
            }

            Self::toggle_hidden(&mut hiddens, blink.timer < blink.delay / 2.0, entity);
        }
    }
}
