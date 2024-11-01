use bevy::{
    app::{App, Startup},
    asset::Assets,
    color::Color,
    prelude::{
        Camera2dBundle, Commands, Component, IntoSystemConfigs, Mesh, Query, Rectangle, ResMut,
        Resource, With,
    },
    sprite::{ColorMaterial, MaterialMesh2dBundle, Mesh2dHandle},
    utils::default,
    DefaultPlugins,
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use noise::NoiseFn;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WorldInspectorPlugin::new())
        .add_systems(Startup, (generate_world, generate_world_squares).chain())
        .run();
}

#[derive(Resource)]
struct Noise {
    seed: u32,
    scale: f64,
    octaves: u8,
    persistence: f64,
    lacunarity: f64,
    exponent: f64,
    max_height: f64,
}

fn get_height(noise: &Noise, x: f64, y: f64) -> f64 {
    let simplex = noise::Simplex::new(noise.seed);

    let mut amplitude = 1.0;
    let mut frequency = noise.scale;
    let mut noise_height = 0.0;

    for _ in 0..noise.octaves {
        let x_val = x * frequency;
        let y_val = y * frequency;

        let noise_val = simplex.get([x_val, y_val]) as f64;
        noise_height += noise_val * amplitude;

        amplitude *= noise.persistence;
        frequency *= noise.lacunarity;
    }

    noise_height = noise_height.max(0.0);
    noise_height = noise_height.powf(noise.exponent);

    noise_height * noise.max_height
}

fn get_height_map(
    noise: &Noise,
    width: u32,
    height: u32,
    offset_x: u32,
    offset_y: u32,
) -> Vec<f64> {
    let mut height_map = Vec::with_capacity((width * height) as usize);
    for y in offset_y..(height + offset_y) {
        for x in offset_x..(width + offset_x) {
            height_map.push(get_height(noise, x as f64, y as f64));
        }
    }
    height_map
}

#[derive(Component)]
struct World {
    width: u32,
    height: u32,
    noise: Noise,
}

fn generate_world(mut commands: Commands) {
    let noise = Noise {
        seed: 259,
        scale: 1.0,
        octaves: 8,
        persistence: 0.75,
        lacunarity: 1.75,
        exponent: 4.0,
        max_height: 1.0,
    };

    // window size
    let width = 100;
    let height = 100;

    println!("Generating world with size: {}x{}", width, height);

    commands.spawn(World {
        width,
        height,
        noise,
    });
}

fn height_to_color(height: f64, max_height: f64) -> Color {
    println!("height: {}, max_height: {}", height, max_height);
    Color::hsl(0.0, 0.0, ((height + max_height / 2.0) / max_height) as f32)
}

fn generate_world_squares(
    query: Query<&World, With<World>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2dBundle::default());

    for world in query.iter() {
        let height_map = get_height_map(&world.noise, world.width, world.height, 0, 0);

        for y in 0..world.height {
            for x in 0..world.width {
                let height = height_map[(y * world.width + x) as usize];
                let color = height_to_color(height, world.noise.max_height);

                commands.spawn(MaterialMesh2dBundle {
                    mesh: Mesh2dHandle(meshes.add(Rectangle::new(3.0, 3.0))),
                    material: materials.add(color),
                    transform: bevy::transform::components::Transform {
                        translation: bevy::math::Vec3::new((x * 3) as f32, (y * 3) as f32, 0.0),
                        ..default()
                    },
                    ..default()
                });
            }
        }
    }
}
