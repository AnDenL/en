use crate::en::*;

#[system]
fn player_control(world: &mut World) {
    for (_id, (vel,ren ,player)) in world.query_mut::<(&mut Vel,&mut Render, &Player)>() {
        let mut x_move: f32 = 0.0;
        let mut y_move: f32 = 0.0;
        let speed = player.speed;

        if is_key_down(KeyCode::D) { x_move += 1.0; }
        if is_key_down(KeyCode::A) { x_move -= 1.0; }
        if is_key_down(KeyCode::W) { y_move += 1.0; }
        if is_key_down(KeyCode::S) { y_move -= 1.0; }

        let move_dir = vec2(x_move, y_move).normalize_or_zero();

        if x_move > 0.0 {
            ren.flip_x = false;
        } else if x_move < 0.0 {
            ren.flip_x = true;
        }

        vel.x += move_dir.x * speed;
        vel.y += move_dir.y * speed;
    }
}