use comfy::{IRect, IVec2};

use super::{AnimationParams, AnnotatedSpriteParams, SpriteSheetParams};

pub const BLOCK_SPRITES: SpriteSheetParams = SpriteSheetParams {
    texture: "F00_CrouchGuard",
    count_x: 1,
    count_y: 2,
};

pub const BLOCK: AnimationParams = AnimationParams {
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
        },
    ],
    looping: false,
};

pub const WALKING_FORWARD_SPRITES: SpriteSheetParams = SpriteSheetParams {
    texture: "F00_Forward",
    count_x: 2,
    count_y: 3,
};

#[allow(dead_code)]
pub const WALKING_FORWARD: AnimationParams = AnimationParams {
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

pub const WALKING_BACKWARD_SPRITES: SpriteSheetParams = SpriteSheetParams {
    texture: "F00_Backward",
    count_x: 2,
    count_y: 3,
};

pub const WALKING_BACKWARD: AnimationParams = AnimationParams {
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

pub const GOOSE_FORWARDS_SPRITES: SpriteSheetParams = SpriteSheetParams {
    texture: "goose_16_walking",
    count_x: 1,
    count_y: 8,
};

pub const GOOSE_FORWARDS: AnimationParams = AnimationParams {
    sprites: &[
        AnnotatedSpriteParams {
            sprite_sheet: GOOSE_FORWARDS_SPRITES,
            hurtbox: true,
            hitbox: None,
            duration: 5,
        },
        AnnotatedSpriteParams {
            sprite_sheet: GOOSE_FORWARDS_SPRITES,
            hurtbox: true,
            hitbox: None,
            duration: 5,
        },
        AnnotatedSpriteParams {
            sprite_sheet: GOOSE_FORWARDS_SPRITES,
            hurtbox: true,
            hitbox: None,
            duration: 5,
        },
        AnnotatedSpriteParams {
            sprite_sheet: GOOSE_FORWARDS_SPRITES,
            hurtbox: true,
            hitbox: None,
            duration: 5,
        },
        AnnotatedSpriteParams {
            sprite_sheet: GOOSE_FORWARDS_SPRITES,
            hurtbox: true,
            hitbox: None,
            duration: 5,
        },
        AnnotatedSpriteParams {
            sprite_sheet: GOOSE_FORWARDS_SPRITES,
            hurtbox: true,
            hitbox: None,
            duration: 5,
        },
        AnnotatedSpriteParams {
            sprite_sheet: GOOSE_FORWARDS_SPRITES,
            hurtbox: true,
            hitbox: None,
            duration: 5,
        },
        AnnotatedSpriteParams {
            sprite_sheet: GOOSE_FORWARDS_SPRITES,
            hurtbox: true,
            hitbox: None,
            duration: 5,
        },
    ],
    looping: true,
};

pub const GOOSE_STANDING_SPRITES: SpriteSheetParams = SpriteSheetParams {
    texture: "goose_16_idle",
    count_x: 1,
    count_y: 2,
};

pub const GOOSE_STANDING_ANIMATION: AnimationParams = AnimationParams {
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
        },
    ],
    looping: true,
};

pub const GOOSE_ATTACK_SPRITES: SpriteSheetParams = SpriteSheetParams {
    texture: "goose_16_attack",
    count_x: 1,
    count_y: 4,
};

pub const GOOSE_ATTACK_ANIMATION: AnimationParams = AnimationParams {
    sprites: &[
        AnnotatedSpriteParams {
            sprite_sheet: GOOSE_ATTACK_SPRITES,
            hurtbox: true,
            hitbox: None,
            duration: 30,
        },
        AnnotatedSpriteParams {
            sprite_sheet: GOOSE_ATTACK_SPRITES,
            hurtbox: true,
            hitbox: None,
            duration: 30,
        },
        AnnotatedSpriteParams {
            sprite_sheet: GOOSE_ATTACK_SPRITES,
            hurtbox: true,
            hitbox: Some(IRect {
                offset: IVec2::new(24, 3),
                size: IVec2::new(5, 5),
            }),
            duration: 30,
        },
        AnnotatedSpriteParams {
            sprite_sheet: GOOSE_ATTACK_SPRITES,
            hurtbox: true,
            hitbox: None,
            duration: 30,
        },
    ],
    looping: false,
};

#[allow(dead_code)]
pub const FOO_STANDING_ANIMATION: AnimationParams = AnimationParams {
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

pub const ATTACK_SPRITES: SpriteSheetParams = SpriteSheetParams {
    texture: "F00_Attack_0",
    count_x: 2,
    count_y: 3,
};

#[allow(unused)]
pub const ATTACK_ANIMATION: AnimationParams = AnimationParams {
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
            hitbox: Some(IRect {
                offset: IVec2::new(10, 10),
                size: IVec2::new(10, 10),
            }),
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

pub const RECOIL_SPRITES: SpriteSheetParams = SpriteSheetParams {
    texture: "F00_Damage",
    count_x: 2,
    count_y: 2,
};

pub const RECOIL_ANIMATION: AnimationParams = AnimationParams {
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
