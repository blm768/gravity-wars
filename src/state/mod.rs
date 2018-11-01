use std::fmt::Debug;

use cgmath::Vector2;
use specs::prelude::*;
use specs::shred::Fetch;
use specs_derive::Component;

use rendering::scene::Camera;

pub mod mapgen;

#[derive(Debug, Component)]
#[storage(VecStorage)]
pub struct Position {
    position: Vector2<f32>,
}

impl Position {
    pub fn new(x: f32, y: f32) -> Position {
        Position {
            position: Vector2::new(x, y),
        }
    }
}

pub trait EntityRenderer: Debug {
    fn render(&self, entity: &Entity);
}

#[derive(Debug, Component)]
#[storage(VecStorage)]
pub struct Renderable {
    renderer: Box<EntityRenderer + Send + Sync>,
}

pub struct RenderEntities {}

impl<'a> System<'a> for RenderEntities {
    type SystemData = (Entities<'a>, ReadStorage<'a, Renderable>);

    fn run(&mut self, (entities, renderables): Self::SystemData) {
        for (entity, renderable) in (&entities, &renderables).join() {
            renderable.renderer.render(&entity)
        }
    }
}

pub struct GameState {
    pub world: World,
}

impl GameState {
    pub fn new() -> GameState {
        let mut world = World::new();
        world.register::<Position>();

        let camera = Camera::new();
        world.add_resource(camera);

        GameState { world }
    }

    pub fn camera(&self) -> Fetch<Camera> {
        self.world.read_resource::<Camera>()
    }

    pub fn render(self) {
        let mut renderer = RenderEntities {};
        renderer.run_now(&self.world.res);
    }
}
