mod ship;
mod asteriod;
mod constants;
mod wall;

use ship::*;
use wall::*;
use asteriod::*;
use constants::*;

use bevy::{
    math::*, prelude::*
};

#[derive(Component)]
struct GameCamera;

#[derive(Component)]
struct KinematicObject;


#[derive(Component, Deref, DerefMut)]
struct Velocity(Vec2);

#[derive(Component)]
struct MovementMagnitude {
    x: f32,
    y: f32,
}


enum ColliderShape {
    Circle,
    Rectangle
}


#[derive(Component)]
struct Collider(ColliderShape);

#[derive(Event, Default)]
struct CollisionEvent;


/**
 * Overview of the game:
 *   - The player must live as long as possible
 *   - The player can move left, right, up, and down - but cannot move outside of the window.
 *   - Asteroids 
 *   - The player can destroy enemies
 */
fn main() {

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
            update_asteroids,
            check_if_firing,
            update_rockets.after(check_if_firing)
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
    // commands.spawn((
    //     SpriteBundle {
    //         transform: Transform {
    //             translation: vec3(0.0, 0.0, 0.0),
    //             ..default()
    //         },
    //         sprite: Sprite {
    //             // custom_size: Some(vec2(SHIP_WIDTH, SHIP_HEIGHT)),
    //             custom_size: Some(SHIP_SPEC),
    //             ..default()
    //         }, 
    //         texture: ship_texture,
    //         ..default()
    //     },
    //     Ship,
    //     Collider,
    //     Velocity(Vec2::new(0., 0.))
    // ));

    commands.spawn(ShipBundle::new(ship_texture));

    // Spawn Walls
    commands.spawn(WallBundle::new(GameWall::Top)).insert(KinematicObject);
    commands.spawn(WallBundle::new(GameWall::Bottom)).insert(KinematicObject);
    commands.spawn(WallBundle::new(GameWall::Right)).insert(KinematicObject);
    commands.spawn(WallBundle::new(GameWall::Left)).insert(KinematicObject);

    
    // Spawn Start Asteroids
    let asteroid_texture = asset_server.load("enemys/asteriod_base.png");


    commands.spawn(AsteroidBundle::new(asteroid_texture));

}

fn update_kinematic_objects(time: Res<Time<Fixed>>, mut query: Query<&mut Transform, With<KinematicObject>>) {
    for mut transform in query.iter_mut() {
        transform.translation.y += SHIP_SPEED * time.delta_seconds();
    }
}

fn update_asteroids(
    mut asteroid_query: Query<(&mut Transform, &mut Asteroid), With<Asteroid>>,
    camera_query: Query<&Transform, (With<GameCamera>, Without<Asteroid>)>,
) {

    let camera_transform = camera_query.single();

    for (mut transform, mut asteroid) in asteroid_query.iter_mut() {
        let is_outside_window: bool = asteroid.is_outside_window(&transform, &camera_transform);

        if is_outside_window { 
            let new_translation = asteroid.reset(camera_transform);
            transform.translation.x = new_translation.x;
            transform.translation.y = new_translation.y;
        } 
    }
}

fn update_rockets(
    mut commands: Commands,
    timestep: Res<Time<Fixed>>,
    mut rocket_query: Query<(Entity, &mut Transform, &mut Rocket), With<Rocket>>,
    camera_query: Query<&Transform, (With<GameCamera>, Without<Rocket>)>,
) {

    let camera_transform = camera_query.single();

    for (entity, mut rocket_transform, mut rocket) in rocket_query.iter_mut() {

        if rocket.is_outside_window(&rocket_transform, &camera_transform) {
            commands.entity(entity).despawn();
        } else {
            rocket_transform.translation.y += ROCKET_SPEED * timestep.delta_seconds()
        }
    }
}


fn collision_checks(
    mut commands: Commands, 
    mut ship_query: Query<(&mut Velocity, &mut Transform), With<Ship>>, 
    asset_collider: Query<(Entity, &Transform), (With<Collider>, Without<Ship>)>,
    mut collision_events: EventWriter<CollisionEvent>,
) {
    let (mut ship_velocity, ship_transform) = ship_query.single_mut();

    for ( _collider, transform  ) in &asset_collider {

        // Types of collision:
        //    - Ship to asteroid
        //    - rocket to asteroid
        // Eventually:
        //    - ship to health
        //    - ship to shield
        //    - ship to powerups
        
    }
}


fn get_y_bounds(camera_query: &Query<&Transform, (With<GameCamera>, Without<Ship>)> ) -> (f32, f32) {
    let camera_transform = *camera_query.single();

    let upper_bound: f32 = TOP_WALL - WALL_THICKNESS / 2.0 - SHIP_SPEC.y / 2.0 - SHIP_PADDING + camera_transform.translation.y;
    let lower_bound: f32 = BOTTOM_WALL - WALL_THICKNESS / 2.0 + SHIP_SPEC.y / 2.0 + SHIP_PADDING + camera_transform.translation.y;

    (upper_bound, lower_bound)
}   

fn ship_movement(
    keyboard_input: Res<ButtonInput<KeyCode>>, 
    timestep: Res<Time<Fixed>>, 
    // mut ship_query: Query<(&mut Transform, &mut Velocity), With<Ship>>,
    mut ship_query: Query<&mut Transform, With<Ship>>,
    camera_query: Query<&Transform, (With<GameCamera>, Without<Ship>)>
) {

    let ( upper_bound, lower_bound ) = get_y_bounds(&camera_query);

    let left_bound: f32 = LEFT_WALL + (SHIP_SPEC.x / 2.0) + SHIP_PADDING;
    let right_bound: f32 = RIGHT_WALL - (SHIP_SPEC.x / 2.0) - SHIP_PADDING;

    for mut transform in ship_query.iter_mut() {
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

        transform.translation.y = new_ship_position_y.clamp(lower_bound, upper_bound);
        transform.translation.x = new_ship_position_x.clamp(left_bound, right_bound);
    }

}

fn check_if_firing(
    keyboard_input: Res<ButtonInput<KeyCode>>, 
    timestep: Res<Time<Fixed>>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut ship_query: Query<(&mut Transform, &mut Ship), With<Ship>>
) {

    let (ship_transform, mut ship_properties) = ship_query.single_mut();

    if keyboard_input.pressed(KeyCode::Space) && ship_properties.cooldown_length == 0.0 {

        ship_properties.fire_rocket(&mut commands, &ship_transform, asset_server, texture_atlas_layouts)

    } else if ship_properties.cooldown_length > 0.0 {
        ship_properties.cooldown_length -= timestep.delta_seconds();
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

