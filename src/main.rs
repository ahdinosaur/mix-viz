use bevy::{
    color::palettes::css::{BLUE, RED},
    prelude::*,
};
use rand::{random_range, rng, seq::IndexedRandom};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, (init_camera, init_peers, init_places))
        .add_systems(
            Update,
            (
                update_position_from_target,
                update_transform_from_position.after(update_position_from_target),
                update_peer_targets,
            ),
        )
        .run();
}

#[derive(Component, Default)]
#[require(Position, Target, Mesh2d)]
struct Peer;

#[derive(Component, Default)]
#[require(Position, Mesh2d)]
struct Place;

#[derive(Component, Default, Clone)]
struct Position {
    x: f32,
    y: f32,
}

impl Position {
    fn random() -> Self {
        Self {
            x: random_range(-1000_f32..1000_f32),
            y: random_range(-1000_f32..1000_f32),
        }
    }
}

#[derive(Component, Default)]
struct Target {
    next: Option<Position>,
}

/*
#[derive(Component, Default)]
struct FavoritePlaces {
    places: Vec<Entity>,
}
*/

#[derive(Component, Default)]
#[require(Camera2d)]
struct Camera;

fn init_camera(mut commands: Commands) {
    commands.spawn(Camera);
}

fn init_peers(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    const NUM_PEERS: usize = 100;
    let mesh = meshes.add(Circle::new(10_f32));
    let material = materials.add(ColorMaterial::from_color(RED));
    for _ in 0..NUM_PEERS {
        commands.spawn((
            Peer,
            Position::random(),
            Mesh2d(mesh.clone()),
            MeshMaterial2d(material.clone()),
        ));
    }
}

fn init_places(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,

    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    const NUM_PLACES: usize = 10;
    let mesh = meshes.add(Rectangle::new(10_f32, 10_f32));
    let material = materials.add(ColorMaterial::from_color(BLUE));
    for _ in 0..NUM_PLACES {
        commands.spawn((
            Place,
            Position::random(),
            Mesh2d(mesh.clone()),
            MeshMaterial2d(material.clone()),
        ));
    }
}

fn update_transform_from_position(mut query: Query<(&Position, &mut Transform)>) {
    for (position, mut transform) in &mut query {
        transform.translation.x = position.x;
        transform.translation.y = position.y;
    }
}

fn update_position_from_target(time: Res<Time>, mut query: Query<(&Target, &mut Position)>) {
    let speed = 100.0; // units per second

    for (target, mut position) in &mut query {
        if let Some(next) = &target.next {
            // Calculate the difference vector from current position to target position
            let dx = next.x - position.x;
            let dy = next.y - position.y;
            // Compute the distance to the target
            let distance = (dx * dx + dy * dy).sqrt();

            if distance > 0.0 {
                // Calculate how much to move this frame
                let step = speed * time.delta_secs();
                if step >= distance {
                    // If we can reach (or overshoot) the target this frame, snap to it.
                    position.x = next.x;
                    position.y = next.y;
                } else {
                    // Move proportionally in the direction of the target
                    position.x += dx / distance * step;
                    position.y += dy / distance * step;
                }
            }
        }
    }
}

fn update_peer_targets(
    mut peers_query: Query<(&Peer, &Position, &mut Target)>,
    places_query: Query<(&Place, &Position)>,
) {
    let place_positions: Vec<Position> = places_query
        .iter()
        .map(|(_, position)| position)
        .cloned()
        .collect();

    for (_, position, mut target) in &mut peers_query {
        // Has peer reached it's target?
        if let Some(next) = &target.next {
            if next.x == position.x && next.y == position.y {
                // Reset target
                target.next = None;
            }
        }

        // If peer has no target
        if target.next.is_none() {
            target.next = place_positions.choose(&mut rng()).cloned();
        }
    }
}
