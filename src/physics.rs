use crate::components::{Pos, Collider, Vel};
use macroquad::prelude::Rect;

pub fn update_physics(world: &mut hecs::World, dt: f32) {
    let colliders: Vec<(hecs::Entity, Rect, bool)> = world
        .query::<(&Pos, &Collider)>()
        .iter()
        .map(|(id, (pos, col))| {
            (id, Rect::new(pos.x - col.w / 2.0, pos.y - col.h / 2.0, col.w, col.h), col.is_static)
        })
        .collect();

    for (_id, (pos, vel)) in world.query_mut::<(&mut Pos, &mut Vel)>() {
        pos.x += vel.x * dt;
        pos.y += vel.y * dt;
        vel.x -= vel.x * vel.d * dt;
        vel.y -= vel.y * vel.d * dt;
    }

    for (id_a, (pos_a, col_a, vel_a)) in world.query_mut::<(&mut Pos, &Collider, &mut Vel)>() {
        if col_a.is_static { continue; }

        let rect_a = Rect::new(pos_a.x - col_a.w / 2.0, pos_a.y - col_a.h / 2.0, col_a.w, col_a.h);

        for (id_b, rect_b, _) in &colliders {
            if id_a == *id_b { continue; }

            if let Some(overlap) = rect_a.intersect(*rect_b) {
                let center_a_x = rect_a.x + rect_a.w / 2.0;
                let center_b_x = rect_b.x + rect_b.w / 2.0;
                let center_a_y = rect_a.y + rect_a.h / 2.0;
                let center_b_y = rect_b.y + rect_b.h / 2.0;

                if overlap.w < overlap.h {
                    let sign = if center_a_x < center_b_x { -1.0 } else { 1.0 };
                    pos_a.x += overlap.w * sign;
                    vel_a.x = 0.0;
                } else {
                    let sign = if center_a_y < center_b_y { -1.0 } else { 1.0 };
                    pos_a.y += overlap.h * sign;
                    vel_a.y = 0.0;
                }
            }
        }
    }
}