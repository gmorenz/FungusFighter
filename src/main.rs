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

enum GameState {
    Playing(PlayingState),
    ScoreScreen {
        winner: usize,
    },
}

struct PlayingState {
    players: [Player; 2],
}

impl GameState {
    fn new(_c: &mut EngineState) -> Self {
        GameState::Playing(PlayingState {
            players: [
                Player { loc: -0.5, state: PlayerState::Idle, health: 3 },
                Player { loc: 0.5, state: PlayerState::Idle, health: 3 },
            ]
        })
    }
}

fn setup(_state: &mut GameState, _c: &mut EngineContext) {
    let mut camera = main_camera_mut();
    camera.zoom = 2.0;
}

fn update(state: &mut GameState, _c: &mut EngineContext) {
    match state {
        GameState::Playing(playing_state) => {
            let state_transition = playing_state.update();
            playing_state.render();
            if let Some(new_state) = state_transition {
                *state = new_state;
            }

        }
        GameState::ScoreScreen { winner } => {
            clear_background(WHITE);
            let msg = format!("Player {} won!", *winner + 1);
            draw_text(&msg, Vec2::ZERO, BLACK, TextAlign::Center);
        }
    }
}

impl PlayingState {
    fn update(&mut self) -> Option<GameState> {
        // Transition states

        for (i, p) in self.players.iter_mut().enumerate() {
            if p.health == 0 {
                return Some(GameState::ScoreScreen { winner: 1 - i });
            }

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

        if matches!(self.players[0].state, PlayerState::Idle) {
            if is_key_down(KeyCode::Space) {
                self.players[0].state = PlayerState::Attacking{ frame: 0 };
            } else {
                match (is_key_down(KeyCode::A), is_key_down(KeyCode::D)) {
                    // TODO: Check if this is framerate dependent.
                    (true, false) => self.players[0].move_(-PLAYER_SPEED),
                    (false, true) => self.players[0].move_(PLAYER_SPEED),
                    (true, true) | (false, false) => (),
                }
            }
        }

        if matches!(self.players[1].state, PlayerState::Idle) {
            if is_key_down(KeyCode::Down) {
                self.players[1].state = PlayerState::Attacking{ frame: 0 };
            } else {
                match (is_key_down(KeyCode::Left), is_key_down(KeyCode::Right)) {
                    (true, false) => self.players[1].move_(-PLAYER_SPEED),
                    (false, true) => self.players[1].move_(PLAYER_SPEED),
                    (true, true) | (false, false) => (),
                }
            }
        }

        // Handle attacks
        let hurtboxes = self.hurtboxes();
        let hitboxes = self.hitboxes();

        let hits = [
            hitboxes[0].is_some_and(|hitbox| hitbox.intersects(&hurtboxes[1])),
            hitboxes[1].is_some_and(|hitbox| hitbox.intersects(&hurtboxes[0])),
        ];

        match hits {
            [true, true] => for p in self.players.iter_mut() {
                p.state = PlayerState::AttackConnected{ frame: 0 };
            },
            [true, false] => {
                self.players[0].state = PlayerState::AttackConnected{ frame: 0 };
                self.players[1].health = self.players[1].health.saturating_sub(1);
            },
            [false, true] => {
                self.players[1].state = PlayerState::AttackConnected{ frame: 0 };
                self.players[0].health = self.players[0].health.saturating_sub(1);
            },
            [false, false] => (),
        }

        None
    }

    fn hitboxes(&self) -> [Option<AABB>; 2] {
        self.players.each_ref().map(|p| match &p.state {
            PlayerState::AttackConnected{ .. } | PlayerState::Idle => None,
            PlayerState::Attacking{ .. } => Some(AABB::from_center_size(p.center(), ATTACK_HITBOX)),
        })
    }


    fn hurtboxes(&self) -> [AABB; 2] {
        self.players.each_ref().map(|p| match &p.state {
            PlayerState::Idle => AABB::from_center_size(Vec2{ x: p.loc, y: 0.0 }, IDLE_HURTBOX),
            PlayerState::AttackConnected{ .. } | PlayerState::Attacking { .. } => AABB::from_center_size(p.center(), ATTACK_HURTBOX),
        })
    }

    fn render(&self) {
        clear_background(WHITE);

        for b in self.hurtboxes() {
            draw_rect(b.center(), b.size(), DARKGREEN, 1);
        }

        draw_rect(Vec2{ x: -0.75, y: 0.4 }, Vec2{ x: self.players[0].health as f32 / 10.0, y: 0.05 }, DARKGREEN, 1);
        draw_rect(Vec2{ x: 0.75, y: 0.4 }, Vec2{ x: self.players[1].health as f32 / 10.0, y: 0.05 }, DARKGREEN, 1);
    }
}