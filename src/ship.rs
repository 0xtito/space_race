use bevy::{
    math::{bounding::{Aabb2d, BoundingCircle, IntersectsVolume}, vec3, Vec2}, 
    prelude::*
};

use crate::{
    Collider, 
    ColliderShape, 
    CollidableComponentNames,
    Velocity,
    constants::*,
    AnimationIndices,
    AnimationTimer,
    AnimationProperties,
    PlayAnimation
};

#[derive(PartialEq)]
pub enum ShipHealth {
    Full,
    Damaged,
    VeryDamaged,
    Empty
}

#[derive(Component)]
pub struct Ship {
    pub health: ShipHealth,
    pub invulnerable: bool,
    pub invulnerable_timer: AnimationTimer,
    pub animation_indices: AnimationIndices,
    pub cooldown_length: f32,
    pub cooldown_time_left: f32,
}

impl Ship {
    pub fn fire_rocket(
        &mut self, 
        commands: &mut Commands,
        ship_transform: &Transform,
        asset_server: Res<AssetServer>, 
        mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    ) {

        // Means you can now fire the rocket
        // Just for precaution right now because
        // when i call fire_rocket I am making that check + incrementing anyway
        if self.cooldown_time_left == 0.0 {
            let rocket_bundle = RocketBundle::new(
                &asset_server,
                &mut texture_atlas_layouts,
                &ship_transform
            );

            self.cooldown_time_left = self.cooldown_length;

            commands.spawn(rocket_bundle).insert(PlayAnimation(crate::AnimatableAsset::Rocket));
        }
        
    }
    pub fn take_damage(&mut self) -> ShipHealth  {

        // Testing
        // self.health = ShipHealth::Empty;

        match self.health {
            ShipHealth::Full => {
                self.health = ShipHealth::Damaged;
                ShipHealth::Damaged
            },
            ShipHealth::Damaged => {
                self.health = ShipHealth::VeryDamaged;
                ShipHealth::VeryDamaged
            },
            ShipHealth::VeryDamaged => {
                self.health = ShipHealth::Empty;
                ShipHealth::Empty
                
            },
            ShipHealth::Empty => {
                self.health = ShipHealth::Empty;
                ShipHealth::Empty
            },
        }



    }
}

#[derive(Bundle)]
pub struct ShipBundle {
    ship: Ship,
    sprite_bundle: SpriteSheetBundle,
    collider: Collider,
}

impl ShipBundle {

    pub fn new(
        ship_texture: Handle<Image>,
        texture_atlas_layouts: &mut ResMut<Assets<TextureAtlasLayout>>,
        cooldown_length: f32
    ) -> ShipBundle {

        let ship_layout = TextureAtlasLayout::from_grid(Vec2::new(48.0, 48.0), 5, 1, None, None);

        let texture_atlas_layout = texture_atlas_layouts.add(ship_layout);

        let animation_indices = AnimationIndices {
            first: 1,
            last: 4
        };


        ShipBundle {
            ship: Ship {
                health: ShipHealth::Full,
                invulnerable: false,
                invulnerable_timer: AnimationTimer(Timer::from_seconds(2.0, TimerMode::Repeating)),
                animation_indices,
                cooldown_length,
                cooldown_time_left: cooldown_length
            },
            sprite_bundle: SpriteSheetBundle {
                transform: Transform {
                    translation: vec3(0.0, 0.0, 0.0),
                    scale: SHIP_APPLIED_SCALE,
                    ..default()
                },
                sprite: Sprite {
                    ..default()
                }, 
                atlas: TextureAtlas {
                    layout: texture_atlas_layout,
                    index: 1
                },
                texture: ship_texture,
                ..default()
            },
            collider: Collider {
                name: CollidableComponentNames::Ship,
                shape: ColliderShape::Circle
            }
        }
    }
}




#[derive(Component, Debug)]
pub struct Rocket {
    pub animation_indices: AnimationIndices,
    pub animation_timer: AnimationTimer,
    pub hit_target: bool
}

impl Rocket {
    pub fn is_outside_window(&self, rocket_transform: &Transform, camera_transform: &Transform) -> bool {

        let despawn_threshold: f32 = TOP_WALL + WALL_THICKNESS / 2.0 + SHIP_SPEC.y / 2.0 + SHIP_PADDING + camera_transform.translation.y;

        rocket_transform.translation.y > despawn_threshold 
    }

    pub fn check_collision(
        &mut self, 
        rocket_transform: &Transform, 
        other_transform: &Transform, 
        other_name: &CollidableComponentNames
    ) -> bool {

        let rocket_rectangle = Aabb2d::new(
            rocket_transform.translation.truncate(),
            rocket_transform.scale.truncate() / 2.0
        );

        if *other_name == CollidableComponentNames::Asteroid {

            let asteroid_circle = BoundingCircle::new(
                other_transform.translation.truncate(),
                ASTEROID_SCALED_RADIUS
            );

            match rocket_rectangle.intersects(&asteroid_circle) {
                true => {

                    
                    true
                },
                false => {
                    false
                }
            }
        } else { false }

    }
}

#[derive(Bundle)]
pub struct RocketBundle {
    rocket: Rocket,
    sprite_bundle: SpriteSheetBundle,
    collider: Collider,
}

impl RocketBundle {

    pub fn new(
        asset_server: &Res<AssetServer>, 
        texture_atlas_layouts: &mut ResMut<Assets<TextureAtlasLayout>>, 
        spawn_location: &Transform
    ) -> RocketBundle {
        let rocket_texture = asset_server.load("weapons/rocket_sprites_3.png");

        let layout = TextureAtlasLayout::from_grid(Vec2::new(32.0, 32.0), 3, 1, None, None);

        let texture_atlas_layout = texture_atlas_layouts.add(layout);

        let animation_indices = AnimationIndices {
            first: 0,
            last: 2
        };



        RocketBundle {
            sprite_bundle: SpriteSheetBundle {
                transform: Transform {
                    translation: spawn_location.translation.clone(),
                    scale: ROCKET_APPLIED_SCALE,
                    ..default()
                },
                sprite: Sprite {
                    ..default()
                }, 
                atlas: TextureAtlas {
                    layout: texture_atlas_layout,
                    index: 0
                },
                texture: rocket_texture,
                ..default()
            },
            collider: Collider {
                name: CollidableComponentNames::Rocket,
                shape: ColliderShape::Rectangle
            },
            rocket: Rocket {
                animation_indices,
                animation_timer: AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
                hit_target: false
            },
        }
    }
}

