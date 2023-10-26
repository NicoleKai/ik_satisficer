use bevy_mod_picking::DefaultPickingPlugins;
use bevy_transform_gizmo::TransformGizmoPlugin;
// use ik2::Limb;

use bevy::prelude::*;

// use ik_satisficer::{self, IKSatisficer, Limb, LimbNode, Positioned};
// use ik2;
use ik3::{self, FabrikChain};
use itertools::Itertools;

// #[derive(Component)]
// pub struct IKSatisficerComponent(IKSatisficer);
#[derive(Component)]
pub struct ChainComponent(FabrikChain);

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            DefaultPickingPlugins,
            TransformGizmoPlugin::default(),
        ))
        .insert_resource(Msaa::Sample4)
        .insert_resource(ClearColor(Color::BLACK))
        .add_systems(Startup, setup)
        .add_systems(Update, recompute_limb)
        .add_systems(Update, render_limb)
        .run();
}

#[derive(Component)]
struct ControlBall;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let joints = vec![
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(1.0, 0.0, 0.0),
        Vec3::new(2.0, 0.0, 0.0),
    ];
    let chain = FabrikChain::new(joints);
    // let mut limb = Limb::new(3, 5, Vec3::new(0.0, 0.0, 0.0));
    // i luv panicks 💜 php time
    // limb.solve().unwrap();
    // println!("{:?}", &limb);

    commands.spawn(ChainComponent(chain));

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

    let mesh = meshes.add(Mesh::from(shape::UVSphere {
        radius: 0.2,
        ..Default::default()
    }));
    let material = materials.add(StandardMaterial::default());
    commands.spawn((
        PbrBundle {
            mesh,
            material,
            ..Default::default()
        },
        bevy_mod_picking::PickableBundle::default(),
        bevy_mod_picking::backends::raycast::RaycastPickTarget::default(),
        bevy_transform_gizmo::GizmoTransformable,
        ControlBall,
    ));
    // // ground plane
    // commands.spawn(PbrBundle {
    //     mesh: meshes.add(shape::Plane::from_size(50.).into()),
    //     material: materials.add(Color::SILVER.into()),
    //     ..default()
    // });

    // The camera
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0., 6., 7.).looking_at(Vec3::new(0., 0., 0.), Vec3::Y),
            ..default()
        },
        bevy_mod_picking::backends::raycast::RaycastPickCamera::default(),
        bevy_transform_gizmo::GizmoPickSource::default(),
    ));
}

fn recompute_limb(
    query_ball: Query<(&ControlBall, &Transform), Changed<Transform>>,
    mut query_limb: Query<&mut ChainComponent>,
) {
    let Ok((_ball, new_target)) = query_ball.get_single() else {
        return;
    };

    for mut limb in query_limb.iter_mut() {
        dbg!(&limb.0);
        limb.0.target = new_target.translation;
        limb.0.solve().unwrap();
    }
}
pub fn render_limb(mut query: Query<&mut ChainComponent>, mut gizmos: Gizmos) {
    for limb in &mut query {
        gizmos.sphere(limb.0.target, Quat::default(), 0.5, Color::GREEN);
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
    }
}
