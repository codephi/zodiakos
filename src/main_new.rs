//! Space colonization game with resource management - Bevy 0.16 version

use bevy::{
    prelude::*,
    sprite::{Mesh2d, Mesh2dHandle},
    window::PrimaryWindow,
    ecs::system::ParamSet,
    core_pipeline::bloom::Bloom,
    text::{TextColor, TextFont, JustifyText},
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

#[derive(Component)]
struct StarSprite;

// Resources
#[derive(Resource, Default)]
struct DragState {
    is_dragging: bool,
    start_star: Option<Entity>,
    current_line: Option<Entity>,
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
) {
    // Camera with HDR and Bloom
    commands.spawn((
        Camera2d::default(),
        Bloom::default(),
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
        Mesh2d(star_mesh.clone()),
        Transform::from_xyz(home_pos.x, home_pos.y, 1.0),
        ColorMaterial2d::from(home_color),
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
        StarSprite,
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
            Mesh2d(star_mesh.clone()),
            Transform::from_xyz(pos.x, pos.y, 1.0),
            ColorMaterial2d::from(star_color),
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
            StarSprite,
        ));
    }

    // UI Setup
    // Title
    commands.spawn((
        Text::new("ZODIAKOS - Space Colonization"),
        TextFont {
            font_size: 24.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
    ));

    // Instructions
    commands.spawn((
        Text::new("Click and drag between stars to create connections"),
        TextFont {
            font_size: 14.0,
            ..default()
        },
        TextColor(Color::srgb(0.8, 0.8, 0.8)),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
    ));

    // Resource panel (top right)
    commands.spawn((
        Text::new("Resources"),
        TextFont {
            font_size: 16.0,
            ..default()
        },
        TextColor(Color::srgb(0.9, 0.9, 0.9)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            right: Val::Px(10.0),
            ..default()
        },
        ResourcePanel,
    ));

    // Star info panel (right side)
    commands.spawn((
        Text::new("Click on a star to see details\nClick on a connection line to see connection info"),
        TextFont {
            font_size: 14.0,
            ..default()
        },
        TextColor(Color::srgb(0.9, 0.9, 0.9)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(200.0),
            right: Val::Px(10.0),
            width: Val::Px(300.0),
            ..default()
        },
        StarInfoPanel,
    ));
}

// System functions would go here - simplified versions for now
fn star_hover_system() {}
fn handle_mouse_input() {}
fn star_selection_system() {}
fn connection_selection_system() {}
fn update_dragging_line() {}
fn update_connections() {}
fn collect_resources_system() {}
fn update_ui() {}