mod physics;

use macroquad::audio::*;
use macroquad::prelude::*;
use macroquad::rand::*;
use physics::*;
use std::ops::RangeInclusive;

const NO_SLIDE: u8 = 1;
const GROUND_LEVEL: u8 = 2;
const DEADLY: u8 = 4;
const COIN: u8 = 8;
const NOT_TAKEN: u8 = 16;

enum ScavengerAnim {
    Idle,
    Run,
    Fall,
}

impl ScavengerAnim {
    fn frames(&self) -> RangeInclusive<i32> {
        match self {
            Self::Idle => 1..=7,
            Self::Run => 8..=15,
            Self::Fall => 20..=23,
        }
    }
}

struct Platform {
    solid: Solid,
    initial_position: Vec2,
    move_sequence: Vec<PlatformMove>,
    move_index: usize,
    move_timer: i32,
}

enum PlatformMove {
    ToTarget { target: Vec2, steps: i32 },
    Pause { steps: i32 },
}

impl Platform {
    fn new(world: &mut World, pos: Vec2, size: Vec2, flags: u8) -> Self {
        let solid = world.add_solid(pos, size, flags);
        Platform {
            solid,
            initial_position: pos,
            move_sequence: Vec::new(),
            move_index: 0,
            move_timer: 0,
        }
    }

    fn reset(&mut self, world: &mut World) {
        self.move_index = 0;
        self.move_timer = 0;
        world.set_solid_pos(self.solid, self.initial_position);
    }

    fn then_moving(self, target: Vec2, steps: i32) -> Self {
        let mut move_sequence = self.move_sequence;
        move_sequence.push(PlatformMove::ToTarget { target, steps });
        Platform {
            move_sequence,
            ..self
        }
    }

    fn then_pausing(self, steps: i32) -> Self {
        let mut move_sequence = self.move_sequence;
        move_sequence.push(PlatformMove::Pause { steps });
        Platform {
            move_sequence,
            ..self
        }
    }
}

fn sfx(sound: &Sound) {
    play_sound(
        sound,
        PlaySoundParams {
            looped: false,
            volume: 0.3,
        },
    );
}

fn down(value: &mut f32, amount: f32) -> f32 {
    *value += amount;
    *value
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Scavenger Drop".to_owned(),
        window_width: 1024,
        window_height: 800,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let snd_pickup = load_sound_from_bytes(include_bytes!("../assets/pickupCoin.wav"))
        .await
        .unwrap();
    let snd_land = load_sound_from_bytes(include_bytes!("../assets/land.wav"))
        .await
        .unwrap();
    let snd_die = load_sound_from_bytes(include_bytes!("../assets/explosion.wav"))
        .await
        .unwrap();
    let snd_step = load_sound_from_bytes(include_bytes!("../assets/step.wav"))
        .await
        .unwrap();
    let snd_wise_crack = load_sound_from_bytes(include_bytes!("../assets/theobstacleistheway.ogg"))
        .await
        .unwrap();
    //let snd_jump = load_sound_from_bytes(include_bytes!("../assets/jump.wav"))
    //    .await
    //    .unwrap();
    let onebit = Texture2D::from_file_with_format(
        include_bytes!("../assets/kenney/Tilemap/monochrome_tilemap_transparent_packed.png"),
        None,
    );
    onebit.set_filter(FilterMode::Nearest);
    let scavenger = Texture2D::from_file_with_format(
        include_bytes!("../assets/02_Bounty_hunter_sprites.png"),
        None,
    );
    scavenger.set_filter(FilterMode::Nearest);

    let mut world = World::new();
    //let start_pos = vec2(200.0, 10000.0);
    let start_pos = Vec2::ZERO;
    let player = world.add_actor(start_pos, vec2(32.0, 32.0), 0);

    // The "Level"
    let level = &mut 32.0;
    let mut platforms = vec![
        Platform::new(&mut world, vec2(-50.0, *level), vec2(100.0, 64.0), NO_SLIDE),
        // Possible drop example
        Platform::new(
            &mut world,
            vec2(250.0, down(level, 317.0)),
            vec2(200.0, 32.0),
            NO_SLIDE,
        ),
        // Death drop example
        Platform::new(
            &mut world,
            vec2(-300.0, down(level, 10.0)),
            vec2(200.0, 32.0),
            NO_SLIDE,
        ),
        // Slide example
        Platform::new(
            &mut world,
            vec2(100.0, down(level, 40.0)),
            vec2(32.0, 300.0),
            0,
        ),
        // No slide example
        Platform::new(&mut world, vec2(550.0, *level), vec2(32.0, 300.0), NO_SLIDE),
        Platform::new(
            &mut world,
            vec2(200.0, down(level, 270.0)),
            vec2(352.0, 32.0),
            NO_SLIDE,
        ),
        // Easy steps
        Platform::new(
            &mut world,
            vec2(130.0, down(level, 130.0)),
            vec2(96.0, 32.0),
            NO_SLIDE,
        ),
        Platform::new(
            &mut world,
            vec2(-70.0, down(level, 100.0)),
            vec2(96.0, 32.0),
            NO_SLIDE,
        ),
        Platform::new(
            &mut world,
            vec2(-270.0, down(level, 100.0)),
            vec2(96.0, 32.0),
            NO_SLIDE,
        ),
        Platform::new(
            &mut world,
            vec2(-470.0, down(level, 100.0)),
            vec2(96.0, 32.0),
            NO_SLIDE,
        )
        .then_moving(vec2(300.0, *level), 300)
        .then_moving(vec2(-470.0, *level), 200),
        Platform::new(
            &mut world,
            vec2(450.0, down(level, 100.0)),
            vec2(32.0, 200.0),
            0,
        ),
        Platform::new(
            &mut world,
            vec2(150.0, down(level, 400.0)),
            vec2(128.0, 128.0),
            0,
        ),
        // A few back and forth jumps
        Platform::new(
            &mut world,
            vec2(0.0, down(level, 200.0)),
            vec2(32.0, 192.0),
            0,
        ),
        Platform::new(
            &mut world,
            vec2(200.0, down(level, 200.0)),
            vec2(32.0, 600.0),
            0,
        ),
        Platform::new(
            &mut world,
            vec2(16.0, down(level, 500.0)),
            vec2(32.0, 192.0),
            0,
        ),
        Platform::new(
            &mut world,
            vec2(-50.0, down(level, 500.0)),
            vec2(192.0, 28.0),
            0,
        ),
        Platform::new(&mut world, vec2(142.0, *level), vec2(64.0, 28.0), NO_SLIDE)
            .then_pausing(180)
            .then_moving(vec2(600.0, *level), 300)
            .then_pausing(180)
            .then_moving(vec2(142.0, *level), 300),
        Platform::new(
            &mut world,
            vec2(500.0, down(level, 100.0)),
            vec2(32.0, 192.0),
            0,
        ),
        Platform::new(
            &mut world,
            vec2(600.0, down(level, 300.0)),
            vec2(192.0, 28.0),
            0,
        ),
        Platform::new(
            &mut world,
            vec2(800.0, down(level, 200.0)),
            vec2(96.0, 28.0),
            NO_SLIDE,
        )
        .then_pausing(180)
        .then_moving(vec2(-400.0, *level), 200)
        .then_moving(vec2(-700.0, *level - 800.0), 200)
        .then_pausing(180)
        .then_moving(vec2(-400.0, *level), 100)
        .then_moving(vec2(800.0, *level), 100),
        Platform::new(
            &mut world,
            vec2(-732.0, down(level, -800.0)),
            vec2(32.0, 192.0),
            NO_SLIDE,
        ),
        Platform::new(
            &mut world,
            vec2(-968.0, down(level, 300.0)),
            vec2(128.0, 32.0),
            NO_SLIDE,
        ),
        Platform::new(&mut world, vec2(-732.0, *level), vec2(32.0, 384.0), 0),
        Platform::new(
            &mut world,
            vec2(-792.0, down(level, 368.0)),
            vec2(64.0, 16.0),
            DEADLY,
        ),
        Platform::new(&mut world, vec2(-700.0, *level), vec2(64.0, 16.0), DEADLY),
        Platform::new(&mut world, vec2(-968.0, *level), vec2(32.0, 384.0), 0),
        Platform::new(
            &mut world,
            vec2(-968.0, down(level, 600.0)),
            vec2(128.0, 32.0),
            0,
        ),
        Platform::new(
            &mut world,
            vec2(-840.0, down(level, 240.0)),
            vec2(128.0, 32.0),
            0,
        )
        .then_moving(vec2(-840.0, *level + 300.0), 200)
        .then_moving(vec2(-840.0, *level), 200),
        Platform::new(
            &mut world,
            vec2(-580.0, down(level, 640.0)),
            vec2(128.0, 32.0),
            0,
        )
        .then_moving(vec2(-580.0, *level - 200.0), 200)
        .then_moving(vec2(-580.0, *level), 200),
        Platform::new(
            &mut world,
            vec2(-320.0, down(level, 200.0)),
            vec2(128.0, 32.0),
            0,
        )
        .then_moving(vec2(-320.0, *level - 600.0), 200)
        .then_moving(vec2(-320.0, *level), 200),
        Platform::new(
            &mut world,
            vec2(-192.0, down(level, -400.0)),
            vec2(128.0, 32.0),
            NO_SLIDE,
        ),
        Platform::new(
            &mut world,
            vec2(192.0, down(level, 300.0)),
            vec2(64.0, 32.0),
            NO_SLIDE,
        ),
        Platform::new(
            &mut world,
            vec2(256.0, down(level, -600.0)),
            vec2(64.0, 32.0),
            NO_SLIDE,
        )
        .then_pausing(60)
        .then_moving(vec2(256.0, *level + 600.0), 200)
        .then_pausing(60)
        .then_moving(vec2(256.0, *level), 200),
        Platform::new(
            &mut world,
            vec2(320.0, down(level, 300.0)),
            vec2(64.0, 32.0),
            NO_SLIDE,
        ),
        Platform::new(
            &mut world,
            vec2(600.0, down(level, 300.0)),
            vec2(32.0, 32.0),
            NO_SLIDE,
        ),
        Platform::new(
            &mut world,
            vec2(900.0, down(level, 300.0)),
            vec2(32.0, 32.0),
            NO_SLIDE,
        ),
        Platform::new(
            &mut world,
            vec2(500.0, down(level, 300.0)),
            vec2(32.0, 32.0),
            NO_SLIDE,
        ),
        Platform::new(
            &mut world,
            vec2(200.0, *level - 200.0),
            vec2(256.0, 30.0),
            NO_SLIDE,
        ),
        Platform::new(
            &mut world,
            vec2(184.0, down(level, 300.0)),
            vec2(64.0, 16.0),
            DEADLY,
        ),
        Platform::new(
            &mut world,
            vec2(200.0, down(level, 16.0)),
            vec2(32.0, 1700.0),
            0,
        ),
        Platform::new(
            &mut world,
            vec2(484.0, down(level, 200.0)),
            vec2(64.0, 16.0),
            DEADLY,
        ),
        Platform::new(
            &mut world,
            vec2(500.0, down(level, 16.0)),
            vec2(32.0, 1700.0),
            0,
        ),
        Platform::new(
            &mut world,
            vec2(232.0, *level + 300.0),
            vec2(64.0, 16.0),
            DEADLY,
        )
        .then_moving(vec2(232.0, *level), 200)
        .then_moving(vec2(232.0, *level + 300.0), 200),
        Platform::new(
            &mut world,
            vec2(436.0, down(level, 300.0)),
            vec2(64.0, 16.0),
            DEADLY,
        )
        .then_moving(vec2(436.0, *level + 300.0), 200)
        .then_moving(vec2(436.0, *level), 200),
        Platform::new(
            &mut world,
            vec2(232.0, down(level, 300.0)),
            vec2(64.0, 16.0),
            DEADLY,
        )
        .then_moving(vec2(232.0, *level), 200)
        .then_moving(vec2(232.0, *level + 300.0), 200),
        Platform::new(
            &mut world,
            vec2(436.0, down(level, 300.0)),
            vec2(64.0, 16.0),
            DEADLY,
        )
        .then_moving(vec2(436.0, *level + 300.0), 200)
        .then_moving(vec2(436.0, *level), 200),
        Platform::new(
            &mut world,
            vec2(232.0, down(level, 300.0)),
            vec2(32.0, 16.0),
            DEADLY,
        ),
        Platform::new(
            &mut world,
            vec2(232.0, down(level, 268.0)),
            vec2(96.0, 16.0),
            NO_SLIDE,
        ),
        Platform::new(&mut world, vec2(404.0, *level), vec2(96.0, 16.0), NO_SLIDE),
        Platform::new(
            &mut world,
            vec2(0.0, down(level, 400.0)),
            vec2(256.0, 16.0),
            NO_SLIDE,
        ),
        Platform::new(&mut world, vec2(320.0, *level), vec2(256.0, 16.0), NO_SLIDE),
        Platform::new(
            &mut world,
            vec2(-32.0, down(level, 200.0)),
            vec2(256.0, 16.0),
            NO_SLIDE,
        ),
        Platform::new(&mut world, vec2(288.0, *level), vec2(256.0, 16.0), NO_SLIDE),
        Platform::new(
            &mut world,
            vec2(-16.0, down(level, 250.0)),
            vec2(256.0, 16.0),
            NO_SLIDE,
        ),
        Platform::new(&mut world, vec2(306.0, *level), vec2(256.0, 16.0), NO_SLIDE),
        Platform::new(
            &mut world,
            vec2(350.0, down(level, 200.0)),
            vec2(32.0, 1000.0),
            0,
        ),
        // DUMMY
        Platform::new(
            &mut world,
            vec2(256.0, down(level, 1000.0)),
            vec2(300.0, 32.0),
            NO_SLIDE,
        ),
        // GROUND
        Platform::new(
            &mut world,
            vec2(-1000.0, down(level, 300.0)),
            vec2(2000.0, 32.0),
            GROUND_LEVEL | NO_SLIDE,
        ),
        Platform::new(&mut world, vec2(-1000.0, 0.0), vec2(32.0, *level), NO_SLIDE),
        Platform::new(&mut world, vec2(1000.0, 0.0), vec2(32.0, *level), NO_SLIDE),
    ];
    for platform in platforms.iter() {
        if world.solid_has_flag(platform.solid, DEADLY) {
            continue;
        }
        if gen_range(0.0, 1.0) > 0.7 {
            let collider = world.solid_collider(platform.solid);
            if collider.dimension.x >= 32.0 {
                let x = gen_range(16.0, collider.dimension.x - 16.0) + collider.position.x - 16.0;
                world.add_actor(
                    vec2(x, collider.position.y - 32.0),
                    vec2(32.0, 32.0),
                    COIN | NOT_TAKEN,
                );
            }
        }
    }
    let mut dx = 0.0;
    let mut dy = 1.0;
    let mut delta = 1.0 / 60.0;
    let mut camera_target = Vec2::ZERO;
    let mut s_anim_index = 0;
    let mut s_anim_time = 8;
    let mut s_anim;
    let mut timer = 0;
    let mut coins = 0;
    let mut game_ended = false;
    loop {
        clear_background(BLACK);

        world.step_particles();
        let mut pos = world.actor_pos(player);
        delta += get_frame_time();
        while delta > 0.9 / 60.0 {
            delta -= 1.0 / 60.0;
            let (_, coin_candidate) = world.move_h(player, dx);
            if let Some(coin_candidate) = coin_candidate {
                if world.actor_has_flag(coin_candidate, COIN | NOT_TAKEN) {
                    world.actor_unset_flag(coin_candidate, NOT_TAKEN);
                    coins += 1;
                    sfx(&snd_pickup);
                }
            }
            let (floor, coin_candidate) = world.move_v(player, dy);
            if let Some(coin_candidate) = coin_candidate {
                if world.actor_has_flag(coin_candidate, COIN | NOT_TAKEN) {
                    world.actor_unset_flag(coin_candidate, NOT_TAKEN);
                    coins += 1;
                    sfx(&snd_pickup);
                }
            }
            pos = world.actor_pos(player);
            let mut control = 0.5;
            if let Some(floor) = floor {
                if dy > 8.0 || world.solid_has_flag(floor, DEADLY) {
                    dx = 0.0;
                    world.set_actor_pos(player, start_pos);
                    for platform in platforms.iter_mut() {
                        platform.reset(&mut world);
                    }
                    let coin_actors: Vec<_> = world
                        .actors()
                        .filter(|(_, collider)| collider.flags & COIN != 0)
                        .map(|(coin, _)| coin)
                        .collect();
                    for coin in coin_actors {
                        world.actor_set_flag(coin, NOT_TAKEN);
                    }
                    timer = 0;
                    coins = 0;
                    sfx(&snd_die);
                } else if dy > 6.0 / 60.0 {
                    sfx(&snd_land);
                    for _ in 0..20 {
                        world.add_particle(
                            pos + vec2(16.0, 32.0),
                            vec2(gen_range(-2.0, 2.0), gen_range(-2.0, 0.0)),
                        );
                    }
                } else if !world.solid_has_flag(floor, GROUND_LEVEL) {
                    if pos != Vec2::ZERO {
                        timer += 1;
                    }
                } else {
                    if !game_ended {
                        sfx(&snd_wise_crack);
                    }
                    game_ended = true;
                    for _ in 0..3 {
                        world.add_particle(
                            pos + vec2(16.0, 32.0),
                            vec2(gen_range(-10.0, 10.0), gen_range(-10.0, 0.0)),
                        );
                    }
                }
                dy = 0.0;
                control = 1.0;
            } else {
                timer += 1;
            }
            let wall = floor
                .is_none()
                .then_some(
                    world
                        .collide_solids(pos + vec2(0.0, 0.05), vec2(32.0, 32.0 - 0.1))
                        .filter(|(solid, _)| !world.solid_has_flag(*solid, NO_SLIDE)),
                )
                .flatten();

            if is_key_down(KeyCode::Right)
                || is_key_down(KeyCode::D)
                || mouse_position_local().x > 0.0 && is_mouse_button_down(MouseButton::Left)
            {
                if wall.is_some() {
                    dx = 5.0;
                    //sfx(&snd_jump);
                } else {
                    dx = (dx + control * 8.0 / 60.0).min(5.0);
                }
            } else if is_key_down(KeyCode::Left)
                || is_key_down(KeyCode::A)
                || mouse_position_local().x < 0.0 && is_mouse_button_down(MouseButton::Left)
            {
                if wall.is_some() {
                    dx = -5.0;
                    //sfx(&snd_jump);
                } else {
                    dx = (dx - control * 8.0 / 60.0).max(-5.0);
                }
            } else if dx > 0.0 {
                dx = (dx - control * 16.0 / 60.0).max(0.0);
            } else if dx < 0.0 {
                dx = (dx + control * 16.0 / 60.0).min(0.0);
            }

            if let Some((_, rect)) = wall {
                if dy > 0.0 {
                    dy = (dy - 32.0 / 60.0).max(2.0);
                } else {
                    dy = (dy + 32.0 / 60.0).min(2.0);
                }
                let dv = vec2(gen_range(0.0, 2.0), 0.0);
                let wall_pos = vec2(rect.x, gen_range(rect.top(), rect.bottom()));
                if pos.x > wall_pos.x {
                    world.add_particle(wall_pos, dv);
                } else {
                    world.add_particle(wall_pos, -dv);
                }
            }

            if dy != 0.0 {
                s_anim = ScavengerAnim::Fall;
            } else if dx != 0.0 {
                s_anim = ScavengerAnim::Run;
            } else {
                s_anim = ScavengerAnim::Idle;
            }

            dy += GRAVITY.y;

            for Platform {
                solid,
                move_sequence,
                move_index,
                move_timer,
                ..
            } in platforms.iter_mut()
            {
                if let Some(current) = move_sequence.get(*move_index) {
                    *move_timer += 1;
                    let anim_steps = match current {
                        PlatformMove::ToTarget { target, steps } => {
                            let position = world.solid_pos(*solid);
                            let delta = (*target - position) / (*steps - *move_timer + 1) as f32;
                            world.solid_move(*solid, delta);
                            *steps
                        }
                        PlatformMove::Pause { steps } => *steps,
                    };
                    if *move_timer >= anim_steps {
                        *move_timer = 0;
                        *move_index = (*move_index + 1) % move_sequence.len();
                    }
                }
            }

            s_anim_time -= 1;
            if s_anim_time <= 0 {
                s_anim_time = 8;
                s_anim_index += 1;
            }
            if !s_anim.frames().contains(&s_anim_index) {
                s_anim_index = *s_anim.frames().start();
            }
            if s_anim_time == 8 && s_anim_index % 4 == 0 && matches!(s_anim, ScavengerAnim::Run) {
                sfx(&snd_step);
            }
        }

        let mut camera =
            Camera2D::from_display_rect(Rect::new(0.0, 0.0, screen_width(), screen_height()));
        camera.zoom *= vec2(1.0, -1.0);
        let camera_delta =
            (pos + vec2(16.0, 16.0) + Vec2::Y * 128.0 - camera_target).clamp_length_max(2000.0);
        if camera_delta.length_squared() > 10.0 {
            camera_target += camera_delta.clamp_length_min(50.0) * get_frame_time();
        } else if camera_delta.length() > 300.0 {
            //camera_target = pos + vec2(16.0, 16.0);
            camera_target += camera_delta / camera_delta.length() * 400.0;
        }
        camera.target = camera_target;
        set_camera(&camera);
        //draw_rectangle(pos.x, pos.y, 32.0, 32.0, RED);
        draw_texture_ex(
            &scavenger,
            pos.x - 16.0,
            pos.y - 28.0,
            WHITE,
            DrawTextureParams {
                source: Some(Rect::new(s_anim_index as f32 * 34.0 + 1.0, 0.0, 32.0, 30.0)),
                dest_size: Some(vec2(64.0, 60.0)),
                flip_x: dx < 0.0,
                ..Default::default()
            },
        );

        for Platform { solid, .. } in platforms.iter() {
            let Collider {
                position: Vec2 { x, y },
                dimension,
                flags,
            } = world.solid_collider(*solid);
            let tl = if flags & DEADLY != 0 {
                vec2(48.0, 144.0)
            } else if flags & NO_SLIDE != 0 {
                vec2(240.0, 80.0)
            } else {
                vec2(240.0, 144.0)
            };
            let rows = (dimension.y / 16.0) as i32;
            let cols = (dimension.x / 16.0) as i32;
            for r in 0..rows {
                let dy = if r == 0 || flags & DEADLY != 0 {
                    0.0
                } else if r == rows - 1 {
                    32.0
                } else {
                    16.0
                };
                for c in 0..cols {
                    let dx = if c == 0 || flags & DEADLY != 0 {
                        0.0
                    } else if c == cols - 1 {
                        32.0
                    } else {
                        16.0
                    };
                    if dx == 16.0 && dy == 16.0 && ((r * 73856093) ^ (c * 19349663)) & 0xF < 14 {
                        continue;
                    }
                    draw_texture_ex(
                        &onebit,
                        x + c as f32 * 16.0,
                        y + r as f32 * 16.0,
                        WHITE,
                        DrawTextureParams {
                            source: Some(Rect::new(tl.x + dx, tl.y + dy, 16.0, 16.0)),
                            ..Default::default()
                        },
                    );
                }
            }
        }

        for (_, coin_collider) in world.actors() {
            if coin_collider.flags & (COIN | NOT_TAKEN) != COIN | NOT_TAKEN {
                continue;
            }
            draw_texture_ex(
                &onebit,
                coin_collider.position.x,
                coin_collider.position.y,
                WHITE,
                DrawTextureParams {
                    source: Some(Rect::new(32.0, 64.0, 16.0, 16.0)),
                    dest_size: Some(vec2(32.0, 32.0)),
                    ..Default::default()
                },
            );
        }

        draw_texture_ex(
            &onebit,
            0.0,
            0.0,
            WHITE,
            DrawTextureParams {
                source: Some(Rect::new(32.0, 80.0, 16.0, 16.0)),
                ..Default::default()
            },
        );

        // In-game tutorial, mostly sorted top to bottom
        draw_text(
            "Level graphics: 1-Bit Platformer Pack by Kenney (kenney.nl)",
            -100.0,
            1000.0,
            30.0,
            WHITE,
        );
        draw_text(
            "Character graphics: Chriss Ulysseo (modified by Bytekeeper)",
            -100.0,
            1500.0,
            30.0,
            WHITE,
        );
        draw_text(
            "How did I end up on this tower? I need to get down.",
            -300.0,
            150.0,
            30.0,
            WHITE,
        );
        draw_text(
            "Surely, it's owner wouldn't mind me 'cleaning' up a bit.",
            -320.0,
            180.0,
            30.0,
            WHITE,
        );

        draw_text("My trusty old soul-stone.", 0.0, -52.0, 24.0, WHITE);
        draw_text(
            "Should anything happen to me, I will be returned here.",
            -50.0,
            -26.0,
            24.0,
            WHITE,
        );

        draw_text("This, I can barely reach.", 250.0, 330.0, 24.0, WHITE);

        draw_text(
            "This is too far of a drop for me.",
            -350.0,
            320.0,
            24.0,
            WHITE,
        );

        draw_text(
            "Some walls, I cannot slide down.",
            550.0,
            380.0,
            24.0,
            WHITE,
        );
        draw_text("I can slide down here.", 0.0, 380.0, 24.0, WHITE);
        draw_text(
            "When sliding, I can jump from the wall.",
            140.0,
            600.0,
            24.0,
            WHITE,
        );

        for particle in world.particles() {
            draw_line(
                particle.last_position.x,
                particle.last_position.y,
                particle.position.x,
                particle.position.y,
                1.0,
                WHITE,
            );
        }

        set_default_camera();
        draw_text(
            &format!("Height: {:.1}m", (*level - pos.y - 32.0) / 16.0),
            0.0,
            28.0,
            30.0,
            WHITE,
        );
        draw_text(&format!("Coins: {}", coins), 0.0, 60.0, 30.0, WHITE);
        let s = timer / 60;
        let ms = ((timer % 60) as f32 / 60.0 * 1000.0) as i32;
        let m = s / 60;
        let s = s % 60;
        draw_text(
            &format!("Time: {:02}:{:02}:{:03}", m, s, ms),
            0.0,
            100.0,
            40.0,
            WHITE,
        );

        next_frame().await
    }
}
