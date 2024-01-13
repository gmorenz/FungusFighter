#![feature(array_methods)]

use comfy::*;

simple_game!("Fighting Fungus", GameState, setup, update);

const IDLE_HURTBOX: Vec2 = Vec2 { x: 0.1, y: 0.5 };
const ATTACK_HURTBOX: Vec2 = Vec2 { x: 0.3, y: 0.4 };
const ATTACK_HITBOX: Vec2 = Vec2 { x: 0.325, y: 0.35 };
const PLAYER_SPEED: f32 = 0.01;
const ATTACK_DURATION: u32 = 30;

enum PlayerState {
    Idle,
    AttackConnected{ frame: u32 },
    Attacking { frame: u32 },
}

struct Player {
    loc: f32,
    health: u32,

    state: PlayerState,
}

impl Player {
    fn move_(&mut self, x: f32) {
        self.loc += x;
        self.loc = self.loc.clamp(-1.0, 1.0);
    }

    fn center(&self) -> Vec2 {
        Vec2 {
            x: self.loc,
            y: 0.0,
        }
    }
}

struct GameState {
    players: [Player; 2]
}

impl GameState {
    fn new(_c: &mut EngineState) -> Self {
        GameState {
            players: [
                Player { loc: -0.5, state: PlayerState::Idle, health: 3 },
                Player { loc: 0.5, state: PlayerState::Idle, health: 3 },
            ]
        }
    }
}

fn setup(_state: &mut GameState, _c: &mut EngineContext) {
    let mut camera = main_camera_mut();
    camera.zoom = 2.0;
}

fn update(state: &mut GameState, _c: &mut EngineContext) {
    clear_background(WHITE);

    let players = &mut state.players;

    // Transition states

    for p in players.iter_mut() {
        use PlayerState::*;
        p.state = match p.state {
            Idle => Idle,
            AttackConnected{ frame } | Attacking{ frame } if frame >= ATTACK_DURATION => {
                Idle
            }
            Attacking { frame } => Attacking { frame: frame + 1 },
            AttackConnected { frame } => AttackConnected { frame: frame + 1 },
        }
    }

    // HANDLE INPUT

    if matches!(players[0].state, PlayerState::Idle) {
        if is_key_down(KeyCode::Space) {
            players[0].state = PlayerState::Attacking{ frame: 0 };
        }
        // TODO: Last button takes priority, by listening for events
        // TODO: Check if this is framerate dependent.
        else if is_key_down(KeyCode::A) {
            players[0].move_(-PLAYER_SPEED);
        }
        else if is_key_down(KeyCode::D) {
            players[0].move_(PLAYER_SPEED);
        }
    }

    if matches!(players[1].state, PlayerState::Idle) {
        if is_key_down(KeyCode::Down) {
            players[1].state = PlayerState::Attacking{ frame: 0 };
        }
        else if is_key_down(KeyCode::Left) {
            players[1].move_(-PLAYER_SPEED);
        }
        else if is_key_down(KeyCode::Right) {
            players[1].move_(PLAYER_SPEED);
        }
    }

    // Handle attacks

    let hurtboxes = players.each_ref().map(|p| match &p.state {
        PlayerState::Idle => AABB::from_center_size(Vec2{ x: p.loc, y: 0.0 }, IDLE_HURTBOX),
        PlayerState::AttackConnected{ .. } | PlayerState::Attacking { .. } => AABB::from_center_size(p.center(), ATTACK_HURTBOX),
    });

    let hitboxes = players.each_ref().map(|p| match &p.state {
        PlayerState::AttackConnected{ .. } | PlayerState::Idle => None,
        PlayerState::Attacking{ .. } => Some(AABB::from_center_size(p.center(), ATTACK_HITBOX)),
    });

    let hits = [
        hitboxes[0].is_some_and(|hitbox| hitbox.intersects(&hurtboxes[1])),
        hitboxes[1].is_some_and(|hitbox| hitbox.intersects(&hurtboxes[0])),
    ];

    match hits {
        [true, true] => for p in players.iter_mut() {
            p.state = PlayerState::AttackConnected{ frame: 0 };
        },
        [true, false] => {
            players[0].state = PlayerState::AttackConnected{ frame: 0 };
            players[1].health = players[1].health.saturating_sub(1);
        },
        [false, true] => {
            players[1].state = PlayerState::AttackConnected{ frame: 0 };
            players[0].health = players[0].health.saturating_sub(1);
        },
        [false, false] => (),
    }

    // RENDER

    for b in hurtboxes {
        draw_rect(b.center(), b.size(), DARKGREEN, 1);
    }

    draw_rect(Vec2{ x: -0.75, y: 0.4 }, Vec2{ x: players[0].health as f32 / 10.0, y: 0.05 }, DARKGREEN, 1);
    draw_rect(Vec2{ x: 0.75, y: 0.4 }, Vec2{ x: players[1].health as f32 / 10.0, y: 0.05 }, DARKGREEN, 1);

}