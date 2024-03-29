#![allow(dead_code)]

use bevy::{
    math::{Vec2, Vec3}, 
    render::color::Color
};

pub const WINDOW_HEIGHT: f32 = 600.;
pub const WINDOW_WIDTH: f32 = 360.;

pub const GRID_SIZE: f32 = 40.0;

pub const BACKGROUND_DIMENSIONS: Vec2 = Vec2::new(360.0, 5760.0);

// will eventually be decided by the players
pub const KINEMATIC_OBJECTS_SPEED: f32 = 150.;




// pub const SHIP_SPRITE_WIDTH: f32 = 50.0;
// pub const SHIP_SPRITE_HEIGHT: f32 = 50.0;
pub const SHIP_GAME_WIDTH: f32 = 50.0;
pub const SHIP_GAME_HEIGHT: f32 = 50.0;
pub const SHIP_TRUE_WIDTH: f32 = 27.0;
pub const SHIP_TRUE_HEIGHT: f32 = 25.0;
pub const SHIP_SPEED: f32 = 100.;
pub const SHIP_SPEC: Vec2 = Vec2::new(SHIP_GAME_WIDTH, SHIP_GAME_HEIGHT);
pub const SHIP_PADDING: f32 = 0.0;
pub const SHIP_APPLIED_SCALE: Vec3 = Vec3::new(1.5, 1.5, 1.0);


pub const MAGNITUDE_FORCE: f32 = 1.5;



// Asteroid
pub const ASTEROID_GAME_WIDTH: f32 = 100.0;
pub const ASTEROID_GAME_HEIGHT: f32 = 100.0;
pub const ASTEROID_TRUE_WIDTH: f32 = 37.0;
pub const ASTEROID_TRUE_HEIGHT: f32 = 32.0;
pub const ASTEROID_APPLIED_SCALE: Vec3 = Vec3::new(1.5, 1.5, 1.0);
pub const ASTEROID_SCALED_RADIUS: f32 = ASTEROID_TRUE_WIDTH * ASTEROID_APPLIED_SCALE.x / 2.0;

// Rocket
pub const ROCKET_SPEED: f32 = 300.;
pub const ROCKET_GAME_WIDTH: f32 = 50.0;
pub const ROCKET_GAME_HEIGHT: f32 = 50.0;
pub const ROCKET_TRUE_WIDTH: f32 = 6.0;
pub const ROCKET_TRUE_HEIGHT: f32 = 11.0;
pub const ROCKET_SPEC: Vec2 = Vec2::new(ROCKET_GAME_WIDTH, ROCKET_GAME_HEIGHT);
pub const ROCKET_APPLIED_SCALE: Vec3 = Vec3::new(1.5, 1.5, 1.0);
pub const ASTEROID_HALF_SIZE: Vec2 = Vec2::new(
    ROCKET_TRUE_WIDTH,
    ROCKET_TRUE_WIDTH
);



// Default Wall Positions
pub const TOP_WALL: f32 = WINDOW_HEIGHT / 2.;
pub const BOTTOM_WALL: f32 = -WINDOW_HEIGHT / 2.;
pub const RIGHT_WALL: f32 = WINDOW_WIDTH / 2.;
pub const LEFT_WALL: f32 = -WINDOW_WIDTH / 2.;

pub const WALL_THICKNESS: f32 = 10.0;
pub const TRANSPARENT_WALL_COLOR: Color = Color::rgba(0.0, 0.0, 0.0, 0.0);
pub const WALL_COLOR: Color = Color::rgb(0.0, 0.0, 0.0);