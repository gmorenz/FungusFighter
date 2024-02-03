#![feature(array_methods)]

mod animation;

use std::ops::ControlFlow;
use std::rc::Rc;

use animation::{Animation, AnimationData};
use bytemuck::Pod;
use comfy::bytemuck::Zeroable;
use comfy::include_dir::{include_dir, Dir, DirEntry};
use comfy::*;
use ggrs::{GgrsError, P2PSession, PlayerHandle, SessionBuilder, SessionState};
use matchbox_socket::{PeerId, WebRtcSocket};

simple_game!("Fighting Fungus", App, setup, update);

/// The screen is always [-1, 1]
///
/// This means 30 pixels in 0.2 (1 fifth) of a half window.
const SPRITE_PIXELS_PER_WINDOW_POINT: f32 = 30. / 0.2;

// const IDLE_HURTBOX: Vec2 = Vec2 { x: 0.1, y: 0.5 };
// const ATTACK_HURTBOX: Vec2 = Vec2 { x: 0.3, y: 0.4 };
// const ATTACK_HITBOX: Vec2 = Vec2 { x: 0.325, y: 0.35 };
const PLAYER_SPEED: f32 = 0.01;
// const ATTACK_DURATION: u32 = 30;

#[derive(Clone)]
enum PlayerState {
    Idle,
    Recoiling,
    Attacking,
}

#[derive(Clone, Copy)]
enum Direction {
    East,
    West,
}

#[derive(Clone)]
struct Player {
    facing: Direction,
    loc: f32,
    health: u32,

    // Animation counts frames, and is authoratative
    animation: Animation,
    state: PlayerState,
}

enum App {
    Loading { socket: Option<WebRtcSocket> },
    InGame(Game),
}

struct Game {
    session: P2PSession<GGRSConfig>,
    local_player: PlayerHandle,

    // time variables for tick rate
    last_update: Instant,
    accumulator: Duration,

    state: GameState,

    animations: Animations,
}

type Animations = HashMap<&'static str, Rc<AnimationData>>;

#[derive(Clone)]
enum GameState {
    Playing(PlayingState),
    ScoreScreen { winner: usize },
}

#[derive(Clone)]
struct PlayingState {
    players: [Player; 2],
}

/// `GGRSConfig` holds all type parameters for GGRS Sessions
#[derive(Debug)]
struct GGRSConfig;
impl ggrs::Config for GGRSConfig {
    type Input = Input;
    type State = GameState;
    type Address = PeerId;
}

#[repr(C)]
#[derive(PartialEq, Eq, Copy, Clone, Pod, Zeroable)]
pub struct Input {
    /// Bit 0: left
    ///
    /// Bit 1: right
    ///
    /// Bit 2: attack
    // TODO: Bitfield crate? Needs to be ": Pod"
    input_bits: u8,
}

impl Input {
    const LEFT: u8 = 0b001;
    const RIGHT: u8 = 0b010;
    const ATTACK: u8 = 0b100;
}

impl App {
    fn new(_e: &mut EngineState) -> Self {
        info!("Constructing socket...");
        // TODO: Use builder, more channels.
        let (socket, message_loop) = WebRtcSocket::new_ggrs("ws://70.29.57.216/foo");
        std::thread::spawn(move || {
            futures_lite::future::block_on(message_loop).unwrap();
            panic!("Network socket message loop exited");
        });

        App::Loading {
            socket: Some(socket),
        }
    }
}

static ASSETS_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/assets");

fn setup(_app: &mut App, c: &mut EngineContext) {
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

fn update(app: &mut App, _c: &mut EngineContext) {
    match app {
        App::Loading { socket } => {
            let socket_ref = socket.as_mut().unwrap();
            socket_ref.update_peers();
            let connected_count = socket_ref.connected_peers().count();
            println!("Waiting for {} more player(s)...", 1 - connected_count);

            if cfg!(feature="local") || connected_count == 1 {
                let mut session = SessionBuilder::<GGRSConfig>::new()
                    .with_num_players(2)
                    .with_fps(60)
                    .unwrap();

                let mut socket: WebRtcSocket = socket.take().unwrap();

                if cfg!(feature = "local") {
                    for i in 0.. 2 {
                        session = session.add_player(ggrs::PlayerType::Local, i).unwrap();
                    }
                } else {
                    for (i, player) in socket.players().into_iter().enumerate() {
                        session = session.add_player(player, i).unwrap();
                    }
                }

                let session = session.start_p2p_session(socket).unwrap();

                let local_player;
                if cfg!(feature = "local") {
                    local_player = 0;
                } else {
                    (local_player,) = session
                        .local_player_handles()
                        .into_iter()
                        .collect_tuple()
                        .unwrap();
                }

                let animations = animation::load_animations();

                *app = App::InGame(Game {
                    session,
                    local_player,

                    last_update: Instant::now(),
                    accumulator: Duration::ZERO,

                    state: GameState::Playing(PlayingState {
                        players: [
                            Player {
                                facing: Direction::East,
                                animation: animations["idle"].to_anim(),
                                loc: -0.5,
                                state: PlayerState::Idle,
                                health: 3,
                            },
                            Player {
                                facing: Direction::West,
                                animation: animations["idle"].to_anim(),
                                loc: 0.5,
                                state: PlayerState::Idle,
                                health: 3,
                            },
                        ],
                    }),

                    animations,
                });
            }
        }
        App::InGame(game) => game.update(),
    }
}

const FPS: f64 = 60.0;

impl Game {
    fn update(&mut self) {
        // communicate, receive and send packets
        // TODO: Do we need this? It does it implicitly in advance_frame.
        self.session.poll_remote_clients();

        // print GGRS events
        for event in self.session.events() {
            println!("Event: {:?}", event);
        }

        // this is to keep ticks between clients synchronized.
        // if a client is ahead, it will run frames slightly slower to allow catching up
        let mut fps_delta = 1. / FPS;
        if self.session.frames_ahead() > 0 {
            fps_delta *= 1.1;
        }

        // get delta time from last iteration and accumulate it
        let now = Instant::now();
        let delta = now.duration_since(self.last_update);
        self.accumulator = self.accumulator.saturating_add(delta);
        self.last_update = now;

        // if enough time is accumulated, we run a frame
        while self.accumulator.as_secs_f64() > fps_delta {
            self.accumulator -= Duration::from_secs_f64(fps_delta);

            // frames are only happening if the sessions are synchronized
            if self.session.current_state() == SessionState::Running {
                self.session
                    .add_local_input(self.local_player, get_local_input())
                    .unwrap();

                if cfg!(feature = "local") {
                    // TODO: Hook up player 2s controls.
                    self.session
                        .add_local_input(1, Input{ input_bits: 0 })
                        .unwrap();
                }

                match self.session.advance_frame() {
                    Ok(requests) => self.handle_requests(requests),
                    Err(GgrsError::PredictionThreshold) => {
                        println!("Frame {} skipped", self.session.current_frame())
                    }
                    Err(e) => panic!("{}", e),
                }
            }
        }

        self.render();
    }

    fn handle_requests(&mut self, requests: Vec<ggrs::GgrsRequest<GGRSConfig>>) {
        for req in requests {
            match req {
                ggrs::GgrsRequest::SaveGameState { cell, frame } => {
                    cell.save(frame, Some(self.state.clone()), None);
                }
                ggrs::GgrsRequest::LoadGameState { cell, frame: _ } => {
                    self.state = cell.load().unwrap();
                }
                ggrs::GgrsRequest::AdvanceFrame { inputs } => {
                    self.advance_frame(inputs);
                }
            }
        }
    }

    fn render(&self) {
        match &self.state {
            GameState::Playing(playing_state) => playing_state.render(),
            GameState::ScoreScreen { winner } => {
                clear_background(WHITE);
                let msg = format!("Player {} won!", *winner + 1);
                draw_text(&msg, Vec2::ZERO, BLACK, TextAlign::Center);
            }
        }
    }

    fn advance_frame(&mut self, inputs: Vec<(Input, ggrs::InputStatus)>) {
        match &mut self.state {
            GameState::Playing(playing_state) => {
                let state_transition = playing_state.update(inputs, &self.animations);
                if let Some(new_state) = state_transition {
                    self.state = new_state;
                }
            }
            // TODO: start the next round at some point
            GameState::ScoreScreen { .. } => (),
        }
    }
}

fn get_local_input() -> Input {
    let mut bits = 0u8;

    if is_key_down(KeyCode::A) {
        bits |= Input::LEFT;
    }
    if is_key_down(KeyCode::D) {
        bits |= Input::RIGHT;
    }
    if is_key_down(KeyCode::Space) {
        bits |= Input::ATTACK;
    }

    Input { input_bits: bits }
}

impl Input {
    fn is_attack_pressed(self) -> bool {
        self.input_bits & Self::ATTACK != 0
    }

    fn is_left_pressed(self) -> bool {
        self.input_bits & Self::LEFT != 0
    }

    fn is_right_pressed(self) -> bool {
        self.input_bits & Self::RIGHT != 0
    }
}

impl PlayingState {
    fn update(
        &mut self,
        inputs: Vec<(Input, ggrs::InputStatus)>,
        anims: &Animations,
    ) -> Option<GameState> {
        // Transition states

        for (i, p) in self.players.iter_mut().enumerate() {
            if p.health == 0 {
                return Some(GameState::ScoreScreen { winner: 1 - i });
            }

            // TODO: Handle return
            if matches!(p.animation.next_frame(), ControlFlow::Break(())) {
                p.start_idle(anims);
            }
        }

        // HANDLE INPUT

        for i in 0..2 {
            if matches!(self.players[i].state, PlayerState::Idle { .. }) {
                if inputs[i].0.is_attack_pressed() {
                    self.players[i].start_attack(anims);
                } else {
                    let left = inputs[i].0.is_left_pressed();
                    let right = inputs[i].0.is_right_pressed();
                    match (left, right) {
                        // TODO: Check if this is framerate dependent.
                        (true, false) => self.players[i].move_(-PLAYER_SPEED),
                        (false, true) => self.players[i].move_(PLAYER_SPEED),
                        (true, true) | (false, false) => (),
                    }
                }
            }
        }

        // Handle attacks
        let hurtboxes = self.hurtboxes();
        let hitboxes = self.hitboxes();

        let hits = [
            hitboxes[0]
                .zip(hurtboxes[1])
                .is_some_and(|(hit, hurt)| hit.intersects(&hurt)),
            hitboxes[1]
                .zip(hurtboxes[0])
                .is_some_and(|(hit, hurt)| hit.intersects(&hurt)),
        ];

        match hits {
            [true, true] => {
                for p in self.players.iter_mut() {
                    p.start_recoil(anims);
                }
            }
            [true, false] => {
                self.players[1].health = self.players[1].health.saturating_sub(1);
                self.players[1].start_recoil(anims);
            }
            [false, true] => {
                self.players[0].health = self.players[0].health.saturating_sub(1);
                self.players[0].start_recoil(anims);
            }
            [false, false] => (),
        }

        None
    }

    fn hitboxes(&self) -> [Option<AABB>; 2] {
        self.players.each_ref().map(|p| p.hitbox())
    }

    fn hurtboxes(&self) -> [Option<AABB>; 2] {
        self.players.each_ref().map(|p| p.hurtbox())
    }

    fn render(&self) {
        clear_background(WHITE);

        for b in self.hurtboxes() {
            if let Some(b) = b {
                draw_rect_outline(b.center(), b.size(), 0.01, DARKGREEN, 1);
            }
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
        let hb = self.animation.sprite().hitbox?;
        Some(AABB::from_center_size(self.center(), hb))
    }

    fn hurtbox(&self) -> Option<AABB> {
        let hb = self.animation.sprite().hurtbox?;
        Some(AABB::from_center_size(self.center(), hb))
    }

    fn render_sprite(&self) {
        self.animation.render(self.center(), self.facing);
    }

    fn start_attack(&mut self, anims: &Animations) {
        self.state = PlayerState::Attacking;
        self.animation = anims["attack"].to_anim();
    }

    fn start_idle(&mut self, anims: &Animations) {
        self.state = PlayerState::Idle;
        self.animation = anims["idle"].to_anim();
    }

    fn start_recoil(&mut self, anims: &Animations) {
        self.state = PlayerState::Recoiling;
        self.animation = anims["recoil"].to_anim();
    }
}
