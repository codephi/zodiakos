//! This example illustrates bloom post-processing in 2d.

use bevy::{
    core_pipeline::{
        bloom::{BloomCompositeMode, BloomSettings},
        tonemapping::Tonemapping,
    },
    prelude::*,
    sprite::MaterialMesh2dBundle,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, update_bloom_settings)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                hdr: true, // 1. HDR is required for bloom
                ..default()
            },
            tonemapping: Tonemapping::TonyMcMapface, // 2. Using a tonemapper that desaturates with exposure prevents over-saturation from bloom
            ..default()
        },
        BloomSettings::default(), // 3. Enable bloom for the camera
    ));

    // Circle mesh
    commands.spawn(MaterialMesh2dBundle {
        mesh: meshes.add(Circle::new(100.)).into(),
        // 4. Put something bright in a dark environment to see the effect
        material: materials.add(ColorMaterial::from(Color::srgb(7.5, 0.0, 7.5))),
        transform: Transform::from_translation(Vec3::new(-200., 0., 0.)),
        ..default()
    });

    // Hexagon mesh
    commands.spawn(MaterialMesh2dBundle {
        mesh: meshes.add(RegularPolygon::new(100., 6)).into(),
        // 4. Put something bright in a dark environment to see the effect
        material: materials.add(ColorMaterial::from(Color::srgb(6.25, 9.4, 9.1))),
        transform: Transform::from_translation(Vec3::new(200., 0., 0.)),
        ..default()
    });

    // UI
    commands.spawn(
        TextBundle::from_section("", TextStyle::default()).with_style(Style {
            position_type: PositionType::Absolute,
            bottom: Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
        }),
    );
}

// ------------------------------------------------------------------------------------------------

fn update_bloom_settings(
    mut camera: Query<(Entity, Option<&mut BloomSettings>), With<Camera>>,
    mut text: Query<&mut Text>,
    mut commands: Commands,
    keycode: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    let bloom_settings = camera.single_mut();
    let mut text = text.single_mut();
    let text = &mut text.sections[0].value;

    match bloom_settings {
        (entity, Some(mut bloom_settings)) => {
            text.clear();
            text.push_str("Bloom enabled (");
            text.push_str(match bloom_settings.composite_mode {
                BloomCompositeMode::EnergyConserving => "Energy-conserving",
                BloomCompositeMode::Additive => "Additive",
            });
            text.push_str(" mode)\n");
            text.push_str("Controls:\n");
            text.push_str("  A - Toggle bloom mode (Energy-conserving or Additive)\n");
            text.push_str("  Space - Toggle bloom on/off\n");
            text.push_str("\n");
            text.push_str("Intensity (Q/E): ");
            text.push_str(&format!("{:.2}", bloom_settings.intensity));
            text.push_str("\n");

            text.push_str("Low Frequency Boost (W/S): ");
            text.push_str(&format!("{:.2}", bloom_settings.low_frequency_boost));
            text.push_str("\n");

            text.push_str("Low Frequency Boost Curvature (R/F): ");
            text.push_str(&format!(
                "{:.2}",
                bloom_settings.low_frequency_boost_curvature
            ));
            text.push_str("\n");

            text.push_str("High Pass Frequency (T/G): ");
            text.push_str(&format!("{:.2}", bloom_settings.high_pass_frequency));
            text.push_str("\n");

            text.push_str("Threshold (Y/H): ");
            text.push_str(&format!(
                "{:.2}",
                bloom_settings.prefilter_settings.threshold
            ));
            text.push_str("\n");

            text.push_str("Soft Threshold (U/J): ");
            text.push_str(&format!(
                "{:.2}",
                bloom_settings.prefilter_settings.threshold_softness
            ));
            text.push_str("\n");

            if keycode.pressed(KeyCode::KeyA) {
                bloom_settings.composite_mode = match bloom_settings.composite_mode {
                    BloomCompositeMode::EnergyConserving => BloomCompositeMode::Additive,
                    BloomCompositeMode::Additive => BloomCompositeMode::EnergyConserving,
                };
            }

            if keycode.just_pressed(KeyCode::Space) {
                commands.entity(entity).remove::<BloomSettings>();
            }

            let dt = time.delta_seconds();

            if keycode.pressed(KeyCode::KeyQ) {
                bloom_settings.intensity -= dt;
            }
            if keycode.pressed(KeyCode::KeyE) {
                bloom_settings.intensity += dt;
            }
            bloom_settings.intensity = bloom_settings.intensity.clamp(0.0, 1.0);

            if keycode.pressed(KeyCode::KeyW) {
                bloom_settings.low_frequency_boost -= dt / 10.0;
            }
            if keycode.pressed(KeyCode::KeyS) {
                bloom_settings.low_frequency_boost += dt / 10.0;
            }
            bloom_settings.low_frequency_boost = bloom_settings.low_frequency_boost.clamp(0.0, 1.0);

            if keycode.pressed(KeyCode::KeyR) {
                bloom_settings.low_frequency_boost_curvature -= dt / 10.0;
            }
            if keycode.pressed(KeyCode::KeyF) {
                bloom_settings.low_frequency_boost_curvature += dt / 10.0;
            }
            bloom_settings.low_frequency_boost_curvature =
                bloom_settings.low_frequency_boost_curvature.clamp(0.0, 1.0);

            if keycode.pressed(KeyCode::KeyT) {
                bloom_settings.high_pass_frequency -= dt / 10.0;
            }
            if keycode.pressed(KeyCode::KeyG) {
                bloom_settings.high_pass_frequency += dt / 10.0;
            }
            bloom_settings.high_pass_frequency = bloom_settings.high_pass_frequency.clamp(0.0, 1.0);

            if keycode.pressed(KeyCode::KeyY) {
                bloom_settings.prefilter_settings.threshold -= dt;
            }
            if keycode.pressed(KeyCode::KeyH) {
                bloom_settings.prefilter_settings.threshold += dt;
            }
            bloom_settings.prefilter_settings.threshold =
                bloom_settings.prefilter_settings.threshold.max(0.0);

            if keycode.pressed(KeyCode::KeyU) {
                bloom_settings.prefilter_settings.threshold_softness -= dt / 10.0;
            }
            if keycode.pressed(KeyCode::KeyJ) {
                bloom_settings.prefilter_settings.threshold_softness += dt / 10.0;
            }
            bloom_settings.prefilter_settings.threshold_softness = bloom_settings
                .prefilter_settings
                .threshold_softness
                .clamp(0.0, 1.0);
        }

        (entity, None) => {
            text.clear();
            text.push_str("Bloom disabled (Space to toggle)");

            if keycode.just_pressed(KeyCode::Space) {
                commands.entity(entity).insert(BloomSettings::default());
            }
        }
    }
}
