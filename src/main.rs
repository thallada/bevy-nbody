use bevy::{prelude::*, render::pass::ClearColor};
use bigbang::{Entity, GravTree};
use rand::Rng;

struct Simulation(GravTree<Entity>);

fn main() {
    let mut rng = rand::thread_rng();
    let mut bodies = vec![];
    for _ in 0..100 {
        bodies.push(Entity {
            x: rng.gen_range(-400., 400.),
            y: rng.gen_range(-400., 400.),
            z: 0.,
            vx: rng.gen_range(-20., 20.),
            vy: rng.gen_range(-20., 20.),
            vz: 0.,
            mass: rng.gen_range(1., 20.),
            radius: rng.gen_range(1., 10.),
        });
    }
    App::build()
        .add_resource(Msaa { samples: 4 })
        .add_default_plugins()
        .add_resource(ClearColor(Color::rgb(0.1, 0.1, 0.1)))
        .add_resource(Simulation(GravTree::new(&bodies, 0.02)))
        .add_startup_system(add_bodies.system())
        .add_system(time_step.system())
        .add_system(update_bodies.system())
        .run();
}

fn add_bodies(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    asset_server: Res<AssetServer>,
    grav_tree: Res<Simulation>,
) {
    let texture = asset_server.load("assets/circle.png").unwrap();
    // TODO: texture transparency not working
    commands
        .spawn(Camera2dComponents::default())
        .spawn(UiCameraComponents::default());

    for body in grav_tree.0.as_vec() {
        commands
            .spawn(SpriteComponents {
                mesh: meshes.add(Mesh::from(shape::Quad::new(Vec2::new(
                    (body.radius / 100.) as f32,
                    (body.radius / 100.) as f32,
                )))),
                material: materials.add(texture.into()),
                translation: Translation(Vec3::new(body.x as f32, body.y as f32, body.z as f32)),
                draw: Draw {
                    is_transparent: true,
                    ..Default::default()
                },
                ..Default::default()
            })
            .with(body);
    }
}

fn time_step(mut grav_tree: ResMut<Simulation>) {
    grav_tree.0 = grav_tree.0.time_step();
}

fn update_bodies(grav_tree: Res<Simulation>, mut body_query: Query<(&Entity, &mut Translation)>) {
    let updated_bodies = grav_tree.0.as_vec();
    // for body in updated_bodies.iter() {
    //     println!("body: {}", body.as_string());
    // }

    let mut index = 0;
    for (_, mut translation) in &mut body_query.iter() {
        let updated_body = &updated_bodies[index];
        translation.0.set_x(updated_body.x as f32);
        translation.0.set_y(updated_body.y as f32);
        translation.0.set_z(updated_body.z as f32);
        index += 1;
    }
}
