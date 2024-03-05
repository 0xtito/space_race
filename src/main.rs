// use bevy::{math::{bounding::{Aabb2d, BoundingCircle, IntersectsVolume}, *}, prelude::*};
use bevy::{
    math::{bounding::{Aabb2d, BoundingCircle, BoundingVolume, IntersectsVolume}, *}, prelude::*
};
use rand::Rng;

const WINDOW_HEIGHT: f32 = 600.;
const WINDOW_WIDTH: f32 = 360.;

// const BACKGROUND_DIMENSIONS: f32 = 5760;
const BACKGROUND_DIMENSIONS: Vec2 = Vec2::new(360.0, 5760.0);


const SHIP_WIDTH: f32 = 50.0;
const SHIP_HEIGHT: f32 = 50.0;
const SHIP_SPEED: f32 = 100.;
const SHIP_SPEC: Vec2 = Vec2::new(SHIP_WIDTH, SHIP_HEIGHT);
const SHIP_PADDING: f32 = 0.0;

const MAGNITUDE_FORCE: f32 = 1.5;


// Default Wall Positions
const TOP_WALL: f32 = WINDOW_HEIGHT / 2.;
const BOTTOM_WALL: f32 = -WINDOW_HEIGHT / 2.;
const RIGHT_WALL: f32 = WINDOW_WIDTH / 2.;
const LEFT_WALL: f32 = -WINDOW_WIDTH / 2.;

const WALL_THICKNESS: f32 = 10.0;
const WALL_COLOR: Color = Color::rgb(0.0, 0.0, 0.0);



#[derive(Component)]
struct GameCamera;

#[derive(Component)]
struct KinematicObject;


#[derive(Component)]
struct Ship;

#[derive(Component, Deref, DerefMut)]
struct Velocity(Vec2);

#[derive(Component)]
struct MovementMagnitude {
    x: f32,
    y: f32,
}



#[derive(Component)]
struct Collider;

#[derive(Event, Default)]
struct CollisionEvent;


#[derive(Component)]
struct Asteroid {
    health: AsteroidHealth,
}

impl Asteroid {
    fn reset() -> Vec2 {
        let x = rand::thread_rng().gen_range(0.0..=1.0) * WINDOW_WIDTH - WINDOW_WIDTH / 2.0;
        let y = WINDOW_HEIGHT / 2.0;

        Vec2::new(x, y)
    }

    fn take_damage(&mut self)  {

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
}

enum AsteroidHealth {
    Full,
    Damaged,
    VeryDamaged,
    Empty
}

#[derive(Bundle)]
struct AsteroidBundle {
    asteroid: Asteroid,
    sprite_bundle: SpriteBundle,
    collider: Collider,
}

impl AsteroidBundle {
    fn new(texture: Handle<Image>) -> AsteroidBundle {
        let x = rand::thread_rng().gen_range(0.0..=1.0) * WINDOW_WIDTH - WINDOW_WIDTH / 2.0;
        let y = WINDOW_HEIGHT / 2.0;

        AsteroidBundle {
            asteroid: Asteroid {
                health: AsteroidHealth::Full
            },
            sprite_bundle: SpriteBundle {
                transform: Transform::from_translation(Vec3::new(x, y, 0.0)),
                texture: texture.clone(),
                ..Default::default()
            },
            collider: Collider,
        }
    }
}

#[derive(Component)]
struct Wall;

enum GameWall {
    Top,
    Bottom,
    Left,
    Right
}

impl GameWall {
    fn position(&self) -> Vec2 {
        match self {
            GameWall::Top => Vec2::new(0., TOP_WALL),
            GameWall::Bottom => Vec2::new(0., BOTTOM_WALL),
            GameWall::Right => Vec2::new(RIGHT_WALL, 0.),
            GameWall::Left => Vec2::new(LEFT_WALL, 0.)
        }
    }

    fn size(&self) -> Vec2 {
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
struct WallBundle {
    sprite_bundle: SpriteBundle,
    collider: Collider,
}

impl WallBundle {
    
    fn new(location: GameWall) -> WallBundle {
        WallBundle {
            sprite_bundle: SpriteBundle {
                transform: Transform {
                    translation: location.position().extend(0.0),
                    scale: location.size().extend(1.0),
                    ..default()
                },
                sprite: Sprite {
                    color: WALL_COLOR,
                    ..default()
                },
                ..default()
            },
            collider: Collider
        }
    }
}


/**
 * Overview of the game:
 *   - The player must live as long as possible
 *   - The player can move left, right, up, and down - but cannot move outside of the window.
 *   - Asteroids 
 *   - The player can destroy enemies
 */
fn main() {
    // println!("Hello, world!");

    // Window 
    let primary_window = Window {
        title: "Space Shooter".to_string(),
        resolution: (WINDOW_WIDTH, WINDOW_HEIGHT).into(),
        resizable: false,
        ..default()
    };


    App::new()
        .add_plugins(DefaultPlugins.set( WindowPlugin {
            primary_window: Some(primary_window),
            ..default()
        }).set(ImagePlugin::default_nearest()) )
        .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .add_event::<CollisionEvent>()
        .add_systems(Update, bevy::window::close_on_esc)
        .add_systems(Startup, setup)
        .add_systems(FixedUpdate, (
            ship_movement, 
            update_kinematic_objects,
            // update_asteroids
        ).chain())
        // .add_systems(FixedUpdate, (update_kinematic_objects, ship_movement, collision_checks).chain())
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {

    // Camera setup
    commands.spawn(Camera2dBundle::default()).insert((GameCamera, KinematicObject));

    // Background setup
    let void_layer_1_texture = asset_server.load("background/void_layer_1.png");
    let stars_layer_2_texture = asset_server.load("background/stars_layer_2.png");
    let stars_layer_3_texture = asset_server.load("background/stars_layer_3.png");

    let background_vec = vec![
        void_layer_1_texture,
        stars_layer_2_texture,
        stars_layer_3_texture
    ];

    // Not this most optimal way but works for now
    load_in_background(&mut commands, background_vec, 4);


    let ship_texture = asset_server.load("ship/ship_full.png");
    // Spawn Spaceship
    commands.spawn((
        SpriteBundle {
            transform: Transform {
                translation: vec3(0.0, 0.0, 0.0),
                ..default()
            },
            sprite: Sprite {
                // custom_size: Some(vec2(SHIP_WIDTH, SHIP_HEIGHT)),
                custom_size: Some(SHIP_SPEC),
                ..default()
            }, 
            texture: ship_texture,
            ..default()
        },
        Ship,
        Collider,
        Velocity(Vec2::new(0., 0.))
    ));

    // Spawn Walls
    commands.spawn(WallBundle::new(GameWall::Top)).insert(KinematicObject);
    commands.spawn(WallBundle::new(GameWall::Bottom)).insert(KinematicObject);
    commands.spawn(WallBundle::new(GameWall::Right)).insert(KinematicObject);
    commands.spawn(WallBundle::new(GameWall::Left)).insert(KinematicObject);

    
    // Spawn Start Asteroids
    let asteroid_texture = asset_server.load("enemys/asteriod_base.png");

    commands.spawn(AsteroidBundle::new(asteroid_texture));

    // commands.spawn()



}

fn update_kinematic_objects(time: Res<Time<Fixed>>, mut query: Query<&mut Transform, With<KinematicObject>>) {
    for mut transform in query.iter_mut() {
        transform.translation.y += SHIP_SPEED * time.delta_seconds();
    }
}

// fn update_asteroids(
//     asteroid_query: Query<(&mut Transform, &mut Velocity), With<>>,
//     time: Res<Time<Fixed>>,
// ) {
// //
// }


fn collision_checks(
    // mut commands: Commands, 
    // time: Res<Time<Fixed>>, 
    mut ship_query: Query<(&mut Velocity, &mut Transform), With<Ship>>, 
    asset_collider: Query<(Entity, &Transform), (With<Collider>)>,
    mut collision_events: EventWriter<CollisionEvent>,
) {
    let (mut ship_velocity, ship_transform) = ship_query.single_mut();

    for ( _collider, transform  ) in &asset_collider {

        // 
        
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
enum Collision {
    Top,
    Bottom,
    Right,
    Left
}

fn collision_check_ship_wall(ship: BoundingCircle, wall: Aabb2d) -> Option<Collision> {

    if !ship.intersects(&wall) {
        return None;
    }

    let closet_point_on_wall = wall.closest_point(ship.center());
    let offset = ship.center() - closet_point_on_wall;

    let contact_side = if offset.x.abs() > offset.y.abs() {
        if offset.x < 0.0 {
            Collision::Left
        } else {
            Collision::Right
        }
    } else {
        if offset.y > 0.0 {
            Collision::Top
        } else {
            Collision::Bottom
        }
    };

    Some(contact_side)


}

fn get_upper_bounds(camera_query: &Query<&Transform, (With<GameCamera>, Without<Ship>)> ) -> (f32, f32) {
    let camera_transform = *camera_query.single();

    let upper_bound: f32 = TOP_WALL - WALL_THICKNESS / 2.0 - SHIP_SPEC.y / 2.0 - SHIP_PADDING + camera_transform.translation.y;
    let lower_bound: f32 = BOTTOM_WALL - WALL_THICKNESS / 2.0 + SHIP_SPEC.y / 2.0 + SHIP_PADDING + camera_transform.translation.y;

    (upper_bound, lower_bound)
}   

fn ship_movement(
    keyboard_input: Res<ButtonInput<KeyCode>>, 
    timestep: Res<Time<Fixed>>, 
    mut ship_query: Query<(&mut Transform, &mut Velocity), With<Ship>>,
    camera_query: Query<&Transform, (With<GameCamera>, Without<Ship>)>
) {

    // let camera_transform = camera_query.single();

    let ( upper_bound, lower_bound ) = get_upper_bounds(&camera_query);

    let left_bound: f32 = LEFT_WALL + (SHIP_SPEC.x / 2.0) + SHIP_PADDING;
    let right_bound: f32 = RIGHT_WALL - (SHIP_SPEC.x / 2.0) - SHIP_PADDING;

    for (mut transform, mut _velocity) in ship_query.iter_mut() {
        let mut magnitude = MovementMagnitude {
            x: 0.,
            y: 1.
        };

        if keyboard_input.pressed(KeyCode::KeyW) || keyboard_input.pressed(KeyCode::ArrowUp) {
            magnitude.y += MAGNITUDE_FORCE;
        }

        if keyboard_input.pressed(KeyCode::KeyS) || keyboard_input.pressed(KeyCode::ArrowDown) {
            magnitude.y -= MAGNITUDE_FORCE;

        }

        if keyboard_input.pressed(KeyCode::KeyA) || keyboard_input.pressed(KeyCode::ArrowLeft) {
            magnitude.x -= MAGNITUDE_FORCE;

        }

        if keyboard_input.pressed(KeyCode::KeyD) || keyboard_input.pressed(KeyCode::ArrowRight) {
            magnitude.x += MAGNITUDE_FORCE;
        }

        

        let new_vel_x: f32 = magnitude.x * SHIP_SPEED * timestep.delta_seconds();
        let new_ship_position_x = transform.translation.x + new_vel_x;

        let new_vel_y: f32 = magnitude.y * SHIP_SPEED * timestep.delta_seconds();
        let new_ship_position_y = transform.translation.y + new_vel_y;

        // let upper_bound: f32 = TOP_WALL - WALL_THICKNESS / 2.0 - SHIP_SPEC.y / 2.0 - SHIP_PADDING + camera_transform.translation.y;
        // let lower_bound: f32 = BOTTOM_WALL - WALL_THICKNESS / 2.0 + SHIP_SPEC.y / 2.0 + SHIP_PADDING + camera_transform.translation.y;

        transform.translation.y = new_ship_position_y.clamp(lower_bound, upper_bound);




        transform.translation.x = new_ship_position_x.clamp(left_bound, right_bound);
        // transform.translation.y += new_vel_y;



        

        // velocity.0.x = magnitude.x * SHIP_SPEED * timestep.delta_seconds();
        // velocity.0.y = magnitude.y * SHIP_SPEED * timestep.delta_seconds();
        // velocity.0 = direction.normalize();
        // transform.translation += velocity.0.extend(0.) * SHIP_SPEED * timestep.delta_seconds();
        // transform.translation += velocity.0.extend(1.);



    }

}

fn load_in_background(
    commands: &mut Commands, 
    backgrounds: Vec<Handle<Image>>,
    repeat: u8
) {

    for i in 0..repeat {
        println!("{}", i);
        for bg_texture in backgrounds.iter() {            
            commands.spawn(SpriteBundle {
                transform: Transform {
                    translation: Vec3::new(0., 0.0 + (i as f32 * BACKGROUND_DIMENSIONS.y), 0.0),
                    rotation: Quat::from_rotation_y(std::f32::consts::PI),
                    ..default()
                },
                sprite: Sprite {
                    ..default()
                },
                texture: bg_texture.clone(),
                ..default()
            });
        }
        
    }




}

