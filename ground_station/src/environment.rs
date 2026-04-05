use std::f32::consts::PI;

use crate::data::Data;
use bevy::{prelude::*, transform::components::Transform};

pub struct CanSatEnvironmentPlugin;

impl Plugin for CanSatEnvironmentPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(Update, update_gizmos)
            .add_systems(Update, update_cansat_model)
            .init_gizmo_group::<Gizmos3D>();
    }
}

#[derive(Default, Reflect, GizmoConfigGroup)]
struct Gizmos3D;

#[derive(Component)]
struct CanSatModel;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let model = meshes.add(Cylinder::default());
    let material = materials.add(StandardMaterial::default());
    commands.spawn((
        Name::new("CanSat model"),
        CanSatModel,
        Mesh3d(model),
        MeshMaterial3d(material),
    ));
    commands.spawn(DirectionalLight::default());
}

fn update_cansat_model(mut model: Single<&mut Transform, With<CanSatModel>>, data: Res<Data>) {
    let Some((_time, position)) = data.data_points.position.iter().filter(|f|*f.0 < data.current_time).min_by_key(|p| p.0.abs_diff(data.current_time)) else {
        return;
    };
    let position = data.get_point_relative_position(position);
    model.translation = position;
    
}

fn update_gizmos(mut gizmos: Gizmos<Gizmos3D>, data: Res<Data>) {
    gizmos.grid(
        Quat::from_rotation_x(PI / 2.),
        UVec2::splat(100),
        Vec2::new(2., 2.),
        // Light gray
        Srgba::rgba_u8(92, 92, 92, 30),
    );

    let mut coordinates = vec![];
    for (_time, position) in data.data_points.position.iter() {
            coordinates.push(data.get_point_relative_position(position));
    }

    gizmos.linestrip(coordinates, Color::BLACK);
}
