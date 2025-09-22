//! Space colonization game with resource management

use bevy::{
    prelude::*,
    sprite::MaterialMesh2dBundle,
    window::PrimaryWindow,
    ecs::system::ParamSet,
    core_pipeline::bloom::*,
};
use rand::prelude::*;
use std::collections::HashMap;

// Specialization types for stars
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Specialization {
    None,           // Default - collects resources
    Storage,        // Storage hub - increased resource capacity
    Military,       // Produces warships
    Mining,         // Produces mining ships (increases resource collection)
    Agriculture,    // Produces food and biological resources
    Research,       // Produces scientists and technology
    Medical,        // Produces medical units and health resources
    Industrial,     // Produces builders and construction units
}

impl Specialization {
    fn name(&self) -> &'static str {
        match self {
            Specialization::None => "Resource Extraction",
            Specialization::Storage => "Storage Hub",
            Specialization::Military => "Military Base",
            Specialization::Mining => "Mining Station",
            Specialization::Agriculture => "Agricultural Colony",
            Specialization::Research => "Research Center",
            Specialization::Medical => "Medical Facility",
            Specialization::Industrial => "Industrial Complex",
        }
    }
    
    fn build_time(&self) -> f32 {
        match self {
            Specialization::None => 5.0,        // 5 seconds to start extraction
            Specialization::Storage => 10.0,     // 10 seconds
            Specialization::Military => 20.0,    // 20 seconds
            Specialization::Mining => 15.0,      // 15 seconds
            Specialization::Agriculture => 12.0, // 12 seconds
            Specialization::Research => 25.0,    // 25 seconds
            Specialization::Medical => 15.0,     // 15 seconds
            Specialization::Industrial => 18.0,  // 18 seconds
        }
    }
    
    fn upgrade_time(&self, level: u8) -> f32 {
        let base_time = self.build_time();
        base_time * (level as f32 * 1.5) // Higher levels take longer
    }

    fn icon(&self) -> &'static str {
        match self {
            Specialization::None => "⛏️",
            Specialization::Storage => "📦",
            Specialization::Military => "🚀",
            Specialization::Mining => "⚒️",
            Specialization::Agriculture => "🌾",
            Specialization::Research => "🔬",
            Specialization::Medical => "🏥",
            Specialization::Industrial => "🏭",
        }
    }

    fn production_cost(&self, level: u8) -> Vec<(ResourceType, f32)> {
        let multiplier = 1.0 / (1.0 + (level - 1) as f32 * 0.2); // Higher levels are more efficient
        match self {
            Specialization::None => vec![],
            Specialization::Storage => vec![
                (ResourceType::Iron, 10.0 * multiplier),
                (ResourceType::Silicon, 5.0 * multiplier),
            ],
            Specialization::Military => vec![
                (ResourceType::Iron, 20.0 * multiplier),
                (ResourceType::Uranium, 10.0 * multiplier),
                (ResourceType::Silicon, 15.0 * multiplier),
            ],
            Specialization::Mining => vec![
                (ResourceType::Iron, 15.0 * multiplier),
                (ResourceType::Copper, 10.0 * multiplier),
            ],
            Specialization::Agriculture => vec![
                (ResourceType::Water, 20.0 * multiplier),
                (ResourceType::Food, 10.0 * multiplier),
            ],
            Specialization::Research => vec![
                (ResourceType::Silicon, 20.0 * multiplier),
                (ResourceType::EnergyCrystal, 2.0 * multiplier),
            ],
            Specialization::Medical => vec![
                (ResourceType::Oxygen, 15.0 * multiplier),
                (ResourceType::Water, 10.0 * multiplier),
            ],
            Specialization::Industrial => vec![
                (ResourceType::Iron, 25.0 * multiplier),
                (ResourceType::Copper, 15.0 * multiplier),
                (ResourceType::Silicon, 10.0 * multiplier),
            ],
        }
    }
}

// Unit types produced by specialized stars
#[derive(Debug, Clone)]
struct Unit {
    unit_type: UnitType,
    count: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UnitType {
    Warship,
    MiningShip,
    Farmer,
    Scientist,
    Doctor,
    Builder,
    StorageModule,
}

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

// Building state for stars
#[derive(Debug, Clone, Copy, PartialEq)]
enum BuildingState {
    Ready,
    Building { timer: f32, total_time: f32 },
    Upgrading { timer: f32, total_time: f32 },
}

// Calculate Fibonacci number for connection limit
fn fibonacci(n: u8) -> u32 {
    if n <= 1 {
        return 1;
    }
    
    let mut a = 1u32;
    let mut b = 2u32;
    
    for _ in 2..n {
        let temp = a + b;
        a = b;
        b = temp;
    }
    
    b
}

// Get maximum connections based on star level
fn max_connections_for_level(level: u8) -> u32 {
    fibonacci(level)
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
    specialization: Specialization, // None = extraction; other = specialization (stops extraction)
    specialization_level: u8,      // Level (no limit, follows Fibonacci for connections)
    units: Vec<Unit>,               // Units produced if specialized
    building_state: BuildingState,  // Current construction/upgrade state
    connections_from: Vec<Entity>,  // List of stars connected TO this star
    connections_to: Vec<Entity>,    // List of stars this star connects TO
    base_color: Color,              // Base color based on resources
}

#[derive(Component)]
struct Connection {
    from: Entity,
    to: Entity,
    collection_timer: Timer,
    is_collecting: bool,
    creation_time: f32,  // Time in seconds since creation
}

#[derive(Component)]
struct ConnectionLine;

#[derive(Resource)]
struct SelectedConnection {
    entity: Entity,
    from: Entity,
    to: Entity,
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

// Get star color based on dominant resource type
fn get_star_color_from_resources(resources: &HashMap<ResourceType, f32>) -> Color {
    // Find the dominant resource type
    let mut dominant_resource = None;
    let mut max_amount = 0.0;
    
    for (resource_type, amount) in resources {
        if *amount > max_amount {
            max_amount = *amount;
            dominant_resource = Some(*resource_type);
        }
    }
    
    // Return color based on dominant resource with HDR values for bloom
    match dominant_resource {
        Some(ResourceType::Water) => Color::srgba(0.3, 0.6, 4.0, 1.0),      // Deep blue
        Some(ResourceType::Oxygen) => Color::srgba(0.7, 3.0, 4.0, 1.0),     // Cyan
        Some(ResourceType::Food) => Color::srgba(0.3, 4.0, 0.3, 1.0),       // Green
        Some(ResourceType::Iron) => Color::srgba(2.5, 2.5, 3.0, 1.0),       // Silver-gray
        Some(ResourceType::Copper) => Color::srgba(4.0, 2.0, 0.8, 1.0),     // Orange-copper
        Some(ResourceType::Silicon) => Color::srgba(3.0, 3.0, 4.0, 1.0),    // Light blue-white
        Some(ResourceType::Uranium) => Color::srgba(0.5, 4.0, 0.5, 1.0),    // Radioactive green
        Some(ResourceType::Helium3) => Color::srgba(4.0, 3.0, 0.0, 1.0),    // Yellow-gold
        Some(ResourceType::EnergyCrystal) => Color::srgba(4.0, 0.5, 4.0, 1.0), // Purple
        None => Color::srgba(3.0, 3.0, 3.0, 1.0),                           // Default white
    }
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
                connection_selection_system,
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
    // Camera with HDR and Bloom
    commands.spawn((
        Camera2dBundle::default(),
        BloomSettings {
            intensity: 1.0,
            low_frequency_boost: 0.5,
            low_frequency_boost_curvature: 0.5,
            high_pass_frequency: 1.0,
            prefilter_settings: BloomPrefilterSettings {
                threshold: 0.2,
                threshold_softness: 0.2,
            },
            composite_mode: BloomCompositeMode::Additive,
        },
    ));

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
    // Home star has a special golden color
    let home_color = Color::srgba(4.0, 3.5, 0.5, 1.0);
    let _home_star = commands.spawn((
        MaterialMesh2dBundle {
            mesh: star_mesh.clone().into(),
            material: materials.add(ColorMaterial::from(home_color)),
            transform: Transform::from_xyz(home_pos.x, home_pos.y, 1.0),
            ..default()
        },
        Star {
            id: 0,
            name: "Sol System".to_string(),
            resources: home_resources.clone(),
            max_resources: home_max,
            production_rate: 2.0,
            is_colonized: true,
            is_home_star: true,
            specialization: Specialization::None,
            specialization_level: 1,
            units: vec![],
            building_state: BuildingState::Ready,
            connections_from: vec![],
            connections_to: vec![],
            base_color: home_color,
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
        let star_color = get_star_color_from_resources(&star_resources);
        commands.spawn((
            MaterialMesh2dBundle {
                mesh: star_mesh.clone().into(),
                material: materials.add(ColorMaterial::from(star_color)),
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
                specialization: Specialization::None,
                specialization_level: 1,
                units: vec![],
                building_state: BuildingState::Ready,
                connections_from: vec![],
                connections_to: vec![],
                base_color: star_color,
            },
        ));
    }

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
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mut star_query: Query<(&Transform, &mut Handle<ColorMaterial>, Entity, &Star), Without<SelectedStar>>,
    drag_state: Res<DragState>,
    mut materials: ResMut<Assets<ColorMaterial>>,
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

    for (transform, material_handle, entity, star) in &mut star_query {
        let distance = transform.translation.truncate().distance(cursor_pos);
        
        // Get or create material based on hover state
        if let Some(material) = materials.get_mut(material_handle.id()) {
            let base = star.base_color;
            
            if distance < 25.0 {
                if drag_state.start_star == Some(entity) {
                    // Selected: brighten the color significantly
                    if let Color::Srgba(srgba) = base {
                        material.color = Color::srgba(
                            (srgba.red * 1.5).min(10.0),
                            (srgba.green * 1.5).min(10.0),
                            (srgba.blue * 1.5).min(10.0),
                            srgba.alpha
                        );
                    }
                } else {
                    // Hovered: slightly brighten
                    if let Color::Srgba(srgba) = base {
                        material.color = Color::srgba(
                            (srgba.red * 1.2).min(10.0),
                            (srgba.green * 1.2).min(10.0),
                            (srgba.blue * 1.2).min(10.0),
                            srgba.alpha
                        );
                    }
                }
            } else if drag_state.start_star != Some(entity) {
                // Not hovered: return to base color
                material.color = base;
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
        // First, find target star and check basic conditions
        let mut target_star_data = None;
        for (transform, entity, star) in star_query.iter() {
            let distance = transform.translation.truncate().distance(cursor_pos);
            if distance < 25.0 && Some(entity) != drag_state.start_star {
                target_star_data = Some((entity, star.is_colonized));
                break;
            }
        }
        
        // Process connection if we found a target
        if let Some((target_entity, target_is_colonized)) = target_star_data {
            if let Some(start_star_entity) = drag_state.start_star {
                // Check connection limit
                let can_connect = if let Ok((_transform, _entity, start_star)) = star_query.get(start_star_entity) {
                    let max_connections = max_connections_for_level(start_star.specialization_level);
                    let current_connections = start_star.connections_to.len() as u32;
                    current_connections < max_connections
                } else {
                    false
                };
                
                if can_connect {
                    // Check if connection already exists
                    let mut connection_exists = false;
                    for connection in &existing_connections {
                        if connection.from == start_star_entity && connection.to == target_entity {
                            connection_exists = true;
                            break;
                        }
                    }
                    
                    if !connection_exists {
                        // Update target star
                        if let Ok((_transform, _entity, mut target_star)) = star_query.get_mut(target_entity) {
                            if !target_is_colonized {
                                target_star.is_colonized = true;
                            }
                            target_star.connections_from.push(start_star_entity);
                        }
                        
                        // Update source star
                        if let Ok((_transform, _entity, mut start_star)) = star_query.get_mut(start_star_entity) {
                            start_star.connections_to.push(target_entity);
                        }
                        
                        // Create a connection
                        commands.spawn((
                            MaterialMesh2dBundle {
                                mesh: meshes.add(Rectangle::new(1.0, 1.0)).into(),
                                material: materials.add(ColorMaterial::from(Color::srgb(0.2, 0.8, 0.2))),
                                transform: Transform::from_xyz(0.0, 0.0, -1.0),
                                ..default()
                            },
                            Connection {
                                from: start_star_entity,
                                to: target_entity,
                                collection_timer: Timer::from_seconds(2.0, TimerMode::Repeating),
                                is_collecting: true,
                                creation_time: 0.0,
                            },
                            ConnectionLine,
                        ));
                    }
                }
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
    mut connection_query: Query<(&mut Transform, &mut Connection), Without<Star>>,
    time: Res<Time>,
) {
    for (mut line_transform, mut connection) in &mut connection_query {
        // Update creation time
        connection.creation_time += time.delta_seconds();
        
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

fn connection_selection_system(
    mouse_button: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    connection_query: Query<(Entity, &Connection, &Transform), With<ConnectionLine>>,
    star_query: Query<&Transform, With<Star>>,
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut stars: Query<&mut Star>,
    selected_connection: Option<Res<SelectedConnection>>,
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

    // Check for delete key press on selected connection
    if keyboard.just_pressed(KeyCode::Delete) {
        if let Some(selected) = selected_connection {
            // Remove connection from stars
            if let Ok(mut from_star) = stars.get_mut(selected.from) {
                from_star.connections_to.retain(|&x| x != selected.to);
            }
            if let Ok(mut to_star) = stars.get_mut(selected.to) {
                to_star.connections_from.retain(|&x| x != selected.from);
            }
            
            // Delete the connection entity
            commands.entity(selected.entity).despawn();
            commands.remove_resource::<SelectedConnection>();
            return;
        }
    }

    if !mouse_button.just_pressed(MouseButton::Left) {
        return;
    }

    // Clear previous selection
    commands.remove_resource::<SelectedConnection>();

    // Check if we clicked on a connection
    for (entity, connection, transform) in &connection_query {
        // Get start and end positions
        if let Ok(from_transform) = star_query.get(connection.from) {
            if let Ok(to_transform) = star_query.get(connection.to) {
                let start_pos = from_transform.translation.truncate();
                let end_pos = to_transform.translation.truncate();
                
                // Check if click is near the line segment
                let line_vec = end_pos - start_pos;
                let click_vec = cursor_pos - start_pos;
                let line_len_sq = line_vec.length_squared();
                
                if line_len_sq > 0.0 {
                    // Calculate projection of click onto line
                    let t = (click_vec.dot(line_vec) / line_len_sq).clamp(0.0, 1.0);
                    let closest_point = start_pos + line_vec * t;
                    let distance = cursor_pos.distance(closest_point);
                    
                    // Check if click is close enough to the line
                    if distance < 10.0 {
                        commands.insert_resource(SelectedConnection {
                            entity,
                            from: connection.from,
                            to: connection.to,
                        });
                        break;
                    }
                }
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
    // First, update building timers
    for mut star in &mut star_query {
        match star.building_state {
            BuildingState::Building { mut timer, total_time } => {
                timer -= time.delta_seconds();
                if timer <= 0.0 {
                    star.building_state = BuildingState::Ready;
                } else {
                    star.building_state = BuildingState::Building { timer, total_time };
                }
            },
            BuildingState::Upgrading { mut timer, total_time } => {
                timer -= time.delta_seconds();
                if timer <= 0.0 {
                    star.building_state = BuildingState::Ready;
                    star.specialization_level = star.specialization_level + 1; // No limit on levels
                } else {
                    star.building_state = BuildingState::Upgrading { timer, total_time };
                }
            },
            _ => {}
        }
    }
    
    for mut connection in &mut connection_query {
        if connection.is_collecting {
            connection.collection_timer.tick(time.delta());
            
            if connection.collection_timer.just_finished() {
                // Collect resources from the connected star
                if let Ok(mut star) = star_query.get_mut(connection.to) {
                    // Only produce if building is ready
                    if star.building_state != BuildingState::Ready {
                        continue;
                    }
                    
                    // Only collect resources if star is not specialized
                    if star.specialization == Specialization::None {
                        let production_rate = star.production_rate;
                        for (resource_type, amount) in star.resources.iter_mut() {
                            let collection_amount = (production_rate * 5.0).min(*amount);
                            if collection_amount > 0.0 {
                                *amount -= collection_amount;
                                *player_resources.resources.entry(*resource_type).or_insert(0.0) += collection_amount;
                            }
                        }
                    } else {
                        // Specialized star: consume resources and produce units
                        let production_costs = star.specialization.production_cost(star.specialization_level);
                        let mut can_produce = true;
                        
                        // Check if we have enough resources
                        for (resource_type, cost) in &production_costs {
                            if *player_resources.resources.get(resource_type).unwrap_or(&0.0) < *cost {
                                can_produce = false;
                                break;
                            }
                        }
                        
                        // Produce units if we have resources
                        if can_produce {
                            // Consume resources
                            for (resource_type, cost) in &production_costs {
                                *player_resources.resources.get_mut(resource_type).unwrap() -= cost;
                            }
                            
                            // Produce units based on specialization (more at higher levels)
                            let level_bonus = star.specialization_level;
                            match star.specialization {
                                Specialization::Military => {
                                    star.units.push(Unit { unit_type: UnitType::Warship, count: level_bonus as u32 });
                                },
                                Specialization::Mining => {
                                    star.units.push(Unit { unit_type: UnitType::MiningShip, count: (2 * level_bonus) as u32 });
                                },
                                Specialization::Agriculture => {
                                    star.units.push(Unit { unit_type: UnitType::Farmer, count: (3 * level_bonus) as u32 });
                                },
                                Specialization::Research => {
                                    star.units.push(Unit { unit_type: UnitType::Scientist, count: level_bonus as u32 });
                                },
                                Specialization::Medical => {
                                    star.units.push(Unit { unit_type: UnitType::Doctor, count: (2 * level_bonus) as u32 });
                                },
                                Specialization::Industrial => {
                                    star.units.push(Unit { unit_type: UnitType::Builder, count: (2 * level_bonus) as u32 });
                                },
                                Specialization::Storage => {
                                    star.units.push(Unit { unit_type: UnitType::StorageModule, count: level_bonus as u32 });
                                },
                                _ => {}
                            }
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
    mut star_queries: ParamSet<(
        Query<&Star>,
        Query<&mut Star>,
    )>,
    game_state: Res<GameState>,
    keyboard: Res<ButtonInput<KeyCode>>,
    connection_query: Query<(Entity, &Connection, &Transform), With<ConnectionLine>>,
    mut commands: Commands,
    selected_connection: Option<Res<SelectedConnection>>,
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
            // First read the star data
            let star_data = star_queries.p0().get(selected_entity).ok().map(|star| {
                (
                    star.id,
                    star.name.clone(),
                    star.is_home_star,
                    star.is_colonized,
                    star.specialization,
                    star.specialization_level,
                    star.production_rate,
                    star.units.clone(),
                    star.resources.clone(),
                    star.max_resources.clone(),
                    star.building_state,
                    star.connections_from.len() as u32,
                    star.connections_to.len() as u32,
                )
            });
            
            if let Some((id, name, is_home_star, is_colonized, specialization, level, production_rate, units, resources, max_resources, building_state, connections_from, connections_to)) = star_data {
                let mut info_text = format!(
                    "=== STAR INFO ===\n{} (ID: {})\n",
                    name, id
                );
                
                if is_home_star {
                    info_text.push_str("HOME SYSTEM\n");
                }
                
                if is_colonized {
                    info_text.push_str("Status: COLONIZED\n");
                    info_text.push_str(&format!("Specialization: {} {} (Level {})
", 
                        specialization.icon(), 
                        specialization.name(),
                        level
                    ));
                    
                    // Show building state
                    match building_state {
                        BuildingState::Building { timer, total_time } => {
                            let progress = ((total_time - timer) / total_time * 100.0) as u32;
                            info_text.push_str(&format!("⚙️ BUILDING: {}% complete ({:.1}s remaining)\n", progress, timer));
                        },
                        BuildingState::Upgrading { timer, total_time } => {
                            let progress = ((total_time - timer) / total_time * 100.0) as u32;
                            info_text.push_str(&format!("⬆️ UPGRADING: {}% complete ({:.1}s remaining)\n", progress, timer));
                        },
                        BuildingState::Ready => {
                            info_text.push_str("✅ OPERATIONAL\n");
                        }
                    }
                    
                    // Show specialization options if colonized
                    if !is_home_star && building_state == BuildingState::Ready {
                        info_text.push_str("\n=== CHANGE SPECIALIZATION ===\n");
                        info_text.push_str("[1] Resource Extraction\n");
                        info_text.push_str("[2] Storage Hub\n");
                        info_text.push_str("[3] Military Base\n");
                        info_text.push_str("[4] Mining Station\n");
                        info_text.push_str("[5] Agricultural Colony\n");
                        info_text.push_str("[6] Research Center\n");
                        info_text.push_str("[7] Medical Facility\n");
                        info_text.push_str("[8] Industrial Complex\n");
                        
                        // No level limit, show upgrade option and next Fibonacci connection limit
                        let next_max_conn = max_connections_for_level(level + 1);
                        info_text.push_str(&format!("\n[U] UPGRADE to Level {} (Next max connections: {})\n", level + 1, next_max_conn));
                        
                        // Show connection progression for next few levels
                        info_text.push_str("\nConnection Limit Progression:\n");
                        for i in 0..3 {
                            let future_level = level + i + 1;
                            info_text.push_str(&format!("  Level {}: {} connections\n", 
                                future_level, 
                                max_connections_for_level(future_level)
                            ));
                        }
                    }
                } else {
                    info_text.push_str("Status: UNCOLONIZED\n");
                }
                
                let max_conn = max_connections_for_level(level);
                info_text.push_str(&format!("Connections: {} inbound, {} outbound (max outbound: {})\n", connections_from, connections_to, max_conn));
                
                if specialization == Specialization::None {
                    info_text.push_str(&format!("Production Rate: {:.1}/s\n", production_rate));
                } else {
                    info_text.push_str("Production: SPECIALIZED\n");
                    
                    // Show units produced
                    if !units.is_empty() {
                        info_text.push_str("\nUnits Produced:\n");
                        for unit in &units {
                            info_text.push_str(&format!("  {} x{}\n", 
                                format!("{:?}", unit.unit_type), 
                                unit.count
                            ));
                        }
                    }
                    
                    // Show production cost
                    info_text.push_str("\nProduction Cost/cycle:\n");
                    for (resource_type, cost) in specialization.production_cost(level) {
                        info_text.push_str(&format!("  {} {}: {:.1}\n",
                            resource_type.icon(),
                            resource_type.name(),
                            cost
                        ));
                    }
                }
                
                info_text.push_str("\nResources:\n");
                
                for (resource_type, amount) in &resources {
                    let max = max_resources.get(resource_type).unwrap_or(&0.0);
                    info_text.push_str(&format!(
                        "{} {}: {:.1}/{:.1}\n",
                        resource_type.icon(),
                        resource_type.name(),
                        amount,
                        max
                    ));
                }
                
                text.sections[0].value = info_text;
                
                // Handle specialization selection
                if is_colonized && !is_home_star {
                    if let Ok(mut selected_star) = star_queries.p1().get_mut(selected_entity) {
                        let mut new_spec = None;
                        
                        if keyboard.just_pressed(KeyCode::Digit1) { new_spec = Some(Specialization::None); }
                        else if keyboard.just_pressed(KeyCode::Digit2) { new_spec = Some(Specialization::Storage); }
                        else if keyboard.just_pressed(KeyCode::Digit3) { new_spec = Some(Specialization::Military); }
                        else if keyboard.just_pressed(KeyCode::Digit4) { new_spec = Some(Specialization::Mining); }
                        else if keyboard.just_pressed(KeyCode::Digit5) { new_spec = Some(Specialization::Agriculture); }
                        else if keyboard.just_pressed(KeyCode::Digit6) { new_spec = Some(Specialization::Research); }
                        else if keyboard.just_pressed(KeyCode::Digit7) { new_spec = Some(Specialization::Medical); }
                        else if keyboard.just_pressed(KeyCode::Digit8) { new_spec = Some(Specialization::Industrial); }
                        
                        if let Some(spec) = new_spec {
                            if selected_star.specialization != spec {
                                selected_star.specialization = spec;
                                selected_star.specialization_level = 1; // Reset level when changing
                                selected_star.units.clear();
                                let build_time = spec.build_time();
                                selected_star.building_state = BuildingState::Building { 
                                    timer: build_time, 
                                    total_time: build_time 
                                };
                            }
                        }
                        
                        // Handle upgrade (no level limit)
                        if keyboard.just_pressed(KeyCode::KeyU) {
                            if selected_star.building_state == BuildingState::Ready {
                                let upgrade_time = selected_star.specialization.upgrade_time(selected_star.specialization_level);
                                selected_star.building_state = BuildingState::Upgrading { 
                                    timer: upgrade_time, 
                                    total_time: upgrade_time 
                                };
                            }
                        }
                    }
                }
            }
        } else if let Some(selected_conn) = selected_connection {
            // Show connection details
            let mut info = String::from("CONNECTION DETAILS\n");
            info.push_str("================\n\n");
            
            // Get connection details
            if let Ok((entity, connection, _transform)) = connection_query.get(selected_conn.entity) {
                // Get star names/IDs
                let from_info = if let Ok(star) = star_queries.p0().get(connection.from) {
                    if star.is_home_star {
                        "Home Star".to_string()
                    } else {
                        format!("Star (Level {})", star.specialization_level)
                    }
                } else {
                    "Unknown".to_string()
                };
                
                let to_info = if let Ok(star) = star_queries.p0().get(connection.to) {
                    if star.is_home_star {
                        "Home Star".to_string()
                    } else {
                        format!("Star (Level {})", star.specialization_level)
                    }
                } else {
                    "Unknown".to_string()
                };
                
                info.push_str(&format!("From: {}\n", from_info));
                info.push_str(&format!("To: {}\n", to_info));
                info.push_str(&format!("\nTime Existed: {:.1} seconds\n", connection.creation_time));
                info.push_str(&format!("Status: {}\n", if connection.is_collecting { "Active" } else { "Inactive" }));
                
                // Get owner info (the 'from' star is the owner)
                if let Ok(owner_star) = star_queries.p0().get(connection.from) {
                    info.push_str(&format!("\nOwner Specialization: {}\n", owner_star.specialization.name()));
                    info.push_str(&format!("Owner Level: {}\n", owner_star.specialization_level));
                }
                
                info.push_str("\n[DELETE] - Remove connection\n");
            }
            
            text.sections[0].value = info;
        } else {
            text.sections[0].value = "Click on a star to see details\nClick on a connection line to see connection info".to_string();
        }
    }
}
