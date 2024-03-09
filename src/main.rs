mod ship;
mod asteroid;
mod constants;
mod wall;

use std::time::Duration;

use ship::*;
use wall::*;
use asteroid::*;
use constants::*;

use bevy::{
     math::*, 
     prelude::*, window::WindowResolution,
};

use rand::{distributions::Uniform, Rng};

#[derive(Component)]
struct GameCamera {
    moving: bool
}

#[derive(Resource, PartialEq)]
enum GameState {
    Playing,
    Paused,
    GameOver
}

#[derive(Resource, PartialEq)]
enum GameDifficulty {
    Easy,
    Medium,
    Hard
}

#[derive(Component)]
struct KinematicObject;

#[derive(Component)]
struct Background;


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

enum SoundVariants {
    ShipExplosion,
    DamageToShip,
    RocketExplosion,
    // BackgroundMusic
}

#[derive(Resource, Debug)]
struct SpawnTimer(Timer);

// #[derive(Event)]
enum ExplosionAnimations {
    ShipExplosion,
    AsteroidExplosion,
    DamageToShip
}

enum AnimatableAsset {
    Rocket,
    Asteroid,
    Ship
}

#[derive(Component)]
struct PlayAnimation(AnimatableAsset);

#[derive(Component, Debug, Deref, DerefMut)]
pub struct AnimationTimer(Timer);

#[derive(Component, Debug)]
pub struct AnimationIndices {
    first: usize,
    last: usize,
}

#[derive(Component, Debug)]
struct AnimationProperties {
    indices: AnimationIndices,
    timer: AnimationTimer
}

#[derive(Event)]
struct ExplosionEvent {
    explosion_type: ExplosionAnimations,
    entity: Entity
}

#[derive(Component, PartialEq, Clone, Debug)]
enum CollidableComponentNames {
    Ship,
    Rocket,
    Asteroid
}

#[derive(Component)]
struct Collider {
    name: CollidableComponentNames,
    shape: ColliderShape
}



#[derive(Resource, Debug, Deref, DerefMut)]
struct Grid {
    #[deref]
    cells: Vec<Vec<Vec<(Entity, CollidableComponentNames, Transform)>>>,
    grid_size: f32,
}




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
        resolution: WindowResolution::new(WINDOW_WIDTH, WINDOW_HEIGHT),
        present_mode: bevy::window::PresentMode::AutoVsync,
        resizable: false,
        ..default()
    };

    let cells: Vec<Vec<Vec<(Entity, CollidableComponentNames, Transform)>>> = vec![vec![Vec::new(); (WINDOW_HEIGHT / GRID_SIZE) as usize]; (WINDOW_WIDTH / GRID_SIZE) as usize];


    let grid = Grid {
        cells,
        grid_size: GRID_SIZE,
    };

    let game_state: GameState = GameState::Playing;
    
    let game_difficulty = GameDifficulty::Hard;

    App::new()
        .add_plugins(DefaultPlugins.set( WindowPlugin {
            primary_window: Some(primary_window),
            ..default()
        }).set(ImagePlugin::default_nearest()))
        .insert_resource(grid)
        .insert_resource(game_state)
        .insert_resource(game_difficulty)
        .insert_resource(SpawnTimer(Timer::from_seconds(1.0, TimerMode::Repeating)))
        .add_event::<ExplosionEvent>()
        .add_systems(Startup, setup) 
        .add_systems(Update, (
            bevy::window::close_on_esc, 
            spawn_asteroids,
            play_animations
        ))
        .add_systems(FixedUpdate, (
            asteroid_manager,
            ship_movement, 
            update_kinematic_objects,
            check_if_firing,
            update_rockets.after(check_if_firing),
            update_grid,
            collision_checks.after(update_grid),
            explosion_event_listener.after(collision_checks),
        ).chain())
        .run();
}

fn setup(
    mut commands: Commands, 
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {

    let camera_bundle = Camera2dBundle::default();

    let camera_transform = camera_bundle.transform.clone();

    load_in_background(&mut commands, &asset_server);

    // Camera setup
    commands
        .spawn(camera_bundle)
        .insert(
            (
            GameCamera {
                moving: true
            }, 
            KinematicObject)
        );

    // Spawn Ship
    let ship_texture = asset_server.load("ship/ship_spritesheet_empty_space.png");
    
    commands.spawn(
        ShipBundle::new(ship_texture, &mut texture_atlas_layouts,
        0.5)
    );

    // Spawn Walls
    commands.spawn(WallBundle::new(GameWall::Top)).insert(KinematicObject);
    commands.spawn(WallBundle::new(GameWall::Bottom)).insert(KinematicObject);
    commands.spawn(WallBundle::new(GameWall::Right)).insert(KinematicObject);
    commands.spawn(WallBundle::new(GameWall::Left)).insert(KinematicObject);


    let asteroid_sprite_texture: Handle<Image> = asset_server.load("enemys/asteroid_explosion_sprite.png");

    commands.spawn(AsteroidBundle::new(
        asteroid_sprite_texture,
        &camera_transform,
        &mut texture_atlas_layouts,
        None
    ));

}

fn update_grid(
    mut grid: ResMut<Grid>,
    collidable_query: Query<(Entity, &Transform, &Collider), (With<Collider>, Without<KinematicObject>)>,
    camera_query: Query<&Transform, With<GameCamera>>,
) {

    for cell in grid.cells.iter_mut().flatten() {
        cell.clear();
    }

    let camera_transform = camera_query.single();
    let camera_translation = camera_transform.translation;

    for (entity, transform, collider_info) in collidable_query.iter() {

        let relative_position = transform.translation - camera_translation;

        let formatted_position_x = relative_position.x + (WINDOW_WIDTH / 2.0);
        let formatted_position_y = relative_position.y + (WINDOW_HEIGHT / 2.0);

        let grid_x = ( formatted_position_x / grid.grid_size).floor() as usize;
        let grid_y = ( formatted_position_y / grid.grid_size).floor() as usize;

        // x segments of grid array has a length of 9. Thus, grid_x must be between 0 and 8 (max)
        // y segments of grid array has a length of 15. Thus, grid_y must be between 0 and 14 (max)
        if grid_x > 8 || grid_y > 14 {
            continue;
        };

        grid.cells[grid_x][grid_y].push((entity, collider_info.name.clone(), *transform));
    }

}

fn collision_checks(
    mut res_grid: ResMut<Grid>,
    mut ship_query: Query<&mut Ship>, 
    mut collidable_query: Query<(Entity, &Transform, Option<&mut Asteroid>, Option<&mut Rocket>), With<Collider>>,
    camera_query: Query<&Transform, With<GameCamera>>,
    mut collision_events: EventWriter<ExplosionEvent>,
    game_state_res: Res<GameState>
) {

    let game_state = game_state_res.as_ref();

    if *game_state == GameState::GameOver || *game_state == GameState::Paused {
        return;
    }

    let mut ship = ship_query.single_mut();


    let camera_transform = camera_query.single();
    let camera_translation = camera_transform.translation;

    // let mut raw_grid = grid.as_deref_mut();
    let grid = res_grid.as_mut();

    for ( cur_entity, cur_transform, mut asteroid, mut rocket ) in &mut collidable_query {
        let relative_position = cur_transform.translation - camera_translation;

        let formatted_position_x = relative_position.x + (WINDOW_WIDTH / 2.0);
        let formatted_position_y = relative_position.y + (WINDOW_HEIGHT / 2.0);

        let grid_x = ( formatted_position_x / grid.grid_size).floor() as usize;
        let grid_y = ( formatted_position_y / grid.grid_size).floor() as usize;

        // x segments of grid array has a length of 9. Thus, grid_x must be between 0 and 8 (max)
        // y segments of grid array has a length of 15. Thus, grid_y must be between 0 and 14 (max)
        if grid_x > 8 || grid_y > 14 {
            continue;
        }

        for dx in -1..=1 {
            for dy in -1..=1 {
                let neighbor_x = (grid_x as i32 + dx) as usize;
                let neighbor_y  = (grid_y as i32 + dy) as usize;

                if neighbor_x > 8 || neighbor_y > 14  { continue; }
                
                for (neighbor_entity, neighbor_name, mut neighbor_transform) in &grid[neighbor_x][neighbor_y] {

                    if let Some(ref mut asteroid) = asteroid {

                        if *neighbor_name == CollidableComponentNames::Asteroid {
                            continue;
                        }

                        if asteroid.exploding { continue; }

                        let collided: bool = asteroid.check_collision(
                            &cur_transform, &neighbor_transform, neighbor_name
                        );

                        if !collided { continue; }

                        if *neighbor_name == CollidableComponentNames::Ship {

                            // ship.invulnerable = true;

                            if ship.invulnerable { continue; };

                            let new_ship_health = ship.take_damage();


                            let events = if new_ship_health == ShipHealth::Empty {
                                [
                                    ExplosionEvent {
                                        explosion_type: ExplosionAnimations::AsteroidExplosion,
                                        entity: cur_entity
                                    },
                                    ExplosionEvent {
                                        explosion_type: ExplosionAnimations::ShipExplosion,
                                        entity: *neighbor_entity
                                    }
                                ]
                            } else {
                                [
                                    ExplosionEvent {
                                        explosion_type: ExplosionAnimations::AsteroidExplosion,
                                        entity: cur_entity
                                    },

                                    ExplosionEvent {
                                        explosion_type: ExplosionAnimations::DamageToShip,
                                        entity: *neighbor_entity
                                    }

                                ]
                            };

                            // let events: [ExplosionEvent; 2] = [
                            //     ExplosionEvent {
                            //         explosion_type: ExplosionAnimations::AsteroidExplosion,
                            //         entity: cur_entity
                            //     },
                            //     ExplosionEvent {
                            //         explosion_type: ExplosionAnimations::ShipExplosion,
                            //         entity: *neighbor_entity
                            //     }
                            // ];

                            collision_events.send_batch(events);

                        } else {

                            println!("Asteroid collided with Rocket");

                            collision_events.send(
                                ExplosionEvent {
                                    explosion_type: ExplosionAnimations::AsteroidExplosion,
                                    entity: cur_entity
                                }
                            );

                        }                                
                        
                    }

                    // Means current entity is a Rocket
                    if let Some(ref mut rocket) = rocket {

                        if *neighbor_name == CollidableComponentNames::Asteroid {

                            let collided = rocket.check_collision(
                                &cur_transform,
                                &neighbor_transform,
                                neighbor_name
                            );

                            // if !collided { continue; }

                           if collided {
                                println!("Rocket collided with Asteroid");
                                rocket.hit_target = true;
                           } 
                        };
                    }
                }
            }
        }  
    }
}


fn update_kinematic_objects(
    time: Res<Time<Fixed>>, 
    mut query: Query<&mut Transform, With<KinematicObject>>, 
    game_state_res: Res<GameState>
) {

    let game_state = game_state_res.as_ref();

    if *game_state == GameState::GameOver || *game_state == GameState::Paused {
        return;
    }

    for mut transform in query.iter_mut() {

        transform.translation.y += KINEMATIC_OBJECTS_SPEED * time.delta_seconds();
    }
}

fn spawn_asteroids(
    mut commands: Commands,
    asteroid_query: Query<&Transform, (With<Asteroid>, Without<Ship>)>,
    camera_query: Query<&Transform, (With<GameCamera>, Without<Ship>)>,
    time: Res<Time>,
    mut timer: ResMut<SpawnTimer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    asset_server_res: Res<AssetServer>,
    game_state_res: Res<GameState>
) {

    let game_state = game_state_res.as_ref();

    if *game_state == GameState::GameOver ||  *game_state == GameState::Paused {
        return;
    }

    if !timer.0.tick(time.delta()).just_finished() {
        return;
    }

    let overall_asteroid_count = asteroid_query.iter().count();


    if overall_asteroid_count > 15 {
        return;
    }

    let mut rng = rand::thread_rng();

    let camera_transform = camera_query.single();

    let mut spawn_position = Vec3::new(
        rng.gen_range((-WINDOW_WIDTH / 2.0 + (ASTEROID_SCALED_RADIUS))..(WINDOW_WIDTH / 2.0 - (ASTEROID_SCALED_RADIUS))),
        TOP_WALL + 100.0 + camera_transform.translation.y, 
        0.0,
    );

    // Done to make sure there is no overlap, greater than a asteriod's radius, between asteriods 
    for transform in asteroid_query.iter() {
        while (spawn_position.x - transform.translation.x).abs() < ASTEROID_SCALED_RADIUS / 2.0
            && (spawn_position.y - transform.translation.y).abs() < ASTEROID_SCALED_RADIUS / 2.0
        {
            println!("INSIDE WHILE");
            spawn_position.x = rng.gen_range((-WINDOW_WIDTH / 2.0 + (ASTEROID_SCALED_RADIUS))..(WINDOW_WIDTH / 2.0 - (ASTEROID_SCALED_RADIUS)));
        }
    }

    let asteroid_sprite_texture: Handle<Image> = asset_server_res.load("enemys/asteroid_explosion_sprite.png");

    commands.spawn(AsteroidBundle::new(
        asteroid_sprite_texture,
        camera_transform,
        &mut texture_atlas_layouts,
        Some(spawn_position)
    ));

    // Resetting the timer to a new random duration between 0.5 and 1.5 seconds
    timer.0.set_duration(Duration::from_secs_f32(rng.gen_range(0.5..0.75)));

    timer.0.reset();
}


fn asteroid_manager(
    mut commands: Commands,
    mut asteroid_query: Query<(Entity, &Transform, &Asteroid), With<Asteroid>>,
    camera_query: Query<&Transform, (With<GameCamera>, Without<Asteroid>)>,
    game_state_res: Res<GameState>,
) {

    let game_state = game_state_res.as_ref();

    if *game_state == GameState::GameOver || *game_state == GameState::Paused {
        return;
    }

    let camera_transform = camera_query.single();

    for (entity, transform, asteroid) in asteroid_query.iter_mut() {

        if asteroid.exploding {
            continue;
        }

        let is_outside_window: bool = asteroid.is_outside_window(&transform, &camera_transform);

        if is_outside_window { 
            // let new_translation = asteroid.reset(camera_transform);
            // transform.translation = new_translation.extend(0.0);

            commands.entity(entity).despawn()
        } 
    }
}

fn update_rockets(
    mut commands: Commands,
    timestep: Res<Time<Fixed>>,
    mut rocket_query: Query<(Entity, &mut Transform, &Rocket), With<Rocket>>,
    camera_query: Query<&Transform, (With<GameCamera>, Without<Rocket>)>,
) {

    let camera_transform = camera_query.single();

    for (entity, mut rocket_transform, rocket) in rocket_query.iter_mut() {

        if rocket.hit_target {
            commands.entity(entity).despawn();
            return; 
        }

        if rocket.is_outside_window(&rocket_transform, &camera_transform) {
            commands.entity(entity).despawn();
        } else {
            rocket_transform.translation.y += ROCKET_SPEED * timestep.delta_seconds()
        }
    }
}


fn get_y_bounds(camera_query: &Query<&Transform, (With<GameCamera>, Without<Ship>)> ) -> (f32, f32) {
    let camera_transform = *camera_query.single();

    let upper_bound: f32 = TOP_WALL - (SHIP_SPEC.y / 2.0) + camera_transform.translation.y;
    let lower_bound: f32 = BOTTOM_WALL + (SHIP_SPEC.y / 2.0) + camera_transform.translation.y;

    // let upper_bound: f32 = TOP_WALL - (WALL_THICKNESS / 2.0) - (SHIP_SPEC.y / 2.0) + camera_transform.translation.y;
    // let lower_bound: f32 = BOTTOM_WALL + (WALL_THICKNESS / 2.0) + (SHIP_SPEC.y / 2.0) + camera_transform.translation.y;

    (upper_bound.round(), lower_bound.round())
}   

fn ship_movement(
    keyboard_input: Res<ButtonInput<KeyCode>>, 
    timestep: Res<Time<Fixed>>, 
    mut ship_query: Query<(&mut Transform, &Ship), With<Ship>>,
    camera_query: Query<&Transform, (With<GameCamera>, Without<Ship>)>,
    game_state_res: Res<GameState>
) {

    let game_state = game_state_res.as_ref();

    if *game_state == GameState::GameOver || *game_state == GameState::Paused {
        return;
    }

    let ( upper_bound, lower_bound ) = get_y_bounds(&camera_query);

    let left_bound: f32 = LEFT_WALL + (SHIP_SPEC.x / 2.0);
    let right_bound: f32 = RIGHT_WALL - (SHIP_SPEC.x / 2.0);

    let (mut transform, ship) = ship_query.single_mut();

    if ship.health == ShipHealth::Empty { return }

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

    transform.translation.y = new_ship_position_y.clamp(
        lower_bound, 
        upper_bound
    );
    transform.translation.x = new_ship_position_x.clamp(
        left_bound, 
        right_bound
    );

}

fn check_if_firing(
    keyboard_input: Res<ButtonInput<KeyCode>>, 
    timestep: Res<Time<Fixed>>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut ship_query: Query<(&mut Transform, &mut Ship), With<Ship>>,
    game_state_res: Res<GameState>
) {


    let game_state = game_state_res.as_ref();

    if *game_state == GameState::GameOver || *game_state == GameState::Paused {
        return;
    }

    let (ship_transform, mut ship_properties) = ship_query.single_mut();

    if ship_properties.health == ShipHealth::Empty || 
       ship_properties.invulnerable == true
     {
        return;
    }

    // if keyboard_input.pressed(KeyCode::Space)  {
    if keyboard_input.pressed(KeyCode::Space) && ship_properties.cooldown_time_left == 0.0 {

        ship_properties.fire_rocket(&mut commands, &ship_transform, asset_server, texture_atlas_layouts)

    } else if ship_properties.cooldown_time_left > 0.0 {
        ship_properties.cooldown_time_left -= timestep.delta_seconds();
    }

}

fn explosion_event_listener(
    mut commands: Commands,
    mut collision_events: EventReader<ExplosionEvent>,
    mut ship_query: Query<(Entity, &mut Ship)>,
    mut asteroid_query: Query<(Entity, &mut Asteroid), Without<AnimationTimer>>,
    mut game_state_res: ResMut<GameState>,
)  {

    if !collision_events.is_empty() {

        for ExplosionEvent { explosion_type, entity } in collision_events.read() {

            let (ship_entity, mut ship) = ship_query.single_mut();

            match explosion_type {
                ExplosionAnimations::ShipExplosion => {

                    // let mut ship: Mut<'_, Ship> = ship_query.single_mut();

                    let game_state = game_state_res.as_mut();

                    *game_state = GameState::GameOver;
                    

                    println!("*ship explosion*");

                }, 
                ExplosionAnimations::AsteroidExplosion => {
                    let exploding_asteroid =  asteroid_query.get_mut(*entity);

                    if exploding_asteroid.is_ok() {
                        let (asteroid_entity, mut asteroid)  = exploding_asteroid.unwrap();

                        println!("*asteroid explosion*");

                        commands.entity(asteroid_entity).insert(PlayAnimation(AnimatableAsset::Asteroid));

                        asteroid.exploding = true;
                    }
                }
                
                ExplosionAnimations::DamageToShip => {
                    
                    ship.invulnerable = true;

                    commands.entity(ship_entity).insert(PlayAnimation(AnimatableAsset::Ship));
                }
            }
        }
        
        collision_events.clear()
    }
}

fn play_animations(
    mut commands: Commands,
    mut animatable_comp_query:  Query<(Entity, &mut TextureAtlas, Option<&mut Ship>, Option<&mut Asteroid>, Option<&mut Rocket>), With<PlayAnimation>>,
    game_state_res: Res<GameState>,
    time: Res<Time>,
) {

    let game_state = game_state_res.as_ref();

    if *game_state == GameState::GameOver || *game_state == GameState::Paused {
        return;
    }

    for (entity, mut atlas, ship, asteroid, rocket) in animatable_comp_query.iter_mut() {

        if let Some(mut ship) = ship {

            if ship.invulnerable_timer.elapsed_secs() == 0.0 {
                // atlas.index += 1;

                println!("Inside if ship.invulnerable_timer.elapsed_secs() == 0.0");

                atlas.index = match ship.health {
                    ShipHealth::Full => {
                        1
                    },
                    ShipHealth::Damaged => {
                        3
                    },
                    ShipHealth::VeryDamaged => {
                        4
                    },
                    ShipHealth::Empty => {
                        4
                    }
                }
            }

            ship.invulnerable_timer.tick(time.delta());

            if ship.invulnerable_timer.finished() {
                ship.invulnerable_timer.reset();
                ship.invulnerable = false;

                atlas.index = match ship.health {
                    ShipHealth::Full => {
                        1
                    },
                    ShipHealth::Damaged => {
                        3
                    },
                    ShipHealth::VeryDamaged => {
                        4
                    },
                    ShipHealth::Empty => {
                        4
                    }
                };
                commands.entity(entity).remove::<PlayAnimation>();
            } else {

                let time_elapsed = ship.invulnerable_timer.elapsed_secs();

                // Basing this off the animation length being 1.0 seconds
                // let show_nothing = 
                // (time_elapsed > 0.0 && time_elapsed < 0.2) || 
                // (time_elapsed > 0.4 && time_elapsed < 0.6) || 
                // (time_elapsed > 0.8 && time_elapsed < 1.0);

                // Basing this off the animation length being 2.0 seconds
                let show_nothing = 
                (time_elapsed > 0.0 && time_elapsed < 0.4) || 
                (time_elapsed > 0.8 && time_elapsed < 1.2) || 
                (time_elapsed > 1.6 && time_elapsed < 2.0);

                atlas.index = if show_nothing {
                    0
                } else {
                    match ship.health {
                        ShipHealth::Full => {
                            1
                        },
                        ShipHealth::Damaged => {
                            3
                        },
                        ShipHealth::VeryDamaged => {
                            4
                        },
                        ShipHealth::Empty => {
                            4
                        }
                    }
                }

            }

        }

        if let Some(mut asteroid) = asteroid {

            if !asteroid.exploding { return };
        
            asteroid.animation_timer.tick(time.delta());
    
            if asteroid.animation_timer.just_finished()  {
                commands.entity(entity).despawn();       
            } else if atlas.index == asteroid.animation_indices.last  {
                atlas.index = atlas.index;
            } else {
                atlas.index = atlas.index + 1;
            }
        }

        if let Some(mut rocket) = rocket {
            if rocket.hit_target { continue; }

            // let time_remaining = rocket.animation_timer.remaining_secs();
            // let delta = time.delta();
    
            rocket.animation_timer.tick(time.delta());
    
    
            if rocket.animation_timer.finished() {
    
            // atlas.index += 1;
                atlas.index = if atlas.index == rocket.animation_indices.last {
                    rocket.animation_indices.first
                } else {
                    atlas.index + 1
                };
            }
        }
    }
}

fn load_in_background(
    commands: &mut Commands, 
    asset_server: &Res<AssetServer>,
) {

    // Background setup
    let void_layer_1_texture: Handle<Image> = asset_server.load("background/void_layer_1.png");
    let stars_layer_2_texture: Handle<Image> = asset_server.load("background/stars_layer_2.png");
    let stars_layer_3_texture: Handle<Image> = asset_server.load("background/stars_layer_3.png");

    let backgrounds = vec![
        void_layer_1_texture,
        stars_layer_2_texture,
        stars_layer_3_texture
    ];
    

    for i in 0..3 {
        // println!("{}", i);
        for bg_texture in backgrounds.iter() {            
            commands.spawn((SpriteBundle {
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
            },
            Background
            ));
        }
        
    }
}

