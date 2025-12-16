use avian3d::PhysicsPlugins;
use avian3d::prelude::{Collider, LockedAxes, RigidBody};
use bevy::color::palettes::css;
use bevy::light::AmbientLight;
use bevy::prelude::*;
use bevy_skein::SkeinPlugin;
use bevy_tnua::TnuaUserControlsSystems;
use bevy_tnua::prelude::{TnuaBuiltinJump, TnuaBuiltinWalk, TnuaController, TnuaControllerPlugin};
use bevy_tnua_avian3d::{TnuaAvian3dPlugin, TnuaAvian3dSensorShape};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            PhysicsPlugins::default(),
            SkeinPlugin::default(),
            TnuaAvian3dPlugin::new(Update),
            TnuaControllerPlugin::new(Update),
        ))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                manage_position.in_set(TnuaUserControlsSystems),
                manage_rotation,
            ),
        )
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 2500.0,
            ..default()
        })
        .insert_resource(GameState { level: 0 })
        .run();
}

#[derive(Component)]
struct CameraArm;

#[derive(Resource)]
struct GameState {
    level: usize,
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    game_state: Res<GameState>,
) {
    match game_state.level {
        0 => {
            commands.spawn(SceneRoot(
                asset_server.load(GltfAssetLabel::Scene(0).from_asset("levels.gltf")),
            ));
        }
        _ => {}
    }

    commands
        .spawn((
            Mesh3d(meshes.add(Capsule3d::new(1.0, 2.0))),
            MeshMaterial3d(materials.add(StandardMaterial::from_color(css::GREEN))),
            Collider::capsule(1.0, 2.0),
            Transform::from_xyz(0.0, 5.0, 0.0),
            TnuaController::default(),
            RigidBody::Dynamic,
            TnuaAvian3dSensorShape(Collider::cylinder(0.24, 0.0)),
            LockedAxes::ROTATION_LOCKED,
        ))
        .with_children(|parent| {
            parent
                .spawn((CameraArm, Transform::from_xyz(0.0, 0.0, 0.0)))
                .with_children(|parent| {
                    parent.spawn((Camera3d::default(), Transform::from_xyz(0.0, 1.0, 10.0)));
                });
        });
}

fn manage_position(
    gamepads: Query<&Gamepad>,
    mut query: Query<(&mut TnuaController, &mut Transform)>,
) -> Result {
    let Ok(gamepad) = gamepads.single() else {
        return Ok(());
    };

    let (mut controller, mut transform) = query.single_mut()?;

    let mut direction = Vec3::ZERO;
    let Some(left_stick_x) = gamepad.get(GamepadAxis::LeftStickX) else {
        return Ok(());
    };
    let Some(left_stick_y) = gamepad.get(GamepadAxis::LeftStickY) else {
        return Ok(());
    };
    if left_stick_x.abs() > 0.01 {
        direction.x = left_stick_x;
    }
    if left_stick_y.abs() > 0.01 {
        direction.z = -left_stick_y;
    }

    let rotated_direction = transform.rotation * direction.normalize_or_zero();
    controller.basis(TnuaBuiltinWalk {
        desired_velocity: rotated_direction * 20.0,
        float_height: 2.0,
        ..default()
    });

    if gamepad.pressed(GamepadButton::South) {
        controller.action(TnuaBuiltinJump {
            height: 15.0,
            ..default()
        });
    }

    if transform.translation.y < -20.0 {
        transform.translation = Vec3::new(0.0, 5.0, 0.0)
    }

    Ok(())
}

fn manage_rotation(
    time: Res<Time>,
    mut player_query: Query<&mut Transform, With<TnuaController>>,
    mut camera_query: Query<&mut Transform, (With<CameraArm>, Without<TnuaController>)>,
    gamepads: Query<&Gamepad>,
) -> Result {
    let Ok(gamepad) = gamepads.single() else {
        return Ok(());
    };

    let mut player_transform = player_query.single_mut()?;
    let mut camera_transform = camera_query.single_mut()?;

    let mut direction = Vec3::ZERO;
    let Some(right_stick_x) = gamepad.get(GamepadAxis::RightStickX) else {
        return Ok(());
    };
    let Some(right_stick_y) = gamepad.get(GamepadAxis::RightStickY) else {
        return Ok(());
    };
    if right_stick_x.abs() > 0.01 {
        direction.z = -right_stick_x;
    }
    if right_stick_x.abs() > 0.01 {
        direction.x = right_stick_y;
    }

    let sensitivity = 5.0;
    player_transform.rotate_y(-right_stick_x * sensitivity * time.delta_secs());
    camera_transform.rotate_x(right_stick_y * sensitivity * time.delta_secs());
    let pitch = camera_transform
        .rotation
        .to_euler(EulerRot::XYZ)
        .0
        .clamp(-1.25, 0.25);
    camera_transform.rotation = Quat::from_euler(EulerRot::XYZ, pitch, 0.0, 0.0);

    Ok(())
}
