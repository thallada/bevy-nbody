use argh::FromArgs;
use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, PrintDiagnosticsPlugin},
    input::keyboard::{ElementState, KeyboardInput},
    input::mouse::{MouseButtonInput, MouseMotion},
    prelude::*,
    render::camera::{Camera, OrthographicProjection},
    render::pass::ClearColor,
    window::CursorMoved,
};
use bigbang::{Entity, GravTree};
use rand::Rng;

#[derive(FromArgs)]
#[argh(description = "n-body simulation in bevy using bigbang")]
struct Options {
    #[argh(option, default = "100", short = 'n', description = "number of bodies in the simulation")]
    num_bodies: usize,
    #[argh(option, default = "0.02", short = 't', description = "granularity of simulation (how much each frame impacts movement)")]
    time_step: f64,
    #[argh(option, default = "1280", short = 'w', description = "initial width of spawned window")]
    width: u32,
    #[argh(option, default = "720", short = 'h', description = "initial height of spawned window")]
    height: u32,
    #[argh(option, default = "10.0", short = 's', description = "initial scale of view (bigger = more zoomed out)")]
    scale: f32
}

struct Simulation(GravTree<Entity>);

#[derive(Default)]
struct State {
    mouse_button_event_reader: EventReader<MouseButtonInput>,
    mouse_motion_event_reader: EventReader<MouseMotion>,
    cursor_moved_event_reader: EventReader<CursorMoved>,
    keyboard_event_reader: EventReader<KeyboardInput>,
    cursor_position: Option<Vec2>,
    zooming: bool,
    panning: bool,
    paused: bool,
    follow_body_index: Option<usize>,
}

fn main() {
    let options: Options = argh::from_env();
    App::build()
        .add_resource(Msaa { samples: 4 })
        .add_resource(WindowDescriptor {
            title: "bevy-nbody".to_string(),
            width: options.width,
            height: options.height,
            ..Default::default()
        })
        .add_default_plugins()
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(PrintDiagnosticsPlugin::default())
        .init_resource::<State>()
        .add_resource(ClearColor(Color::rgb(0.01, 0.01, 0.01)))
        .add_resource(Simulation(GravTree::new(
            &initialize_bodies(options.num_bodies, options.width, options.height, options.scale),
            options.time_step,
        )))
        .add_resource(options)
        .add_startup_system(add_bodies.system())
        .add_system(time_step.system())
        .add_system(update_bodies.system())
        .add_system(follow.system())
        .add_system(mouse_input.system())
        .add_system(keyboard_input.system())
        .run();
}

fn initialize_bodies(num: usize, width: u32, height: u32, scale: f32) -> Vec<Entity> {
    let mut rng = rand::thread_rng();
    let mut bodies = vec![];
    for i in 0..num {
        let mass = if i == 0 {
            // big boi
            rng.gen_range(500., 1500.)
        } else {
            rng.gen_range(50., 500.)
        };
        bodies.push(Entity {
            x: rng.gen_range(
                -(width as f64 / 2.) * scale as f64,
                (width as f64 / 2.) * scale as f64,
            ),
            y: rng.gen_range(
                -(height as f64 / 2.) * scale as f64,
                (height as f64 / 2.) * scale as f64,
            ),
            z: 0.,
            vx: rng.gen_range(-50., 50.),
            vy: rng.gen_range(-50., 50.),
            vz: 0.,
            mass,
            radius: mass / 30.,
        });
    }
    bodies
}

fn add_bodies(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    asset_server: Res<AssetServer>,
    grav_tree: Res<Simulation>,
    options: Res<Options>,
) {
    let mut rng = rand::thread_rng();
    let texture = asset_server.load("assets/circle.png").unwrap();
    commands.spawn(Camera2dComponents {
        scale: Scale(options.scale),
        ..Camera2dComponents::default()
    });

    let mut index = 0;
    for body in grav_tree.0.as_vec() {
        commands
            .spawn(SpriteComponents {
                mesh: meshes.add(Mesh::from(shape::Quad::new(Vec2::new(
                    (body.radius / 10.) as f32,
                    (body.radius / 10.) as f32,
                )))),
                material: materials.add(ColorMaterial {
                    color: Color::rgb(
                        rng.gen_range(0.8, 1.),
                        rng.gen_range(0., 0.6),
                        rng.gen_range(0., 0.05),
                    ),
                    texture: texture.into(),
                }),
                translation: Translation(Vec3::new(body.x as f32, body.y as f32, index as f32)),
                draw: Draw {
                    is_transparent: true,
                    ..Default::default()
                },
                ..Default::default()
            })
            .with(body);
        index += 1;
    }
}

fn time_step(state: Res<State>, mut grav_tree: ResMut<Simulation>) {
    if !state.paused {
        grav_tree.0 = grav_tree.0.time_step();
    }
}

fn update_bodies(grav_tree: Res<Simulation>, mut body_query: Query<(&Entity, &mut Translation)>) {
    let updated_bodies = grav_tree.0.as_vec();
    let mut index = 0;
    for (_, mut translation) in &mut body_query.iter() {
        let updated_body = &updated_bodies[index];
        translation.0.set_x(updated_body.x as f32);
        translation.0.set_y(updated_body.y as f32);
        index += 1;
    }
}

fn screen_to_translation_coord(
    screen_coord: Vec2,
    projection: &OrthographicProjection,
    translation: &Translation,
    scale: &Scale,
) -> Vec2 {
    Vec2::new(
        ((projection.left + screen_coord.x()) * scale.0) + translation.x(),
        ((projection.bottom + screen_coord.y()) * scale.0) + translation.y(),
    )
}

fn mouse_input(
    mut state: ResMut<State>,
    grav_tree: Res<Simulation>,
    mouse_button_input_events: Res<Events<MouseButtonInput>>,
    mouse_motion_events: Res<Events<MouseMotion>>,
    cursor_moved_events: Res<Events<CursorMoved>>,
    mut query: Query<(
        &mut Camera,
        &mut Scale,
        &mut Translation,
        &mut OrthographicProjection,
    )>,
) {
    for event in state
        .mouse_button_event_reader
        .iter(&mouse_button_input_events)
    {
        if event.button == MouseButton::Middle && event.state == ElementState::Pressed {
            state.zooming = true;
        } else if event.button == MouseButton::Middle && event.state == ElementState::Released {
            state.zooming = false;
        } else if event.button == MouseButton::Left && event.state == ElementState::Pressed {
            state.panning = true;
            state.follow_body_index = None;
        } else if event.button == MouseButton::Left && event.state == ElementState::Released {
            state.panning = false;
        } else if event.button == MouseButton::Right && event.state == ElementState::Released {
            if let Some(position) = state.cursor_position {
                for (_, scale, translation, projection) in &mut query.iter() {
                    if let Some((closest_index, _)) = grav_tree
                        .0
                        .as_vec()
                        .into_iter()
                        .enumerate()
                        .find(|(_, body)| {
                            let hit_slop = body.radius as f32 * 2. + (10. * scale.0);
                            let translation_coord = screen_to_translation_coord(
                                position,
                                &projection,
                                &translation,
                                &scale,
                            );
                            if (translation_coord.x() - body.x as f32).abs() <= hit_slop
                                && (translation_coord.y() - body.y as f32).abs() <= hit_slop
                            {
                                return true;
                            }
                            false
                        })
                    {
                        state.follow_body_index = Some(closest_index);
                    } else {
                        state.follow_body_index = None;
                    }
                }
            }
        }
    }

    for event in state.mouse_motion_event_reader.iter(&mouse_motion_events) {
        if state.zooming || state.panning {
            for (_, mut scale, mut translation, _) in &mut query.iter() {
                if state.zooming {
                    scale.0 += (event.delta.y() / 500.) * (scale.0 / 3.);
                }
                if state.panning {
                    *translation.x_mut() -= event.delta.x() * (scale.0 / 3.);
                    *translation.y_mut() += event.delta.y() * (scale.0 / 3.);
                }
            }
        }
    }

    for event in state.cursor_moved_event_reader.iter(&cursor_moved_events) {
        state.cursor_position = Some(event.position);
    }
}

fn keyboard_input(
    mut state: ResMut<State>,
    keyboard_input_events: Res<Events<KeyboardInput>>,
    mut grav_tree: ResMut<Simulation>,
    windows: Res<Windows>,
    options: Res<Options>,
    mut query: Query<(&Camera, &Scale, &mut Translation)>,
) {
    for event in state.keyboard_event_reader.iter(&keyboard_input_events) {
        if let Some(key_code) = event.key_code {
            if key_code == KeyCode::R && event.state == ElementState::Pressed {
                for (_, scale, mut translation) in &mut query.iter() {
                    if let Some(window) = windows.get_primary() {
                        translation.0.set_x(0.);
                        translation.0.set_y(0.);
                        grav_tree.0 = GravTree::new(
                            &initialize_bodies(options.num_bodies, window.width, window.height, scale.0),
                            options.time_step,
                        );
                    }
                }
            } else if key_code == KeyCode::Space && event.state == ElementState::Pressed {
                state.paused = !state.paused;
            }
        }
    }
}

fn follow(
    state: Res<State>,
    mut body_query: Query<(&Entity, &mut Translation)>,
    mut camera_query: Query<(&mut Camera, &mut Translation)>,
) {
    if let Some(follow_body_index) = state.follow_body_index {
        for (_, mut translation) in &mut camera_query.iter() {
            if let Some((_, body_translation)) = body_query.iter().iter().nth(follow_body_index) {
                translation.0.set_x(body_translation.0.x());
                translation.0.set_y(body_translation.0.y());
            }
        }
    }
}
