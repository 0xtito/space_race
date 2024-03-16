use bevy::prelude::*;


use crate::{
    ui_plugin::{
        react_to_button_pressed, 
        OnPressedHandler, 
        UiInteractionPlugin
    },
    constants::*
};



pub struct UiScaffoldPlugin;


impl Plugin for UiScaffoldPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(UiInteractionPlugin)
            .add_systems(Startup, setup);
    }
}


fn setup(mut commands: Commands) {


    commands.spawn(
        NodeBundle {
            style: Style {
                // size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                width: Val::Px(WINDOW_WIDTH),
                height: Val::Px(WINDOW_HEIGHT),
                justify_content: JustifyContent::Start,
                align_items: AlignItems::Center,
                display: Display::Flex,
                ..Default::default()
            }, 
            ..Default::default()
        }
    ).with_children( |parent| {
        parent.spawn(ButtonBundle {
            style: Style {
                width: Val::Px(40.0),
                height: Val::Px(40.0),
                border: UiRect::all(Val::Px(2.0)),
                margin: UiRect {
                    left: Val::Percent(1.),
                    top: Val::Percent(1.),
                    ..default()
                },
                align_self: AlignSelf::FlexStart,
                ..default()
            },
            background_color: Color::rgb(0.35, 0.35, 0.35).into(),
            ..default()
        }).add(OnPressedHandler::from(react_to_button_pressed));
        // }).add(OnPressedHandler::from(|| println!("Button pressed")));
    });
}