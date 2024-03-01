use std::cmp::min;
use std::ops::ControlFlow;

use comfy::image::GenericImageView;
use comfy::*;

use crate::{Direction, SPRITE_PIXELS_PER_WINDOW_POINT};

mod data;

struct AnimationParams {
    sprites: &'static [AnnotatedSpriteParams],
    looping: bool,
}

struct AnnotatedSpriteParams {
    // TODO: Move to AnimationParams
    sprite_sheet: SpriteSheetParams,

    hurtbox: bool,
    hitbox: Option<Vec2>,
    duration: usize,
}

struct SpriteSheetParams {
    texture: &'static str,
    count_x: u32,
    count_y: u32,
}

impl SpriteSheetParams {
    const fn single_sprite(texture: &'static str) -> Self {
        SpriteSheetParams {
            texture,
            count_x: 1,
            count_y: 1,
        }
    }
}

#[derive(Clone)]
pub struct Animation {
    data: Rc<AnimationData>,
    sprite_index: usize,
    frame_counter: usize,
}

pub struct AnimationData {
    sprites: Vec<AnnotatedSprite>,
    looping: bool,
}

pub struct AnnotatedSprite {
    texture: TextureHandle,
    source_rect: IRect,
    pub hitbox: Option<Vec2>,
    pub hurtbox: Option<Vec2>,
    size: Vec2,
    duration: usize,
}

pub fn load_animations() -> HashMap<&'static str, Rc<AnimationData>> {
    let mut anims = HashMap::new();

    use data::*;
    anims.insert("block", load_animation(BLOCK).into());
    anims.insert("forward", load_animation(GOOSE_FORWARDS).into());
    anims.insert("backward", load_animation(WALKING_BACKWARD).into());
    anims.insert("standing", load_animation(GOOSE_STANDING_ANIMATION).into());
    anims.insert("attack", load_animation(GOOSE_ATTACK_ANIMATION).into());
    anims.insert("recoil", load_animation(RECOIL_ANIMATION).into());

    anims
}

impl AnimationData {
    pub fn to_anim(self: &Rc<Self>) -> Animation {
        Animation {
            data: Rc::clone(self),
            sprite_index: 0,
            frame_counter: 0,
        }
    }
}

impl Animation {
    pub fn is_instance(&self, data: &Rc<AnimationData>) -> bool {
        Rc::ptr_eq(&self.data, data)
    }
}

fn load_animation(params: AnimationParams) -> AnimationData {
    AnimationData {
        sprites: params
            .sprites
            .into_iter()
            .enumerate()
            .map(|(i, params)| load_sprite(i, params))
            .collect(),
        looping: params.looping,
    }
}

fn load_sprite(i: usize, params: &AnnotatedSpriteParams) -> AnnotatedSprite {
    let texture = texture_id(params.sprite_sheet.texture);
    let assets_lock = ASSETS.borrow();
    let images_lock = assets_lock.texture_image_map.lock();
    let image = images_lock.get(&texture).unwrap();

    let sprite_width = image.width() / params.sprite_sheet.count_x;
    let sprite_height = image.height() / params.sprite_sheet.count_y;

    let x = i as u32 % params.sprite_sheet.count_x;
    let y = i as u32 / params.sprite_sheet.count_x;
    assert!(y < params.sprite_sheet.count_y);

    let sprite_x = sprite_width * x;
    let sprite_y = sprite_height * y;

    let sprite_image =
        comfy::image::imageops::crop_imm(image, sprite_x, sprite_y, sprite_width, sprite_height);

    let mut min_x = sprite_width - 1;
    let mut max_x = 0;
    for (x, _y, value) in sprite_image.pixels() {
        if value.0[3] != 0 {
            min_x = min_x.min(x);
            max_x = max_x.max(x);
        }
    }

    // Crop both sides in symetrically by the smaller amount
    let delta_x = min(min_x, sprite_width - 1 - max_x);

    let sprite_size = IVec2 {
        x: (sprite_width - 2 * delta_x) as i32,
        y: sprite_height as i32,
    };

    let sprite_offset = IVec2 {
        x: sprite_x as i32 + delta_x as i32,
        y: sprite_y as i32,
    };

    let hurtbox = Vec2 {
        x: sprite_size.x as f32 / SPRITE_PIXELS_PER_WINDOW_POINT,
        y: sprite_size.y as f32 / SPRITE_PIXELS_PER_WINDOW_POINT,
    };

    AnnotatedSprite {
        texture,
        source_rect: IRect {
            offset: sprite_offset,
            size: sprite_size,
        },
        hurtbox: params.hurtbox.then(|| hurtbox),
        hitbox: params.hitbox,
        size: hurtbox,
        duration: params.duration,
    }
}

impl Animation {
    pub fn next_frame(&mut self) -> ControlFlow<()> {
        self.frame_counter += 1;
        if self.data.sprites[self.sprite_index].duration <= self.frame_counter {
            self.sprite_index += 1;
            self.frame_counter = 0;

            if self.sprite_index >= self.data.sprites.len() {
                self.sprite_index = 0;
                if !self.data.looping {
                    return ControlFlow::Break(());
                }
            }
        }
        ControlFlow::Continue(())
    }

    pub fn sprite(&self) -> &AnnotatedSprite {
        &self.data.sprites[self.sprite_index]
    }

    /// Player 1 faces right (no flip); player 2 faces left (flip).
    pub fn render(&self, location: Vec2, facing: Direction) {
        let sprite = self.sprite();
        draw_sprite_ex(
            sprite.texture,
            location,
            WHITE,
            2,
            DrawTextureParams {
                dest_size: Some(sprite.size.as_world_size()),
                source_rect: Some(sprite.source_rect),
                scroll_offset: Vec2::ZERO,
                rotation: 0.,
                flip_x: matches!(facing, Direction::West),
                flip_y: false,
                pivot: None,
                blend_mode: BlendMode::Alpha,
            },
        );
    }
}
