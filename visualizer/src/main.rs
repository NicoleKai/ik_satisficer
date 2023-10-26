//! Demonstrates how to work with Cubic curves.

use std::process::exit;

use bevy::{
    math::{cubic_splines::CubicCurve, vec3},
    prelude::*,
    window::ExitCondition,
};

use ik_satisficer::{self, IKSatisficer, Limb, Positioned};

#[derive(Component)]
pub struct IKSatisficerComponent(IKSatisficer);

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
    let ik_satisficer = IKSatisficer::new(1, limb);
    commands.spawn(IKSatisficerComponent(ik_satisficer));

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

pub fn render_limb(mut query: Query<&mut IKSatisficerComponent>, mut gizmos: Gizmos) {
    for mut ik in &mut query {
        let goal = Vec3::new(1.0, 1.0, 1.0);
        ik.0.satisfice(goal.into()).unwrap();
        gizmos.sphere(goal, Quat::default(), 0.3, Color::GREEN);
        let mut joint_points: Vec<Vec3> = Vec::new();
        for node in ik.0.nodes() {
            match node {
                ik_satisficer::LimbNode::Joint(j) => {
                    gizmos.sphere(j.pos().into(), Quat::default(), 0.15, Color::BLUE);
                    joint_points.push(j.pos().into());
                }
                ik_satisficer::LimbNode::Segment(s) => {
                    // gizmos.sphere(s.pos().into(), Quat::default(), 0.1, Color::YELLOW_GREEN);
                }
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
