//! Demonstrates how to work with Cubic curves.

use bevy::{
    math::{cubic_splines::CubicCurve, vec3},
    prelude::*,
};

use ik_satisficer::{self, Limb, Positioned};

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
    let limb = Limb::new(3);
    commands.spawn(LimbComponent(limb));

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

pub fn render_limb(mut query: Query<&LimbComponent>, mut gizmos: Gizmos) {
    for limb_compnent in &mut query {
        let mut joint_points: Vec<Vec3> = Vec::new();
        for node in limb_compnent.0.nodes() {
            match node {
                ik_satisficer::LimbNode::Joint(j) => {
                    gizmos.sphere(j.pos().into(), Quat::default(), 0.15, Color::BLUE);
                    joint_points.push(j.pos().into());
                }
                ik_satisficer::LimbNode::Segment(_) => {}
                ik_satisficer::LimbNode::Terminus(t) => {
                    gizmos.sphere(t.pos().into(), Quat::default(), 0.2, Color::RED);
                }
                ik_satisficer::LimbNode::Limb(_) => todo!(),
            }
        }
        gizmos.linestrip(joint_points, Color::ORANGE);

        // // Draw the curve
        // gizmos.linestrip(cubic_curve.0.iter_positions(50), Color::WHITE);
        // // position takes a point from the curve where 0 is the initial point
        // // and 1 is the last point
        // transform.translation = cubic_curve.0.position(t);
    }
}
