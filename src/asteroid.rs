
use bevy::{
    math::{
        bounding::{Aabb2d, BoundingCircle, IntersectsVolume}, 
        Vec2
    }, 
    prelude::*, 
    sprite::Sprite
};

use rand::Rng;

use crate::{
    constants::*, 
    AnimationIndices, 
    AnimationProperties, 
    AnimationTimer, 
    CollidableComponentNames, 
    Collider, 
    ColliderShape
};

#[derive(Component, Debug)]
pub struct Asteroid {
    pub exploding: bool,
}

impl Asteroid {
    pub fn take_damage(&mut self)  {
        self.exploding = true;
    }
    
    pub fn is_outside_window(&self, asteroid_transform: &Transform, camera_transform: &Transform) -> bool {

        let lower_bound: f32 = BOTTOM_WALL - WALL_THICKNESS / 2.0 - SHIP_SPEC.y / 2.0 - SHIP_PADDING + camera_transform.translation.y;


        asteroid_transform.translation.y < lower_bound 
        
    }

    pub fn check_collision(
        &mut self, 
        asteroid_transform: &Transform, 
        other_transform: &Transform, 
        other_name: &CollidableComponentNames,
    ) -> bool {
        let asteroid_circle = BoundingCircle::new(
            asteroid_transform.translation.truncate(),
            ASTEROID_SCALED_RADIUS
        );

        match other_name {
            // Rocket and Asteroid Collide
            CollidableComponentNames::Rocket => {

                let rocket_rectangle = Aabb2d::new(
                    other_transform.translation.truncate(),
                    other_transform.scale.truncate() / 2.0
                );

                match asteroid_circle.intersects(&rocket_rectangle) {
                    true => {
                        self.take_damage();
                        true
                    },
                    false => {
                        false
                    }
                }
            }
            // Ship and Asteroid Collide
            CollidableComponentNames::Ship => {

                let ship_circle = BoundingCircle::new(
                    other_transform.translation.truncate(),
                    SHIP_TRUE_WIDTH * SHIP_APPLIED_SCALE.x / 2.0
                );

                if asteroid_circle.intersects(&ship_circle) {
                    
                    true
                } else {
                    false
                }
            }
            // Asteroid to Asteroid Collide
            CollidableComponentNames::Asteroid => {
                println!("dn check_collision | Asteroid to Asteroid -  Should not be in here!");
                false
            }
        }
    }
}


#[derive(Bundle)]
pub struct AsteroidBundle {
    pub asteroid: Asteroid,
    pub sprite_bundle: SpriteSheetBundle,
    pub collider: Collider,
    pub animation: AnimationProperties
}

impl AsteroidBundle {
    pub fn new(
        texture: Handle<Image>, 
        camera_transform: &Transform ,
        texture_atlas_layouts: &mut ResMut<Assets<TextureAtlasLayout>>,
        spawn_location: Option<Vec3>
    ) -> AsteroidBundle {

        let camera_translation_y = camera_transform.translation.y;

        let x = rand::thread_rng().gen_range(0.0..=1.0) * WINDOW_WIDTH - WINDOW_WIDTH / 2.0;
        let y: f32 = camera_translation_y + WINDOW_HEIGHT / 2.0;

        let asteroid_layout = TextureAtlasLayout::from_grid(Vec2::new(96.0, 96.0), 8, 1, None, None);

        let texture_atlas_layout = texture_atlas_layouts.add(asteroid_layout);


        let animation_indices = AnimationIndices { first: 0, last: 7 };


        AsteroidBundle {
            asteroid: Asteroid {
                exploding: false,
            },
            sprite_bundle: SpriteSheetBundle {
                transform: Transform {
                    translation: if spawn_location.is_some() { spawn_location.unwrap() } else { Vec2::new(x, y).extend(0.) },
                    scale: ASTEROID_APPLIED_SCALE,
                    ..default()
                },
                sprite: Sprite {
                    ..default()
                },
                atlas: TextureAtlas {
                    layout: texture_atlas_layout,
                    index: 0,
                },
                texture: texture.clone(),
                ..default()
            },
            collider: Collider {
                name: CollidableComponentNames::Asteroid,
                shape: ColliderShape::Circle
            },
            animation: AnimationProperties {
                // asset: crate::AnimatableAsset::Asteroid,
                indices: animation_indices,
                timer: AnimationTimer(Timer::from_seconds(0.12, TimerMode::Once))
            }
        }
    }
}