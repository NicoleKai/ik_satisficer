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

use bevy::{ecs::schedule::ScheduleGraph, prelude::*};

use egui_plot::{BoxPlot, Line, Plot, PlotPoint, PlotPoints, PlotUi};
use ik3::{self, FabrikChain, MotionHeuristics, PoseDiscrepancy};
use itertools::Itertools;
use strum::{EnumIter, IntoEnumIterator};

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
pub struct LimbData {
    real_limb: FabrikChain,
    phantom_limb: FabrikChain,
}

impl LimbData {
    fn get_mut(&mut self, state: &LimbState) -> &mut FabrikChain {
        match state {
            LimbState::RealLimb => &mut self.real_limb,
            LimbState::FantasyLimb => &mut self.phantom_limb,
        }
    }
    fn get(&self, state: &LimbState) -> &FabrikChain {
        match state {
            LimbState::RealLimb => &self.real_limb,
            LimbState::FantasyLimb => &self.phantom_limb,
        }
    }
}

#[derive(Component, Default)]
pub struct VelocityDisplay(Vec<Vec<f32>>);

fn main() {
    let window = Window {
        title: "IK Satisficer Demo [DISPLAY_TESTING]".to_string(),
        ..default()
    };
    App::new()
        .add_state::<LimbState>()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(window),
                ..default()
            }),
            DefaultPickingPlugins,
            bevy_egui::EguiPlugin,
            TransformGizmoPlugin::default(),
        ))
        .add_event::<SyncTransforms>()
        .add_event::<RecomputeLimb>()
        .add_event::<MoveLimb>()
        .insert_resource(Msaa::Sample4)
        .insert_resource(ClearColor(Color::BLACK))
        .init_resource::<UiState>()
        .add_systems(Startup, setup)
        .add_systems(Update, display_ui)
        .add_systems(
            Update,
            move_limb
                .run_if(on_event::<GizmoUpdate>().or_else(on_event::<MoveLimb>()))
                .before(recompute_limb),
        )
        .add_systems(
            Update,
            recompute_limb
                .run_if(on_event::<GizmoUpdate>().or_else(on_event::<RecomputeLimb>()))
                .before(sync_ctrl_ball_transform),
        )
        .add_systems(
            Update,
            handle_limb_switch.run_if(resource_changed::<State<LimbState>>()),
        )
        .add_systems(
            Update,
            sync_ball_transform.run_if(on_event::<SyncTransforms>()),
        )
        .add_systems(
            Update,
            sync_ctrl_ball_transform.run_if(on_event::<SyncTransforms>()),
        )
        .add_systems(
            Update,
            sync_segment_transform.run_if(on_event::<SyncTransforms>()),
        )
        .run();
}

#[derive(Component, Default, Debug)]
struct ControlBall {
    index: usize,
}

#[derive(Component, Default, Debug)]
struct InnerBall {
    index: usize,
}

#[derive(Component, Default, Debug)]
struct Segment {
    index: usize,
}

#[derive(Event, Default)]
struct SyncTransforms;

#[derive(Event, Default)]
struct RecomputeLimb;

#[derive(Event, Default)]
struct MoveLimb;

#[derive(Bundle, Default)]
struct ControlBallBundle {
    pbr: PbrBundle,
    pickable: PickableBundle,
    raycast: RaycastPickTarget,
    gizmo_transformable: GizmoTransformable,
    control_ball: ControlBall,
}

#[derive(Bundle, Default)]
struct InnerBallBundle {
    pbr: PbrBundle,
    inner_ball: InnerBall,
}

#[derive(Bundle, Default)]
struct SegmentBundle {
    pbr: PbrBundle,
    segment: Segment,
}

#[derive(States, Default, Debug, Hash, PartialEq, Eq, Clone, EnumIter, strum::Display)]
enum LimbState {
    #[default]
    RealLimb,
    FantasyLimb,
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut ev_sync_transforms: EventWriter<SyncTransforms>,
) {
    let joints = vec![
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(1.0, 0.0, 0.0),
        Vec3::new(2.0, 0.0, 0.0),
        Vec3::new(3.0, 0.0, 0.0),
        Vec3::new(4.0, 0.0, 0.0),
    ];
    let real_limb = FabrikChain::new(joints, MotionHeuristics::default());
    let phantom_limb = real_limb.clone();
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

    let control_ball_mesh = meshes.add(Mesh::from(shape::UVSphere {
        radius: 0.29,
        ..Default::default()
    }));

    let ball_mesh = meshes.add(Mesh::from(shape::UVSphere {
        radius: 0.3,
        ..default()
    }));
    let material = materials.add(StandardMaterial::default());
    let translucent_material = materials.add(StandardMaterial {
        alpha_mode: AlphaMode::Mask(0.5),
        base_color: Color::rgba(0.7, 0.7, 1.0, 0.2),
        ..default()
    });
    for i in 0..real_limb.joints.len() {
        commands.spawn(InnerBallBundle {
            pbr: PbrBundle {
                mesh: ball_mesh.clone(),
                material: material.clone(),
                transform: Transform::from_translation(real_limb.joints[i]),
                ..Default::default()
            },
            inner_ball: InnerBall { index: i },
            ..default()
        });
        commands.spawn(ControlBallBundle {
            pbr: PbrBundle {
                mesh: control_ball_mesh.clone(),
                material: translucent_material.clone(),
                transform: Transform::from_translation(real_limb.joints[i]),
                ..Default::default()
            },
            control_ball: ControlBall { index: i },
            ..default()
        });
    }

    for i in 0..real_limb.lengths.len() {
        let cylinder_mesh = meshes.add(Mesh::from(shape::Cylinder {
            radius: 0.15,
            height: real_limb.lengths[i],
            ..default()
        }));
        commands.spawn(SegmentBundle {
            pbr: PbrBundle {
                mesh: cylinder_mesh,
                material: material.clone(),
                transform: real_limb.segment_transforms[i],
                ..Default::default()
            },
            segment: Segment { index: i },
            ..default()
        });
    }
    commands.spawn(LimbData {
        real_limb,
        phantom_limb,
    });

    ev_sync_transforms.send_default();
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

fn sync_ball_transform(
    mut query_chain: Query<&mut LimbData>,
    mut query_ball: Query<(&InnerBall, &mut Transform)>,
    limb_state: Res<State<LimbState>>,
) {
    let chain = query_chain.single_mut();
    let limb = chain.get(&limb_state.get());

    for (ball, mut transform) in query_ball.iter_mut() {
        *transform = Transform::from_translation(limb.joints[ball.index]);
    }
}

fn handle_limb_switch(mut ev_sync_transforms: EventWriter<SyncTransforms>) {
    dbg!("pls");
    ev_sync_transforms.send_default();
}

fn sync_ctrl_ball_transform(
    mut query_chain: Query<&mut LimbData>,
    mut query_ctrl_ball: Query<(&ControlBall, &mut Transform)>,
    limb_state: Res<State<LimbState>>,
) {
    let chain = query_chain.single_mut();
    let limb = chain.get(&limb_state.get());
    for (ctrl_ball, mut transform) in query_ctrl_ball.iter_mut() {
        *transform = Transform::from_translation(limb.joints[ctrl_ball.index]);
    }
}

fn sync_segment_transform(
    mut query_chain: Query<&mut LimbData>,
    mut query_segment: Query<(&Segment, &mut Transform)>,
    limb_state: Res<State<LimbState>>,
) {
    let chain = query_chain.single_mut();
    let limb = chain.get(&limb_state.get());
    for (segment, mut transform) in query_segment.iter_mut() {
        *transform = limb.segment_transforms[segment.index];
    }
}

fn move_limb(
    query_ctrl_ball: Query<(&ControlBall, &Transform)>,
    mut query_chain: Query<&mut LimbData>,
    mut ev_gizmo: EventReader<GizmoUpdate>,
    mut ev_recompute: EventWriter<RecomputeLimb>,
    limb_state: Res<State<LimbState>>,
) {
    let mut excluded: Vec<usize> = Vec::new();
    if ev_gizmo.is_empty() {
        return;
    }

    let mut chain = query_chain.single_mut();
    let limb = chain.get_mut(limb_state.get());
    limb.targets.clear();
    for event in ev_gizmo.iter() {
        let (ball, transform) = query_ctrl_ball
            .get(event.entity)
            .expect("Something is moving but it's not a ball!");
        excluded.push(ball.index);
        limb.targets
            .push((ball.index, transform.translation.clone()));
    }
    ev_recompute.send_default();
}

fn recompute_limb(
    mut query_chain: Query<&mut LimbData>,
    mut query_velocity_display: Query<&mut VelocityDisplay>,
    mut ev_sync_transforms: EventWriter<SyncTransforms>,
    limb_state: Res<State<LimbState>>,
) {
    let mut chain = query_chain.single_mut();
    let limb = chain.get_mut(limb_state.get());

    limb.solve(10, PoseDiscrepancy::default());

    if !limb.angular_velocities.is_empty() {
        query_velocity_display
            .single_mut()
            .0
            .push(limb.angular_velocities.clone());
    }
    ev_sync_transforms.send_default();
}

fn display_ui(
    mut context: EguiContexts,
    mut query: Query<&mut VelocityDisplay>,
    mut query_chain: Query<&mut LimbData>,
    mut ui_state: ResMut<UiState>,
    mut ev_sync_transforms: EventWriter<SyncTransforms>,
    limb_state_ro: ResMut<State<LimbState>>,
    mut limb_state: ResMut<NextState<LimbState>>,
) {
    let mut chain = query_chain.single_mut();
    egui::Window::new("Limb Control").show(context.ctx_mut(), |ui| {
        let mut velocity_display = query.single_mut();
        if ui.button("Reset graph").clicked() {
            velocity_display.0.clear();
        }
        if ui.button("Reset all").clicked() {
            velocity_display.0.clear();
            chain.real_limb.reset();
            chain.phantom_limb.reset();
            ev_sync_transforms.send_default();
        }
        if ui
            .checkbox(&mut ui_state.lock_ground, "Lock Ground")
            .changed()
        {
            chain.real_limb.lock_ground = ui_state.lock_ground;
            chain.phantom_limb.lock_ground = ui_state.lock_ground;
        }

        for possible_mode in LimbState::iter() {
            let name = possible_mode.to_string();
            if ui
                .radio_value(&mut limb_state_ro.clone(), possible_mode.clone(), name)
                .clicked()
            {
                limb_state.set(possible_mode);
            }
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
