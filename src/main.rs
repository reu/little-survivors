use bevy::{prelude::*, sprite::Anchor};
use rand::{seq::SliceRandom, thread_rng, Rng};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_systems(Startup, setup)
        .add_systems(Update, keyboard_input_system)
        .add_systems(Update, z_index_system)
        .add_systems(Update, seek_system)
        .add_systems(Update, camera_move_system.after(movement_system))
        .add_systems(
            Update,
            movement_system
                .after(keyboard_input_system)
                .after(seek_system),
        )
        .run();
}

#[derive(Component, Default)]
struct Moveable {
    speed: f32,
}

#[derive(Component, Default)]
struct Velocity(Vec3);

#[derive(Component)]
struct Seek {
    target: Entity,
}

#[derive(Component)]
struct CameraTarget;

fn camera_move_system(
    mut camera: Query<&mut Transform, With<Camera>>,
    targets: Query<&Transform, (With<CameraTarget>, Without<Camera>)>,
) {
    if let Ok(mut camera) = camera.get_single_mut() {
        if let Ok(target) = targets.get_single() {
            camera.translation = camera.translation.lerp(target.translation, 0.01);
        }
    }
}

fn seek_system(
    mut seekers: Query<(&Transform, &Seek, &Moveable, &mut Velocity)>,
    seekables: Query<&Transform>,
) {
    for (me, seek, moveable, mut velocity) in seekers.iter_mut() {
        if let Ok(other) = seekables.get(seek.target) {
            let desired = (other.translation - me.translation).normalize_or_zero();
            let desired = desired * moveable.speed;
            velocity.0 = desired;
        }
    }
}

fn keyboard_input_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut entities: Query<(&Moveable, &mut Velocity)>,
) {
    for (moveable, mut velocity) in entities.iter_mut() {
        velocity.0 = Vec3::default();

        if keyboard.pressed(KeyCode::KeyW) {
            velocity.0.y += moveable.speed;
        }
        if keyboard.pressed(KeyCode::KeyS) {
            velocity.0.y -= moveable.speed;
        }
        if keyboard.pressed(KeyCode::KeyA) {
            velocity.0.x -= moveable.speed;
        }
        if keyboard.pressed(KeyCode::KeyD) {
            velocity.0.x += moveable.speed;
        }
    }
}

fn movement_system(time: Res<Time>, mut entities: Query<(&Velocity, &mut Transform)>) {
    for (velocity, mut transform) in entities.iter_mut() {
        transform.translation += velocity.0 * time.delta_seconds();
        if velocity.0.x > 0.0 {
            transform.scale = Vec3::new(1.0, 1.0, 1.0);
        } else if velocity.0.x < 0.0 {
            transform.scale = Vec3::new(-1.0, 1.0, 1.0);
        }

        if velocity.0.length() > 0.0 {
            let phase = (time.elapsed_seconds() * velocity.0.length() * 0.8).sin();
            transform.rotation.z = phase.remap(-1.0, 1.0, -0.06, 0.06);
            transform.scale.y = phase.remap(-1.0, 1.0, 0.9, 1.1);
        } else {
            transform.scale.y = 1.0;
            transform.rotation.z = 0.0;
        }
    }
}

fn z_index_system(mut entities: Query<&mut Transform, Without<Camera>>) {
    for mut transform in entities.iter_mut() {
        transform.translation.z = -1.0 * transform.translation.y;
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle {
        transform: Transform::from_scale(Vec3::new(0.25, 0.25, 0.25)),
        ..default()
    });

    let mut mage = commands.spawn(SpriteBundle {
        texture: asset_server.load("mage.png"),
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        sprite: Sprite {
            anchor: Anchor::BottomCenter,
            ..default()
        },
        ..default()
    });

    mage.insert(Moveable { speed: 40.0 })
        .insert(Velocity::default())
        .insert(CameraTarget);

    let mage_id = mage.id();

    let mut rng = thread_rng();
    for _ in 0..1000 {
        commands
            .spawn(SpriteBundle {
                texture: asset_server.load(*["orc.png", "knight.png"].choose(&mut rng).unwrap()),
                transform: Transform::from_xyz(
                    rng.gen_range(-200.0..200.0) * 1.5,
                    rng.gen_range(-200.0..200.0) * 1.5,
                    0.0,
                ),
                ..default()
            })
            .insert(Moveable { speed: rng.gen_range(10.0..20.0) })
            .insert(Seek { target: mage_id })
            .insert(Velocity::default());
    }
}
