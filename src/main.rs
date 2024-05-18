mod animation;

use std::mem;
use std::ops::ControlFlow;

use ::include_dir::{Dir, DirEntry};
use animation::{Animation, AnimationData};
use bytemuck::Pod;
use comfy::bytemuck::Zeroable;
use comfy::*;
use ggrs::{GgrsError, NonBlockingSocket, P2PSession, SessionBuilder, SessionState};
use matchbox_socket::{PeerId, WebRtcSocket};

simple_game!("Goose Fighter", App, setup, update);

/// The screen is always [-1, 1]
///
/// This means 30 pixels in 0.2 (1 fifth) of a half window.
const SPRITE_PIXELS_PER_WINDOW_POINT: f32 = 16. / 0.2;

const PLAYER_SPEED: f32 = 0.01;

#[derive(Clone)]
enum PlayerState {
    Idle,
    Recoiling,
    Attacking,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Direction {
    East,
    West,
}

#[derive(Clone)]
struct Player {
    facing: Direction,
    loc: Vec2,
    endurance: u32,
    velocity: Vec2,
    tint: Color,

    // Animation counts frames, and is authoratative
    animation: Animation,
    state: PlayerState,
}

enum App {
    StartMenu { server: String },
    Connecting { socket: Option<WebRtcSocket> },
    InGame(Game),
}

struct Game {
    session: P2PSession<GGRSConfig>,

    // time variables for tick rate
    last_update: Instant,
    accumulator: Duration,

    state: GameState,

    animations: Animations,
}

type Animations = HashMap<String, Rc<AnimationData>>;

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
    const JUMP: u8 = 0b1000;
}

impl App {
    fn new(_e: &mut EngineState) -> Self {
        App::StartMenu {
            // server: "localhost:3536".into(),
            server: "gregs-macbook-air:3536".into(),
        }
    }
}

fn assets_dir() -> Dir<'static> {
    // TODO: figure this stuff out...

    // TODO: Instead, always include the directory into the executable, and fall
    // back to it when unable to read from the directory at runtime.
    // #[cfg(target_arch = "wasm32")]
    {
        use ::include_dir::include_dir;
        static ASSETS_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/assets");
        return ASSETS_DIR.clone();
    }

    // #[cfg(not(target_arch = "wasm32"))]
    // {
    //     let path = concat!(env!("CARGO_MANIFEST_DIR"), "/assets");
    //     Dir::from_fs(path).unwrap()
    // }
}

fn setup(_app: &mut App, c: &mut EngineContext) {
    let mut camera = main_camera_mut();
    camera.zoom = 2.0;

    // Load textures
    let dir = assets_dir();
    for entry in dir.find("**/*.png").unwrap() {
        if let DirEntry::File(file) = entry {
            c.load_texture_from_bytes(
                file.path().file_stem().unwrap().to_str().unwrap(),
                file.contents(),
            );
        }
    }
}

#[derive(Default)]
struct FakeSocket {
    queue: Vec<(PeerId, ggrs::Message)>,
}

impl NonBlockingSocket<PeerId> for FakeSocket {
    fn send_to(&mut self, msg: &ggrs::Message, addr: &PeerId) {
        self.queue.push((*addr, msg.clone()));
    }

    fn receive_all_messages(&mut self) -> Vec<(PeerId, ggrs::Message)> {
        mem::take(&mut self.queue)
    }
}

fn update(app: &mut App, _c: &mut EngineContext) {
    match app {
        App::StartMenu { ref mut server } => {
            clear_background(WHITE);

            let new_app = comfy::egui::CentralPanel::default().show(&comfy::egui(), |ui| {
                ui.with_layout(
                    egui::Layout::top_down_justified(egui::Align::Center),
                    |ui| {
                        ui.add(egui::Label::new("Goose Fighter"));
                        if ui.button("Start Local").clicked() {
                            let mut session = SessionBuilder::<GGRSConfig>::new()
                                .with_num_players(2)
                                .with_fps(60)
                                .unwrap();

                            for i in 0..2 {
                                session = session.add_player(ggrs::PlayerType::Local, i).unwrap();
                            }

                            let session = session.start_p2p_session(FakeSocket::default()).unwrap();

                            return Some(start_game(session));
                        }

                        if ui.button("Start Remote").clicked() {
                            info!("Constructing socket...");
                            // TODO: Use builder, more channels.
                            // let (socket, message_loop) = WebRtcSocket::new_ggrs("ws://206.172.98.17:3536/?next=2");
                            // let (socket, message_loop) = WebRtcSocket::new_ggrs("ws://206.172.98.17:80/foo");
                            // TODO: Sort of a injection vulnerability.
                            let (socket, message_loop) =
                                WebRtcSocket::new_ggrs(format!("ws://{server}/foo"));

                            #[cfg(not(target_arch = "wasm32"))]
                            std::thread::spawn(move || {
                                futures_lite::future::block_on(message_loop).unwrap();
                                panic!("Network socket message loop exited");
                            });
                            #[cfg(target_arch = "wasm32")]
                            wasm_bindgen_futures::spawn_local(async move {
                                message_loop.await.unwrap();
                                panic!("Network socket message loop exited");
                            });

                            return Some(App::Connecting {
                                socket: Some(socket),
                            });
                        }
                        ui.text_edit_singleline(server);
                        None
                    },
                )
            });

            if let Some(new_app) = new_app.inner.inner {
                *app = new_app;
            }

            // // Render text
            // draw_text(&"Goose Fighter", Vec2 { x: 0., y: 0.4 }, BLACK, TextAlign::Center);

            // Menu
            //  - Start Local
            //  - Connect to server [IP]
        }
        App::Connecting { socket } => {
            let socket_ref = socket.as_mut().unwrap();
            socket_ref.update_peers();
            let connected_count = socket_ref.connected_peers().count();
            print!("\rWaiting for {} more player(s)...", 1 - connected_count);

            if connected_count == 1 {
                println!();

                let mut session = SessionBuilder::<GGRSConfig>::new()
                    .with_num_players(2)
                    .with_fps(60)
                    .unwrap();

                let mut socket: WebRtcSocket = socket.take().unwrap();

                for (i, player) in socket.players().into_iter().enumerate() {
                    session = session.add_player(player, i).unwrap();
                }

                let session = session.start_p2p_session(socket).unwrap();

                *app = start_game(session);
            }
        }
        App::InGame(game) => game.update(),
    }
}

const MAX_ENDURANCE: u32 = 60 * 10;
const ENDURANCE_ADD_ATTACK: u32 = 60;
const ENDURANCE_SUB_HIT: u32 = 3 * 60;

fn start_game(session: P2PSession<GGRSConfig>) -> App {
    let animations = animation::load_animations();

    App::InGame(Game {
        session,

        last_update: Instant::now(),
        accumulator: Duration::ZERO,

        state: GameState::Playing(PlayingState {
            players: [
                Player {
                    tint: Color::rgb(1.0, 0.8, 0.8),
                    facing: Direction::East,
                    animation: animations["standing"].to_anim(),
                    loc: Vec2::new(-0.5, 0.0),
                    velocity: Vec2::ZERO,
                    state: PlayerState::Idle,
                    endurance: MAX_ENDURANCE,
                },
                Player {
                    tint: Color::rgb(0.8, 0.8, 1.0),
                    facing: Direction::West,
                    animation: animations["standing"].to_anim(),
                    loc: Vec2::new(0.5, 0.0),
                    velocity: Vec2::ZERO,
                    state: PlayerState::Idle,
                    endurance: MAX_ENDURANCE,
                },
            ],
        }),

        animations,
    })
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
                let mut handles = self.session.local_player_handles();
                handles.sort();
                for (idx, player) in handles.into_iter().enumerate() {
                    self.session
                        .add_local_input(player, get_local_input(idx))
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

fn get_local_input(idx: usize) -> Input {
    let mut bits = 0u8;

    match idx {
        0 => {
            if is_key_down(KeyCode::A) {
                bits |= Input::LEFT;
            }
            if is_key_down(KeyCode::D) {
                bits |= Input::RIGHT;
            }
            if is_key_down(KeyCode::Space) {
                bits |= Input::ATTACK;
            }
            if is_key_down(KeyCode::W) {
                bits |= Input::JUMP;
            }
        }
        1 => {
            if is_key_down(KeyCode::Left) {
                bits |= Input::LEFT;
            }
            if is_key_down(KeyCode::Right) {
                bits |= Input::RIGHT;
            }
            if is_key_down(KeyCode::Down) {
                bits |= Input::ATTACK;
            }
            if is_key_down(KeyCode::Up) {
                bits |= Input::JUMP;
            }
        }
        _ => unimplemented!(),
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

    fn is_jump_pressed(self) -> bool {
        self.input_bits & Self::JUMP != 0
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
            p.endurance = p.endurance.saturating_sub(1); // Tick endurance down each frame.
            if p.endurance == 0 {
                return Some(GameState::ScoreScreen { winner: 1 - i });
            }

            // TODO: Handle return
            if matches!(p.animation.next_frame(), ControlFlow::Break(())) {
                p.start_idle();
            }
        }

        if self.players[0].loc.x < self.players[1].loc.x {
            self.players[0].facing = Direction::East;
            self.players[1].facing = Direction::West;
        } else if self.players[0].loc.x > self.players[1].loc.x {
            self.players[0].facing = Direction::West;
            self.players[1].facing = Direction::East;
        }
        // Else, equal, change nothing.

        // HANDLE INPUT

        // NOTE: start_idle may not be called after this point.
        // we rely on input handling to put us in the right walking animation.

        for i in 0..2 {
            let mut x_accel: f32 = 0.0;
            if matches!(self.players[i].state, PlayerState::Idle { .. }) {
                if inputs[i].0.is_attack_pressed() {
                    self.players[i].start_attack(anims);
                } else {
                    let left = inputs[i].0.is_left_pressed();
                    let right = inputs[i].0.is_right_pressed();
                    let jump = inputs[i].0.is_jump_pressed();

                    let forwards = (left && (self.players[i].facing == Direction::West))
                        || (right && (self.players[i].facing == Direction::East));
                    let backwards = (left && (self.players[i].facing == Direction::East))
                        || (right && (self.players[i].facing == Direction::West));

                    match (forwards, backwards, jump) {
                        (_, _, true) if self.players[i].loc.y == 0.0 => {
                            // TODO: Jump?
                            self.players[i].velocity.y = 0.05;
                            self.players[i].ensure_standing(anims);
                        }
                        (true, false, _) => {
                            x_accel = PLAYER_SPEED;
                            self.players[i].ensure_walking_forwards(anims);
                        }
                        (false, true, _) => {
                            x_accel = -PLAYER_SPEED;
                            self.players[i].ensure_walking_backwards(anims);
                        }
                        (true, true, _) | (false, false, _) => {
                            self.players[i].ensure_standing(anims);
                        }
                    }
                }
            }
            self.players[i].accelerate(x_accel);
        }

        for p in &mut self.players {
            p.update_loc();
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
            [true, false] => self.players[1].handle_hit(anims),
            [false, true] => self.players[0].handle_hit(anims),
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
            Vec2 { x: -0.5, y: 0.4 },
            Vec2 {
                x: self.players[0].endurance as f32 / MAX_ENDURANCE as f32 * 0.95,
                y: 0.05,
            },
            self.players[0].tint,
            1,
        );
        draw_rect(
            Vec2 { x: 0.5, y: 0.4 },
            Vec2 {
                x: self.players[1].endurance as f32 / MAX_ENDURANCE as f32 * 0.95,
                y: 0.05,
            },
            self.players[1].tint,
            1,
        );
    }
}

impl Player {
    fn accelerate(&mut self, speed: f32) {
        let x_transform = match self.facing {
            Direction::East => 1.0,
            Direction::West => -1.0,
        };
        if self.loc.y == 0.0 && self.velocity.y == 0.0 {
            self.velocity.x = speed * x_transform;
        } else {
            self.velocity.x += 0.1 * speed * x_transform;
            self.velocity.y -= 0.002;
        }
    }

    fn update_loc(&mut self) {
        self.loc += self.velocity;
        self.loc.x = self.loc.x.clamp(-1.0, 1.0);
        if self.loc.y < 0.0 {
            // TODO: No landing animation?
            self.loc.y = 0.0;
            self.velocity.y = 0.0;
        }
    }

    fn center(&self) -> Vec2 {
        self.loc
    }

    fn hitbox(&self) -> Option<AABB> {
        let mut hb = self.animation.sprite().hitbox?;
        if matches!(self.facing, Direction::West) {
            // Reflect.
            (hb.min.x, hb.max.x) = (-hb.max.x, -hb.min.x);
        }
        hb.min += self.center();
        hb.max += self.center();
        Some(hb)
    }

    fn hurtbox(&self) -> Option<AABB> {
        let mut hb = self.animation.sprite().hurtbox?;
        if matches!(self.facing, Direction::West) {
            // Reflect.
            (hb.min.x, hb.max.x) = (-hb.max.x, -hb.min.x);
        }
        hb.min += self.center();
        hb.max += self.center();
        Some(hb)
    }

    fn render_sprite(&self) {
        self.animation.render(self.tint, self.center(), self.facing);
    }

    fn start_attack(&mut self, anims: &Animations) {
        self.state = PlayerState::Attacking;
        self.endurance = (self.endurance + ENDURANCE_ADD_ATTACK).min(MAX_ENDURANCE);
        self.animation = anims["attack"].to_anim();
    }

    fn start_idle(&mut self) {
        self.state = PlayerState::Idle;
    }

    fn ensure_standing(&mut self, anims: &Animations) {
        if !self.animation.is_instance(&anims["standing"]) {
            self.animation = anims["standing"].to_anim();
        }
    }

    fn ensure_walking_forwards(&mut self, anims: &Animations) {
        if !self.animation.is_instance(&anims["forward"]) {
            self.animation = anims["forward"].to_anim();
        }
    }

    fn ensure_walking_backwards(&mut self, anims: &Animations) {
        if !self.animation.is_instance(&anims["backward"]) {
            self.animation = anims["backward"].to_anim();
        }
    }

    fn is_walking_backwards(&mut self, anims: &Animations) -> bool {
        self.animation.is_instance(&anims["backward"])
    }

    fn start_recoil(&mut self, anims: &Animations) {
        self.state = PlayerState::Recoiling;
        self.animation = anims["recoil"].to_anim();
    }

    fn start_block(&mut self, anims: &Animations) {
        self.state = PlayerState::Recoiling; // TODO: Different state?
        self.animation = anims["block"].to_anim();
    }

    fn handle_hit(&mut self, anims: &Animations) {
        if self.is_walking_backwards(anims) {
            self.start_block(anims);
        } else {
            self.endurance = self.endurance.saturating_sub(ENDURANCE_SUB_HIT);
            self.start_recoil(anims);
        }
    }
}
