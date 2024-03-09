use bevy::{
    math::Vec2, prelude::*
};

use crate::{
    constants::*,
    // Collider, ColliderShape
};

#[derive(Component)]
struct Wall;

pub enum GameWall {
    Top,
    Bottom,
    Left,
    Right
}

impl GameWall {
    pub fn position(&self) -> Vec2 {

        match self {
            GameWall::Top => Vec2::new(0., TOP_WALL),
            GameWall::Bottom => Vec2::new(0., BOTTOM_WALL),
            GameWall::Right => Vec2::new(RIGHT_WALL, 0.),
            GameWall::Left => Vec2::new(LEFT_WALL, 0.)
        }
    }

    pub fn size(&self) -> Vec2 {
        let box_height = TOP_WALL - BOTTOM_WALL;
        let box_width = RIGHT_WALL - LEFT_WALL;

        assert!(box_height > 0.);
        assert!(box_width > 0.);

        match self {
            GameWall::Left | GameWall::Right => {
                Vec2::new(WALL_THICKNESS, box_height + WALL_THICKNESS)
            }
            GameWall::Top | GameWall::Bottom => {
                Vec2::new(box_width + WALL_THICKNESS, WALL_THICKNESS)
            }
        }
    }
}

#[derive(Bundle)]
pub struct WallBundle {
    sprite_bundle: SpriteBundle,
    // collider: Collider,
}

impl WallBundle {
    
    pub fn new(location: GameWall) -> WallBundle {
        WallBundle {
            sprite_bundle: SpriteBundle {
                transform: Transform {
                    translation: location.position().extend(0.0),
                    scale: location.size().extend(1.0),
                    ..default()
                },
                sprite: Sprite {
                    color: TRANSPARENT_WALL_COLOR,
                    ..default()
                },
                ..default()
            },
        }
    }
}
