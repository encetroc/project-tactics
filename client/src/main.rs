use bevy::{prelude::*, text::FontSize, window::WindowResolution};
use project_tactics_client::network::{ConnectionSnapshot, NetworkPlugin, NetworkSet};

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.025, 0.04, 0.06)))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Project Tactics — connected clients".into(),
                resolution: WindowResolution::new(960, 540),
                canvas: Some("#game".into()),
                fit_canvas_to_parent: true,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(NetworkPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, update_status.after(NetworkSet))
        .run();
}

#[derive(Component)]
struct StatusText;

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    commands.spawn((
        Text::new("Connecting…"),
        TextFont {
            font_size: FontSize::Px(15.0),
            ..default()
        },
        Node {
            position_type: PositionType::Absolute,
            top: px(16),
            left: px(16),
            right: px(16),
            ..default()
        },
        StatusText,
    ));
}

fn update_status(snapshot: Res<ConnectionSnapshot>, mut text: Query<&mut Text, With<StatusText>>) {
    if !snapshot.is_changed() {
        return;
    }
    for mut text in &mut text {
        text.0 = snapshot.summary();
    }
    // Accessible DOM mirror also makes real browser replication testable.
    #[cfg(target_arch = "wasm32")]
    if let Some(document) = web_sys::window().and_then(|window| window.document())
        && let Some(status) = document.get_element_by_id("connection-status")
    {
        status.set_text_content(Some(&snapshot.summary()));
        let _ = status.set_attribute("data-state", snapshot.phase);
        let _ = status.set_attribute("data-count", &snapshot.sessions.len().to_string());
    }
}
