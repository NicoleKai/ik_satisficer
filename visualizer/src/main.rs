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
    let chain = FabrikChain::new(joints);
    // let mut limb = Limb::new(3, 5, Vec3::new(0.0, 0.0, 0.0));
    // i luv panicks 💜 php time
    // limb.solve().unwrap();
    // println!("{:?}", &limb);

    commands.spawn(ChainComponent(chain));
    commands.spawn(VelocityDisplay::default());

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
    mut query_chain: Query<&mut ChainComponent>,
    mut query_velocity_display: Query<&mut VelocityDisplay>,
) {
    let Ok((_ball, new_target)) = query_ball.get_single() else {
        return;
    };

    for mut chain in query_chain.iter_mut() {
        chain.0.solve(new_target.translation, 10);
        for i in 0..chain.0.angular_velocities.len() {
            query_velocity_display
                .single_mut()
                .0
                .push(chain.0.angular_velocities.clone());
        }
        // dbg!(&chain.0.angles);
        // dbg!(&chain.0.prev_angles);
        // dbg!(&limb.0);
        // limb.0.target = new_target.translation;
        // limb.0.solve().unwrap();
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

            // let a = chain.0.joints[i];
            // let b = chain.0.joints[i - 1];

            // let ab_vector = b - a;
            // let ab_vector = ab_vector.normalize();

            // let theta = std::f32::consts::PI / 2.0; // 90 degrees in radians

            // let quaternion = Quat::from_axis_angle(ab_vector, theta);

            // gizmos.rect((a + b) / 2.0, quaternion, Vec2::splat(1.0), Color::BLUE);

            // let ab_vector = b - a;
            // let rotation_axis = ab_vector.normalize();

            // let arbitrary_vector = if rotation_axis.x.abs() > 0.9 {
            //     Vec3::new(0.0, 1.0, 0.0)
            // } else {
            //     Vec3::new(1.0, 0.0, 0.0)
            // };

            // let orthogonal_vector = rotation_axis.cross(arbitrary_vector).normalize();

            // let reference_vector = Vec3::new(1.0, 0.0, 0.0);
            // let quaternion = Quat::from_rotation_arc(reference_vector, orthogonal_vector);

            // gizmos.rect((a + b) / 2.0, quaternion, Vec2::splat(1.0), Color::BLUE);

            // let ab_vector = b - a;
            // let ab_vector = ab_vector.normalize();

            // let reference_vector = Vec3::Y;
            // let rotation_axis = reference_vector.cross(ab_vector).normalize();

            // let cos_theta = reference_vector.dot(ab_vector);
            // let theta = cos_theta.acos();

            // let quaternion = Quat::from_axis_angle(rotation_axis, theta);
            // //  * Quat::from_rotation_x(90f32.to_radians());
            // gizmos.rect((a + b) / 2.0, quaternion, Vec2::splat(1.), Color::BLUE)
        }
        // for i in 1..self.joints.len() {
        //     let a = self.joints[i + 1];
        //     let b = self.joints[i];
        //     (b - a).
        // }

        gizmos.linestrip(chain.0.joints.clone(), Color::ORANGE);
        // gizmos.sphere(limb.0.target, Quat::default(), 0.5, Color::GREEN);
        // for segment in &limb.0.segments {
        //     gizmos.sphere(
        //         segment.start - Vec3::splat(0.01),
        //         Quat::default(),
        //         0.1,
        //         Color::BLUE,
        //     );
        //     gizmos.line(segment.start, segment.end, Color::ORANGE);
        //     gizmos.sphere(segment.end, Quat::default(), 0.1, Color::RED);
        // }
    }
}

// fn display_ui(mut context: EguiContexts, mut query: Query<&mut VelocityDisplay>) {
//     egui::Window::new("Limb Control").show(context.ctx_mut(), |ui| {
//         for velocity_display in query.iter() {
//             // velocity display is [angle velocity][time]
//             // while we need [time][angle velocity]
//             let mut velocities: Vec<Vec<[f64; 2]>> = Vec::new();
//             let Some(first_len) = velocity_display.0.first().map(|x| x.len()) else {
//                 return;
//             };
//             for _ in 0..first_len {
//                 velocities.push(Vec::new());
//             }
//             for x in 0..velocity_display.0.len() {
//                 for y in 0..velocity_display.0[x].len() {
//                     let new_point = [x as f64, velocity_display.0[x][y] as f64];
//                     match velocities.get_mut(y) {
//                         Some(y_ptr) => {
//                             y_ptr.push(new_point);
//                         }
//                         None => {
//                             velocities.push(vec![new_point]);
//                         }
//                     }
//                 }
//             }
//             let velocities = velocities
//                 .into_iter()
//                 .map(|x| PlotPoints::new(x))
//                 .collect_vec();
//             let lines = velocities.into_iter().map(|x| Line::new(x)).collect_vec();

//             Plot::new("velocity").view_aspect(2.0).show(ui, |plot_ui| {
//                 for line in lines {
//                     plot_ui.line(line);
//                 }
//             });
//         }
//     });
// }
fn display_ui(mut context: EguiContexts, mut query: Query<&mut VelocityDisplay>) {
    egui::Window::new("Limb Control").show(context.ctx_mut(), |ui| {
        for velocity_display in query.iter() {
            if let Some(first_len) = velocity_display.0.first().map(|x| x.len()) {
                let mut velocities: Vec<Vec<[f64; 2]>> = vec![Vec::new(); first_len];

                for (x, x_values) in velocity_display.0.iter().enumerate() {
                    for (y, &y_value) in x_values.iter().enumerate() {
                        let new_point = [x as f64, y_value as f64];
                        velocities[y].push(new_point);
                    }
                }

                let lines = velocities
                    .into_iter()
                    .map(|x| Line::new(PlotPoints::new(x)))
                    .collect_vec();

                Plot::new("velocity").view_aspect(2.0).show(ui, |plot_ui| {
                    for line in lines {
                        plot_ui.line(line);
                    }
                });
            }
        }
    });
}
