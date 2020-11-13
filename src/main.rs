use bevy::prelude::*;
use bevy::sprite::collide_aabb::{collide, Collision};
use bevy_prototype_lyon::prelude::*;

const PADDLE_WIDTH: f32 = 20.0;
const PADDLE_HEIGHT: f32 = 120.0;
const PADDLE_OFFSET: f32 = 240.0;

const PADDLE_VEL: f32 = 480.0;

const BALL_RADIUS: f32 = 20.0;

struct Velocity(Vec2);
struct Radius(f32);

enum PaddleSide {
    Left,
    Right,
}

struct Paddle(PaddleSide);
impl Paddle {
    fn start_pos(side: PaddleSide, window_width: f32) -> Vec2 {
        match side {
            PaddleSide::Left => Vec2::new(-window_width / 2.0 + PADDLE_OFFSET, 0.0),
            PaddleSide::Right => Vec2::new(window_width / 2.0 - PADDLE_OFFSET - PADDLE_WIDTH, 0.0),
        }
    }
}

struct Ball;

fn init(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    windows: Res<Windows>,
) {
    let material = materials.add(Color::rgb(1.0, 1.0, 1.0).into());

    let window = windows.get_primary().unwrap();
    let (width, _height) = {
        let (w, h) = (window.width(), window.height());
        (w as f32, h as f32)
    };

    // camera and ball
    commands
        .spawn(Camera2dComponents::default())
        .spawn(primitive(
            material.clone(),
            &mut meshes,
            ShapeType::Circle(BALL_RADIUS),
            TessellationMode::Fill(&FillOptions::default()),
            Vec3::new(0.0, 0.0, 0.0),
        ))
        .with(Velocity(Vec2::new(125.0, 225.0)))
        .with(Radius(BALL_RADIUS))
        .with(Ball);

    // left paddle
    commands
        .spawn(primitive(
            material.clone(),
            &mut meshes,
            ShapeType::Rectangle {
                width: PADDLE_WIDTH,
                height: PADDLE_HEIGHT,
            },
            TessellationMode::Fill(&FillOptions::default()),
            Paddle::start_pos(PaddleSide::Left, width).extend(0.0),
        ))
        .with(Paddle(PaddleSide::Left))
        .with(Velocity(Vec2::zero()));

    // right paddle
    commands
        .spawn(primitive(
            material,
            &mut meshes,
            ShapeType::Rectangle {
                width: PADDLE_WIDTH,
                height: PADDLE_HEIGHT,
            },
            TessellationMode::Fill(&FillOptions::default()),
            Paddle::start_pos(PaddleSide::Right, width).extend(0.0),
        ))
        .with(Paddle(PaddleSide::Right))
        .with(Velocity(Vec2::zero()));
}

fn integrate(time: Res<Time>, mut query: Query<(&mut Transform, &Velocity)>) {
    query.iter_mut().for_each(|(mut transform, vel)| {
        *transform.translation.x_mut() += vel.0.x() * time.delta_seconds;
        *transform.translation.y_mut() += vel.0.y() * time.delta_seconds;
    });
}

fn wall_collide(windows: Res<Windows>, mut query: Query<(&mut Velocity, &Transform, &Radius)>) {
    let window = windows.get_primary().unwrap();
    let (width, height) = {
        let (w, h) = (window.width(), window.height());
        (w as f32, h as f32)
    };

    query.iter_mut().for_each(|(mut vel, transform, radius)| {
        let pos = transform.translation;
        if (-width / 2.0 >= pos.x() - radius.0 && vel.0.x() < 0.0)
            || (width / 2.0 <= pos.x() + radius.0 && vel.0.x() > 0.0)
        {
            *vel.0.x_mut() *= -1.0;
        } else if (-height / 2.0 >= pos.y() - radius.0 && vel.0.y() < 0.0)
            || (height / 2.0 <= pos.y() + radius.0 && vel.0.y() > 0.0)
        {
            *vel.0.y_mut() *= -1.0;
        }
    });
}

fn paddle_collide(
    mut ball_query: Query<(&Ball, &Transform, &mut Velocity)>,
    paddle_query: Query<(&Paddle, &Transform)>,
) {
    for (_, ball_transform, mut vel) in ball_query.iter_mut() {
        for (_, paddle_transform) in paddle_query.iter() {
            let collision = collide(
                paddle_transform.translation
                    + Vec3::new(PADDLE_WIDTH / 2.0, PADDLE_HEIGHT / 2.0, 0.0),
                Vec2::new(PADDLE_WIDTH, PADDLE_HEIGHT),
                ball_transform.translation,
                Vec2::new(2.0 * BALL_RADIUS, 2.0 * BALL_RADIUS),
            );

            match collision {
                Some(Collision::Left) => {
                    if vel.0.x() < 0.0 {
                        *vel.0.x_mut() *= -1.0;
                    }
                }
                Some(Collision::Right) => {
                    if vel.0.x() > 0.0 {
                        *vel.0.x_mut() *= -1.0;
                    }
                }
                Some(Collision::Top) => {
                    if vel.0.y() > 0.0 {
                        *vel.0.y_mut() *= -1.0;
                    }
                }
                Some(Collision::Bottom) => {
                    if vel.0.y() < 0.0 {
                        *vel.0.y_mut() *= -1.0;
                    }
                }
                None => {}
            }

            if collision.is_some() {
                break;
            }
        }
    }
}

pub struct PhysicsPlugin;
impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system(integrate.system())
            .add_system(wall_collide.system())
            .add_system(paddle_collide.system());
    }
}

fn move_paddles(input: Res<Input<KeyCode>>, mut query: Query<(&Paddle, &mut Velocity)>) {
    let (l_up, l_down) = (input.pressed(KeyCode::W), input.pressed(KeyCode::S));
    let (r_up, r_down) = (input.pressed(KeyCode::Up), input.pressed(KeyCode::Down));

    // kind of ugly
    let l_vel = (l_up as u8 as f32 * PADDLE_VEL) - (l_down as u8 as f32 * PADDLE_VEL);
    let r_vel = (r_up as u8 as f32 * PADDLE_VEL) - (r_down as u8 as f32 * PADDLE_VEL);

    query.iter_mut().for_each(|(paddle, mut vel)| match paddle {
        Paddle(PaddleSide::Left) => *vel.0.y_mut() = l_vel,
        Paddle(PaddleSide::Right) => *vel.0.y_mut() = r_vel,
    })
}

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_plugin(PhysicsPlugin)
        .add_startup_system(init.system())
        .add_system(move_paddles.system())
        .run();
}
