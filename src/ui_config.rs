extern crate midir;

// use bevy::ecs::query::QueryFilter;
// use bevy::gizmos::config;

use bevy::prelude::*;

use bevy_egui::{egui, EguiContexts, EguiPlugin};

// use std::thread::JoinHandle;
use crate::*;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin)
            .add_systems(Update, ui_config_system);
    }
}

fn decimal_to_intiger_color(arr: &[f32; 4]) -> egui::Color32 {
    //return [(arr[0]*256.)as  u8,(arr[1]*256.)as u8,(arr[2]) ]
    let converted = arr.iter().map(|x| (x * 256.) as u8).collect::<Vec<u8>>();
    return egui::Color32::from_rgba_premultiplied(
        converted[0],
        converted[1],
        converted[2],
        converted[3],
    );
}

pub fn ui_config_system(
    mut contexts: EguiContexts,
    mut config: ResMut<Configuration>,
    mut bloom_settings: Query<&mut BloomSettings>,
    //mut keys: Query<(&mut Transform, &KeyNote)>,
    window: Query<&Window>,
    notes_placement: Res<NotePlacemnt>,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<ColorMaterial>>,
    old_keys: Query<Entity, With<KeyboardElement>>,
    commands: Commands,
    keyboard_note_meshes: ResMut<KeyboardNoteMeshes>,
    mut note_offset: ResMut<NoteOffset>,
) {
    let white_top = config.white_color_top.to_f32_array();
    let white_bottom = config.white_color_bottom.to_f32_array();
    let black_top = config.black_color_top.to_f32_array();
    let black_bottom = config.black_color_bottom.to_f32_array();
    let mut w_t = decimal_to_intiger_color(&white_top);
    let mut w_b = decimal_to_intiger_color(&white_bottom);
    let mut b_t = decimal_to_intiger_color(&black_top);
    let mut b_b = decimal_to_intiger_color(&black_bottom);

    let mut k_w = decimal_to_intiger_color(&config.keyboard_white_color.to_f32_array());
    let mut k_w_a = decimal_to_intiger_color(&config.keyboard_white_color_active.to_f32_array());
    let mut k_b = decimal_to_intiger_color(&config.keyboard_black_color.to_f32_array());
    let mut k_b_a = decimal_to_intiger_color(&config.keyboard_black_color_active.to_f32_array());
    let mut k_f = decimal_to_intiger_color(&config.keyboard_felt_color.to_f32_array());

    egui::Window::new("Config").show(contexts.ctx_mut(), |ui| {
        ui.label("baka");
        ui.label("white top color");
        ui.color_edit_button_srgba(&mut w_t);
        ui.label("white bottom color");
        ui.color_edit_button_srgba(&mut w_b);
        ui.checkbox(&mut config.sync_white_notes, "sync white notes");

        ui.label("black top color");
        ui.color_edit_button_srgba(&mut b_t);
        ui.label("black bottom color");
        ui.color_edit_button_srgba(&mut b_b);
        ui.checkbox(&mut config.sync_black_notes, "sync black notes");
        ui.add(egui::Slider::new(&mut config.note_speed, 100.0..=300.0).text("note speed"));
        let k_height = ui.add(
            egui::Slider::new(&mut config.keyboard_height, 100.0..=300.0).text("keyboard height"),
        );
        let s_note =
            ui.add(egui::Slider::new(&mut config.starting_note, 21..=108).text("starting note"));
        let e_note =
            ui.add(egui::Slider::new(&mut config.ending_note, 21..=108).text("ending note"));

        ui.checkbox(&mut config.enable_bloom, "enable bloom");
        if config.enable_bloom {
            ui.add(
                egui::Slider::new(&mut config.bloom_intensity, 0.0..=1.0).text("bloom intensity"),
            );
            let additive_button = ui.button("additive");
            if additive_button.clicked() {
                config.bloom_composite_mode = BloomCompositeMode::Additive;
            }
            let efficent_button = ui.button("efficent");
            if efficent_button.clicked() {
                config.bloom_composite_mode = BloomCompositeMode::EnergyConserving;
            }
        }
        ui.label("keyboard white color");
        let w_color = ui.color_edit_button_srgba(&mut k_w);
        ui.label("keyboard black color");
        let b_color = ui.color_edit_button_srgba(&mut k_b);
        ui.label("keyboard white active color");
        ui.color_edit_button_srgba(&mut k_w_a);
        ui.label("keyboard black active color");
        ui.color_edit_button_srgba(&mut k_b_a);
        ui.checkbox(
            &mut config.sync_keyboard_active_color,
            "sync active keyboard keys",
        );
        ui.label("keyboard felt color");
        let felt = ui.color_edit_button_srgba(&mut k_f);

        if ui.button("save default").clicked() {
            save_config(&config);
        }

        let keyboard_gen_button = ui.add(egui::Button::new("Generate Keyboard"));
        if s_note.changed()
            || e_note.changed()
            || keyboard_gen_button.clicked()
            || k_height.changed()
            || felt.changed()
            || w_color.changed()
            || b_color.changed()
        {
            note_offset.offset = count_whites(21, config.starting_note, &notes_placement.blacks);
            note_offset.whites_count = count_whites(
                config.starting_note,
                config.ending_note + 1,
                &notes_placement.blacks,
            );

            draw_keyboard(
                &config,
                commands,
                window,
                meshes,
                materials,
                notes_placement,
                old_keys,
                keyboard_note_meshes,
                &note_offset,
            );
        }
    });
    for mut bs in &mut bloom_settings {
        if config.enable_bloom {
            bs.intensity = config.bloom_intensity;
            bs.composite_mode = config.bloom_composite_mode;
        } else {
            bs.intensity = 0.;
        }
    }
    if config.sync_white_notes {
        w_b = w_t;
    }
    if config.sync_black_notes {
        b_b = b_t;
    }
    if config.sync_keyboard_active_color {
        k_b_a = k_w_a;
    }
    config.white_color_top = compress_color(w_t);
    config.white_color_bottom = compress_color(w_b);
    config.black_color_top = compress_color(b_t);
    config.black_color_bottom = compress_color(b_b);

    config.keyboard_white_color = compress_color(k_w);
    config.keyboard_white_color_active = compress_color(k_w_a);
    config.keyboard_black_color = compress_color(k_b);
    config.keyboard_black_color_active = compress_color(k_b_a);
    config.keyboard_felt_color = compress_color(k_f);
}

fn compress_color(color: egui::Color32) -> Srgba {
    return Srgba {
        red: color.to_array()[0] as f32 / 256.,
        green: color.to_array()[1] as f32 / 256.,
        blue: color.to_array()[2] as f32 / 256.,
        alpha: color.to_array()[3] as f32 / 256.,
    };
}
fn format_color(color: Srgba) -> String {
    let mut out = String::new();
    out.push_str(&color.red.to_string());
    out += ",";
    out.push_str(&color.green.to_string());
    out += ",";
    out.push_str(&color.blue.to_string());
    out += ",";
    out.push_str(&color.alpha.to_string());
    out
}

fn save_config(config: &Configuration) {
    let mut out: String = String::new();

    out.push_str(
        format!(
            "note_speed:{}\nstarting_note:{}\nending_note:{}\nenable_bloom:{}\nnote_width:{}\nsync_white_notes:{}\nsync_black_notes:{}\nsync_keyboard_active_color:{}\nkeyboard_height:{}\nshow_keyboard:{}\n",
            config.note_speed, config.starting_note, config.ending_note, config.enable_bloom,config.note_width,config.sync_white_notes,config.sync_black_notes,config.sync_keyboard_active_color,config.keyboard_height,config.show_keyboard
        )
        .as_str(),
    );
    out.push_str(
        format!(
            "white_top:{}\nwhite_bottom:{}\nblack_top:{}\nblack_bottom:{}\nkeyboard_black:{}\nkeyboard_black_active:{}\nkeyboard_white:{}\nkeyboard_white_active{}\nkeyboard_felt:{}\nbackground_color:{}\n",
            format_color(config.white_color_top),
            format_color(config.white_color_bottom),
            format_color(config.black_color_top),
            format_color(config.black_color_bottom),
            format_color(config.keyboard_black_color),
            format_color(config.keyboard_black_color_active),
            format_color(config.keyboard_white_color),
            format_color(config.keyboard_white_color_active),
            format_color(config.keyboard_felt_color),
            format_color(config.background_color),
        )
        .as_str(),
    );

    let mut ofile = File::create("save_default.txt").expect("unable to create file");

    ofile.write_all(out.as_bytes()).expect("unable to write");
}
