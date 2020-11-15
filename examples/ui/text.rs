use bevy::prelude::*;

/// This example illustrates how to create text and update it in a system. It displays the current FPS in the upper left hand corner.
fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_plugin(bevy_webgl2::WebGL2Plugin::default())
        .add_startup_system(setup.system())
        .run();
}

// A unit struct to help identify the FPS UI component, since there may be many Text components
struct FpsText;

fn setup(commands: &mut Commands, asset_server: Res<AssetServer>) {
    commands
        // 2d camera
        .spawn(UiCameraComponents::default())
        // texture
        .spawn(TextComponents {
            style: Style {
                align_self: AlignSelf::FlexEnd,
                ..Default::default()
            },
            text: Text {
                value: "OK".to_string(),
                font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                style: TextStyle {
                    font_size: 60.0,
                    color: Color::WHITE,
                    ..Default::default()
                },
            },
            ..Default::default()
        })
        .with(FpsText);
}
