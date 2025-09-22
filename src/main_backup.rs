//! A game where you can connect balls by dragging lines between them.

use bevy::{
    prelude::*,
    sprite::MaterialMesh2dBundle,
    window::PrimaryWindow,
};
use rand::prelude::*;

// Components
#[derive(Component)]
struct Ball {
    id: usize,
}

#[derive(Component)]
struct Connection {
    from: Entity,
    to: Entity,
}

#[derive(Component)]
struct DraggingLine;

// Resources
#[derive(Resource, Default)]
struct DragState {
    is_dragging: bool,
    start_ball: Option<Entity>,
    current_line: Option<Entity>,
}

#[derive(Resource)]
struct BallMaterials {
    normal: Handle<ColorMaterial>,
    hovered: Handle<ColorMaterial>,
    selected: Handle<ColorMaterial>,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_resource::<DragState>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                ball_hover_system,
                handle_mouse_input,
                update_dragging_line,
                update_connections,
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

    // Store materials for balls
    let ball_materials = BallMaterials {
        normal: materials.add(ColorMaterial::from(Color::srgb(0.3, 0.3, 0.8))),
        hovered: materials.add(ColorMaterial::from(Color::srgb(0.5, 0.5, 1.0))),
        selected: materials.add(ColorMaterial::from(Color::srgb(0.6, 0.6, 1.0))),
    };

    // Create ball mesh handle
    let ball_mesh = meshes.add(Circle::new(20.0));

    // Spawn balls at random positions with minimum distance
    let mut rng = rand::thread_rng();
    let mut positions: Vec<Vec2> = Vec::new();
    let min_distance = 70.0; // Minimum distance between balls (slightly less for better distribution)
    let max_attempts = 500; // Increased attempts for better placement
    let margin = 40.0; // Margin from screen edges
    let num_balls = 15; // Number of balls to spawn
    
    for i in 0..num_balls {
        let mut position_found = false;
        let mut attempts = 0;
        let mut pos = Vec2::ZERO;
        
        while !position_found && attempts < max_attempts {
            // Generate position with margin from edges
            let x = rng.gen_range((-400.0 + margin)..(400.0 - margin));
            let y = rng.gen_range((-300.0 + margin)..(300.0 - margin));
            pos = Vec2::new(x, y);
            
            // Check if this position is far enough from all existing balls
            position_found = true;
            for existing_pos in &positions {
                if pos.distance(*existing_pos) < min_distance {
                    position_found = false;
                    break;
                }
            }
            
            attempts += 1;
        }
        
        // If we couldn't find a valid position after max attempts, try a grid-based fallback
        if !position_found && attempts >= max_attempts {
            // Use a grid position as fallback
            let grid_cols = 5;
            let grid_x = (i % grid_cols) as f32;
            let grid_y = (i / grid_cols) as f32;
            pos = Vec2::new(
                -200.0 + grid_x * 100.0,
                -150.0 + grid_y * 100.0,
            );
        }
        
        positions.push(pos);
        
        commands.spawn((
            MaterialMesh2dBundle {
                mesh: ball_mesh.clone().into(),
                material: ball_materials.normal.clone(),
                transform: Transform::from_xyz(pos.x, pos.y, 1.0),
                ..default()
            },
            Ball { id: i },
        ));
    }

    commands.insert_resource(ball_materials);

    // Instructions text
    commands.spawn(
        TextBundle::from_section(
            "Click and drag from one ball to another to connect them",
            TextStyle {
                font_size: 18.0,
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
}

fn ball_hover_system(
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mut ball_query: Query<(&Transform, &mut Handle<ColorMaterial>, Entity), With<Ball>>,
    ball_materials: Res<BallMaterials>,
    drag_state: Res<DragState>,
) {
    let Ok(window) = windows.get_single() else {
        return;
    };
    
    let Ok((camera, camera_transform)) = camera_q.get_single() else {
        return;
    };

    // Get cursor position in world coordinates
    let Some(cursor_pos) = window.cursor_position()
        .and_then(|cursor| camera.viewport_to_world_2d(camera_transform, cursor))
    else {
        return;
    };

    for (transform, mut material, entity) in &mut ball_query {
        let distance = transform.translation.truncate().distance(cursor_pos);
        
        // Check if cursor is over the ball (within radius of 20)
        if distance < 20.0 {
            if drag_state.start_ball == Some(entity) {
                *material = ball_materials.selected.clone();
            } else {
                *material = ball_materials.hovered.clone();
            }
        } else if drag_state.start_ball != Some(entity) {
            *material = ball_materials.normal.clone();
        }
    }
}

fn handle_mouse_input(
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    ball_query: Query<(&Transform, Entity), With<Ball>>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut drag_state: ResMut<DragState>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
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

    if mouse_button.just_pressed(MouseButton::Left) {
        // Check if we clicked on a ball
        for (transform, entity) in &ball_query {
            let distance = transform.translation.truncate().distance(cursor_pos);
            
            if distance < 20.0 {
                // Start dragging from this ball
                drag_state.is_dragging = true;
                drag_state.start_ball = Some(entity);
                
                // Create a temporary line that follows the mouse
                let line_entity = commands.spawn((
                    MaterialMesh2dBundle {
                        mesh: meshes.add(Rectangle::new(1.0, 1.0)).into(),
                        material: materials.add(ColorMaterial::from(Color::srgba(0.5, 0.5, 1.0, 0.5))),
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
        // Check if we released on a different ball
        for (transform, entity) in &ball_query {
            let distance = transform.translation.truncate().distance(cursor_pos);
            
            if distance < 20.0 && Some(entity) != drag_state.start_ball {
                // Create a permanent connection
                if let Some(start_ball) = drag_state.start_ball {
                    commands.spawn((
                        MaterialMesh2dBundle {
                            mesh: meshes.add(Rectangle::new(1.0, 1.0)).into(),
                            material: materials.add(ColorMaterial::from(Color::srgb(0.3, 0.3, 0.8))),
                            transform: Transform::from_xyz(0.0, 0.0, -1.0),
                            ..default()
                        },
                        Connection {
                            from: start_ball,
                            to: entity,
                        },
                    ));
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
        drag_state.start_ball = None;
        drag_state.current_line = None;
    }
}

fn update_dragging_line(
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    ball_query: Query<&Transform, With<Ball>>,
    mut line_query: Query<&mut Transform, (With<DraggingLine>, Without<Ball>)>,
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

    if let Some(start_ball) = drag_state.start_ball {
        if let Ok(start_transform) = ball_query.get(start_ball) {
            let start_pos = start_transform.translation.truncate();
            
            // Update the temporary line
            if let Some(line_entity) = drag_state.current_line {
                if let Ok(mut line_transform) = line_query.get_mut(line_entity) {
                    let direction = cursor_pos - start_pos;
                    let length = direction.length();
                    let angle = direction.y.atan2(direction.x);
                    
                    line_transform.translation.x = start_pos.x + direction.x / 2.0;
                    line_transform.translation.y = start_pos.y + direction.y / 2.0;
                    line_transform.rotation = Quat::from_rotation_z(angle);
                    line_transform.scale.x = length;
                    line_transform.scale.y = 3.0; // Line thickness
                }
            }
        }
    }
}

fn update_connections(
    ball_query: Query<&Transform, With<Ball>>,
    mut connection_query: Query<(&mut Transform, &Connection), Without<Ball>>,
) {
    for (mut line_transform, connection) in &mut connection_query {
        if let Ok(from_transform) = ball_query.get(connection.from) {
            if let Ok(to_transform) = ball_query.get(connection.to) {
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
                line_transform.scale.y = 3.0; // Line thickness
            }
        }
    }
}