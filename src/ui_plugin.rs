use bevy::{ecs::system::{EntityCommand, SystemId}, prelude::*};

use event_handler_macro::EventHandler;

use std::marker::PhantomData;

use crate::AppState;

#[derive(Component, Debug, Clone, EventHandler)]
pub struct OnPressed {
    system_id: SystemId,
    pub active: bool,
}

pub struct UiInteractionPlugin;

#[derive(Component)]
pub struct MainMenuRootNode;

/// This is the plugin that will handle all of the UI interactions
/// A Derivated of @eidloi's version showed in his youtube video:
/// https://youtu.be/s1lQD-R_kqg?si=VHW4Qz1AIVv1o5JB
impl Plugin for UiInteractionPlugin {
    fn build(&self, app: &mut App) {
            app.add_systems(Update, handle_pressed);
    }
}

// Call this system when you need to do something
pub fn handle_pressed(
        mut interaction_query: Query<(Entity, &Interaction), (Changed<Interaction>, With<OnPressed>)>,
        mut handler_query: Query<&mut OnPressed>,
        mut commands: Commands,
) {
        for (entity, interaction) in &mut interaction_query {
                println!("inside of handle_pressed");
                let handler = handler_query.get_mut(entity).unwrap().into_inner();
                if *interaction == Interaction::Pressed {
                        let mut active_handler = handler.clone();
                        active_handler.active = true;
                        commands.entity(entity).insert(active_handler);
                        commands.run_system(handler.system_id);
                        commands.entity(entity).insert(handler.clone());
                }
        }
}

pub fn react_to_button_pressed(
    mut pressed_query: Query<(Entity, &OnPressed), With<OnPressed>>,
    state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>
) {
    for (entity, on_event) in &mut pressed_query {
        if on_event.active {
                println!("Button with entity {:?} was pressed", entity);

                let game_state = state.get();

                println!("Current Game State: {:?}", game_state);
                match *game_state {
                    AppState::InGame => {
                        println!("Setting next state to Paused");
                        next_state.set(AppState::Paused);
                    },
                    AppState::Paused => {
                        println!("Setting next state to InGame");
                        next_state.set(AppState::InGame);
                    },
                    _ => {}
                }
        }
    }
}