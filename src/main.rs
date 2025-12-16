use avian3d::PhysicsPlugins;
use avian3d::prelude::{Collider, LockedAxes, RigidBody};
use bevy::color::palettes::css;
use bevy::light::{AmbientLight, DirectionalLightShadowMap};
use bevy::prelude::*;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
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
            PanOrbitCameraPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                manage_position.in_set(TnuaUserControlsSystems),
                manage_rotation,
                reset_camera,
            ),
        )
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 2500.0,
            ..default()
        })
        .insert_resource(DirectionalLightShadowMap { size: 2048 })
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

    // Ground
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(500.0, 500.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.3, 0.5, 0.3))),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    commands.spawn((
        Mesh3d(meshes.add(Capsule3d::new(1.0, 2.0))),
        MeshMaterial3d(materials.add(StandardMaterial::from_color(css::GREEN))),
        Collider::capsule(1.0, 2.0),
        Transform::from_xyz(0.0, 5.0, 0.0),
        TnuaController::default(),
        RigidBody::Dynamic,
        TnuaAvian3dSensorShape(Collider::cylinder(0.24, 0.0)),
        LockedAxes::ROTATION_LOCKED,
        children![(
            CameraArm,
            Transform::from_xyz(0.0, 0.0, 0.0),
            children![(
                Transform::from_xyz(0.0, 1.0, 20.0),
                PanOrbitCamera {
                    enabled: false,
                    ..default()
                },
            )]
        )],
    ));

    commands.spawn((
        DirectionalLight {
            illuminance: light_consts::lux::AMBIENT_DAYLIGHT,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(5.0, 70.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn manage_position(
    gamepads: Query<&Gamepad>,
    mut controller_q: Query<(&mut TnuaController, &mut Transform)>,
) -> Result {
    let Ok(gamepad) = gamepads.single() else {
        return Ok(());
    };

    let (mut controller, mut transform) = controller_q.single_mut()?;

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
    mut player_q: Query<&mut Transform, With<TnuaController>>,
    mut arm_q: Query<&mut Transform, (With<CameraArm>, Without<TnuaController>)>,
    mut camera_q: Query<&mut PanOrbitCamera>,
    gamepads: Query<&Gamepad>,
) -> Result {
    let Ok(gamepad) = gamepads.single() else {
        return Ok(());
    };

    let mut player_transform = player_q.single_mut()?;
    let mut arm_transform = arm_q.single_mut()?;

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
    arm_transform.rotate_x(right_stick_y * sensitivity * time.delta_secs());
    let pitch = arm_transform
        .rotation
        .to_euler(EulerRot::XYZ)
        .0
        .clamp(-1.25, 0.25);
    arm_transform.rotation = Quat::from_euler(EulerRot::XYZ, pitch, 0.0, 0.0);

    let sensitivity = 1.2;
    if let Some(right_z) = gamepad.analog().get(GamepadAxis::RightZ)
        && right_z > 0.
    {
        let mut camera = camera_q.single_mut()?;
        camera.target_radius += sensitivity;
    }
    if let Some(left_z) = gamepad.analog().get(GamepadAxis::LeftZ)
        && left_z > 0.
    {
        let mut camera = camera_q.single_mut()?;
        camera.target_radius -= sensitivity;
    }

    Ok(())
}

fn reset_camera(
    mut camera: ParamSet<(
        Query<&mut Transform, With<CameraArm>>,
        Query<(&mut Transform, &mut PanOrbitCamera)>,
    )>,
    gamepads: Query<&Gamepad>,
) -> Result {
    let Ok(gamepad) = gamepads.single() else {
        return Ok(());
    };
    if gamepad.just_pressed(GamepadButton::RightThumb) {
        let mut arm_q = camera.p0();
        let mut arm_transform = arm_q.single_mut()?;
        *arm_transform = Transform::from_xyz(0.0, 0.0, 0.0);

        let mut camera_q = camera.p1();
        let (mut camera_transform, mut camera) = camera_q.single_mut()?;
        *camera_transform = Transform::from_xyz(0.0, 1.0, 20.0);
        *camera = PanOrbitCamera {
            enabled: false,
            ..default()
        };
    }
    Ok(())
}
