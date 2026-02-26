use inventory;

automod::dir!("src/systems");

#[allow(dead_code)]
pub struct GameSystem {
    pub name: &'static str,
    pub func: fn(&mut SysCtx),
}

inventory::collect!(GameSystem);

pub struct SysCtx<'a> {
    pub world: &'a mut hecs::World,
    pub sprites: &'a mut crate::sprite_manager::SpriteManager,
    pub dt: f32,
}

pub fn run_all_systems(ctx: &mut SysCtx) {
    for system in inventory::iter::<GameSystem> {
        (system.func)(ctx);
    }
}