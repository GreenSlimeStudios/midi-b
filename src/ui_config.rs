extern crate midir;

// use bevy::ecs::query::QueryFilter;
// use bevy::gizmos::config;

use bevy::prelude::*;

use bevy_egui::{
    egui::{self, RichText, TextBuffer, TextEdit},
    EguiContexts, EguiPlugin,
};
use std::fs::read_to_string;

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

    mut app_background_color: ResMut<ClearColor>,
) {
    let white_top = config.white_color_top.to_f32_array();
    let white_bottom = config.white_color_bottom.to_f32_array();
    let black_top = config.black_color_top.to_f32_array();
    let black_bottom = config.black_color_bottom.to_f32_array();
    let background_color = config.background_color.to_f32_array();
    let keyboard_filler_color = config.keyboard_filler_color.to_f32_array();

    let mut k_f_c = decimal_to_intiger_color(&keyboard_filler_color);
    let mut b_c = decimal_to_intiger_color(&background_color);

    let mut w_t = decimal_to_intiger_color(&white_top);
    let mut w_b = decimal_to_intiger_color(&white_bottom);
    let mut b_t = decimal_to_intiger_color(&black_top);
    let mut b_b = decimal_to_intiger_color(&black_bottom);

    let mut k_w = decimal_to_intiger_color(&config.keyboard_white_color.to_f32_array());
    let mut k_w_a = decimal_to_intiger_color(&config.keyboard_white_color_active.to_f32_array());
    let mut k_b = decimal_to_intiger_color(&config.keyboard_black_color.to_f32_array());
    let mut k_b_a = decimal_to_intiger_color(&config.keyboard_black_color_active.to_f32_array());
    let mut k_f = decimal_to_intiger_color(&config.keyboard_felt_color.to_f32_array());

    let mut is_color_changed: bool = false;

    egui::Window::new("Config").show(contexts.ctx_mut(), |ui| {
        // ui.label("baka");

        ui.label("Color of notes played");
        // ui.heading("Color of notes played");
        // ui.add_space(4.);

        // ui.separator();
        egui::Grid::new("note_color").show(ui, |ui| {
            ui.label("");
            ui.label("top");
            ui.label("bottom");
            ui.label("sync").on_hover_text(
                "when enabled the note will be a solid if not you can make a gradient",
            );
            ui.end_row();

            // ui.label("white top color");

            ui.label("white");
            if ui.color_edit_button_srgba(&mut w_t).changed() {
                is_color_changed = true;
            }

            // ui.label("white bottom color");
            if ui.color_edit_button_srgba(&mut w_b).changed() {
                is_color_changed = true;
            }
            ui.checkbox(&mut config.sync_white_notes, "").on_hover_text(
                "when enabled the note will be a solid if not you can make a gradient",
            );

            ui.end_row();

            ui.label("black");
            // ui.label("black top color");
            if ui.color_edit_button_srgba(&mut b_t).changed() {
                is_color_changed = true;
            }
            // ui.label("black bottom color");
            if ui.color_edit_button_srgba(&mut b_b).changed() {
                is_color_changed = true;
            }
            ui.checkbox(&mut config.sync_black_notes, "").on_hover_text(
                "when enabled the note will be a solid if not you can make a gradient",
            );
        });
        ui.spacing();
        ui.separator();

        ui.add(egui::Slider::new(&mut config.note_speed, 100.0..=300.0).text("note speed"));
        let k_height = ui.add(
            egui::Slider::new(&mut config.keyboard_height, 0.0..=300.0).text("keyboard height"),
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

        // let mut w_color: Response;
        // let mut b_color: Response;

        ui.separator();
        // ui.label("Keyboard colors");
        ui.label(RichText::new("keyboard colors"));
        // ui.separator();
        egui::Grid::new("keyboard_color").show(ui, |ui| {
            ui.label("");
            ui.label("idle").on_hover_text(
                "color of white notes on keyboard while inactive (currently not being pressed)",
            );
            ui.label("active").on_hover_text(
                "color of black notes on keyboard while active (currently pressed) ",
            );

            ui.end_row();

            ui.label("white");
            // ui.label("keyboard white color");
            if ui.color_edit_button_srgba(&mut k_w).changed() {
                is_color_changed = true;
            }
            // ui.label("keyboard white active color");
            if ui.color_edit_button_srgba(&mut k_w_a).changed() {
                is_color_changed = true;
            }

            ui.end_row();

            ui.label("black");
            // ui.label("keyboard black color");
            if ui.color_edit_button_srgba(&mut k_b).changed() {
                is_color_changed = true;
            }
            // ui.label("keyboard black active color");
            if ui.color_edit_button_srgba(&mut k_b_a).changed() {
                is_color_changed = true;
            }

            ui.end_row();

            ui.label("felt");
            if ui.color_edit_button_srgba(&mut k_f).changed() {
                is_color_changed = true;
            }
            ui.end_row();
            ui.label("back");
            if ui.color_edit_button_srgba(&mut b_c).changed() {
                is_color_changed = true;
            }
            ui.end_row();
            ui.label("fill").on_hover_text(
                "the color that is filled in between the keys on the virtual keyboard",
            );
            if ui.color_edit_button_srgba(&mut k_f_c).changed() {
                is_color_changed = true;
            }
        });
        if ui
            .checkbox(
                &mut config.sync_keyboard_active_color,
                "sync active keyboard keys",
            )
            .changed()
        {
            is_color_changed = true;
        };

        ui.separator();

        let load_default_button = ui.button("load default settings");
        if load_default_button.clicked() {
            load_config(&mut config, "./saves/do_not_alter.sav.txt");
        }

        ui.horizontal(|ui| {
            ui.add(
                TextEdit::singleline(&mut config.save_file_name).hint_text("enter save file name"),
            );
            if ui.button("add").clicked() && !config.save_file_name.is_empty() {
                save_config(
                    &config,
                    &("./saves/".to_owned() + config.save_file_name.as_str() + ".sav"),
                );
            }
        });
        ui.separator();
        let mut config_loaded: bool = false;

        ui.label("Detected save files");
        for entry in fs::read_dir("./saves/").unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_file() {
                if path.display().to_string().ends_with(".sav") {
                    let save_name = path.file_stem().unwrap().to_string_lossy();
                    ui.horizontal(|ui| {
                        if ui.button("Load").clicked() {
                            load_config(&mut config, &path.display().to_string().as_str());
                            config_loaded = true;
                        }
                        if ui.button("Save").clicked() {
                            save_config(&mut config, &path.display().to_string().as_str());
                        }
                        if ui.button("Delete").clicked() {
                            match fs::remove_file(path.clone()) {
                                Ok(a) => {
                                    println!("removed config file {:?}", a)
                                }
                                Err(e) => {
                                    println!("error removing config file: {:?}", e);
                                }
                            }
                        }

                        ui.label(save_name + "");
                    });

                    // println!("{}", &save_name);
                }
            }
        }

        ui.separator();
        let keyboard_gen_button = ui.add(egui::Button::new("Generate Keyboard"));
        if s_note.changed()
            || load_default_button.clicked()
            || e_note.changed()
            || keyboard_gen_button.clicked()
            || k_height.changed()
            || is_color_changed
            || config_loaded
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

            *app_background_color = ClearColor(Color::from(config.background_color));
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
    if is_color_changed {
        config.white_color_top = compress_color(w_t);
        config.white_color_bottom = compress_color(w_b);
        config.black_color_top = compress_color(b_t);
        config.black_color_bottom = compress_color(b_b);

        config.keyboard_white_color = compress_color(k_w);
        config.keyboard_white_color_active = compress_color(k_w_a);
        config.keyboard_black_color = compress_color(k_b);
        config.keyboard_black_color_active = compress_color(k_b_a);
        config.keyboard_felt_color = compress_color(k_f);

        config.background_color = compress_color(b_c);
        config.keyboard_filler_color = compress_color(k_f_c);
    }
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

fn save_config(config: &Configuration, path: &str) {
    let mut out: String = String::new();

    out.push_str(
        format!(
            "note_speed:{}\nstarting_note:{}\nending_note:{}\nenable_bloom:{}\nbloom_intensity:{}\nnote_width:{}\nsync_white_notes:{}\nsync_black_notes:{}\nsync_keyboard_active_color:{}\nkeyboard_height:{}\nshow_keyboard:{}\n",
            config.note_speed, config.starting_note, config.ending_note, config.enable_bloom,config.bloom_intensity,config.note_width,config.sync_white_notes,config.sync_black_notes,config.sync_keyboard_active_color,config.keyboard_height,config.show_keyboard
        )
        .as_str(),
    );
    out.push_str(
        format!(
            "white_top:{}\nwhite_bottom:{}\nblack_top:{}\nblack_bottom:{}\nkeyboard_black:{}\nkeyboard_black_active:{}\nkeyboard_white:{}\nkeyboard_white_active:{}\nkeyboard_felt:{}\nkeyboard_filler:{}\nbackground_color:{}\n",
            format_color(config.white_color_top),
            format_color(config.white_color_bottom),
            format_color(config.black_color_top),
            format_color(config.black_color_bottom),
            format_color(config.keyboard_black_color),
            format_color(config.keyboard_black_color_active),
            format_color(config.keyboard_white_color),
            format_color(config.keyboard_white_color_active),
            format_color(config.keyboard_felt_color),
            format_color(config.keyboard_filler_color),
            format_color(config.background_color),
        )
        .as_str(),
    );

    out.push_str(
        format!(
            "bloom_composite_mode:{}\n",
            match config.bloom_composite_mode {
                BloomCompositeMode::Additive => "additive",
                BloomCompositeMode::EnergyConserving => "efficent",
            }
        )
        .as_str(),
    );

    let mut ofile = File::create(path).expect("unable to create file");

    ofile.write_all(out.as_bytes()).expect("unable to write");
}

fn load_config(config: &mut Configuration, path: &str) {
    // for line in read_to_string("./saves/save_default.sav.txt")
    for line in read_to_string(path).unwrap().lines() {
        let values: Vec<&str> = line.split(":").collect();
        println!("{}:{}", &values[0], &values[1]);
        match values[0] {
            "white_top" => {
                let colors: Vec<f32> = values[1]
                    .split(",")
                    .filter_map(|x| x.parse::<f32>().ok())
                    .collect();
                config.white_color_top = Srgba {
                    red: colors[0],
                    green: colors[1],
                    blue: colors[2],
                    alpha: colors[3],
                };
            }
            "white_bottom" => {
                let colors: Vec<f32> = values[1]
                    .split(",")
                    .filter_map(|x| x.parse::<f32>().ok())
                    .collect();
                config.white_color_bottom = Srgba {
                    red: colors[0],
                    green: colors[1],
                    blue: colors[2],
                    alpha: colors[3],
                };
            }
            "black_top" => {
                let colors: Vec<f32> = values[1]
                    .split(",")
                    .filter_map(|x| x.parse::<f32>().ok())
                    .collect();
                config.black_color_top = Srgba {
                    red: colors[0],
                    green: colors[1],
                    blue: colors[2],
                    alpha: colors[3],
                };
            }
            "black_bottom" => {
                let colors: Vec<f32> = values[1]
                    .split(",")
                    .filter_map(|x| x.parse::<f32>().ok())
                    .collect();
                config.black_color_bottom = Srgba {
                    red: colors[0],
                    green: colors[1],
                    blue: colors[2],
                    alpha: colors[3],
                };
            }
            "keyboard_white" => {
                let colors: Vec<f32> = values[1]
                    .split(",")
                    .filter_map(|x| x.parse::<f32>().ok())
                    .collect();
                config.keyboard_white_color = Srgba {
                    red: colors[0],
                    green: colors[1],
                    blue: colors[2],
                    alpha: colors[3],
                };
            }
            "keyboard_white_active" => {
                let colors: Vec<f32> = values[1]
                    .split(",")
                    .filter_map(|x| x.parse::<f32>().ok())
                    .collect();
                config.keyboard_white_color_active = Srgba {
                    red: colors[0],
                    green: colors[1],
                    blue: colors[2],
                    alpha: colors[3],
                };
            }
            "keyboard_black" => {
                let colors: Vec<f32> = values[1]
                    .split(",")
                    .filter_map(|x| x.parse::<f32>().ok())
                    .collect();
                config.keyboard_black_color = Srgba {
                    red: colors[0],
                    green: colors[1],
                    blue: colors[2],
                    alpha: colors[3],
                };
            }
            "keyboard_black_active" => {
                let colors: Vec<f32> = values[1]
                    .split(",")
                    .filter_map(|x| x.parse::<f32>().ok())
                    .collect();
                config.keyboard_black_color_active = Srgba {
                    red: colors[0],
                    green: colors[1],
                    blue: colors[2],
                    alpha: colors[3],
                };
            }
            "keyboard_felt" => {
                let colors: Vec<f32> = values[1]
                    .split(",")
                    .filter_map(|x| x.parse::<f32>().ok())
                    .collect();
                config.keyboard_felt_color = Srgba {
                    red: colors[0],
                    green: colors[1],
                    blue: colors[2],
                    alpha: colors[3],
                };
            }
            "keyboard_filler" => {
                let colors: Vec<f32> = values[1]
                    .split(",")
                    .filter_map(|x| x.parse::<f32>().ok())
                    .collect();
                config.keyboard_filler_color = Srgba {
                    red: colors[0],
                    green: colors[1],
                    blue: colors[2],
                    alpha: colors[3],
                };
            }
            "background_color" => {
                let colors: Vec<f32> = values[1]
                    .split(",")
                    .filter_map(|x| x.parse::<f32>().ok())
                    .collect();
                config.background_color = Srgba {
                    red: colors[0],
                    green: colors[1],
                    blue: colors[2],
                    alpha: colors[3],
                };
            }
            "note_speed" => {
                config.note_speed = values[1].parse::<f32>().unwrap();
            }
            "starting_note" => {
                config.starting_note = values[1].parse().unwrap();
            }
            "bloom_intensity" => {
                config.bloom_intensity = values[1].parse().unwrap();
            }
            "keyboard_height" => {
                config.keyboard_height = values[1].parse().unwrap();
            }
            "ending_note" => {
                config.ending_note = values[1].parse().unwrap();
            }
            "enable_bloom" => {
                config.enable_bloom = values[1].parse().unwrap();
            }
            "note_width" => {
                config.note_width = values[1].parse().unwrap();
            }
            "sync_white_notes" => {
                config.sync_white_notes = values[1].parse().unwrap();
            }
            "sync_black_notes" => {
                config.sync_black_notes = values[1].parse().unwrap();
            }
            "show_keyboard" => {
                config.show_keyboard = values[1].parse().unwrap();
            }
            "sync_keyboard_active_color" => {
                config.sync_keyboard_active_color = values[1].parse().unwrap();
            }
            "bloom_composite_mode" => match values[1] {
                "efficent" => config.bloom_composite_mode = BloomCompositeMode::EnergyConserving,
                "additive" => config.bloom_composite_mode = BloomCompositeMode::Additive,
                _ => {
                    println!("invalide bloom composite mode");
                }
            },
            _ => {
                println!("detected unknown config line: {}", &values[0]);
            }
        }
    }
}
