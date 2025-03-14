use bevy::{
    color::palettes::css::{BLUE, RED},
    prelude::*,
};
use rand::{random_range, rng, seq::IndexedRandom};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(
            Startup,
            (init_camera, init_places, init_peers.after(init_places)),
        )
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
#[require(Position, FavoritePlaces, Mesh2d)]
struct Peer;

#[derive(Component, Default)]
#[require(Position, Mesh2d)]
struct Place;

#[derive(Component, Default, Debug, Copy, Clone)]
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
    position: Position,
}

#[derive(Component, Default)]
struct FavoritePlaces {
    places: Vec<Entity>,
}

#[derive(Component, Default)]
#[require(Camera2d)]
struct Camera;

fn init_camera(mut commands: Commands) {
    commands.spawn(Camera);
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

fn init_peers(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    places_query: Query<Entity, With<Place>>,
) {
    let places: Vec<Entity> = places_query.iter().collect();

    const NUM_PEERS: usize = 100;
    let mesh = meshes.add(Circle::new(10_f32));
    let material = materials.add(ColorMaterial::from_color(RED));
    for _ in 0..NUM_PEERS {
        commands.spawn((
            Peer,
            Position::random(),
            Mesh2d(mesh.clone()),
            MeshMaterial2d(material.clone()),
            FavoritePlaces {
                places: places.choose_multiple(&mut rng(), 3).cloned().collect(),
            },
        ));
    }
}

fn update_transform_from_position(mut query: Query<(&Position, &mut Transform)>) {
    for (position, mut transform) in &mut query {
        transform.translation.x = position.x;
        transform.translation.y = position.y;
    }
}

fn update_position_from_target(
    time: Res<Time>,
    mut query: Query<(Option<&Target>, &mut Position)>,
) {
    let speed = 100.0; // units per second

    for (maybe_target, mut position) in &mut query {
        let Some(target) = maybe_target else {
            continue;
        };
        // Calculate the difference vector from current position to target position
        let dx = target.position.x - position.x;
        let dy = target.position.y - position.y;
        // Compute the distance to the target
        let distance = (dx * dx + dy * dy).sqrt();

        if distance > 0.0 {
            // Calculate how much to move this frame
            let step = speed * time.delta_secs();
            if step >= distance {
                // If we can reach (or overshoot) the target this frame, snap to it.
                position.x = target.position.x;
                position.y = target.position.y;
            } else {
                // Move proportionally in the direction of the target
                position.x += dx / distance * step;
                position.y += dy / distance * step;
            }
        }
    }
}

fn update_peer_targets(
    mut commands: Commands,
    peers_query: Query<(Entity, &Position, Option<&Target>, &FavoritePlaces), With<Peer>>,
    places_query: Query<&Position, With<Place>>,
) {
    for (peer_entity, position, maybe_target, favorite_places) in peers_query.iter() {
        match maybe_target {
            Some(target) => {
                // If the peer already has a target and has reached it...
                if target.position.x == position.x && target.position.y == position.y {
                    // Remove the target component to indicate "no target"
                    commands.entity(peer_entity).remove::<Target>();
                }
            }
            None => {
                // If the peer has no target, choose a random place from favorites.
                let place_positions: Vec<Position> = favorite_places
                    .places
                    .iter()
                    .filter_map(|&place_entity| places_query.get(place_entity).ok())
                    .cloned()
                    .collect();

                if let Some(new_target) = place_positions.choose(&mut rng()).cloned() {
                    commands.entity(peer_entity).insert(Target {
                        position: new_target,
                    });
                }
            }
        }
    }
}
