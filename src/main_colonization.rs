//! Space colonization game with resource management

use bevy::{
    prelude::*,
    sprite::MaterialMesh2dBundle,
    window::PrimaryWindow,
};
use rand::prelude::*;
use std::collections::HashMap;

// Resource types in the game
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum ResourceType {
    // Basic life resources
    Water,      // 💧
    Oxygen,     // 🌬️
    Food,       // 🌱
    
    // Construction minerals
    Iron,       // 🪨
    Copper,     // ⚡
    Silicon,    // 💻
    
    // Energy resources
    Uranium,    // ☢️
    Helium3,    // 🔋
    EnergyCrystal, // ✨
}

impl ResourceType {
    fn color(&self) -> Color {
        match self {
            ResourceType::Water => Color::srgb(0.0, 1.0, 1.0),
            ResourceType::Oxygen => Color::srgb(0.7, 0.9, 1.0),
            ResourceType::Food => Color::srgb(0.0, 1.0, 0.0),
            ResourceType::Iron => Color::srgb(0.5, 0.5, 0.6),
            ResourceType::Copper => Color::srgb(0.72, 0.45, 0.20),
            ResourceType::Silicon => Color::srgb(0.8, 0.8, 0.9),
            ResourceType::Uranium => Color::srgb(0.0, 1.0, 0.0),
            ResourceType::Helium3 => Color::srgb(1.0, 0.8, 0.0),
            ResourceType::EnergyCrystal => Color::srgb(1.0, 0.0, 1.0),
        }
    }

    fn icon(&self) -> &'static str {
        match self {
            ResourceType::Water => "💧",
            ResourceType::Oxygen => "🌬️",
            ResourceType::Food => "🌱",
            ResourceType::Iron => "🪨",
            ResourceType::Copper => "⚡",
            ResourceType::Silicon => "💻",
            ResourceType::Uranium => "☢️",
            ResourceType::Helium3 => "🔋",
            ResourceType::EnergyCrystal => "✨",
        }
    }

    fn name(&self) -> &'static str {
        match self {
            ResourceType::Water => "Water",
            ResourceType::Oxygen => "Oxygen",
            ResourceType::Food => "Food",
            ResourceType::Iron => "Iron",
            ResourceType::Copper => "Copper",
            ResourceType::Silicon => "Silicon",
            ResourceType::Uranium => "Uranium",
            ResourceType::Helium3 => "Helium-3",
            ResourceType::EnergyCrystal => "Energy Crystal",
        }
    }
}

// Components
#[derive(Component)]
struct Star {
    id: usize,
    name: String,
    resources: HashMap<ResourceType, f32>,
    max_resources: HashMap<ResourceType, f32>,
    production_rate: f32,  // Resources per second
    is_colonized: bool,
    is_home_star: bool,
}

#[derive(Component)]
struct Connection {
    from: Entity,
    to: Entity,
    collection_timer: Timer,
    is_collecting: bool,
}

#[derive(Component)]
struct DraggingLine;

#[derive(Component)]
struct StarInfoPanel;

#[derive(Component)]
struct ResourcePanel;

#[derive(Component)]
struct SelectedStar;

// Resources
#[derive(Resource, Default)]
struct DragState {
    is_dragging: bool,
    start_star: Option<Entity>,
    current_line: Option<Entity>,
}

#[derive(Resource)]
struct StarMaterials {
    normal: Handle<ColorMaterial>,
    hovered: Handle<ColorMaterial>,
    selected: Handle<ColorMaterial>,
    colonized: Handle<ColorMaterial>,
    home: Handle<ColorMaterial>,
}

#[derive(Resource)]
struct PlayerResources {
    resources: HashMap<ResourceType, f32>,
}

impl Default for PlayerResources {
    fn default() -> Self {
        let mut resources = HashMap::new();
        // Start with a small amount of each resource
        resources.insert(ResourceType::Water, 50.0);
        resources.insert(ResourceType::Oxygen, 30.0);
        resources.insert(ResourceType::Food, 40.0);
        resources.insert(ResourceType::Iron, 20.0);
        resources.insert(ResourceType::Copper, 15.0);
        resources.insert(ResourceType::Silicon, 10.0);
        resources.insert(ResourceType::Uranium, 5.0);
        resources.insert(ResourceType::Helium3, 2.0);
        resources.insert(ResourceType::EnergyCrystal, 1.0);
        Self { resources }
    }
}

#[derive(Resource)]
struct GameState {
    selected_star: Option<Entity>,
}

// Star name generator
fn generate_star_name(rng: &mut ThreadRng) -> String {
    let prefixes = ["Alpha", "Beta", "Gamma", "Delta", "Epsilon", "Zeta", "Eta", "Theta", "Iota", "Kappa"];
    let suffixes = ["Centauri", "Orionis", "Draconis", "Pegasi", "Andromedae", "Leonis", "Aquarii", "Scorpii", "Tauri", "Geminorum"];
    let prefix = prefixes[rng.gen_range(0..prefixes.len())];
    let suffix = suffixes[rng.gen_range(0..suffixes.len())];
    format!("{} {}", prefix, suffix)
}

// Generate random resources for a star
fn generate_star_resources(rng: &mut ThreadRng, is_home: bool) -> (HashMap<ResourceType, f32>, HashMap<ResourceType, f32>) {
    let mut resources = HashMap::new();
    let mut max_resources = HashMap::new();
    
    if is_home {
        // Home star has balanced resources
        for resource in [
            ResourceType::Water, ResourceType::Oxygen, ResourceType::Food,
            ResourceType::Iron, ResourceType::Copper, ResourceType::Silicon,
        ] {
            let amount = rng.gen_range(100.0..200.0);
            resources.insert(resource, amount);
            max_resources.insert(resource, amount);
        }
    } else {
        // Other stars have random resources (1-3 types)
        let num_resources = rng.gen_range(1..=3);
        let all_resources = [
            ResourceType::Water, ResourceType::Oxygen, ResourceType::Food,
            ResourceType::Iron, ResourceType::Copper, ResourceType::Silicon,
            ResourceType::Uranium, ResourceType::Helium3, ResourceType::EnergyCrystal,
        ];
        
        let mut selected_resources = all_resources.to_vec();
        selected_resources.shuffle(rng);
        
        for i in 0..num_resources {
            let resource = selected_resources[i];
            let amount = match resource {
                ResourceType::EnergyCrystal | ResourceType::Helium3 => rng.gen_range(5.0..30.0),
                ResourceType::Uranium => rng.gen_range(10.0..50.0),
                _ => rng.gen_range(50.0..150.0),
            };
            resources.insert(resource, amount);
            max_resources.insert(resource, amount);
        }
    }
    
    (resources, max_resources)
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_resource::<DragState>()
        .init_resource::<PlayerResources>()
        .insert_resource(GameState { selected_star: None })
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                star_hover_system,
                handle_mouse_input,
                star_selection_system,
                update_dragging_line,
                update_connections,
                collect_resources_system,
                update_ui,
            ),
        )
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // Camera
    commands.spawn(Camera2dBundle::default());

    // Store materials for stars
    let star_materials = StarMaterials {
        normal: materials.add(ColorMaterial::from(Color::srgb(0.8, 0.8, 0.8))),
        hovered: materials.add(ColorMaterial::from(Color::srgb(1.0, 1.0, 0.8))),
        selected: materials.add(ColorMaterial::from(Color::srgb(1.0, 1.0, 0.5))),
        colonized: materials.add(ColorMaterial::from(Color::srgb(0.5, 0.8, 1.0))),
        home: materials.add(ColorMaterial::from(Color::srgb(0.2, 1.0, 0.2))),
    };

    // Create star mesh handle
    let star_mesh = meshes.add(Circle::new(25.0));

    // Spawn stars at random positions with minimum distance
    let mut rng = rand::thread_rng();
    let mut positions: Vec<Vec2> = Vec::new();
    let min_distance = 90.0;
    let max_attempts = 500;
    let margin = 50.0;
    let num_stars = 12;
    
    // Generate home star first (center position)
    let home_pos = Vec2::new(0.0, 0.0);
    positions.push(home_pos);
    
    let (home_resources, home_max) = generate_star_resources(&mut rng, true);
    let _home_star = commands.spawn((
        MaterialMesh2dBundle {
            mesh: star_mesh.clone().into(),
            material: star_materials.home.clone(),
            transform: Transform::from_xyz(home_pos.x, home_pos.y, 1.0),
            ..default()
        },
        Star {
            id: 0,
            name: "Sol System".to_string(),
            resources: home_resources,
            max_resources: home_max,
            production_rate: 2.0,
            is_colonized: true,
            is_home_star: true,
        },
    )).id();
    
    // Generate other stars
    for i in 1..num_stars {
        let mut position_found = false;
        let mut attempts = 0;
        let mut pos = Vec2::ZERO;
        
        while !position_found && attempts < max_attempts {
            let x = rng.gen_range((-350.0 + margin)..(350.0 - margin));
            let y = rng.gen_range((-250.0 + margin)..(250.0 - margin));
            pos = Vec2::new(x, y);
            
            position_found = true;
            for existing_pos in &positions {
                if pos.distance(*existing_pos) < min_distance {
                    position_found = false;
                    break;
                }
            }
            
            attempts += 1;
        }
        
        positions.push(pos);
        
        let (star_resources, star_max) = generate_star_resources(&mut rng, false);
        commands.spawn((
            MaterialMesh2dBundle {
                mesh: star_mesh.clone().into(),
                material: star_materials.normal.clone(),
                transform: Transform::from_xyz(pos.x, pos.y, 1.0),
                ..default()
            },
            Star {
                id: i,
                name: generate_star_name(&mut rng),
                resources: star_resources,
                max_resources: star_max,
                production_rate: rng.gen_range(0.5..2.5),
                is_colonized: false,
                is_home_star: false,
            },
        ));
    }

    commands.insert_resource(star_materials);

    // UI Setup
    // Title
    commands.spawn(
        TextBundle::from_section(
            "ZODIAKOS - Space Colonization",
            TextStyle {
                font_size: 24.0,
                color: Color::WHITE,
                ..default()
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        }),
    );

    // Instructions
    commands.spawn(
        TextBundle::from_section(
            "Click stars to select | Drag to connect and colonize",
            TextStyle {
                font_size: 14.0,
                color: Color::srgb(0.8, 0.8, 0.8),
                ..default()
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(40.0),
            left: Val::Px(10.0),
            ..default()
        }),
    );

    // Resource panel
    commands.spawn((
        TextBundle::from_section(
            "Resources:",
            TextStyle {
                font_size: 16.0,
                color: Color::WHITE,
                ..default()
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(70.0),
            right: Val::Px(10.0),
            ..default()
        }),
        ResourcePanel,
    ));

    // Star info panel
    commands.spawn((
        TextBundle::from_section(
            "",
            TextStyle {
                font_size: 14.0,
                color: Color::WHITE,
                ..default()
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            bottom: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        }),
        StarInfoPanel,
    ));
}

fn star_hover_system(
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mut star_query: Query<(&Transform, &mut Handle<ColorMaterial>, Entity, &Star), Without<SelectedStar>>,
    star_materials: Res<StarMaterials>,
    drag_state: Res<DragState>,
) {
    let Ok(window) = windows.get_single() else {
        return;
    };
    
    let Ok((camera, camera_transform)) = camera_q.get_single() else {
        return;
    };

    let Some(cursor_pos) = window.cursor_position()
        .and_then(|cursor| camera.viewport_to_world_2d(camera_transform, cursor))
    else {
        return;
    };

    for (transform, mut material, entity, star) in &mut star_query {
        let distance = transform.translation.truncate().distance(cursor_pos);
        
        if distance < 25.0 {
            if drag_state.start_star == Some(entity) {
                *material = star_materials.selected.clone();
            } else {
                *material = star_materials.hovered.clone();
            }
        } else if drag_state.start_star != Some(entity) {
            if star.is_home_star {
                *material = star_materials.home.clone();
            } else if star.is_colonized {
                *material = star_materials.colonized.clone();
            } else {
                *material = star_materials.normal.clone();
            }
        }
    }
}

fn star_selection_system(
    mouse_button: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    star_query: Query<(&Transform, Entity, &Star)>,
    mut game_state: ResMut<GameState>,
    mut commands: Commands,
) {
    if !mouse_button.just_pressed(MouseButton::Left) {
        return;
    }

    let Ok(window) = windows.get_single() else {
        return;
    };
    
    let Ok((camera, camera_transform)) = camera_q.get_single() else {
        return;
    };

    let Some(cursor_pos) = window.cursor_position()
        .and_then(|cursor| camera.viewport_to_world_2d(camera_transform, cursor))
    else {
        return;
    };

    // Clear previous selection
    if let Some(prev_selected) = game_state.selected_star {
        commands.entity(prev_selected).remove::<SelectedStar>();
    }

    // Check if we clicked on a star
    for (transform, entity, _star) in &star_query {
        let distance = transform.translation.truncate().distance(cursor_pos);
        
        if distance < 25.0 {
            game_state.selected_star = Some(entity);
            commands.entity(entity).insert(SelectedStar);
            break;
        }
    }
}

fn handle_mouse_input(
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mut star_query: Query<(&Transform, Entity, &mut Star)>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut drag_state: ResMut<DragState>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    existing_connections: Query<&Connection>,
) {
    let Ok(window) = windows.get_single() else {
        return;
    };
    
    let Ok((camera, camera_transform)) = camera_q.get_single() else {
        return;
    };

    let Some(cursor_pos) = window.cursor_position()
        .and_then(|cursor| camera.viewport_to_world_2d(camera_transform, cursor))
    else {
        return;
    };

    if mouse_button.just_pressed(MouseButton::Left) {
        // Check if we clicked on a colonized star
        for (transform, entity, star) in &star_query {
            let distance = transform.translation.truncate().distance(cursor_pos);
            
            if distance < 25.0 && star.is_colonized {
                drag_state.is_dragging = true;
                drag_state.start_star = Some(entity);
                
                let line_entity = commands.spawn((
                    MaterialMesh2dBundle {
                        mesh: meshes.add(Rectangle::new(1.0, 1.0)).into(),
                        material: materials.add(ColorMaterial::from(Color::srgba(0.5, 1.0, 0.5, 0.5))),
                        transform: Transform::from_xyz(0.0, 0.0, 0.0),
                        ..default()
                    },
                    DraggingLine,
                )).id();
                
                drag_state.current_line = Some(line_entity);
                break;
            }
        }
    }

    if mouse_button.just_released(MouseButton::Left) && drag_state.is_dragging {
        // Check if we released on a different star
        for (transform, entity, mut star) in &mut star_query {
            let distance = transform.translation.truncate().distance(cursor_pos);
            
            if distance < 25.0 && Some(entity) != drag_state.start_star && !star.is_colonized {
                // Check if connection already exists
                let mut connection_exists = false;
                for connection in &existing_connections {
                    if (connection.from == drag_state.start_star.unwrap() && connection.to == entity) ||
                       (connection.to == drag_state.start_star.unwrap() && connection.from == entity) {
                        connection_exists = true;
                        break;
                    }
                }
                
                if !connection_exists {
                    if let Some(start_star) = drag_state.start_star {
                        // Colonize the star
                        star.is_colonized = true;
                        
                        // Create a connection
                        commands.spawn((
                            MaterialMesh2dBundle {
                                mesh: meshes.add(Rectangle::new(1.0, 1.0)).into(),
                                material: materials.add(ColorMaterial::from(Color::srgb(0.2, 0.8, 0.2))),
                                transform: Transform::from_xyz(0.0, 0.0, -1.0),
                                ..default()
                            },
                            Connection {
                                from: start_star,
                                to: entity,
                                collection_timer: Timer::from_seconds(2.0, TimerMode::Repeating),
                                is_collecting: true,
                            },
                        ));
                    }
                }
                break;
            }
        }
        
        // Clean up the temporary line
        if let Some(line_entity) = drag_state.current_line {
            commands.entity(line_entity).despawn();
        }
        
        // Reset drag state
        drag_state.is_dragging = false;
        drag_state.start_star = None;
        drag_state.current_line = None;
    }
}

fn update_dragging_line(
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    star_query: Query<&Transform, With<Star>>,
    mut line_query: Query<&mut Transform, (With<DraggingLine>, Without<Star>)>,
    drag_state: Res<DragState>,
) {
    if !drag_state.is_dragging {
        return;
    }

    let Ok(window) = windows.get_single() else {
        return;
    };
    
    let Ok((camera, camera_transform)) = camera_q.get_single() else {
        return;
    };

    let Some(cursor_pos) = window.cursor_position()
        .and_then(|cursor| camera.viewport_to_world_2d(camera_transform, cursor))
    else {
        return;
    };

    if let Some(start_star) = drag_state.start_star {
        if let Ok(start_transform) = star_query.get(start_star) {
            let start_pos = start_transform.translation.truncate();
            
            if let Some(line_entity) = drag_state.current_line {
                if let Ok(mut line_transform) = line_query.get_mut(line_entity) {
                    let direction = cursor_pos - start_pos;
                    let length = direction.length();
                    let angle = direction.y.atan2(direction.x);
                    
                    line_transform.translation.x = start_pos.x + direction.x / 2.0;
                    line_transform.translation.y = start_pos.y + direction.y / 2.0;
                    line_transform.rotation = Quat::from_rotation_z(angle);
                    line_transform.scale.x = length;
                    line_transform.scale.y = 4.0;
                }
            }
        }
    }
}

fn update_connections(
    star_query: Query<&Transform, With<Star>>,
    mut connection_query: Query<(&mut Transform, &Connection), Without<Star>>,
) {
    for (mut line_transform, connection) in &mut connection_query {
        if let Ok(from_transform) = star_query.get(connection.from) {
            if let Ok(to_transform) = star_query.get(connection.to) {
                let start_pos = from_transform.translation.truncate();
                let end_pos = to_transform.translation.truncate();
                let direction = end_pos - start_pos;
                let length = direction.length();
                let angle = direction.y.atan2(direction.x);
                
                line_transform.translation.x = start_pos.x + direction.x / 2.0;
                line_transform.translation.y = start_pos.y + direction.y / 2.0;
                line_transform.translation.z = -1.0;
                line_transform.rotation = Quat::from_rotation_z(angle);
                line_transform.scale.x = length;
                line_transform.scale.y = 4.0;
            }
        }
    }
}

fn collect_resources_system(
    time: Res<Time>,
    mut connection_query: Query<&mut Connection>,
    mut star_query: Query<&mut Star>,
    mut player_resources: ResMut<PlayerResources>,
) {
    for mut connection in &mut connection_query {
        if connection.is_collecting {
            connection.collection_timer.tick(time.delta());
            
            if connection.collection_timer.just_finished() {
                // Collect resources from the connected star
                if let Ok(mut star) = star_query.get_mut(connection.to) {
                    let production_rate = star.production_rate;
                    for (resource_type, amount) in star.resources.iter_mut() {
                        let collection_amount = (production_rate * 5.0).min(*amount);
                        if collection_amount > 0.0 {
                            *amount -= collection_amount;
                            *player_resources.resources.entry(*resource_type).or_insert(0.0) += collection_amount;
                        }
                    }
                    
                    // Check if star is depleted
                    let total_resources: f32 = star.resources.values().sum();
                    if total_resources < 0.1 {
                        connection.is_collecting = false;
                    }
                }
            }
        }
    }
}

fn update_ui(
    player_resources: Res<PlayerResources>,
    mut resource_panel_query: Query<&mut Text, (With<ResourcePanel>, Without<StarInfoPanel>)>,
    mut star_info_query: Query<&mut Text, (With<StarInfoPanel>, Without<ResourcePanel>)>,
    star_query: Query<&Star>,
    game_state: Res<GameState>,
) {
    // Update resource panel
    if let Ok(mut text) = resource_panel_query.get_single_mut() {
        let mut resource_text = "=== RESOURCES ===\n".to_string();
        
        for (resource_type, amount) in &player_resources.resources {
            resource_text.push_str(&format!(
                "{} {}: {:.1}\n",
                resource_type.icon(),
                resource_type.name(),
                amount
            ));
        }
        
        text.sections[0].value = resource_text;
    }
    
    // Update star info panel
    if let Ok(mut text) = star_info_query.get_single_mut() {
        if let Some(selected_entity) = game_state.selected_star {
            if let Ok(star) = star_query.get(selected_entity) {
                let mut info_text = format!(
                    "=== STAR INFO ===\n{} (ID: {})\n",
                    star.name, star.id
                );
                
                if star.is_home_star {
                    info_text.push_str("HOME SYSTEM\n");
                }
                
                if star.is_colonized {
                    info_text.push_str("Status: COLONIZED\n");
                } else {
                    info_text.push_str("Status: UNCOLONIZED\n");
                }
                
                info_text.push_str(&format!("Production Rate: {:.1}/s\n", star.production_rate));
                info_text.push_str("\nResources:\n");
                
                for (resource_type, amount) in &star.resources {
                    let max = star.max_resources.get(resource_type).unwrap_or(&0.0);
                    info_text.push_str(&format!(
                        "{} {}: {:.1}/{:.1}\n",
                        resource_type.icon(),
                        resource_type.name(),
                        amount,
                        max
                    ));
                }
                
                text.sections[0].value = info_text;
            }
        } else {
            text.sections[0].value = "Click on a star to see details".to_string();
        }
    }
}