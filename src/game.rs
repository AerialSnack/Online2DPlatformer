use bevy::{prelude::*, render::camera::ScalingMode};
use bevy_matchbox::prelude::*;
use bevy_ggrs::*;
use bevy_ggrs::prelude::SessionBuilder;
use avian2d::prelude::*;
use crate::GameState;
use crate::input::{Config, get_input_direction, InputPlugin, INPUT_LEFT, INPUT_RIGHT, INPUT_UP, INPUT_UP_PRESSED};

pub struct GamePlugin;

// Define collision layers
const WALL_LAYER: u32 = 0b01;
const PLAYER_LAYER: u32 = 0b10;
const GROUND_LAYER: u32 = 0b100; // Different from WALL_LAYER

#[derive(Component)]
struct Ground; // Add a component to identify the ground

#[derive(Component)]
struct WaitingText;

#[derive(Component, Clone)]
struct Player {
    handle: usize,
    jumps_remaining: u8,
    is_grounded: bool,
    previous_input: u8,  // Add field to track previous input
}

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            GgrsPlugin::<Config>::default(),
            PhysicsPlugins::default(),
            PhysicsDebugPlugin::default(),
            InputPlugin,
        ))
            .rollback_component_with_clone::<Transform>()
            .rollback_component_with_clone::<LinearVelocity>()
            .rollback_component_with_clone::<Restitution>()
            .rollback_component_with_clone::<Friction>()
            .rollback_component_with_clone::<GravityScale>()
            .rollback_component_with_clone::<CollisionLayers>()
            .rollback_component_with_clone::<Collider>()
            .add_systems(OnEnter(GameState::InGame), (setup, spawn_players, start_matchbox_socket))
            .add_systems(Update, wait_for_players.run_if(in_state(GameState::InGame)))
            .add_systems(GgrsSchedule, move_players.run_if(in_state(GameState::InGame)));
    }
}

fn setup(mut commands: Commands) {
    // Camera setup
    commands.spawn((
        Camera2d,
        OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: 10.,
            },
            ..OrthographicProjection::default_2d()
        },
    ));

    // Border dimensions
    let border_thickness = 0.5;
    let width = 16.0; // Viewport width (assuming 16:10 aspect ratio)
    let height = 10.0; // Matches viewport_height

    // Spawn borders
    // Top wall
    commands.spawn((
        Transform::from_xyz(0.0, height/2.0, 0.0),
        Sprite {
            color: Color::BLACK,
            custom_size: Some(Vec2::new(width, border_thickness)),
            ..default()
        },
        RigidBody::Static,
        Collider::rectangle(width, border_thickness),
        CollisionLayers::new([WALL_LAYER], !WALL_LAYER),
    ));

    // Bottom wall (ground)
    commands.spawn((
        Transform::from_xyz(0.0, -height/2.0, 0.0),
        Sprite {
            color: Color::BLACK,
            custom_size: Some(Vec2::new(width, border_thickness)),
            ..default()
        },
        RigidBody::Static,
        Collider::rectangle(width, border_thickness),
        CollisionLayers::new([GROUND_LAYER], !GROUND_LAYER),
        Ground,
    ));

    // Left wall
    commands.spawn((
        Transform::from_xyz(-width/2.0, 0.0, 0.0),
        Sprite {
            color: Color::BLACK,
            custom_size: Some(Vec2::new(border_thickness, height)),
            ..default()
        },
        RigidBody::Static,
        Collider::rectangle(border_thickness, height),
        CollisionLayers::new([WALL_LAYER], !WALL_LAYER),
    ));

    // Right wall
    commands.spawn((
        Transform::from_xyz(width/2.0, 0.0, 0.0),
        Sprite {
            color: Color::BLACK,
            custom_size: Some(Vec2::new(border_thickness, height)),
            ..default()
        },
        RigidBody::Static,
        Collider::rectangle(border_thickness, height),
        CollisionLayers::new([WALL_LAYER], !WALL_LAYER),
    ));

    // Net
    commands.spawn((
        Transform::from_xyz(0.0, -height/4.0, 0.0),
        Sprite {
            color: Color::BLACK,
            custom_size: Some(Vec2::new(border_thickness, height * 0.5)),
            ..default()
        },
        RigidBody::Static,
        Collider::rectangle(border_thickness, height * 0.5),
        CollisionLayers::new([WALL_LAYER], !WALL_LAYER),
    ));

    // Spawn waiting text
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            WaitingText,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Waiting for other player..."),
                TextFont {
                    font_size: 30.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

fn start_matchbox_socket(mut commands: Commands) {
    let room_url = "ws://ec2-54-67-37-240.us-west-1.compute.amazonaws.com:3536/extreme_bevy?next=2";
    info!("connecting to matchbox server: {room_url}");
    commands.insert_resource(MatchboxSocket::new_unreliable(room_url));
}

fn wait_for_players(
    mut socket: ResMut<MatchboxSocket>, 
    mut commands: Commands,
    waiting_text: Query<Entity, With<WaitingText>>,
) {
    if socket.get_channel(0).is_err() {
        return; // we've already started
    }

    // Check for new connections
    socket.update_peers();
    let players = socket.players();

    let num_players = 2;
    if players.len() < num_players {
        return; // wait for more players
    }

    info!("All peers have joined, going in-game");

    // Remove waiting text
    if let Ok(entity) = waiting_text.get_single() {
        commands.entity(entity).despawn_recursive();
    }

    // create a GGRS P2P session
    let mut session_builder = SessionBuilder::<Config>::new()
        .with_num_players(num_players)
        .with_input_delay(2);

    for (i, player) in players.into_iter().enumerate() {
        session_builder = session_builder
            .add_player(player, i)
            .expect("failed to add player");
    }

    // move the channel out of the socket (required because GGRS takes ownership of it)
    let channel = socket.take_channel(0).unwrap();

    // start the GGRS session
    let ggrs_session = session_builder
        .start_p2p_session(channel)
        .expect("failed to start session");

    commands.insert_resource(bevy_ggrs::Session::P2P(ggrs_session));
}

// Helper function to add common physics components to a player
fn add_player_physics(commands: &mut Commands, entity: Entity) {
    commands.entity(entity).insert((
        RigidBody::Dynamic,
        LockedAxes::ROTATION_LOCKED,
        LinearVelocity::default(),
        Restitution::new(0.0),
        Friction::new(0.01),
        GravityScale(1.0), // Enable gravity for jumping
    ));
}

fn spawn_players(mut commands: Commands, asset_server: Res<AssetServer>) {
    let scale = 0.0025;
    let sprite_height = 440.0;
    let sprite_width = 200.0;
    
    // Player 1
    let player1 = commands
        .spawn((
            Player { 
                handle: 0,
                jumps_remaining: 2,
                is_grounded: false,
                previous_input: 0,
            },
            Transform::from_translation(Vec3::new(-2., 0., 0.))
                .with_scale(Vec3::splat(scale)),
            Sprite {
                image: asset_server.load("sprites/ice3.png"),
                ..default()
            },
        ))
        .add_rollback()
        .id();
    
    add_player_physics(&mut commands, player1);

    // Spawn collider as child
    commands.spawn((
        Collider::rectangle(sprite_width, sprite_height),
        CollisionLayers::new(
            [PLAYER_LAYER],
            !(PLAYER_LAYER) | WALL_LAYER | GROUND_LAYER
        ),
    ))
    .add_rollback()
    .set_parent(player1);

    // Player 2
    let player2 = commands
        .spawn((
            Player { 
                handle: 1,
                jumps_remaining: 2,
                is_grounded: false,
                previous_input: 0,
            },
            Transform::from_translation(Vec3::new(2., 0., 0.))
                .with_scale(Vec3::splat(scale)),
            Sprite {
                image: asset_server.load("sprites/zapp.png"),
                ..default()
            },
        ))
        .add_rollback()
        .id();
    
    add_player_physics(&mut commands, player2);

    // Spawn collider as child
    commands.spawn((
        Collider::rectangle(sprite_width, sprite_height),
        CollisionLayers::new(
            [PLAYER_LAYER],
            !(PLAYER_LAYER) | WALL_LAYER | GROUND_LAYER
        ),
    ))
    .add_rollback()
    .set_parent(player2);
}

fn move_players(
    mut query: Query<(Entity, &mut Transform, &mut LinearVelocity, &mut Sprite, &mut Player)>,
    mut collision_events: EventReader<Collision>,
    inputs: Res<PlayerInputs<Config>>,
    children_query: Query<&Parent>,
    ground_query: Query<Entity, With<Ground>>,
) {
    let ground_entity = if let Ok(entity) = ground_query.get_single() {
        entity
    } else {
        return;
    };

    for (player_entity, mut transform, mut velocity, mut sprite, mut player) in query.iter_mut() {
        // Handle movement and jumping first
        let (input, _) = inputs[player.handle];
        
        // Flip sprite based on movement direction
        if input & INPUT_LEFT != 0 {
            sprite.flip_x = true;
        } else if input & INPUT_RIGHT != 0 {
            sprite.flip_x = false;
        }

        // Handle horizontal movement
        let direction = get_input_direction(input);
        let move_speed = 7.;
        velocity.0.x = direction.x * move_speed;

        // Handle jumping - check if UP was just pressed by comparing with previous input
        let just_pressed_up = (input & INPUT_UP != 0) && (player.previous_input & INPUT_UP == 0);
        let is_jumping = if just_pressed_up && player.jumps_remaining > 0 {
            info!("Player {} jumping, {} jumps remaining", player.handle, player.jumps_remaining - 1);
            velocity.0.y = 10.0; // Jump impulse
            player.jumps_remaining -= 1;
            true
        } else {
            false
        };

        // Store current input for next frame
        player.previous_input = input;

        // Reset grounded state
        player.is_grounded = false;

        // Check for collisions and reset jumps
        for Collision(contacts) in collision_events.read() {
            // Get parent entities if the colliding entities are children
            let entity1_parent = children_query.get(contacts.entity1).ok().map(Parent::get);
            let entity2_parent = children_query.get(contacts.entity2).ok().map(Parent::get);
            
            // Check if either entity (or its parent) is our player
            let is_player_collision = player_entity == contacts.entity1 
                || player_entity == contacts.entity2
                || Some(player_entity) == entity1_parent 
                || Some(player_entity) == entity2_parent;

            // Check if one of the entities is the ground
            let has_ground = contacts.entity1 == ground_entity || contacts.entity2 == ground_entity;

            if is_player_collision && has_ground {
                player.is_grounded = true;
                // Only reset jumps if we're not currently jumping and don't have max jumps
                if !is_jumping && player.jumps_remaining < 2 {
                    info!("Player {} touched ground, resetting jumps", player.handle);
                    player.jumps_remaining = 2;
                }
            }
        }
    }
}