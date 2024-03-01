use std::ops::ControlFlow;
use std::{cmp::min, fs};

use comfy::anyhow::Context;
use comfy::image::GenericImageView;
use comfy::*;
use serde::Deserialize;

use crate::{Direction, SPRITE_PIXELS_PER_WINDOW_POINT};

#[derive(Deserialize)]
struct AnimationParams {
    sprite_sheet: SpriteSheetParams,
    sprites: Vec<AnnotatedSpriteParams>,
    looping: bool,
}

#[derive(Deserialize)]
struct AnnotatedSpriteParams {
    hurtbox: bool,
    /// Pixel coords in the current tile of the spritesheet.
    /// Top-left is (0,0).
    hitbox: Option<PixelRect>,
    duration: usize,
}

#[derive(Debug, Clone, Copy, Deserialize)]
struct PixelRect {
    offset: [u32; 2],
    size: [u32; 2],
}

impl From<PixelRect> for IRect {
    fn from(r: PixelRect) -> Self {
        IRect {
            offset: IVec2 {
                x: r.offset[0] as i32,
                y: r.offset[1] as i32,
            },
            size: IVec2 {
                x: r.size[0] as i32,
                y: r.size[1] as i32,
            },
        }
    }
}

#[derive(Deserialize)]
struct SpriteSheetParams {
    texture: String,
    count_x: u32,
    count_y: u32,
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

/// Using player frame of refence, world scale.
pub struct AnnotatedSprite {
    texture: TextureHandle,
    source_rect: PixelRect,
    pub hitbox: Option<AABB>,
    pub hurtbox: Option<AABB>,
    size: Vec2,
    duration: usize,
}

pub fn load_animations() -> HashMap<String, Rc<AnimationData>> {
    let mut anims = HashMap::new();

    // TODO: remove the runtime dependency on this specific path
    for entry in fs::read_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/assets")).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        let Some(ext) = path.extension() else {
            continue;
        };
        if ext == "ron" {
            let name = path.file_stem().unwrap().to_str().unwrap().to_string();
            let contents = fs::read_to_string(path).unwrap();
            let data = Rc::new(load_animation(
                ron::from_str::<AnimationParams>(&contents)
                    .with_context(|| name.clone())
                    .unwrap_or_else(|e| panic!("{e}")),
            ));
            anims.insert(name, data);
        }
    }

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

fn load_animation(anim: AnimationParams) -> AnimationData {
    AnimationData {
        looping: anim.looping,
        sprites: anim
            .sprites
            .into_iter()
            .enumerate()
            .map(|(i, sprite)| load_sprite(i, &anim.sprite_sheet, &sprite))
            .collect(),
    }
}

fn load_sprite(
    i: usize,
    sprite_sheet: &SpriteSheetParams,
    sprite: &AnnotatedSpriteParams,
) -> AnnotatedSprite {
    let texture = texture_id(&sprite_sheet.texture);
    let assets_lock = ASSETS.borrow();
    let images_lock = assets_lock.texture_image_map.lock();
    let image = images_lock.get(&texture).unwrap();

    let sprite_width = image.width() / sprite_sheet.count_x;
    let sprite_height = image.height() / sprite_sheet.count_y;

    let x = i as u32 % sprite_sheet.count_x;
    let y = i as u32 / sprite_sheet.count_x;
    assert!(y < sprite_sheet.count_y);

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

    let sprite_size = [(sprite_width - 2 * delta_x) as u32, sprite_height as u32];

    let sprite_offset = [sprite_x + delta_x, sprite_y];

    let hurtbox = AABB::from_center_size(
        Vec2::ZERO,
        Vec2 {
            x: sprite_size[0] as f32 / SPRITE_PIXELS_PER_WINDOW_POINT,
            y: sprite_size[1] as f32 / SPRITE_PIXELS_PER_WINDOW_POINT,
        },
    );

    let hitbox = sprite.hitbox.map(|rect| {
        // in: pixel, y-down, (0,0) topleft
        // out: float, y-up, (0,0) center of player

        let center_x = sprite_width as f32 / 2.;
        let center_y = sprite_height as f32 / 2.;
        let x = (rect.offset[0] as f32 - center_x) / SPRITE_PIXELS_PER_WINDOW_POINT;
        let y = -1. * (rect.offset[1] as f32 - center_y) / SPRITE_PIXELS_PER_WINDOW_POINT;

        let w = rect.size[0] as f32 / SPRITE_PIXELS_PER_WINDOW_POINT;
        let h = rect.size[1] as f32 / SPRITE_PIXELS_PER_WINDOW_POINT;

        AABB::from_top_left(Vec2 { x, y }, Vec2 { x: w, y: h })
    });

    AnnotatedSprite {
        texture,
        source_rect: PixelRect {
            offset: sprite_offset,
            size: sprite_size,
        },
        hurtbox: sprite.hurtbox.then(|| hurtbox),
        hitbox,
        size: hurtbox.size(),
        duration: sprite.duration,
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
                source_rect: Some(sprite.source_rect.into()),
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
