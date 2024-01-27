#![feature(array_methods)]

mod animation;

use animation::{load_animation, Animation, IDLE_ANIMATION};
use comfy::include_dir::{include_dir, Dir, DirEntry};
use comfy::*;

simple_game!("Fighting Fungus", GameState, setup, update);

/// The screen is always [-1, 1]
///
/// This means 30 pixels in 0.2 (1 fifth) of a half window.
const SPRITE_PIXELS_PER_WINDOW_POINT: f32 = 30. / 0.2;

// const IDLE_HURTBOX: Vec2 = Vec2 { x: 0.1, y: 0.5 };
// const ATTACK_HURTBOX: Vec2 = Vec2 { x: 0.3, y: 0.4 };
// const ATTACK_HITBOX: Vec2 = Vec2 { x: 0.325, y: 0.35 };
const PLAYER_SPEED: f32 = 0.01;
const ATTACK_DURATION: u32 = 30;

enum PlayerState {
    Idle,
    AttackConnected { frame: u32 },
    Attacking { frame: u32 },
}

struct Player {
    loc: f32,
    health: u32,

    // Animation counts frames, and is authoratative
    animation: Animation,
    state: PlayerState,
}

enum GameState {
    Loading,
    Playing(PlayingState),
    ScoreScreen { winner: usize },
}

struct PlayingState {
    players: [Player; 2],
}

impl GameState {
    fn new(_e: &mut EngineState) -> Self {
        GameState::Loading
    }
}

static ASSETS_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/assets");

fn setup(_state: &mut GameState, c: &mut EngineContext) {
    let mut camera = main_camera_mut();
    camera.zoom = 2.0;

    // Load textures
    for entry in ASSETS_DIR.find("**/*.png").unwrap() {
        if let DirEntry::File(file) = entry {
            c.load_texture_from_bytes(
                file.path().file_stem().unwrap().to_str().unwrap(),
                file.contents(),
            );
        }
    }
}

fn update(state: &mut GameState, c: &mut EngineContext) {
    match state {
        GameState::Loading => {
            *state = GameState::Playing(PlayingState {
                players: [
                    Player {
                        animation: load_animation(IDLE_ANIMATION),
                        loc: -0.5,
                        state: PlayerState::Idle,
                        health: 3,
                    },
                    Player {
                        animation: load_animation(IDLE_ANIMATION),
                        loc: 0.5,
                        state: PlayerState::Idle,
                        health: 3,
                    },
                ],
            });
            return update(state, c);
        }

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

            // TODO: Handle return
            p.animation.next_frame();

            use PlayerState::*;
            p.state = match p.state {
                Idle => Idle,
                AttackConnected { frame } | Attacking { frame } if frame >= ATTACK_DURATION => Idle,
                Attacking { frame } => Attacking { frame: frame + 1 },
                AttackConnected { frame } => AttackConnected { frame: frame + 1 },
            }
        }

        // HANDLE INPUT

        if matches!(self.players[0].state, PlayerState::Idle { .. }) {
            if is_key_down(KeyCode::Space) {
                self.players[0].state = PlayerState::Attacking { frame: 0 };
            } else {
                match (is_key_down(KeyCode::A), is_key_down(KeyCode::D)) {
                    // TODO: Check if this is framerate dependent.
                    (true, false) => self.players[0].move_(-PLAYER_SPEED),
                    (false, true) => self.players[0].move_(PLAYER_SPEED),
                    (true, true) | (false, false) => (),
                }
            }
        }

        if matches!(self.players[1].state, PlayerState::Idle { .. }) {
            if is_key_down(KeyCode::Down) {
                self.players[1].state = PlayerState::Attacking { frame: 0 };
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
            [true, true] => {
                for p in self.players.iter_mut() {
                    p.state = PlayerState::AttackConnected { frame: 0 };
                }
            }
            [true, false] => {
                self.players[0].state = PlayerState::AttackConnected { frame: 0 };
                self.players[1].health = self.players[1].health.saturating_sub(1);
            }
            [false, true] => {
                self.players[1].state = PlayerState::AttackConnected { frame: 0 };
                self.players[0].health = self.players[0].health.saturating_sub(1);
            }
            [false, false] => (),
        }

        None
    }

    fn hitboxes(&self) -> [Option<AABB>; 2] {
        self.players.each_ref().map(|p| p.hitbox())
    }

    fn hurtboxes(&self) -> [AABB; 2] {
        self.players.each_ref().map(|p| p.hurtbox())
    }

    fn render(&self) {
        clear_background(WHITE);

        for b in self.hurtboxes() {
            draw_rect_outline(b.center(), b.size(), 0.01, DARKGREEN, 1);
        }

        for b in self.hitboxes() {
            if let Some(b) = b {
                // TODO: Not pixel perfect, border extends past hitbox.
                draw_rect_outline(b.center(), b.size(), 0.01, DARKRED, 2);
            }
        }

        for p in &self.players {
            p.render_sprite();
        }

        draw_rect(
            Vec2 { x: -0.75, y: 0.4 },
            Vec2 {
                x: self.players[0].health as f32 / 10.0,
                y: 0.05,
            },
            DARKGREEN,
            1,
        );
        draw_rect(
            Vec2 { x: 0.75, y: 0.4 },
            Vec2 {
                x: self.players[1].health as f32 / 10.0,
                y: 0.05,
            },
            DARKGREEN,
            1,
        );
    }
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

    fn hitbox(&self) -> Option<AABB> {
        self.animation
            .sprite()
            .hitbox
            .map(|hitbox| AABB::from_center_size(self.center(), hitbox))
    }

    fn hurtbox(&self) -> AABB {
        AABB::from_center_size(self.center(), self.animation.sprite().hurtbox)
    }

    fn render_sprite(&self) {
        self.animation.render(self.center());
    }
}
