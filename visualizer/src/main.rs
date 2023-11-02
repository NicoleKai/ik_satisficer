use bevy_egui::EguiContexts;
use bevy_mod_picking::{
    prelude::{Click, Drag, Move, Pointer, RaycastPickTarget},
    selection::PickSelection,
    DefaultPickingPlugins, PickableBundle,
};
use bevy_transform_gizmo::{
    GizmoTransformable, GizmoUpdate, RotationOriginOffset, TransformGizmoEvent,
    TransformGizmoPlugin,
};
// use ik2::Limb;

use bevy::{ecs::schedule::ScheduleGraph, prelude::*};

use egui_plot::{BoxPlot, Line, Plot, PlotPoint, PlotPoints, PlotUi};
// use ik_satisficer::{self, IKSatisficer, Limb, LimbNode, Positioned};
// use ik2;
use ik3::{self, FabrikChain};
use itertools::Itertools;

#[derive(Resource)]
pub struct UiState {
    lock_ground: bool,
}

impl Default for UiState {
    fn default() -> Self {
        Self { lock_ground: true }
    }
}

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
        .init_resource::<UiState>()
        .add_systems(Startup, setup)
        .add_systems(Update, recompute_limb)
        // .add_systems(Update, render_limb)
        .add_systems(Update, display_ui)
        .run();
}

#[derive(Component, Default, Debug)]
struct ControlBall {
    index: usize,
}

#[derive(Component, Default, Debug)]
struct Segment {
    index: usize,
}

#[derive(Bundle, Default)]
struct ControlBallBundle {
    pbr: PbrBundle,
    pickable: PickableBundle,
    raycast: RaycastPickTarget,
    gizmo_transformable: GizmoTransformable,
    control_ball: ControlBall,
}

#[derive(Bundle, Default)]
struct SegmentBundle {
    pbr: PbrBundle,
    segment: Segment,
}

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

    let sphere_mesh = meshes.add(Mesh::from(shape::UVSphere {
        radius: 0.3,
        ..Default::default()
    }));
    let material = materials.add(StandardMaterial::default());
    for i in 0..chain.joints.len() {
        commands.spawn(ControlBallBundle {
            pbr: PbrBundle {
                mesh: sphere_mesh.clone(),
                material: material.clone(),
                transform: Transform::from_translation(chain.joints[i]),
                ..Default::default()
            },
            control_ball: ControlBall { index: i },
            ..default()
        });
    }

    for i in 0..chain.lengths.len() {
        let cylinder_mesh = meshes.add(Mesh::from(shape::Cylinder {
            radius: 0.15,
            height: chain.lengths[i],
            ..default()
        }));
        commands.spawn(SegmentBundle {
            pbr: PbrBundle {
                mesh: cylinder_mesh,
                material: material.clone(),
                transform: chain.segment_transforms[i],
                ..Default::default()
            },
            segment: Segment { index: i },
            ..default()
        });
    }
    // for i in 0..chain.lengths.len() {
    //     commands.spawn()
    // }
    commands.spawn(ChainComponent(chain));
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
    mut query_ball: Query<(&ControlBall, &mut Transform), Without<Segment>>,
    mut query_segment: Query<(&Segment, &mut Transform), Without<ControlBall>>,
    mut query_chain: Query<&mut ChainComponent>,
    mut query_velocity_display: Query<&mut VelocityDisplay>,
    mut ev_gizmo: EventReader<GizmoUpdate>,
    // selected_items_query: Query<
    //     Entity,
    //     With<PickSelection, GlobalTransform, Option<&RotationOriginOffset>>,
    // >,
) {
    let mut excluded: Vec<usize> = Vec::new();
    if ev_gizmo.is_empty() {
        return;
    }

    let mut chain = query_chain.single_mut();
    chain.0.targets.clear();
    for event in ev_gizmo.iter() {
        // dbg!(&query_ball.iter().map(|(e, _b, _t)| e).collect_vec());
        let (ball, transform) = query_ball
            .get(event.entity)
            .expect("Something is moving but it's not a ball!");
        excluded.push(ball.index);
        // chain.0.joints[ball.index].clone_from(&transform.translation);
        chain
            .0
            .targets
            .push((ball.index, transform.translation.clone()));
    }
    for (ball, mut transform) in query_ball.iter_mut() {
        if !excluded.contains(&ball.index) {
            dbg!(&ball);
            *transform = Transform::from_translation(chain.0.joints[ball.index]);
        }
    }

    for (segment, mut transform) in query_segment.iter_mut() {
        *transform = chain.0.segment_transforms[segment.index];
    }
    chain.0.solve(10);
    if !chain.0.angular_velocities.is_empty() {
        query_velocity_display
            .single_mut()
            .0
            .push(chain.0.angular_velocities.clone());
    }
}

// pub fn render_limb(mut query: Query<&mut ChainComponent>, mut gizmos: Gizmos) {
//     for chain in &mut query {
//         for joint in &chain.0.joints {
//             gizmos.sphere(*joint, Quat::default(), 0.2, Color::ORANGE_RED);
//         }
//         // for transform in chain.0.segment_transforms.iter() {
//         //     gizmos.rect(
//         //         transform.translation,
//         //         transform.rotation,
//         //         Vec2::splat(1.0),
//         //         Color::BLUE,
//         //     );
//         // }
//         // gizmos.linestrip(chain.0.joints.clone(), Color::ORANGE);
//     }
// }

fn display_ui(
    mut context: EguiContexts,
    mut query: Query<&mut VelocityDisplay>,
    mut query_chain: Query<&mut ChainComponent>,
    mut query_ball: Query<(&ControlBall, &mut Transform), Without<Segment>>,
    mut query_segment: Query<(&Segment, &mut Transform), Without<ControlBall>>,
    mut ui_state: ResMut<UiState>,
) {
    let mut chain = query_chain.single_mut();
    egui::Window::new("Limb Control").show(context.ctx_mut(), |ui| {
        let mut velocity_display = query.single_mut();
        if ui.button("Reset graph").clicked() {
            velocity_display.0.clear();
        }
        if ui.button("Reset all").clicked() {
            velocity_display.0.clear();
            chain.0.reset();
            for (ball, mut transform) in query_ball.iter_mut() {
                *transform = Transform::from_translation(chain.0.joints[ball.index]);
            }

            for (segment, mut transform) in query_segment.iter_mut() {
                *transform = chain.0.segment_transforms[segment.index];
            }

            // *query_ball.single_mut().1 =
            //     Transform::from_translation(query_chain.single().0.get_ee().to_owned());
        }
        if ui
            .checkbox(&mut ui_state.lock_ground, "Lock Ground")
            .changed()
        {
            chain.0.lock_ground = ui_state.lock_ground;
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
