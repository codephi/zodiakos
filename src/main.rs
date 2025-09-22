//! Space colonization game with resource management

use bevy::{
    core_pipeline::{bloom::*, tonemapping::Tonemapping},
    ecs::system::ParamSet,
    prelude::*,
    render::mesh::Indices,
    sprite::MaterialMesh2dBundle,
    window::PrimaryWindow,
};
use rand::prelude::*;
use std::collections::HashMap;

// Specialization types for stars
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Specialization {
    None,        // Default - collects resources
    Storage,     // Storage hub - increased resource capacity
    Military,    // Produces warships
    Mining,      // Produces mining ships (increases resource collection)
    Agriculture, // Produces food and biological resources
    Research,    // Produces scientists and technology
    Medical,     // Produces medical units and health resources
    Industrial,  // Produces builders and construction units
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
            Specialization::None => 5.0,         // 5 seconds to start extraction
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
    Water,  // 💧
    Oxygen, // 🌬️
    Food,   // 🌱

    // Construction minerals
    Iron,    // 🪨
    Copper,  // ⚡
    Silicon, // 💻

    // Energy resources
    Uranium,       // ☢️
    Helium3,       // 🔋
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

// Calculate route distance to nearest storage hub through connection paths
// This measures the number of connection hops, not physical distance
// A star can be physically close but have a long route distance if not directly connected
fn calculate_distance_to_nearest_storage(
    star_entity: Entity,
    star_query: &Query<&Star>,
    visited: &mut Vec<Entity>,
) -> Option<u32> {
    // Check if already visited to avoid cycles
    if visited.contains(&star_entity) {
        return None;
    }
    visited.push(star_entity);
    
    if let Ok(star) = star_query.get(star_entity) {
        // If this star is a storage hub, route distance is 0
        if star.is_storage_hub {
            return Some(0);
        }
        
        // Check all connection routes to find shortest path
        let mut min_route_distance = None;
        
        // Check routes through incoming connections
        for &connected_entity in &star.connections_from {
            if let Some(dist) = calculate_distance_to_nearest_storage(connected_entity, star_query, visited) {
                let route_dist = dist + 1; // Add 1 hop for this connection
                min_route_distance = Some(min_route_distance.map_or(route_dist, |d: u32| d.min(route_dist)));
            }
        }
        
        // Check routes through outgoing connections
        for &connected_entity in &star.connections_to {
            if let Some(dist) = calculate_distance_to_nearest_storage(connected_entity, star_query, visited) {
                let route_dist = dist + 1; // Add 1 hop for this connection
                min_route_distance = Some(min_route_distance.map_or(route_dist, |d: u32| d.min(route_dist)));
            }
        }
        
        min_route_distance
    } else {
        None
    }
}

// Find all cycles of 3 or more stars in the connection graph
fn find_cycles_in_graph(
    stars: &Query<(Entity, &Star)>,
    min_cycle_size: usize,
) -> Vec<Vec<Entity>> {
    let mut cycles = Vec::new();
    let mut visited = Vec::new();
    
    for (entity, _star) in stars.iter() {
        if !visited.contains(&entity) {
            let mut path = Vec::new();
            find_cycles_dfs(entity, entity, &mut path, &mut visited, &mut cycles, stars, min_cycle_size, None);
        }
    }
    
    // Remove duplicate cycles (same nodes in different order)
    let mut unique_cycles = Vec::new();
    for cycle in cycles {
        let mut sorted_cycle = cycle.clone();
        sorted_cycle.sort_by_key(|e| e.index());
        if !unique_cycles.iter().any(|existing: &Vec<Entity>| {
            let mut sorted_existing = existing.clone();
            sorted_existing.sort_by_key(|e| e.index());
            sorted_existing == sorted_cycle
        }) {
            unique_cycles.push(cycle);
        }
    }
    
    unique_cycles
}

fn find_cycles_dfs(
    current: Entity,
    start: Entity,
    path: &mut Vec<Entity>,
    visited: &mut Vec<Entity>,
    cycles: &mut Vec<Vec<Entity>>,
    stars: &Query<(Entity, &Star)>,
    min_size: usize,
    parent: Option<Entity>,
) {
    path.push(current);
    visited.push(current);
    
    if let Ok((_entity, star)) = stars.get(current) {
        // Check all connected stars
        let mut connected: Vec<Entity> = star.connections_to.clone();
        connected.extend(star.connections_from.clone());
        
        for &next in &connected {
            // Skip parent to avoid immediate backtracking
            if Some(next) == parent {
                continue;
            }
            
            // If we found the start and path is long enough, we have a cycle
            if next == start && path.len() >= min_size {
                cycles.push(path.clone());
            } 
            // Continue DFS if not visited in current path
            else if !path.contains(&next) {
                find_cycles_dfs(next, start, path, visited, cycles, stars, min_size, Some(current));
            }
        }
    }
    
    path.pop();
}

// Calculate production efficiency based on route distance to storage hub
// Stars need supply routes to maintain efficiency - the longer the route, the less efficient
fn production_rate_modifier_from_distance(route_distance: Option<u32>) -> f32 {
    match route_distance {
        None => 0.1,    // No route to storage hub: 10% production (isolated)
        Some(0) => 1.0, // Is a storage hub: 100% production
        Some(1) => 0.9, // 1 connection hop: 90% production
        Some(2) => 0.75, // 2 connection hops: 75% production
        Some(3) => 0.6,  // 3 connection hops: 60% production
        Some(4) => 0.45, // 4 connection hops: 45% production
        Some(5) => 0.35, // 5 connection hops: 35% production
        Some(d) => (0.3 / (d as f32 - 4.0)).max(0.1), // Longer routes: diminishing returns, min 10%
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
    specialization: Specialization, // None = extraction; other = specialization (stops extraction)
    specialization_level: u8,      // Level (no limit, follows Fibonacci for connections)
    units: Vec<Unit>,               // Units produced if specialized
    building_state: BuildingState,  // Current construction/upgrade state
    connections_from: Vec<Entity>,  // List of stars connected TO this star
    connections_to: Vec<Entity>,    // List of stars this star connects TO
    base_color: Color,              // Base color based on resources
    storage_capacity: HashMap<ResourceType, f32>, // Storage capacity if it's a storage hub
    is_storage_hub: bool,          // Whether this star is a storage hub
}

#[derive(Component)]
struct Connection {
    from: Entity,
    to: Entity,
    collection_timer: Timer,
    is_collecting: bool,
    creation_time: f32, // Time in seconds since creation
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

#[derive(Component)]
struct StarSprite;

#[derive(Component)]
struct StarBorder;

// Constellation data structure
struct Constellation {
    id: u32,
    stars: Vec<Entity>,
    color: Color,
}

#[derive(Component)]
struct ConstellationMarker {
    id: u32,
}

#[derive(Component)]
struct ConfigMenu;

#[derive(Component)]
struct BloomIntensityText;

#[derive(Component)]
struct BloomThresholdText;

#[derive(Component)]
struct BloomBoostText;

#[derive(Resource)]
struct ConfigMenuState {
    visible: bool,
}

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

#[derive(Resource, Default)]
struct ConstellationTracker {
    next_id: u32,
    constellations: Vec<Constellation>,
}

// Star name generator
fn generate_star_name(rng: &mut ThreadRng) -> String {
    let prefixes = [
        "Alpha", "Beta", "Gamma", "Delta", "Epsilon", "Zeta", "Eta", "Theta", "Iota", "Kappa",
    ];
    let suffixes = [
        "Centauri",
        "Orionis",
        "Draconis",
        "Pegasi",
        "Andromedae",
        "Leonis",
        "Aquarii",
        "Scorpii",
        "Tauri",
        "Geminorum",
    ];
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
        Some(ResourceType::Water) => Color::srgba(0.3, 0.6, 4.0, 1.0), // Deep blue
        Some(ResourceType::Oxygen) => Color::srgba(0.7, 3.0, 4.0, 1.0), // Cyan
        Some(ResourceType::Food) => Color::srgba(0.3, 4.0, 0.3, 1.0),  // Green
        Some(ResourceType::Iron) => Color::srgba(2.5, 2.5, 3.0, 1.0),  // Silver-gray
        Some(ResourceType::Copper) => Color::srgba(4.0, 2.0, 0.8, 1.0), // Orange-copper
        Some(ResourceType::Silicon) => Color::srgba(3.0, 3.0, 4.0, 1.0), // Light blue-white
        Some(ResourceType::Uranium) => Color::srgba(0.5, 4.0, 0.5, 1.0), // Radioactive green
        Some(ResourceType::Helium3) => Color::srgba(4.0, 3.0, 0.0, 1.0), // Yellow-gold
        Some(ResourceType::EnergyCrystal) => Color::srgba(4.0, 0.5, 4.0, 1.0), // Purple
        None => Color::srgba(3.0, 3.0, 3.0, 1.0),                      // Default white
    }
}

// Generate random resources for a star
fn generate_star_resources(
    rng: &mut ThreadRng,
    is_home: bool,
) -> (HashMap<ResourceType, f32>, HashMap<ResourceType, f32>) {
    let mut resources = HashMap::new();
    let mut max_resources = HashMap::new();

    if is_home {
        // Home star has balanced resources
        for resource in [
            ResourceType::Water,
            ResourceType::Oxygen,
            ResourceType::Food,
            ResourceType::Iron,
            ResourceType::Copper,
            ResourceType::Silicon,
        ] {
            let amount = rng.gen_range(100.0..200.0);
            resources.insert(resource, amount);
            max_resources.insert(resource, amount);
        }
    } else {
        // Other stars have random resources (1-3 types)
        let num_resources = rng.gen_range(1..=3);
        let all_resources = [
            ResourceType::Water,
            ResourceType::Oxygen,
            ResourceType::Food,
            ResourceType::Iron,
            ResourceType::Copper,
            ResourceType::Silicon,
            ResourceType::Uranium,
            ResourceType::Helium3,
            ResourceType::EnergyCrystal,
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
        .init_resource::<ConstellationTracker>()
        .insert_resource(GameState {
            selected_star: None,
        })
        .insert_resource(ConfigMenuState { visible: false })
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
                detect_and_create_constellations,
                update_star_borders,
                toggle_config_menu,
                update_bloom_settings,
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
        Camera2dBundle {
            camera: Camera {
                hdr: true, // HDR is required for bloom
                ..default()
            },
            tonemapping: Tonemapping::TonyMcMapface, // Prevents over-saturation
            ..default()
        },
        BloomSettings {
            intensity: 0.3,
            low_frequency_boost: 0.3,
            low_frequency_boost_curvature: 0.3,
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

    let (_home_resources, home_max) = generate_star_resources(&mut rng, true);
    
    // Calculate storage capacity (10% of max capacity for each resource)
    let mut storage_capacity = HashMap::new();
    let mut storage_resources = HashMap::new();
    for (resource_type, max_value) in &home_max {
        let capacity = max_value * 10.0; // Storage hub has 10x the capacity
        storage_capacity.insert(*resource_type, capacity);
        // Start with 10% of storage capacity filled
        storage_resources.insert(*resource_type, capacity * 0.1);
    }
    
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
            name: "Sol System (Storage Hub)".to_string(),
            resources: storage_resources, // Use storage resources instead
            max_resources: home_max.clone(),
            production_rate: 2.0,
            is_colonized: true,
            is_home_star: true,
            specialization: Specialization::Storage, // Set as Storage hub
            specialization_level: 1,
            units: vec![],
            building_state: BuildingState::Ready,
            connections_from: vec![],
            connections_to: vec![],
            base_color: home_color,
            storage_capacity,
            is_storage_hub: true,
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
                storage_capacity: HashMap::new(),
                is_storage_hub: false,
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

    // Configuration Menu (initially hidden) - Title
    let menu_title = commands
        .spawn((
            TextBundle::from_section(
                "=== BLOOM CONFIGURATION ===\n[ESC] Toggle Menu | [R] Reset Defaults",
                TextStyle {
                    font_size: 18.0,
                    color: Color::srgb(1.0, 1.0, 0.2),
                    ..default()
                },
            )
            .with_style(Style {
                position_type: PositionType::Absolute,
                top: Val::Px(100.0),
                left: Val::Percent(50.0),
                ..default()
            }),
            ConfigMenu,
        ))
        .id();
    commands.entity(menu_title).insert(Visibility::Hidden);

    // Intensity control
    let intensity_text = commands
        .spawn((
            TextBundle::from_section(
                "Intensity: 1.0 (Q/A to adjust)",
                TextStyle {
                    font_size: 16.0,
                    color: Color::srgb(0.9, 0.9, 0.9),
                    ..default()
                },
            )
            .with_style(Style {
                position_type: PositionType::Absolute,
                top: Val::Px(150.0),
                left: Val::Percent(50.0),
                ..default()
            }),
            ConfigMenu,
            BloomIntensityText,
        ))
        .id();
    commands.entity(intensity_text).insert(Visibility::Hidden);

    // Threshold control
    let threshold_text = commands
        .spawn((
            TextBundle::from_section(
                "Threshold: 0.20 (W/S to adjust)",
                TextStyle {
                    font_size: 16.0,
                    color: Color::srgb(0.9, 0.9, 0.9),
                    ..default()
                },
            )
            .with_style(Style {
                position_type: PositionType::Absolute,
                top: Val::Px(180.0),
                left: Val::Percent(50.0),
                ..default()
            }),
            ConfigMenu,
            BloomThresholdText,
        ))
        .id();
    commands.entity(threshold_text).insert(Visibility::Hidden);

    // Low frequency boost control
    let boost_text = commands
        .spawn((
            TextBundle::from_section(
                "Low Freq Boost: 0.5 (E/D to adjust)",
                TextStyle {
                    font_size: 16.0,
                    color: Color::srgb(0.9, 0.9, 0.9),
                    ..default()
                },
            )
            .with_style(Style {
                position_type: PositionType::Absolute,
                top: Val::Px(210.0),
                left: Val::Percent(50.0),
                ..default()
            }),
            ConfigMenu,
            BloomBoostText,
        ))
        .id();
    commands.entity(boost_text).insert(Visibility::Hidden);
}

fn star_hover_system(
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mut star_query: Query<
        (&Transform, &mut Handle<ColorMaterial>, Entity, &Star),
        Without<SelectedStar>,
    >,
    drag_state: Res<DragState>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let Ok(window) = windows.get_single() else {
        return;
    };

    let Ok((camera, camera_transform)) = camera_q.get_single() else {
        return;
    };

    let Some(cursor_pos) = window
        .cursor_position()
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
                            srgba.alpha,
                        );
                    }
                } else {
                    // Hovered: slightly brighten
                    if let Color::Srgba(srgba) = base {
                        material.color = Color::srgba(
                            (srgba.red * 1.2).min(10.0),
                            (srgba.green * 1.2).min(10.0),
                            (srgba.blue * 1.2).min(10.0),
                            srgba.alpha,
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

    let Some(cursor_pos) = window
        .cursor_position()
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

    let Some(cursor_pos) = window
        .cursor_position()
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

                let line_entity = commands
                    .spawn((
                        MaterialMesh2dBundle {
                            mesh: meshes.add(Rectangle::new(1.0, 1.0)).into(),
                            material: materials
                                .add(ColorMaterial::from(Color::srgba(0.5, 1.0, 0.5, 0.5))),
                            transform: Transform::from_xyz(0.0, 0.0, 0.0),
                            ..default()
                        },
                        DraggingLine,
                    ))
                    .id();

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
                let can_connect = if let Ok((_transform, _entity, start_star)) =
                    star_query.get(start_star_entity)
                {
                    let max_connections =
                        max_connections_for_level(start_star.specialization_level);
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
                        if let Ok((_transform, _entity, mut target_star)) =
                            star_query.get_mut(target_entity)
                        {
                            if !target_is_colonized {
                                target_star.is_colonized = true;
                            }
                            target_star.connections_from.push(start_star_entity);
                        }

                        // Update source star
                        if let Ok((_transform, _entity, mut start_star)) =
                            star_query.get_mut(start_star_entity)
                        {
                            start_star.connections_to.push(target_entity);
                        }

                        // Mark stars as needing borders
                        commands.entity(start_star_entity).insert(StarBorder);
                        commands.entity(target_entity).insert(StarBorder);

                        // Create a connection
                        commands.spawn((
                            MaterialMesh2dBundle {
                                mesh: meshes.add(Rectangle::new(1.0, 1.0)).into(),
                                material: materials
                                    .add(ColorMaterial::from(Color::srgb(0.2, 0.8, 0.2))),
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

    let Some(cursor_pos) = window
        .cursor_position()
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

    let Some(cursor_pos) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world_2d(camera_transform, cursor))
    else {
        return;
    };

    // Check for delete key press on selected connection
    if keyboard.just_pressed(KeyCode::Delete) {
        if let Some(selected) = selected_connection {
            let mut from_has_no_connections = false;
            let mut to_has_no_connections = false;

            // Remove connection from stars
            if let Ok(mut from_star) = stars.get_mut(selected.from) {
                from_star.connections_to.retain(|&x| x != selected.to);
                from_has_no_connections =
                    from_star.connections_to.is_empty() && from_star.connections_from.is_empty();
            }
            if let Ok(mut to_star) = stars.get_mut(selected.to) {
                to_star.connections_from.retain(|&x| x != selected.from);
                to_has_no_connections =
                    to_star.connections_to.is_empty() && to_star.connections_from.is_empty();
            }

            // Remove StarBorder component if star has no more connections
            if from_has_no_connections {
                commands.entity(selected.from).remove::<StarBorder>();
            }
            if to_has_no_connections {
                commands.entity(selected.to).remove::<StarBorder>();
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
    for (entity, connection, _transform) in &connection_query {
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
    mut star_queries: ParamSet<(
        Query<&mut Star>,
        Query<&Star>,
    )>,
    mut player_resources: ResMut<PlayerResources>,
    constellation_tracker: Res<ConstellationTracker>,
) {
    // First, update building timers
    for mut star in &mut star_queries.p0() {
        match star.building_state {
            BuildingState::Building {
                mut timer,
                total_time,
            } => {
                timer -= time.delta_seconds();
                if timer <= 0.0 {
                    star.building_state = BuildingState::Ready;
                } else {
                    star.building_state = BuildingState::Building { timer, total_time };
                }
            }
            BuildingState::Upgrading {
                mut timer,
                total_time,
            } => {
                timer -= time.delta_seconds();
                if timer <= 0.0 {
                    star.building_state = BuildingState::Ready;
                    star.specialization_level = star.specialization_level + 1; // No limit on levels
                } else {
                    star.building_state = BuildingState::Upgrading { timer, total_time };
                }
            }
            _ => {}
        }
    }

    for mut connection in &mut connection_query {
        if connection.is_collecting {
            connection.collection_timer.tick(time.delta());

            if connection.collection_timer.just_finished() {
                // First calculate distance to nearest storage hub
                let mut visited = Vec::new();
                let distance = {
                    let star_readonly = star_queries.p1();
                    calculate_distance_to_nearest_storage(connection.to, &star_readonly, &mut visited)
                };
                let distance_modifier = production_rate_modifier_from_distance(distance);
                
                // Then collect resources from the connected star
                if let Ok(mut star) = star_queries.p0().get_mut(connection.to) {
                    // Only produce if building is ready
                    if star.building_state != BuildingState::Ready {
                        continue;
                    }

                    // Only collect resources if star is not specialized for something other than storage
                    if star.specialization == Specialization::None || star.specialization == Specialization::Storage {
                        // Check if star is in a constellation for bonus
                        let constellation_bonus = check_constellation_bonuses(connection.to, &constellation_tracker);
                        let production_rate = star.production_rate * distance_modifier * constellation_bonus;
                        for (resource_type, amount) in star.resources.iter_mut() {
                            let collection_amount = (production_rate * 5.0).min(*amount);
                            if collection_amount > 0.0 {
                                *amount -= collection_amount;
                                *player_resources.resources.entry(*resource_type).or_insert(0.0) += collection_amount;
                            }
                        }
                    } else {
                        // Specialized star: consume resources and produce units
                        let production_costs = star
                            .specialization
                            .production_cost(star.specialization_level);
                        let mut can_produce = true;

                        // Check if we have enough resources
                        for (resource_type, cost) in &production_costs {
                            if *player_resources
                                .resources
                                .get(resource_type)
                                .unwrap_or(&0.0)
                                < *cost
                            {
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
                                    star.units.push(Unit {
                                        unit_type: UnitType::Warship,
                                        count: level_bonus as u32,
                                    });
                                }
                                Specialization::Mining => {
                                    star.units.push(Unit {
                                        unit_type: UnitType::MiningShip,
                                        count: (2 * level_bonus) as u32,
                                    });
                                }
                                Specialization::Agriculture => {
                                    star.units.push(Unit {
                                        unit_type: UnitType::Farmer,
                                        count: (3 * level_bonus) as u32,
                                    });
                                }
                                Specialization::Research => {
                                    star.units.push(Unit {
                                        unit_type: UnitType::Scientist,
                                        count: level_bonus as u32,
                                    });
                                }
                                Specialization::Medical => {
                                    star.units.push(Unit {
                                        unit_type: UnitType::Doctor,
                                        count: (2 * level_bonus) as u32,
                                    });
                                }
                                Specialization::Industrial => {
                                    star.units.push(Unit {
                                        unit_type: UnitType::Builder,
                                        count: (2 * level_bonus) as u32,
                                    });
                                }
                                Specialization::Storage => {
                                    star.units.push(Unit {
                                        unit_type: UnitType::StorageModule,
                                        count: level_bonus as u32,
                                    });
                                }
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

fn update_star_borders(
    mut commands: Commands,
    star_query: Query<(Entity, &Transform, &Star), With<StarBorder>>,
    border_query: Query<Entity, (With<StarBorder>, Without<Star>)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // Remove existing borders
    for border_entity in &border_query {
        commands.entity(border_entity).despawn();
    }

    // Create new borders for stars with connections
    for (entity, transform, star) in &star_query {
        let has_connections = !star.connections_from.is_empty() || !star.connections_to.is_empty();

        if has_connections {
            // Create a border ring around the star
            commands.spawn((
                MaterialMesh2dBundle {
                    mesh: meshes.add(Circle::new(30.0)).into(),
                    material: materials.add(ColorMaterial::from(Color::srgba(0.2, 1.0, 0.2, 0.3))),
                    transform: Transform::from_xyz(
                        transform.translation.x,
                        transform.translation.y,
                        0.5,
                    ),
                    ..default()
                },
                StarBorder,
            ));

            // Also spawn a second ring for better visibility
            commands.spawn((
                MaterialMesh2dBundle {
                    mesh: meshes.add(Circle::new(32.0)).into(),
                    material: materials.add(ColorMaterial::from(Color::srgba(0.1, 0.8, 0.1, 0.2))),
                    transform: Transform::from_xyz(
                        transform.translation.x,
                        transform.translation.y,
                        0.4,
                    ),
                    ..default()
                },
                StarBorder,
            ));
        } else {
            // Remove StarBorder component if no connections
            commands.entity(entity).remove::<StarBorder>();
        }
    }
}

fn toggle_config_menu(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut menu_state: ResMut<ConfigMenuState>,
    mut menu_query: Query<&mut Visibility, With<ConfigMenu>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        menu_state.visible = !menu_state.visible;

        for mut visibility in &mut menu_query {
            *visibility = if menu_state.visible {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
        }
    }
}

fn update_bloom_settings(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut bloom_query: Query<&mut BloomSettings>,
    mut intensity_text: Query<
        &mut Text,
        (
            With<BloomIntensityText>,
            Without<BloomThresholdText>,
            Without<BloomBoostText>,
        ),
    >,
    mut threshold_text: Query<
        &mut Text,
        (
            With<BloomThresholdText>,
            Without<BloomIntensityText>,
            Without<BloomBoostText>,
        ),
    >,
    mut boost_text: Query<
        &mut Text,
        (
            With<BloomBoostText>,
            Without<BloomIntensityText>,
            Without<BloomThresholdText>,
        ),
    >,
    menu_state: Res<ConfigMenuState>,
) {
    if !menu_state.visible {
        return;
    }

    for mut bloom in &mut bloom_query {
        let mut changed = false;

        // Intensity controls (Q/A)
        if keyboard.just_pressed(KeyCode::KeyQ) {
            bloom.intensity = (bloom.intensity + 0.1).min(3.0);
            changed = true;
        }
        if keyboard.just_pressed(KeyCode::KeyA) {
            bloom.intensity = (bloom.intensity - 0.1).max(0.0);
            changed = true;
        }

        // Threshold controls (W/S)
        if keyboard.just_pressed(KeyCode::KeyW) {
            bloom.prefilter_settings.threshold =
                (bloom.prefilter_settings.threshold + 0.05).min(1.0);
            changed = true;
        }
        if keyboard.just_pressed(KeyCode::KeyS) {
            bloom.prefilter_settings.threshold =
                (bloom.prefilter_settings.threshold - 0.05).max(0.0);
            changed = true;
        }

        // Low frequency boost controls (E/D)
        if keyboard.just_pressed(KeyCode::KeyE) {
            bloom.low_frequency_boost = (bloom.low_frequency_boost + 0.1).min(1.0);
            changed = true;
        }
        if keyboard.just_pressed(KeyCode::KeyD) {
            bloom.low_frequency_boost = (bloom.low_frequency_boost - 0.1).max(0.0);
            changed = true;
        }

        // Reset to defaults (R)
        if keyboard.just_pressed(KeyCode::KeyR) {
            bloom.intensity = 1.0;
            bloom.prefilter_settings.threshold = 0.2;
            bloom.low_frequency_boost = 0.5;
            changed = true;
        }

        if changed {
            // Update text displays
            if let Ok(mut text) = intensity_text.get_single_mut() {
                text.sections[0].value =
                    format!("Intensity: {:.1} (Q/A to adjust)", bloom.intensity);
            }
            if let Ok(mut text) = threshold_text.get_single_mut() {
                text.sections[0].value = format!(
                    "Threshold: {:.2} (W/S to adjust)",
                    bloom.prefilter_settings.threshold
                );
            }
            if let Ok(mut text) = boost_text.get_single_mut() {
                text.sections[0].value = format!(
                    "Low Freq Boost: {:.1} (E/D to adjust)",
                    bloom.low_frequency_boost
                );
            }
        }
    }
}

fn update_ui(
    player_resources: Res<PlayerResources>,
    mut resource_panel_query: Query<&mut Text, (With<ResourcePanel>, Without<StarInfoPanel>)>,
    mut star_info_query: Query<&mut Text, (With<StarInfoPanel>, Without<ResourcePanel>)>,
    mut star_queries: ParamSet<(Query<&Star>, Query<&mut Star>)>,
    game_state: Res<GameState>,
    keyboard: Res<ButtonInput<KeyCode>>,
    connection_query: Query<(Entity, &Connection, &Transform), With<ConnectionLine>>,
    _commands: Commands,
    selected_connection: Option<Res<SelectedConnection>>,
    constellation_tracker: Res<ConstellationTracker>,
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

            if let Some((
                id,
                name,
                is_home_star,
                is_colonized,
                specialization,
                level,
                production_rate,
                units,
                resources,
                max_resources,
                building_state,
                connections_from,
                connections_to,
            )) = star_data
            {
                let mut info_text = format!("=== STAR INFO ===\n{} (ID: {})\n", name, id);

                if is_home_star {
                    info_text.push_str("HOME SYSTEM\n");
                }

                if is_colonized {
                    info_text.push_str("Status: COLONIZED\n");
                    info_text.push_str(&format!(
                        "Specialization: {} {} (Level {})
",
                        specialization.icon(),
                        specialization.name(),
                        level
                    ));

                    // Show building state
                    match building_state {
                        BuildingState::Building { timer, total_time } => {
                            let progress = ((total_time - timer) / total_time * 100.0) as u32;
                            info_text.push_str(&format!(
                                "⚙️ BUILDING: {}% complete ({:.1}s remaining)\n",
                                progress, timer
                            ));
                        }
                        BuildingState::Upgrading { timer, total_time } => {
                            let progress = ((total_time - timer) / total_time * 100.0) as u32;
                            info_text.push_str(&format!(
                                "⬆️ UPGRADING: {}% complete ({:.1}s remaining)\n",
                                progress, timer
                            ));
                        }
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
                        info_text.push_str(&format!(
                            "\n[U] UPGRADE to Level {} (Next max connections: {})\n",
                            level + 1,
                            next_max_conn
                        ));

                        // Show connection progression for next few levels
                        info_text.push_str("\nConnection Limit Progression:\n");
                        for i in 0..3 {
                            let future_level = level + i + 1;
                            info_text.push_str(&format!(
                                "  Level {}: {} connections\n",
                                future_level,
                                max_connections_for_level(future_level)
                            ));
                        }
                    }
                } else {
                    info_text.push_str("Status: UNCOLONIZED\n");
                }

                let max_conn = max_connections_for_level(level);
                info_text.push_str(&format!(
                    "Connections: {} inbound, {} outbound (max outbound: {})\n",
                    connections_from, connections_to, max_conn
                ));

                // Show route distance to storage hub and production efficiency
                let mut visited = Vec::new();
                let route_distance = calculate_distance_to_nearest_storage(selected_entity, &star_queries.p0(), &mut visited);
                let efficiency_modifier = production_rate_modifier_from_distance(route_distance);
                
                if let Some(hops) = route_distance {
                    info_text.push_str(&format!("Supply Route Distance: {} connection(s)\n", hops));
                    info_text.push_str(&format!("Route Status: {}\n", 
                        match hops {
                            0 => "Storage Hub (Direct Supply)",
                            1 => "Adjacent to Storage (Optimal)",
                            2..=3 => "Short Route (Good)",
                            4..=5 => "Long Route (Suboptimal)",
                            _ => "Very Long Route (Poor)",
                        }
                    ));
                } else {
                    info_text.push_str("Supply Route: ⚠️ NO CONNECTION TO STORAGE!\n");
                    info_text.push_str("Status: Isolated (Critical)\n");
                }

                // Check if star is in a constellation
                let constellation_bonus = check_constellation_bonuses(selected_entity, &constellation_tracker);
                if constellation_bonus > 1.0 {
                    info_text.push_str("\n⭐ CONSTELLATION BONUS: 2x Production! ⭐\n");
                    info_text.push_str("This star is part of a constellation.\n");
                    info_text.push_str("No new constellations can be formed with this star.\n\n");
                }
                
                if specialization == Specialization::None {
                    info_text.push_str(&format!("Base Production Rate: {:.1}/s\n", production_rate));
                    info_text.push_str(&format!("Effective Production: {:.1}/s ({:.0}% efficiency)\n", 
                        production_rate * efficiency_modifier * constellation_bonus, efficiency_modifier * constellation_bonus * 100.0));
                } else {
                    info_text.push_str("Production: SPECIALIZED\n");

                    // Show units produced
                    if !units.is_empty() {
                        info_text.push_str("\nUnits Produced:\n");
                        for unit in &units {
                            info_text.push_str(&format!(
                                "  {} x{}\n",
                                format!("{:?}", unit.unit_type),
                                unit.count
                            ));
                        }
                    }

                    // Show production cost
                    info_text.push_str("\nProduction Cost/cycle:\n");
                    for (resource_type, cost) in specialization.production_cost(level) {
                        info_text.push_str(&format!(
                            "  {} {}: {:.1}\n",
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

                        if keyboard.just_pressed(KeyCode::Digit1) {
                            new_spec = Some(Specialization::None);
                        } else if keyboard.just_pressed(KeyCode::Digit2) {
                            new_spec = Some(Specialization::Storage);
                        } else if keyboard.just_pressed(KeyCode::Digit3) {
                            new_spec = Some(Specialization::Military);
                        } else if keyboard.just_pressed(KeyCode::Digit4) {
                            new_spec = Some(Specialization::Mining);
                        } else if keyboard.just_pressed(KeyCode::Digit5) {
                            new_spec = Some(Specialization::Agriculture);
                        } else if keyboard.just_pressed(KeyCode::Digit6) {
                            new_spec = Some(Specialization::Research);
                        } else if keyboard.just_pressed(KeyCode::Digit7) {
                            new_spec = Some(Specialization::Medical);
                        } else if keyboard.just_pressed(KeyCode::Digit8) {
                            new_spec = Some(Specialization::Industrial);
                        }

                        if let Some(spec) = new_spec {
                            if selected_star.specialization != spec {
                                selected_star.specialization = spec;
                                selected_star.specialization_level = 1; // Reset level when changing
                                selected_star.units.clear();
                                
                                // If becoming a storage hub, set up storage capacity
                                if spec == Specialization::Storage {
                                    selected_star.is_storage_hub = true;
                                    let max_resources_copy = selected_star.max_resources.clone();
                                    for (resource_type, max_value) in &max_resources_copy {
                                        let capacity = max_value * 10.0; // Storage hub has 10x capacity
                                        selected_star.storage_capacity.insert(*resource_type, capacity);
                                    }
                                } else {
                                    selected_star.is_storage_hub = false;
                                    selected_star.storage_capacity.clear();
                                }
                                
                                let build_time = spec.build_time();
                                selected_star.building_state = BuildingState::Building {
                                    timer: build_time,
                                    total_time: build_time,
                                };
                            }
                        }

                        // Handle upgrade (no level limit)
                        if keyboard.just_pressed(KeyCode::KeyU) {
                            if selected_star.building_state == BuildingState::Ready {
                                let upgrade_time = selected_star
                                    .specialization
                                    .upgrade_time(selected_star.specialization_level);
                                selected_star.building_state = BuildingState::Upgrading {
                                    timer: upgrade_time,
                                    total_time: upgrade_time,
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
            if let Ok((_entity, connection, _transform)) = connection_query.get(selected_conn.entity)
            {
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
                info.push_str(&format!(
                    "\nTime Existed: {:.1} seconds\n",
                    connection.creation_time
                ));
                info.push_str(&format!(
                    "Status: {}\n",
                    if connection.is_collecting {
                        "Active"
                    } else {
                        "Inactive"
                    }
                ));

                // Get owner info (the 'from' star is the owner)
                if let Ok(owner_star) = star_queries.p0().get(connection.from) {
                    info.push_str(&format!(
                        "\nOwner Specialization: {}\n",
                        owner_star.specialization.name()
                    ));
                    info.push_str(&format!(
                        "Owner Level: {}\n",
                        owner_star.specialization_level
                    ));
                }

                info.push_str("\n[DELETE] - Remove connection\n");
            }

            text.sections[0].value = info;
        } else {
            text.sections[0].value =
                "Click on a star to see details\nClick on a connection line to see connection info"
                    .to_string();
        }
    }
}

// System to detect and create constellations when cycles are formed
fn detect_and_create_constellations(
    stars_query: Query<(Entity, &Star, &Transform)>,
    stars_simple: Query<(Entity, &Star)>,
    mut constellation_tracker: ResMut<ConstellationTracker>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // Find all cycles of 3 or more stars
    let cycles = find_cycles_in_graph(&stars_simple, 3);
    
    for cycle_entities in cycles {
        // Check if this constellation already exists
        let is_new = !constellation_tracker.constellations.iter().any(|c| {
            let mut sorted_cycle = cycle_entities.clone();
            sorted_cycle.sort_by_key(|e| e.index());
            let mut sorted_existing = c.stars.clone();
            sorted_existing.sort_by_key(|e| e.index());
            sorted_cycle == sorted_existing
        });
        
        // Check if any star in this cycle is already part of another constellation
        let has_existing_constellation_star = cycle_entities.iter().any(|&star_entity| {
            constellation_tracker.constellations.iter().any(|existing_constellation| {
                existing_constellation.stars.contains(&star_entity)
            })
        });
        
        // Only create constellation if it's new AND no stars are already in other constellations
        if is_new && !has_existing_constellation_star {
            // Create a new constellation with varied colors
            let hue = (constellation_tracker.next_id as f32 * 137.5) % 360.0; // Golden angle for color distribution
            let color = Color::hsla(
                hue,
                0.7,  // Good saturation
                0.6,  // Medium lightness 
                0.25, // Semi-transparent
            );
            
            let constellation = Constellation {
                id: constellation_tracker.next_id,
                stars: cycle_entities.clone(),
                color,
            };
            
            constellation_tracker.next_id += 1;
            
            // Create visual representation of the constellation
            create_constellation_visual(&constellation, &stars_query, &mut commands, &mut meshes, &mut materials);
            
            constellation_tracker.constellations.push(constellation);
            
            info!("New constellation formed with {} stars!", cycle_entities.len());
        } else if !is_new {
            // Constellation already exists
            debug!("Cycle detected but constellation already exists");
        } else if has_existing_constellation_star {
            // Can't create because stars are already in other constellations
            info!("Cannot form new constellation: one or more stars already belong to existing constellations");
        }
    }
}

// Create visual polygon for constellation
fn create_constellation_visual(
    constellation: &Constellation,
    stars_query: &Query<(Entity, &Star, &Transform)>,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
) {
    // The constellation stars are already in the correct order from the cycle detection
    // So we just need to connect them in that exact order
    let mut star_positions = Vec::new();
    
    // Get positions in the exact order of the cycle
    for &star_entity in &constellation.stars {
        if let Ok((_, _star, transform)) = stars_query.get(star_entity) {
            star_positions.push(Vec2::new(transform.translation.x, transform.translation.y));
        }
    }
    
    if star_positions.len() < 3 {
        return;
    }
    
    // Create a proper mesh for the constellation polygon
    let mut mesh = Mesh::new(
        bevy::render::mesh::PrimitiveTopology::TriangleList,
        bevy::render::render_asset::RenderAssetUsages::RENDER_WORLD,
    );
    
    // Create vertices for the polygon - we'll use a triangle fan from the center
    let center = star_positions.iter().fold(Vec2::ZERO, |acc, &p| acc + p) / star_positions.len() as f32;
    
    let mut vertices = vec![[center.x, center.y, 0.0]]; // Center vertex
    for &pos in &star_positions {
        vertices.push([pos.x, pos.y, 0.0]);
    }
    
    // Create triangle indices for a triangle fan
    let mut indices = Vec::new();
    for i in 0..star_positions.len() as u32 {
        let next = (i + 1) % star_positions.len() as u32;
        indices.push(0); // Center
        indices.push(i + 1);
        indices.push(next + 1);
    }
    
    // Set mesh attributes
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_indices(Indices::U32(indices));
    
    // Create UV coordinates for proper texturing (if needed)
    let mut uvs = vec![[0.5, 0.5]]; // Center UV
    for i in 0..star_positions.len() {
        let angle = (i as f32 * 2.0 * std::f32::consts::PI) / star_positions.len() as f32;
        uvs.push([0.5 + 0.5 * angle.cos(), 0.5 + 0.5 * angle.sin()]);
    }
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    
    // Create the constellation background with the mesh
    let mesh_handle = meshes.add(mesh);
    let material_handle = materials.add(ColorMaterial::from(constellation.color.with_alpha(0.2)));
    
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: mesh_handle.into(),
            material: material_handle,
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, -1.0)), // Behind everything
            ..default()
        },
        ConstellationMarker { id: constellation.id },
    ));
    
    // Draw glowing colored lines connecting the stars in the exact cycle order
    for i in 0..constellation.stars.len() {
        let from_entity = constellation.stars[i];
        let to_entity = constellation.stars[(i + 1) % constellation.stars.len()];
        
        if let (Ok((_, _, from_transform)), Ok((_, _, to_transform))) = (
            stars_query.get(from_entity),
            stars_query.get(to_entity)
        ) {
            let start = Vec2::new(from_transform.translation.x, from_transform.translation.y);
            let end = Vec2::new(to_transform.translation.x, to_transform.translation.y);
            
            let midpoint = (start + end) / 2.0;
            let diff = end - start;
            let distance = diff.length();
            let angle = diff.y.atan2(diff.x);
            
            // Create multiple layers for glow effect
            // Outer glow - very wide and transparent
            commands.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        color: constellation.color.with_alpha(0.15),
                        custom_size: Some(Vec2::new(distance, 30.0)),
                        ..default()
                    },
                    transform: Transform {
                        translation: midpoint.extend(-0.3),
                        rotation: Quat::from_rotation_z(angle),
                        ..default()
                    },
                    ..default()
                },
                ConstellationMarker { id: constellation.id },
            ));
            
            // Middle glow
            commands.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        color: constellation.color.with_alpha(0.3),
                        custom_size: Some(Vec2::new(distance, 16.0)),
                        ..default()
                    },
                    transform: Transform {
                        translation: midpoint.extend(-0.2),
                        rotation: Quat::from_rotation_z(angle),
                        ..default()
                    },
                    ..default()
                },
                ConstellationMarker { id: constellation.id },
            ));
            
            // Core line - bright and solid
            commands.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        color: constellation.color.with_alpha(0.9),
                        custom_size: Some(Vec2::new(distance, 6.0)),
                        ..default()
                    },
                    transform: Transform {
                        translation: midpoint.extend(-0.1),
                        rotation: Quat::from_rotation_z(angle),
                        ..default()
                    },
                    ..default()
                },
                ConstellationMarker { id: constellation.id },
            ));
        }
    }
}

// Check if a star is part of any constellation and apply bonuses
fn check_constellation_bonuses(
    star_entity: Entity,
    constellation_tracker: &ConstellationTracker,
) -> f32 {
    for constellation in &constellation_tracker.constellations {
        if constellation.stars.contains(&star_entity) {
            return 2.0; // 100% bonus (2x multiplier)
        }
    }
    1.0 // No bonus
}
