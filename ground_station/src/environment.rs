use std::f32::consts::PI;

use bevy::prelude::*;
use proj::Proj;

use crate::data::Data;

pub struct CanSatEnvironmentPlugin;

impl Plugin for CanSatEnvironmentPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_gizmos)
            .init_gizmo_group::<Gizmos3D>();
    }
}

#[derive(Default, Reflect, GizmoConfigGroup)]
struct Gizmos3D;

fn update_gizmos(mut gizmos: Gizmos<Gizmos3D>, data: Res<Data>) {
    gizmos.grid(
        Quat::from_rotation_x(PI / 2.),
        UVec2::splat(100),
        Vec2::new(2., 2.),
        // Light gray
        Srgba::rgba_u8(92, 92, 92, 30),
    );

    let mut coordinates = vec![];
    for (i,_data_point) in data.data_points.iter().enumerate() {
        if let Some(position) = data.get_point_relative_position(i) {
            coordinates.push(position);
        }
    }

    gizmos.linestrip(coordinates, Color::WHITE);
}


