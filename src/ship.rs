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
    AnimationIndices
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
    pub cooldown_length: f32,
    pub velocity: Velocity
}

#[derive(Event)]
struct RocketAnimation{
    entity: Entity
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
    pub fn take_damage(&mut self)  {

        // Testing
        self.health = ShipHealth::Empty;

        

        // match self.health {
        //     AsteroidHealth::Full => {
        //         self.health = AsteroidHealth::Damaged;
        //     },
        //     AsteroidHealth::Damaged => {
        //         self.health = AsteroidHealth::VeryDamaged;
        //     },
        //     AsteroidHealth::VeryDamaged => {
        //         self.health = AsteroidHealth::Empty;
                
        //     },
        //     AsteroidHealth::Empty => {
        //         self.health = AsteroidHealth::Empty;
        //     },
        // }

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
                    // scale: Vec2::new(SHIP_GAME_WIDTH / SHIP_TRUE_WIDTH, SHIP_GAME_HEIGHT / SHIP_TRUE_HEIGHT).extend(0.),
                    scale: SHIP_APPLIED_SCALE,
                    ..default()
                },
                sprite: Sprite {
                    // custom_size: Some(SHIP_SPEC),
                    // color: Color::rgba(1.0, 0.0, 0.0, 0.5),
                    ..default()
                }, 
                texture: ship_texture,
                ..default()
            },
            ship: Ship {
                health: ShipHealth::Full,
                invulnerable: false,
                cooldown_length: 1.0,
                velocity: Velocity(Vec2::new(0., 0.))
            },
            // collider: Collider(ColliderShape::Circle),
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
                ASTEROID_TRUE_WIDTH * ASTEROID_APPLIED_SCALE.x / 2.0
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

        // texture_atlas_layouts.ge

        let animation_indices = AnimationIndices {
            first: 0,
            last: 3
        };



        RocketBundle {
            sprite_bundle: SpriteBundle {
                transform: Transform {
                    translation: spawn_location.translation.clone(),
                    // scale: ROCKET_APPLIED_SCALE,
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
            // collider: Collider(ColliderShape::Rectangle),
            collider: Collider {
                name: CollidableComponentNames::Rocket,
                shape: ColliderShape::Rectangle
            },
            rocket: Rocket {
                animation_indices,
                hit_target: false
            }
        }
    }
}

