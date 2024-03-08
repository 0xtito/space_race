mod ship;
mod asteroid;
mod constants;
mod wall;

use ship::*;
use wall::*;
use asteroid::*;
use constants::*;

use bevy::{
     math::*, 
     prelude::*, window::WindowResolution,
};

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

// #[derive(Event)]
enum ExplosionAnimations {
    ShipExplosion,
    AsteroidExplosion
}

#[derive(Component, Debug)]
pub struct AnimationIndices {
    first: usize,
    last: usize,
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

    let grid_width = (WINDOW_WIDTH / GRID_SIZE).ceil();
    let grid_height = (WINDOW_HEIGHT / GRID_SIZE).ceil();

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
        }).set(ImagePlugin::default_nearest()) )
        .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .insert_resource(grid)
        .insert_resource(game_state)
        .insert_resource(game_difficulty)
        .add_event::<ExplosionEvent>() 
        .add_systems(Update, (bevy::window::close_on_esc,player_asteroid_explode))
        .add_systems(Startup, (setup, load_in_background, initiate_explosion))
        .add_systems(FixedUpdate, (
            ship_movement, 
            update_kinematic_objects,
            update_asteroids,
            check_if_firing,
            update_rockets.after(check_if_firing),
            update_grid,
            collision_checks.after(update_grid),
            initiate_explosion.after(collision_checks),
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
    let ship_texture = asset_server.load("ship/ship_full.png");
    
    commands.spawn(ShipBundle::new(ship_texture));

    // Spawn Walls
    commands.spawn(WallBundle::new(GameWall::Top)).insert(KinematicObject);
    commands.spawn(WallBundle::new(GameWall::Bottom)).insert(KinematicObject);
    commands.spawn(WallBundle::new(GameWall::Right)).insert(KinematicObject);
    commands.spawn(WallBundle::new(GameWall::Left)).insert(KinematicObject);


    let asteroid_sprite_texture: Handle<Image> = asset_server.load("enemys/asteroid_explosion_sprite.png");

    commands.spawn(NewAsteroidBundle::new(
        asteroid_sprite_texture,
        &camera_transform,
        &mut texture_atlas_layouts
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

    // println!("grid: {:#?}", grid);

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

                // Not checking current cell 
                if dx == 0 && dy == 0 { continue; }

                if neighbor_x > 8 || neighbor_y > 14  { continue; }
                
                for (neighbor_entity, neighbor_name, mut transform) in &grid[neighbor_x][neighbor_y] {

                    if let Some(ref mut asteroid) = asteroid {
                        if *neighbor_name == CollidableComponentNames::Asteroid {
                            continue;
                        }

                        // if asteroid.exploding { return; }

                        let collided: bool = asteroid.check_collision(
                            &transform, cur_transform, neighbor_name
                        );

                        match collided {
                            true => {
                                // println!("Asteroid collided with {:?}!", neighbor_name);

                                if *neighbor_name == CollidableComponentNames::Ship {

                                    ship.invulnerable = true;

                                    ship.take_damage();

                                    let events: [ExplosionEvent; 2] = [
                                        ExplosionEvent {
                                            explosion_type: ExplosionAnimations::AsteroidExplosion,
                                            entity: cur_entity
                                        },
                                        ExplosionEvent {
                                            explosion_type: ExplosionAnimations::ShipExplosion,
                                            entity: *neighbor_entity
                                        }
                                    ];

                                    collision_events.send_batch(events);

                                } else {

                                    collision_events.send(
                                        ExplosionEvent {
                                            explosion_type: ExplosionAnimations::AsteroidExplosion,
                                            entity: cur_entity
                                        }
                                    );

                                }                                
                            },
                            false => {
                                continue;
                            }
                        }
                        
                    }

                    // Means current entity is a Rocket
                    if let Some(ref mut rocket) = rocket {

                        if *neighbor_name == CollidableComponentNames::Asteroid {

                            let collided = rocket.check_collision(
                                &transform,
                                cur_transform,
                                neighbor_name
                            );

                           if collided {
                                rocket.hit_target = true;
                           } 

                            

                            // println!("cur entity Rocket possibly collided with {:?}", neighbor_name);
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

        transform.translation.y += SHIP_SPEED * time.delta_seconds();
    }
}

fn update_asteroids(
    mut commands: Commands,
    mut asteroid_query: Query<(&mut Transform, &mut Asteroid), With<Asteroid>>,
    camera_query: Query<&Transform, (With<GameCamera>, Without<Asteroid>)>,
    asset_server_res: Res<AssetServer>,
    game_state_res: Res<GameState>,
    game_difficulty_res: Res<GameDifficulty>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>
) {

    let game_state = game_state_res.as_ref();

    if *game_state == GameState::GameOver || *game_state == GameState::Paused {
        return;
    }

    let camera_transform = camera_query.single();

    for (mut transform, mut asteroid) in asteroid_query.iter_mut() {

        if asteroid.exploding {
            continue;
        }

        let is_outside_window: bool = asteroid.is_outside_window(&transform, &camera_transform);

        if is_outside_window { 
            let new_translation = asteroid.reset(camera_transform);
            transform.translation.x = new_translation.x;
            transform.translation.y = new_translation.y;
        } 
    }



    // Spawn another asteriod depending on difficulty
    if asteroid_query.is_empty() {

        let game_difficulty = game_difficulty_res.as_ref();

        let asteroid_texture: Handle<Image> = asset_server_res.load("enemys/asteroid_base.png");

        let mut asteroid_sprite_texture: Handle<Image> = asset_server_res.load("enemys/asteroid_explosion_sprite.png");


        match *game_difficulty {
            GameDifficulty::Easy => {

                commands.spawn_batch([
                    AsteroidBundle::new(asteroid_texture, camera_transform)
                ])

            },
            GameDifficulty::Medium => {

                commands.spawn_batch([
                    AsteroidBundle::new(asteroid_texture.clone(), camera_transform),
                    AsteroidBundle::new(asteroid_texture, camera_transform)
                ])

            }, 
            GameDifficulty::Hard => {

                let asteroid_sprite_texture1: Handle<Image> = asset_server_res.load("enemys/asteroid_explosion_sprite.png");

                let asteroid_sprite_texture2: Handle<Image> = asset_server_res.load("enemys/asteroid_explosion_sprite.png");

                let asteroid_sprite_texture3: Handle<Image> = asset_server_res.load("enemys/asteroid_explosion_sprite.png");




                // commands.spawn_batch([
                //     AsteroidBundle::new(asteroid_texture.clone(),camera_transform),
                //     AsteroidBundle::new(asteroid_texture.clone(), camera_transform),
                //     AsteroidBundle::new(asteroid_texture, camera_transform)
                // ]);

                commands.spawn(
                    NewAsteroidBundle::new(
                        asteroid_sprite_texture,
                        camera_transform,
                        &mut texture_atlas_layouts
                    )
                );

                // commands.spawn_batch([
                //     NewAsteroidBundle::new(
                //         asteroid_sprite_texture1,
                //         camera_transform,
                //         &mut texture_atlas_layouts
                //     ),
                //     NewAsteroidBundle::new(
                //         asteroid_sprite_texture2,
                //         camera_transform,
                //         &mut texture_atlas_layouts
                //     ),
                //     NewAsteroidBundle::new(
                //         asteroid_sprite_texture3,
                //         camera_transform,
                //         &mut texture_atlas_layouts
                //     )
                // ]);

            }
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

    if keyboard_input.pressed(KeyCode::Space) && ship_properties.cooldown_length == 0.0 {

        ship_properties.fire_rocket(&mut commands, &ship_transform, asset_server, texture_atlas_layouts)

    } else if ship_properties.cooldown_length > 0.0 {
        ship_properties.cooldown_length -= timestep.delta_seconds();
    }

}

fn initiate_explosion(
    mut commands: Commands,
    mut collision_events: EventReader<ExplosionEvent>,
    mut ship_query: Query<&mut Ship>,
    mut asteroid_query: Query<(Entity, &mut Asteroid), Without<AnimationTimer>>,
    mut game_state_res: ResMut<GameState>,
)  {


    if !collision_events.is_empty() {

        for ExplosionEvent { explosion_type, entity } in collision_events.read() {

            match explosion_type {
                ExplosionAnimations::ShipExplosion => {

                    let mut ship: Mut<'_, Ship> = ship_query.single_mut();

                    let game_state = game_state_res.as_mut();

                    *game_state = GameState::GameOver;
                    

                    println!("*ship explosion*");

                }, 
                ExplosionAnimations::AsteroidExplosion => {
                    let exploding_asteroid =  asteroid_query.get_mut(*entity);

                    if exploding_asteroid.is_ok() {
                        let (asteroid_entity, mut asteroid)  = exploding_asteroid.unwrap();

                        println!("*asteroid explosion*");

                        asteroid.exploding = true;
                    }
                }
            }
        }
        collision_events.clear()
    }
}

fn player_asteroid_explode(
    mut commands: Commands,
    mut asteroid_query: Query<(&AnimationIndices, &mut AnimationTimer, &mut TextureAtlas, &mut Asteroid, Entity)>,
    game_state_res: Res<GameState>,
    time: Res<Time>,
) {

    let game_state = game_state_res.as_ref();

    if *game_state == GameState::GameOver || *game_state == GameState::Paused {
        return;
    }


    for (_indices, mut timer, mut atlas, asteroid, entity ) in asteroid_query.iter_mut() {

        if !asteroid.exploding { return };

        timer.tick(time.delta());
        
        if timer.just_finished() {
            commands.entity(entity).despawn();
        } else {
            atlas.index = atlas.index + 1;
        };

    }
}

fn load_in_background(
    mut commands:  Commands, 
    // backgrounds: Vec<Handle<Image>>,
    asset_server: Res<AssetServer>,
    // repeat: u8,
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
    

    for i in 0..4 {
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

