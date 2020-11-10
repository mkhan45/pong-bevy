use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;

struct Velocity([f32; 2]);
struct Radius(f32);

fn init(mut commands: Commands, mut materials: ResMut<Assets<ColorMaterial>>, mut meshes: ResMut<Assets<Mesh>>) {
    commands
        .spawn(Camera2dComponents::default())
        .spawn(primitive(
                materials.add(Color::rgb(1.0, 1.0, 1.0).into()),
                &mut meshes,
                ShapeType::Circle(50.0),
                TessellationMode::Fill(&FillOptions::default()),
                Vec3::new(0.0, 0.0, 0.0).into()
        ))
        .with(Velocity([125.0, 225.0]))
        .with(Radius(50.0));
}

fn integrate(time: Res<Time>, mut query: Query<(&mut Transform, &Velocity)>) {
    query.iter_mut().for_each(|(mut transform, vel)| {
        *transform.translation.x_mut() += vel.0[0] * time.delta_seconds;
        *transform.translation.y_mut() += vel.0[1] * time.delta_seconds;
    });
}

fn collide(windows: Res<Windows>, mut query: Query<(&mut Velocity, &Transform, &Radius)>) {
    let window = windows.get_primary().unwrap();
    let (width, height) = {
        let (w, h) = (window.width(), window.height());
        (w as f32, h as f32)
    };

    query.iter_mut().for_each(|(mut vel, transform, radius)| {
        let pos = transform.translation;
        if !(-width/2.0 + radius.0 .. width/2.0 - radius.0).contains(&pos.x()) {
            vel.0[0] *= -1.0;
        }
        if !(-height/2.0 + radius.0 .. height/2.0 - radius.0).contains(&pos.y()) {
            vel.0[1] *= -1.0;
        }
    });
}

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app
            .add_system(integrate.system())
            .add_system(collide.system());
    }
}

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_plugin(PhysicsPlugin)
        .add_startup_system(init.system())
        .run();
}
