use std::cmp::min;
use std::ops::ControlFlow;

use comfy::image::GenericImageView;
use comfy::*;

use crate::{Direction, SPRITE_PIXELS_PER_WINDOW_POINT};

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

struct AnnotatedSpriteParams {
    sprite_sheet: SpriteSheetParams,
    x: u32,
    y: u32,

    hurtbox: bool,
    hitbox: Option<Vec2>,
    duration: usize,
}

pub struct AnimationParams {
    sprites: &'static [AnnotatedSpriteParams], // TODO: + Source Rect
    looping: bool,
}

#[derive(Clone)] // TODO: Remove
pub struct AnnotatedSprite {
    texture: TextureHandle,
    source_rect: IRect,
    pub hitbox: Option<Vec2>,
    pub hurtbox: Option<Vec2>,
    pub size: Vec2,
    duration: usize,
}

#[derive(Clone)]
pub struct Animation {
    sprites: Vec<AnnotatedSprite>, // TODO: + Source Rect
    sprite_index: usize,
    frame_counter: usize,
    looping: bool,
}

pub fn load_animation(params: AnimationParams) -> Animation {
    Animation {
        sprites: params.sprites.into_iter().map(load_sprite).collect(),
        sprite_index: 0,
        frame_counter: 0,
        looping: params.looping,
    }
}

fn load_sprite(params: &AnnotatedSpriteParams) -> AnnotatedSprite {
    let texture = texture_id(params.sprite_sheet.texture);
    let assets_lock = ASSETS.borrow();
    let images_lock = assets_lock.texture_image_map.lock();
    let image = images_lock.get(&texture).unwrap();

    let sprite_width = image.width() / params.sprite_sheet.count_x;
    let sprite_height = image.height() / params.sprite_sheet.count_y;

    let sprite_x = sprite_width * params.x;
    let sprite_y = sprite_height * params.y;

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

pub const IDLE_ANIMATION: AnimationParams = AnimationParams {
    sprites: &[
        AnnotatedSpriteParams {
            sprite_sheet: SpriteSheetParams::single_sprite("Idle_0"),
            x: 0,
            y: 0,
            hurtbox: true,
            hitbox: None,
            duration: 10,
        },
        AnnotatedSpriteParams {
            sprite_sheet: SpriteSheetParams::single_sprite("Idle_1"),
            x: 0,
            y: 0,
            hurtbox: true,
            hitbox: None,
            duration: 10,
        },
        AnnotatedSpriteParams {
            sprite_sheet: SpriteSheetParams::single_sprite("Idle_2"),
            x: 0,
            y: 0,
            hurtbox: true,
            hitbox: None,
            duration: 10,
        },
        AnnotatedSpriteParams {
            sprite_sheet: SpriteSheetParams::single_sprite("Idle_3"),
            x: 0,
            y: 0,
            hurtbox: true,
            hitbox: None,
            duration: 10,
        },
        AnnotatedSpriteParams {
            sprite_sheet: SpriteSheetParams::single_sprite("Idle_4"),
            x: 0,
            y: 0,
            hurtbox: true,
            hitbox: None,
            duration: 10,
        },
    ],
    looping: true,
};

const ATTACK_SPRITES: SpriteSheetParams = SpriteSheetParams {
    texture: "F00_Attack_0",
    count_x: 2,
    count_y: 3,
};

pub const ATTACK_ANIMATION: AnimationParams = AnimationParams {
    sprites: &[
        AnnotatedSpriteParams {
            sprite_sheet: ATTACK_SPRITES,
            x: 0,
            y: 0,
            hurtbox: true,
            hitbox: None,
            duration: 10,
        },
        AnnotatedSpriteParams {
            sprite_sheet: ATTACK_SPRITES,
            x: 1,
            y: 0,
            hurtbox: true,
            hitbox: None,
            duration: 10,
        },
        AnnotatedSpriteParams {
            sprite_sheet: ATTACK_SPRITES,
            x: 0,
            y: 1,
            hurtbox: true,
            hitbox: Some(Vec2 { x: 0.4, y: 0.2 }),
            duration: 10,
        },
        AnnotatedSpriteParams {
            sprite_sheet: ATTACK_SPRITES,
            x: 1,
            y: 1,
            hurtbox: true,
            hitbox: None,
            duration: 10,
        },
        AnnotatedSpriteParams {
            sprite_sheet: ATTACK_SPRITES,
            x: 0,
            y: 2,
            hurtbox: true,
            hitbox: None,
            duration: 10,
        },
    ],
    looping: false,
};

const RECOIL_SPRITES: SpriteSheetParams = SpriteSheetParams {
    texture: "F00_Damage",
    count_x: 2,
    count_y: 2,
};

pub const RECOIL_ANIMATION: AnimationParams = AnimationParams {
    sprites: &[
        AnnotatedSpriteParams {
            sprite_sheet: RECOIL_SPRITES,
            x: 0,
            y: 0,
            hurtbox: false,
            hitbox: None,
            duration: 10,
        },
        AnnotatedSpriteParams {
            sprite_sheet: RECOIL_SPRITES,
            x: 1,
            y: 0,
            hurtbox: false,
            hitbox: None,
            duration: 10,
        },
        AnnotatedSpriteParams {
            sprite_sheet: RECOIL_SPRITES,
            x: 0,
            y: 1,
            hurtbox: false,
            hitbox: None,
            duration: 10,
        },
        AnnotatedSpriteParams {
            sprite_sheet: RECOIL_SPRITES,
            x: 1,
            y: 1,
            hurtbox: false,
            hitbox: None,
            duration: 10,
        },
    ],
    looping: false,
};

impl Animation {
    pub fn next_frame(&mut self) -> ControlFlow<()> {
        self.frame_counter += 1;
        if self.sprites[self.sprite_index].duration <= self.frame_counter {
            self.sprite_index += 1;
            self.frame_counter = 0;

            if self.sprite_index >= self.sprites.len() {
                self.sprite_index = 0;
                if !self.looping {
                    return ControlFlow::Break(());
                }
            }
        }
        ControlFlow::Continue(())
    }

    pub fn sprite(&self) -> &AnnotatedSprite {
        &self.sprites[self.sprite_index]
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
