
use bevy::{
    math::Vec2, 
    prelude::*, 
    sprite::Sprite
};

use rand::Rng;



use crate::{
    constants::*,
    Collider, 
    ColliderShape,
};

#[derive(Component)]
pub struct Asteroid {
    health: AsteroidHealth,
}

impl Asteroid {
    pub fn reset(&mut self, camera_transform: &Transform) -> Vec2 {


        let x = rand::thread_rng().gen_range(0.0..=1.0) * WINDOW_WIDTH - WINDOW_WIDTH / 2.0;
        let y = (WINDOW_HEIGHT / 2.0) + camera_transform.translation.y;

        Vec2::new(x, y)
    }

    pub fn take_damage(&mut self)  {

        match self.health {
            AsteroidHealth::Full => {
                self.health = AsteroidHealth::Damaged;
            },
            AsteroidHealth::Damaged => {
                self.health = AsteroidHealth::VeryDamaged;
            },
            AsteroidHealth::VeryDamaged => {
                self.health = AsteroidHealth::Damaged;
            },
            AsteroidHealth::Empty => {
                self.health = AsteroidHealth::Empty;
            },
        }
    }
    pub fn is_outside_window(&self, asteroid_transform: &Transform, camera_transform: &Transform) -> bool {

        let lower_bound: f32 = BOTTOM_WALL - WALL_THICKNESS / 2.0 - SHIP_SPEC.y / 2.0 - SHIP_PADDING + camera_transform.translation.y;


        asteroid_transform.translation.y < lower_bound 
        
    }
}

pub enum AsteroidHealth {
    Full,
    Damaged,
    VeryDamaged,
    Empty
}

#[derive(Bundle)]
pub struct AsteroidBundle {
    asteroid: Asteroid,
    sprite_bundle: SpriteBundle,
    collider: Collider,
}

impl AsteroidBundle {
    pub fn new(texture: Handle<Image>) -> AsteroidBundle {
        let x = rand::thread_rng().gen_range(0.0..=1.0) * WINDOW_WIDTH - WINDOW_WIDTH / 2.0;
        let y: f32 = WINDOW_HEIGHT / 2.0;

        AsteroidBundle {
            asteroid: Asteroid {
                health: AsteroidHealth::Full
            },
            sprite_bundle: SpriteBundle {
                transform: Transform {
                    translation: Vec2::new(x, y).extend(0.),
                    ..default()
                },
                sprite: Sprite {
                    custom_size: Some(Vec2::new(ASTEROID_GAME_WIDTH, ASTEROID_GAME_HEIGHT)),
                    ..default()
                },
                texture: texture.clone(),
                ..default()
            },
            collider: Collider(ColliderShape::Circle),
        }
    }
}