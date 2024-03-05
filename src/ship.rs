use bevy::{
    math::{vec3, Vec2}, 
    prelude::*
};

use crate::{
    Collider, 
    ColliderShape, 
    Velocity, 
    ROCKET_SPEC, 
    SHIP_PADDING, 
    SHIP_SPEC, 
    TOP_WALL, 
    WALL_THICKNESS
};

pub enum ShipHealth {
    Full,
    Damaged,
    VeryDamaged,
    Empty
}

#[derive(Component)]
pub struct Ship {
    pub health: ShipHealth,
    pub cooldown_length: f32,
    pub velocity: Velocity
}

impl Ship {
    pub fn fire_rocket(
        &mut self, 
        commands: &mut Commands,
        ship_transform: &Transform,
        asset_server: Res<AssetServer>, 
        mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>
    ) {

        // Means you can now fire the rocket
        if self.cooldown_length == 0.0 {
            let rocket_bundle = RocketBundle::new(
                &asset_server,
                &mut texture_atlas_layouts,
                &ship_transform
            );

            self.cooldown_length = 1.0;

            commands.spawn(rocket_bundle);
        }
        
    }
}

#[derive(Bundle)]
pub struct ShipBundle {
    ship: Ship,
    sprite_bundle: SpriteBundle,
    collider: Collider,
}

impl ShipBundle {

    pub fn new(ship_texture: Handle<Image>) -> ShipBundle {

        ShipBundle {
            sprite_bundle: SpriteBundle {
                transform: Transform {
                    translation: vec3(0.0, 0.0, 0.0),
                    ..default()
                },
                sprite: Sprite {
                    custom_size: Some(SHIP_SPEC),
                    ..default()
                }, 
                texture: ship_texture,
                ..default()
            },
            ship: Ship {
                health: ShipHealth::Full,
                cooldown_length: 1.0,
                velocity: Velocity(Vec2::new(0., 0.))
            },
            collider: Collider(ColliderShape::Circle),
            
        }
    }
}


#[derive(Component)]
struct AnimationIndices {
    first: usize,
    last: usize,
}

#[derive(Component)]
pub struct Rocket {
    animation_indices: AnimationIndices
}

impl Rocket {
    pub fn is_outside_window(&self, rocket_transform: &Transform, camera_transform: &Transform) -> bool {

        let despawn_threshold: f32 = TOP_WALL + WALL_THICKNESS / 2.0 + SHIP_SPEC.y / 2.0 + SHIP_PADDING + camera_transform.translation.y;

        rocket_transform.translation.y > despawn_threshold 
    }
}

#[derive(Bundle)]
pub struct RocketBundle {
    rocket: Rocket,
    sprite_bundle: SpriteBundle,
    texture_atlas: TextureAtlas,
    collider: Collider,
}

impl RocketBundle {

    pub fn new(
        asset_server: &Res<AssetServer>, 
        mut texture_atlas_layouts: &mut ResMut<Assets<TextureAtlasLayout>>, 
        spawn_location: &Transform
    ) -> RocketBundle {
        let rocket_texture = asset_server.load("weapons/rocket_sprites_3.png");

        let layout = TextureAtlasLayout::from_grid(Vec2::new(32.0, 32.0), 3, 1, Some(Vec2::new(0., 0.)), Some(Vec2::new(0., 0.)));

        let texture_atlas_layout = texture_atlas_layouts.add(layout);

        let animation_indices = AnimationIndices {
            first: 0,
            last: 3
        };



        RocketBundle {
            sprite_bundle: SpriteBundle {
                transform: Transform {
                    translation: spawn_location.translation.clone(),
                    ..default()
                },
                sprite: Sprite {
                    custom_size: Some(ROCKET_SPEC),
                    ..default()
                }, 
                texture: rocket_texture,
                ..default()
            },
            texture_atlas: TextureAtlas {
                layout: texture_atlas_layout,
                index: animation_indices.first
            },
            collider: Collider(ColliderShape::Rectangle),
            rocket: Rocket {
                animation_indices
            }
        }
    }
}

