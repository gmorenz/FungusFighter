use std::cmp::min;
use std::ops::ControlFlow;
use std::rc::Rc;

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
    data: Rc<AnimationData>,
    sprite_index: usize,
    frame_counter: usize,
}

impl Animation {
    pub fn is_instance(&self, data: &Rc<AnimationData>) -> bool {
        Rc::ptr_eq(&self.data, data)
    }
}

pub struct AnimationData {
    sprites: Vec<AnnotatedSprite>, // TODO: + Source Rect
    looping: bool,
}

pub fn load_animations() -> HashMap<&'static str, Rc<AnimationData>> {
    let mut anims = HashMap::new();

    anims.insert("block", load_animation(BLOCK).into());
    anims.insert("forward", load_animation(WALKING_FORWARD).into());
    anims.insert("backward", load_animation(WALKING_BACKWARD).into());
    anims.insert("standing", load_animation(GOOSE_STANDING_ANIMATION).into());
    anims.insert("attack", load_animation(ATTACK_ANIMATION).into());
    anims.insert("recoil", load_animation(RECOIL_ANIMATION).into());

    anims
}

fn load_animation(params: AnimationParams) -> AnimationData {
    AnimationData {
        sprites: params.sprites.into_iter().enumerate().map(|(i, params)| load_sprite(i, params)).collect(),
        looping: params.looping,
    }
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

const BLOCK_SPRITES: SpriteSheetParams = SpriteSheetParams {
    texture: "F00_CrouchGuard",
    count_x: 1,
    count_y: 2,
};

const BLOCK: AnimationParams = AnimationParams {
    sprites: &[
        AnnotatedSpriteParams {
            sprite_sheet: BLOCK_SPRITES,
            hurtbox: false,
            hitbox: None,
            duration: 10,
        },
        AnnotatedSpriteParams {
            sprite_sheet: BLOCK_SPRITES,
            hurtbox: false,
            hitbox: None,
            duration: 10,
        }
    ],
    looping: false,
};

const WALKING_FORWARD_SPRITES: SpriteSheetParams = SpriteSheetParams {
    texture: "F00_Forward",
    count_x: 2,
    count_y: 3,
};

const WALKING_FORWARD: AnimationParams =  AnimationParams {
    sprites: &[
        AnnotatedSpriteParams {
            sprite_sheet: WALKING_FORWARD_SPRITES,
            hurtbox: true,
            hitbox: None,
            duration: 10,
        },
        AnnotatedSpriteParams {
            sprite_sheet: WALKING_FORWARD_SPRITES,
            hurtbox: true,
            hitbox: None,
            duration: 10,
        },
        AnnotatedSpriteParams {
            sprite_sheet: WALKING_FORWARD_SPRITES,
            hurtbox: true,
            hitbox: None,
            duration: 10,
        },
        AnnotatedSpriteParams {
            sprite_sheet: WALKING_FORWARD_SPRITES,
            hurtbox: true,
            hitbox: None,
            duration: 10,
        },
        AnnotatedSpriteParams {
            sprite_sheet: WALKING_FORWARD_SPRITES,
            hurtbox: true,
            hitbox: None,
            duration: 10,
        },
        AnnotatedSpriteParams {
            sprite_sheet: WALKING_FORWARD_SPRITES,
            hurtbox: true,
            hitbox: None,
            duration: 10,
        },
    ],
    looping: true,
};


const WALKING_BACKWARD_SPRITES: SpriteSheetParams = SpriteSheetParams {
    texture: "F00_Backward",
    count_x: 2,
    count_y: 3,
};

const WALKING_BACKWARD: AnimationParams =  AnimationParams {
    sprites: &[
        AnnotatedSpriteParams {
            sprite_sheet: WALKING_BACKWARD_SPRITES,
            hurtbox: true,
            hitbox: None,
            duration: 10,
        },
        AnnotatedSpriteParams {
            sprite_sheet: WALKING_BACKWARD_SPRITES,
            hurtbox: true,
            hitbox: None,
            duration: 10,
        },
        AnnotatedSpriteParams {
            sprite_sheet: WALKING_BACKWARD_SPRITES,
            hurtbox: true,
            hitbox: None,
            duration: 10,
        },
        AnnotatedSpriteParams {
            sprite_sheet: WALKING_BACKWARD_SPRITES,
            hurtbox: true,
            hitbox: None,
            duration: 10,
        },
        AnnotatedSpriteParams {
            sprite_sheet: WALKING_BACKWARD_SPRITES,
            hurtbox: true,
            hitbox: None,
            duration: 10,
        },
        AnnotatedSpriteParams {
            sprite_sheet: WALKING_BACKWARD_SPRITES,
            hurtbox: true,
            hitbox: None,
            duration: 10,
        },
    ],
    looping: true,
};

const GOOSE_STANDING_SPRITES: SpriteSheetParams = SpriteSheetParams {
    texture: "goose_idle",
    count_x: 1,
    count_y: 2,
};

const GOOSE_STANDING_ANIMATION: AnimationParams = AnimationParams {
    sprites: &[
        AnnotatedSpriteParams {
            sprite_sheet: GOOSE_STANDING_SPRITES,
            hurtbox: true,
            hitbox: None,
            duration: 30,
        },
        AnnotatedSpriteParams {
            sprite_sheet: GOOSE_STANDING_SPRITES,
            hurtbox: true,
            hitbox: None,
            duration: 30,
        }
    ],
    looping: true,
};

#[allow(dead_code)]
const FOO_STANDING_ANIMATION: AnimationParams = AnimationParams {
    sprites: &[
        AnnotatedSpriteParams {
            sprite_sheet: SpriteSheetParams::single_sprite("Idle_0"),
            hurtbox: true,
            hitbox: None,
            duration: 10,
        },
        AnnotatedSpriteParams {
            sprite_sheet: SpriteSheetParams::single_sprite("Idle_1"),
            hurtbox: true,
            hitbox: None,
            duration: 10,
        },
        AnnotatedSpriteParams {
            sprite_sheet: SpriteSheetParams::single_sprite("Idle_2"),
            hurtbox: true,
            hitbox: None,
            duration: 10,
        },
        AnnotatedSpriteParams {
            sprite_sheet: SpriteSheetParams::single_sprite("Idle_3"),
            hurtbox: true,
            hitbox: None,
            duration: 10,
        },
        AnnotatedSpriteParams {
            sprite_sheet: SpriteSheetParams::single_sprite("Idle_4"),
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

const ATTACK_ANIMATION: AnimationParams = AnimationParams {
    sprites: &[
        AnnotatedSpriteParams {
            sprite_sheet: ATTACK_SPRITES,
            hurtbox: true,
            hitbox: None,
            duration: 10,
        },
        AnnotatedSpriteParams {
            sprite_sheet: ATTACK_SPRITES,
            hurtbox: true,
            hitbox: None,
            duration: 10,
        },
        AnnotatedSpriteParams {
            sprite_sheet: ATTACK_SPRITES,
            hurtbox: true,
            hitbox: Some(Vec2 { x: 0.4, y: 0.2 }),
            duration: 10,
        },
        AnnotatedSpriteParams {
            sprite_sheet: ATTACK_SPRITES,
            hurtbox: true,
            hitbox: None,
            duration: 10,
        },
        AnnotatedSpriteParams {
            sprite_sheet: ATTACK_SPRITES,
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

const RECOIL_ANIMATION: AnimationParams = AnimationParams {
    sprites: &[
        AnnotatedSpriteParams {
            sprite_sheet: RECOIL_SPRITES,
            hurtbox: false,
            hitbox: None,
            duration: 10,
        },
        AnnotatedSpriteParams {
            sprite_sheet: RECOIL_SPRITES,
            hurtbox: false,
            hitbox: None,
            duration: 10,
        },
        AnnotatedSpriteParams {
            sprite_sheet: RECOIL_SPRITES,
            hurtbox: false,
            hitbox: None,
            duration: 10,
        },
        AnnotatedSpriteParams {
            sprite_sheet: RECOIL_SPRITES,
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
