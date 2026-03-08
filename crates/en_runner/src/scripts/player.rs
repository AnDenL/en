use en_core::prelude::*;

#[en_component]
pub struct Player{
    pub speed: f32,
}

#[en_system]
fn player_movement_system(
    input: Res<Input>,
    time: Res<Time>,
    mut query: Query<(&mut Transform, &Player)>,
) {
    for (mut transform, player) in query.iter_mut() {
        let mut velocity_x = 0.0;
        let mut velocity_y = 0.0;

        if input.pressed(KeyCode::KeyW) { velocity_y += 1.0; }
        if input.pressed(KeyCode::KeyS) { velocity_y -= 1.0; }
        if input.pressed(KeyCode::KeyA) { velocity_x -= 1.0; }
        if input.pressed(KeyCode::KeyD) { velocity_x += 1.0; }

        transform.x += velocity_x * player.speed * time.delta_time;
        transform.y += velocity_y * player.speed * time.delta_time;
    }
}