use bevy_egui::EguiContexts;
use bevy_mod_picking::DefaultPickingPlugins;
use bevy_transform_gizmo::TransformGizmoPlugin;
// use ik2::Limb;

use bevy::{ecs::schedule::ScheduleGraph, prelude::*};

use egui_plot::{BoxPlot, Line, Plot, PlotPoint, PlotPoints, PlotUi};
// use ik_satisficer::{self, IKSatisficer, Limb, LimbNode, Positioned};
// use ik2;
use ik3::{self, FabrikChain};
use itertools::Itertools;

// #[derive(Component)]
// pub struct IKSatisficerComponent(IKSatisficer);
#[derive(Component)]
pub struct ChainComponent(FabrikChain);

#[derive(Component, Default)]
pub struct VelocityDisplay(Vec<Vec<f32>>);

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            DefaultPickingPlugins,
            bevy_egui::EguiPlugin,
            TransformGizmoPlugin::default(),
        ))
        .insert_resource(Msaa::Sample4)
        .insert_resource(ClearColor(Color::BLACK))
        .add_systems(Startup, setup)
        .add_systems(Update, recompute_limb)
        .add_systems(Update, render_limb)
        .add_systems(Update, display_ui)
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
        Vec3::new(3.0, 0.0, 0.0),
        Vec3::new(4.0, 0.0, 0.0),
    ];
    let ctrl_point_pos = joints.last().cloned().expect("No pos");
    let chain = FabrikChain::new(joints);
    let ee = chain.get_ee().to_owned();
    commands.spawn(ChainComponent(chain));
    commands.spawn(VelocityDisplay::default());

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
            transform: Transform::from_translation(ee),
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
    mut query_chain: Query<&mut ChainComponent>,
    mut query_velocity_display: Query<&mut VelocityDisplay>,
) {
    let Ok((_ball, new_target)) = query_ball.get_single() else {
        return;
    };

    for mut chain in query_chain.iter_mut() {
        chain.0.solve(new_target.translation, 10);
        if !chain.0.angular_velocities.is_empty() {
            query_velocity_display
                .single_mut()
                .0
                .push(chain.0.angular_velocities.clone());
        }
    }
}

pub fn render_limb(mut query: Query<&mut ChainComponent>, mut gizmos: Gizmos) {
    for chain in &mut query {
        for joint in &chain.0.joints {
            gizmos.sphere(*joint, Quat::default(), 0.2, Color::ORANGE_RED);
        }

        for i in 1..chain.0.joints.len() {
            let a = chain.0.joints[i];
            let b = chain.0.joints[i - 1];

            let ab_vector = b - a;
            let ab_vector = ab_vector.normalize();

            let world_axis = Vec3::new(0.0, 1.0, 0.0); // Y-axis as an example

            // Cross to get a perpendicular vector
            let perp_vector = ab_vector.cross(world_axis).normalize();

            // Cross again to get a second perpendicular vector properly aligned
            let perp_vector2 = ab_vector.cross(perp_vector).normalize();

            // Create a quaternion from the perpendicular vector
            let quaternion =
                Quat::from_mat3(&Mat3::from_cols(ab_vector, perp_vector, perp_vector2))
                    * Quat::from_rotation_y(90f32.to_radians());

            gizmos.rect((a + b) / 2.0, quaternion, Vec2::splat(1.0), Color::BLUE);
        }
        gizmos.linestrip(chain.0.joints.clone(), Color::ORANGE);
    }
}

fn display_ui(
    mut context: EguiContexts,
    mut query: Query<&mut VelocityDisplay>,
    mut query_chain: Query<&mut ChainComponent>,
    mut query_ball: Query<(&ControlBall, &mut Transform)>,
) {
    egui::Window::new("Limb Control").show(context.ctx_mut(), |ui| {
        let mut velocity_display = query.single_mut();
        if ui.button("Reset graph").clicked() {
            velocity_display.0.clear();
        }
        if ui.button("Reset all").clicked() {
            velocity_display.0.clear();
            query_chain.single_mut().0.reset();

            *query_ball.single_mut().1 =
                Transform::from_translation(query_chain.single().0.get_ee().to_owned());
        }
        ui.separator();
        // velocity display is [angle velocity][time]
        // while we need [time][angle velocity]
        let mut velocities: Vec<Vec<[f64; 2]>> = Vec::new();
        if let Some(first_len) = velocity_display.0.first().map(|x| x.len()) {
            for _ in 0..first_len {
                velocities.push(Vec::new());
            }
            for x in 0..velocity_display.0.len() {
                for y in 0..velocity_display.0[x].len() {
                    let new_point = [x as f64, velocity_display.0[x][y] as f64];
                    match velocities.get_mut(y) {
                        Some(y_ptr) => {
                            y_ptr.push(new_point);
                        }
                        None => {
                            velocities.push(vec![new_point]);
                        }
                    }
                }
            }
            let lines = velocities
                .into_iter()
                .map(|x| Line::new(PlotPoints::new(x)));

            Plot::new("velocity").view_aspect(2.0).show(ui, |plot_ui| {
                for line in lines {
                    plot_ui.line(line);
                }
            });
        } else {
            ui.label("NO DATA");
        }
    });
}
