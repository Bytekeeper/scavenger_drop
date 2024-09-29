use macroquad::prelude::*;
use slotmap::{new_key_type, DefaultKey, SlotMap};

pub const GRAVITY: Vec2 = vec2(0.0, 6.0 / 60.0);

#[derive(Copy, Clone)]
pub struct Collider {
    pub position: Vec2,
    pub dimension: Vec2,
    pub flags: u8,
}

impl Collider {
    pub fn as_rect(&self) -> Rect {
        Rect::new(
            self.position.x,
            self.position.y,
            self.dimension.x,
            self.dimension.y,
        )
    }
}

pub struct Particle {
    pub position: Vec2,
    pub last_position: Vec2,
    pub velocity: Vec2,
    pub life_time_steps: u32,
}

new_key_type! {
    pub struct Actor;
    pub struct Solid;
}

#[derive(Default)]
pub struct World {
    actors: SlotMap<Actor, Collider>,
    solids: SlotMap<Solid, Collider>,
    particles: SlotMap<DefaultKey, Particle>,
}

impl World {
    pub fn new() -> World {
        World::default()
    }

    pub fn add_particle(&mut self, position: Vec2, velocity: Vec2) {
        self.particles.insert(Particle {
            position,
            last_position: position,
            velocity,
            life_time_steps: 30,
        });
    }

    pub fn particles(&self) -> impl Iterator<Item = &Particle> {
        self.particles.values()
    }

    pub fn step_particles(&mut self) {
        for particle in self.particles.values_mut() {
            particle.velocity += GRAVITY;
            particle.last_position = particle.position;
            particle.position += particle.velocity;
            particle.life_time_steps -= 1;
        }
        self.particles
            .retain(|_, particle| particle.life_time_steps > 0);
    }

    pub fn add_actor(&mut self, position: Vec2, dimension: Vec2) -> Actor {
        self.actors.insert(Collider {
            position,
            dimension,
            flags: 0,
        })
    }

    pub fn set_actor_pos(&mut self, actor: Actor, position: Vec2) {
        self.actors[actor].position = position;
    }

    pub fn actor_pos(&self, actor: Actor) -> Vec2 {
        self.actors[actor].position
    }

    pub fn add_solid(&mut self, position: Vec2, dimension: Vec2, flags: u8) -> Solid {
        self.solids.insert(Collider {
            position,
            dimension,
            flags,
        })
    }

    pub fn solid_collider(&self, solid: Solid) -> Collider {
        self.solids[solid]
    }

    pub fn has_flag(&self, solid: Solid, flag: u8) -> bool {
        self.solids[solid].flags & flag != 0
    }

    pub fn solid_pos(&self, solid: Solid) -> Vec2 {
        self.solids[solid].position
    }

    pub fn solid_move(&mut self, solid: Solid, delta: Vec2) {
        let collider = &mut self.solids[solid];
        let my_rect = collider.as_rect();
        for actor_collider in self.actors.values_mut() {
            let actor_rect = actor_collider.as_rect();
            if actor_rect.overlaps(&my_rect) {
                actor_collider.position += delta;
            }
        }
        collider.position += delta;
    }

    pub fn move_v(&mut self, actor: Actor, dy: f32) -> Option<Solid> {
        let collider = &mut self.actors[actor];
        let mut actor_rect = collider.as_rect();
        actor_rect.x += 0.05;
        actor_rect.y += 0.05;
        actor_rect.w -= 0.1;
        actor_rect.h += dy.abs() - 0.1;
        if dy < 0.0 {
            actor_rect.y += dy;
        }
        for (solid, solid_collider) in self.solids.iter_mut() {
            let solid_rect = solid_collider.as_rect();
            if let Some(intersection) = solid_rect.intersect(actor_rect) {
                if dy > 0.0 {
                    collider.position.y = intersection.y - collider.dimension.y;
                } else {
                    collider.position.y = intersection.top();
                }
                return Some(solid);
            }
        }
        collider.position.y += dy;
        None
    }

    pub fn move_h(&mut self, actor: Actor, dx: f32) -> Option<Solid> {
        let collider = &mut self.actors[actor];
        let mut actor_rect = collider.as_rect();
        actor_rect.x += 0.05;
        actor_rect.y += 0.05;
        actor_rect.w += dx.abs() - 0.1;
        actor_rect.h -= 0.1;
        if dx < 0.0 {
            actor_rect.x += dx;
        }
        for (solid, solid_collider) in self.solids.iter_mut() {
            let solid_rect = solid_collider.as_rect();
            if let Some(intersection) = solid_rect.intersect(actor_rect) {
                if dx > 0.0 {
                    collider.position.x = intersection.x - collider.dimension.x;
                } else {
                    collider.position.x = intersection.right();
                }
                return Some(solid);
            }
        }
        collider.position.x += dx;
        None
    }

    pub fn collide_solids(&self, position: Vec2, dimension: Vec2) -> Option<(Solid, Rect)> {
        let rect = Rect::new(position.x, position.y, dimension.x, dimension.y);
        self.solids
            .iter()
            .filter_map(|(solid, solid_collider)| {
                (solid_collider.as_rect().intersect(rect)).map(|rect| (solid, rect))
            })
            .next()
    }
}
