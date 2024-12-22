use crate::*;

const STARTING_NOTE: i8 = 21;
const ENDING_NOTE: i8 = 108;
const NOTE_SPEED: f32 = 150.;
const NOTE_WIDTH: f32 = 15.;
const BLACK_COLOR_TOP: Srgba = DARK_GRAY;
const BLACK_COLOR_BOTTOM: Srgba = DARK_GRAY;
const WHITE_COLOR_TOP: Srgba = WHITE;
const WHITE_COLOR_BOTTOM: Srgba = WHITE;
const KEYBOARD_WHITE_COLOR: Srgba = WHITE;
const KEYBOARD_WHITE_COLOR_ACTIVE: Srgba = Srgba {
    red: 190. / 256.,
    green: 190. / 256.,
    blue: 190. / 256.,
    alpha: 1.,
};
const KEYBOARD_BLACK_COLOR: Srgba = Srgba {
    red: 76. / 256.,
    green: 76. / 256.,
    blue: 76. / 256.,
    alpha: 1.,
};
const KEYBOARD_BLACK_COLOR_ACTIVE: Srgba = Srgba {
    red: 138. / 256.,
    green: 138. / 256.,
    blue: 138. / 256.,
    alpha: 1.,
};
const KEYBOARD_FELT_COLOR: Srgba = RED;

pub struct ConfigPlugin;

impl Plugin for ConfigPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Configuration {
            show_keyboard: true,
            keyboard_height: 200.,
            enable_bloom: false,
            bloom_composite_mode: BloomCompositeMode::EnergyConserving,
            bloom_intensity: 0.5,
            sync_white_notes: true,
            sync_black_notes: true,
            note_speed: NOTE_SPEED,
            note_width: NOTE_WIDTH,
            black_color_top: BLACK_COLOR_TOP,
            black_color_bottom: BLACK_COLOR_BOTTOM,
            white_color_top: WHITE_COLOR_TOP,
            white_color_bottom: WHITE_COLOR_BOTTOM,
            keyboard_white_color: KEYBOARD_WHITE_COLOR,
            keyboard_black_color: KEYBOARD_BLACK_COLOR,
            keyboard_white_color_active: KEYBOARD_WHITE_COLOR_ACTIVE,
            keyboard_black_color_active: KEYBOARD_BLACK_COLOR_ACTIVE,
            keyboard_felt_color: KEYBOARD_FELT_COLOR,
            sync_keyboard_active_color: false,
            starting_note: STARTING_NOTE,
            ending_note: ENDING_NOTE,
            background_color:BLACK,
        });
    }
}

#[derive(Resource, Clone, Copy)]
pub struct Configuration {
    pub starting_note: i8,
    pub ending_note: i8,
    pub keyboard_height: f32,
    pub show_keyboard: bool,
    pub enable_bloom: bool,
    pub bloom_intensity: f32,
    pub bloom_composite_mode: BloomCompositeMode,
    pub note_speed: f32,
    pub note_width: f32,
    pub black_color_top: Srgba,
    pub black_color_bottom: Srgba,
    pub white_color_top: Srgba,
    pub white_color_bottom: Srgba,
    pub sync_white_notes: bool,
    pub sync_black_notes: bool,
    pub keyboard_white_color: Srgba,
    pub keyboard_black_color: Srgba,
    pub keyboard_white_color_active: Srgba,
    pub keyboard_black_color_active: Srgba,
    pub keyboard_felt_color: Srgba,
    pub sync_keyboard_active_color: bool,
    pub background_color: Srgba,
    // pub vertex_colors: Vec<[f32; 4]>,
    // pub vertex_colors_blacks: Vec<[f32; 4]>,
    // pub vertex_colors_keyboard: Vec<[f32; 4]>,
    // pub vertex_colors_keyboard_blacks: Vec<[f32; 4]>,
    // pub vertex_colors: Vec<[f32; 4]>,
    // pub vertex_colors_blacks: Vec<[f32; 4]>,
}
