use ik2::Limb;

use bevy::prelude::*;

// use ik_satisficer::{self, IKSatisficer, Limb, LimbNode, Positioned};
use ik2;
use itertools::Itertools;

// #[derive(Component)]
// pub struct IKSatisficerComponent(IKSatisficer);
#[derive(Component)]
pub struct LimbComponent(Limb);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, render_limb)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut limb = Limb::new(3, 1, Vec3::new(0.0, 0.8, 0.0));
    // i luv panicks 💜 php time
    limb.solve().unwrap();

    commands.spawn(LimbComponent(limb));

    // let ik_satisficer = IKSatisficer::new(1, limb);
    // commands.spawn(IKSatisficerComponent(ik_satisficer));

    // Some light to see something
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 9000.,
            range: 100.,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(8., 16., 8.),
        ..default()
    });

    // ground plane
    commands.spawn(PbrBundle {
        mesh: meshes.add(shape::Plane::from_size(50.).into()),
        material: materials.add(Color::SILVER.into()),
        ..default()
    });

    // The camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0., 6., 12.).looking_at(Vec3::new(0., 0., 0.), Vec3::Y),
        ..default()
    });
}

pub fn render_limb(mut query: Query<&mut LimbComponent>, mut gizmos: Gizmos) {
    for limb in &mut query {
        // limb.0.solve().unwrap();         // let goal = Vec3::new(1.0, 1.0, 1.0);
        // limb.0.target = goal;
        // limb.0.solve();
        gizmos.sphere(limb.0.target, Quat::default(), 0.3, Color::GREEN);

        // fn sphere(pos: Vec3, size: f32) {
        //     gizmos.sphere
        // }
        for segment in &limb.0.segments {
            gizmos.sphere(
                segment.start - Vec3::splat(0.01),
                Quat::default(),
                0.1,
                Color::BLUE,
            );
            gizmos.line(segment.start, segment.end, Color::ORANGE);
            gizmos.sphere(segment.end, Quat::default(), 0.1, Color::RED);
        }
        // gizmos.linestrip(limb.0.segments.iter().map(|x| x.end), Color::ORANGE);
    }
}
