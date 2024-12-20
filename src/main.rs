// use bevy::core_pipeline::clear_color::ClearColorConfig;
extern crate midir;
use bevy::color::palettes::css::*;
// use bevy::ecs::query::QueryFilter;
// use bevy::gizmos::config;
use bevy::core_pipeline::{
    bloom::{BloomCompositeMode, BloomSettings},
    tonemapping::Tonemapping,
};
use bevy::prelude::*;
use bevy::sprite::MaterialMesh2dBundle;
use bevy::utils::HashMap;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use midir::{Ignore, MidiInput};
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::{stdin, stdout, Write};
use std::sync::{Arc, Mutex};
// use std::thread::JoinHandle;
use bevy::window::WindowResized;

use std::time;

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

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let args2 = args.clone();
    let t1 = std::thread::spawn(move || match run(&args2) {
        Ok(_) => (),
        Err(err) => println!("Error: {}", err),
    });
    // println!("Hello, world!");
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin)
        .add_systems(Startup, setup)
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(Configuration {
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
        })
        .insert_resource(NotePlacemnt {
            notes_position: HashMap::new(),
            blacks: Vec::new(),
            blacks2: Vec::new(),
        })
        .insert_resource(NoteMeshes {
            note_handles: Vec::new(),
        })
        .insert_resource(NoteOffset {
            offset: 0f32,
            whites_count: 52f32,
        })
        .insert_resource(KeyboardNoteMeshes {
            keyboard_handles: Vec::new(),
        })
        .add_systems(Update, window_resize_system)
        .add_systems(Update, ui_config_system)
        .add_systems(Startup, note_placement)
        .insert_resource(ActiveNotes {
            active_notes: Vec::new(),
        })
        //.add_systems(Startup, draw_keyboard)
        // .add_systems(Update, move_keyboard)
        .add_systems(Update, move_notes)
        .add_systems(Update, notes_spawner)
        .add_systems(Update, grow_notes)
        .add_systems(Update, animate_keyboard)
        .run();
}
fn window_resize_system(
    mut resize_reader: EventReader<WindowResized>,
    config: Res<Configuration>,
    note_offset: ResMut<NoteOffset>,
    commands: Commands,
    window: Query<&Window>,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<ColorMaterial>>,
    notes_placement: Res<NotePlacemnt>,
    old_keys: Query<Entity, With<KeyboardElement>>,
    keyboard_note_meshes: ResMut<KeyboardNoteMeshes>,
) {
    for event in resize_reader.read() {
        println!(
            "Window resized to width: {} and height: {}",
            event.width, event.height
        );
        // note_offset.offset=count_whites(config.starting_note, config.ending_note, blacks)

        //generate keyboard
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
        break;
    }
}

#[derive(Resource)]
pub struct NoteOffset {
    pub offset: f32,
    pub whites_count: f32,
}

fn run(args: &Vec<String>) -> Result<(), Box<dyn Error>> {
    let mut ofile = File::create("info.txt").expect("unable to create file");
    ofile.write_all("".as_bytes()).expect("unable to write");

    let mut input = String::new();

    let mut is_debug: bool = false;

    if args.contains(&"debug".to_string()) || args.contains(&"d".to_string()) {
        println!("Debug Mode is on");
        is_debug = true;
    }

    let contents =
        fs::read_to_string("whitelist.txt").expect("Something went wrong reading the config file");
    let mut whitelisted_inputs: Vec<&str> = contents.split("\n").collect();
    whitelisted_inputs.pop();
    // let whitelisted_inputs_u8: Vec<u8> = whitelisted_inputs.into().map(|x| x.parse.unwrap());

    let whitelisted_inputs_u8: Vec<u8> = vec![128,144]; 
        //whitelisted_inputs
        //.into_iter()
        //.map(|x| x.parse().unwrap())
        //.collect();

    println!("Whitelisted inputs\n{:?}", whitelisted_inputs_u8);

    let mut midi_in = MidiInput::new("midir reading input")?;
    midi_in.ignore(Ignore::None);

    // Get an input port (read from console if multiple are available)
    let in_ports = midi_in.ports();
    let in_port = match in_ports.len() {
        0 => return Err("no input port found".into()),
        1 => {
            println!(
                "Choosing the only available input port: {}",
                midi_in.port_name(&in_ports[0]).unwrap()
            );
            &in_ports[0]
        }
        _ => {
            println!("\nAvailable input ports:");
            for (i, p) in in_ports.iter().enumerate() {
                println!("{}: {}", i, midi_in.port_name(p).unwrap());
            }
            print!("Please select input port:\n");
            let input_given: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));
            let input_giv = Arc::clone(&input_given);
            std::thread::spawn(move || loop {
                let given = input_giv.lock().unwrap();
                if *given {
                    return;
                }
                drop(given);
                println!("Please select input port: ");
                std::thread::sleep(time::Duration::from_secs(3));
            });

            stdout().flush()?;
            let mut input = String::new();
            stdin().read_line(&mut input)?;

            let mut given = input_given.lock().unwrap();
            *given = true;

            in_ports
                .get(input.trim().parse::<usize>()?)
                .ok_or("invalid input port selected")?
        }
    };

    println!("\nOpening connection");
    let in_port_name = midi_in.port_name(in_port)?;

    let mut active_notes: Vec<i32> = Vec::new();

    let _conn_in = midi_in.connect(
        in_port,
        "midir-read-input",
        move |_stamp, message, _| {
            if message.len() == 3usize {
                //println!("{:?}", message);
                if message[1] != 1 {
                    if is_debug == true {
                        println!("{:?}", message);
                    }
                    if whitelisted_inputs_u8.contains(&message[0]) {
                        // this checks if the midi input is a note
                        handle_note(message[1].into(), &mut active_notes);
                        write_notes_to_file(&active_notes);
                        //println!("{:?}", active_notes);
                    }
                } else {
                    display_board(&active_notes);
                    //println!("{:?}", active_notes);
                }
            }
        },
        (),
    )?;

    println!(
        "Connection open, reading input from '{}' (press enter to exit) ...",
        in_port_name
    );

    input.clear();
    stdin().read_line(&mut input)?; // wait for next enter key press

    println!("Closing connection");
    Ok(())
}
fn write_notes_to_file(act_notes: &Vec<i32>) {
    let mut out: String = String::new();

    for note in act_notes {
        out.push_str(&note.to_string());
        out.push_str("\n");
    }

    let mut ofile = File::create("info.txt").expect("unable to create file");
     ofile.write_all(out.as_bytes()).expect("unable to write");
}
fn display_board(act_notes: &Vec<i32>) {
    for i in STARTING_NOTE..=ENDING_NOTE {
        if act_notes.contains(&(i as i32)) {
            print!("X");
        } else {
            print!(" ");
        }
    }
    println!();
     write_notes_to_file(act_notes);
}
fn handle_note(note: i32, act_notes: &mut Vec<i32>) {
    // println!("{}",note);
    let mut had_note = false;
    for i in 0..act_notes.len() {
        if act_notes[i] == note {
            act_notes.remove(i);
            had_note = true;
            break;
        }
    }
    if had_note == false {
        act_notes.push(note);
    }
}
fn setup(mut commands: Commands) {
    // commands.spawn((
    //     Camera2dBundle {
    //         camera_2d: Camera2d {
    //             // clear_color: ClearColorConfig::Custom(Color::BLACK),
    //             // cle

    //             // tonemapping: tonemapping::tonymcmapface, // 2. using a tonemapper that desaturates to white is recommended
    //             ..default()
    //         },
    //         // BloomSettings::default(), // 3. enable bloom for the camera
    //         ..default()
    //     },
    //     bevy::core_pipeline::bloom::BloomSettings::default(),
    // ));
    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                hdr: true, // 1. hdr is required for bloom
                ..default()
            },
            tonemapping: Tonemapping::TonyMcMapface, // 2. using a tonemapper that desaturates to white is recommended
            ..default()
        },
        BloomSettings {
            intensity: 0.5,
            // composite_mode: BloomCompositeMode::Additive,
            composite_mode: BloomCompositeMode::EnergyConserving,
            low_frequency_boost: 0.3,
            high_pass_frequency: 0.694,
            // prefilter_settings
            ..Default::default()
        },
    ));
}

fn move_notes(
    mut notes: Query<&mut Transform, With<Note>>,
    time: Res<Time>,
    mut edges: Query<&mut Transform, (With<NoteEdge>, Without<Note>)>,
    config: Res<Configuration>,
) {
    for mut transform in &mut notes {
        transform.translation.y += config.note_speed * time.delta_seconds();
    }
    for mut transform in &mut edges {
        transform.translation.y += config.note_speed * time.delta_seconds();
    }
}
fn grow_notes(
    mut notes: Query<(&mut Transform, &Note)>,
    time: Res<Time>,
    mut meshes: ResMut<Assets<Mesh>>,
    active_notes: Res<ActiveNotes>,
    mut note_meshes: ResMut<NoteMeshes>,
) {
    for (note, handle) in &mut note_meshes.note_handles.iter() {
        // info!("{:?},{:?}", note, handle);
        let mesh = meshes.get_mut(handle).unwrap();
        // mesh.insert_attribute(mesh::ATTRIBUT)
        // let vertex_colors: Vec<[f32; 4]> = vec![
        //     Color::RED.as_rgba_f32(),
        //     Color::RED.as_rgba_f32(),
        //     Color::BLUE.as_rgba_f32(),
        //     Color::BLUE.as_rgba_f32(),
        // ];

        // // Insert the vertex colors as an attribute
        // mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, vertex_colors.clone());

        // info!("{:?}", mesh);
    }
    // info!("{:?}", active_notes.active_notes);

    // for (mut transform, note) in &mut notes.iter_mut() {
    //     //grow em
    //     info!("{:?}, {}", note.note_id, note.id);
    //     // no logs :(
    //     if active_notes.active_notes.contains(&(note.note_id)) {
    //         // transform.scale()
    //         transform.translation.y -= time.delta_seconds() * config.note_speed / 2.;
    //         transform.scale = Vec3::new(
    //             1.,
    //             (time.delta_seconds() * config.note_speed) / 20. + transform.scale.y,
    //             1.,
    //         );
    //         info!("yes");
    //     }
    // }
}

fn notes_spawner(
    mut commands: Commands,
    mut active_notes: ResMut<ActiveNotes>,
    window: Query<&Window>,
    mut transform_notes: Query<(&mut Transform, &Note)>,
    time: Res<Time>,
    notes_placement: Res<NotePlacemnt>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut note_meshes: ResMut<NoteMeshes>,
    config: Res<Configuration>,
    note_offset: Res<NoteOffset>,
) {
    let contents = fs::read_to_string("info.txt").expect("Something went wrong reading the file");
    let mut notes_string: Vec<&str> = contents.split("\n").collect();
    notes_string.pop();
    // println!("{:?}",contents);

    let notes: Vec<i32>;
    let res = &window.single().resolution;
    let n_width = res.width() / note_offset.whites_count;

    if !notes_string.is_empty() {
        notes = notes_string
            .iter()
            .map(|x| x.parse::<i32>().unwrap())
            .collect();
    } else {
        notes = Vec::new();
    }

    for i in 0..notes.len() {
        let nn_width = if notes_placement.blacks.contains(&(notes[i] as i8)) {
            res.width() / note_offset.whites_count * 0.694
        } else {
            res.width() / note_offset.whites_count - 2.
        };

        if !active_notes.active_notes.contains(&notes[i]) {
            // let mesh = Capsule2d {
            //     radius: nn_width / 2.,
            //     half_length: 10.,
            //     // half_size: Vec2::new(nn_width / 2., 10.),
            //     ..Default::default()
            // };
            let mesh = Rectangle {
                half_size: Vec2::new(nn_width / 2. - 2., 1.),
                ..default()
            };
            let vertex_colors: Vec<[f32; 4]> = vec![
                LinearRgba::from(config.white_color_top).to_f32_array(),
                LinearRgba::from(config.white_color_top).to_f32_array(),
                LinearRgba::from(config.white_color_bottom).to_f32_array(),
                LinearRgba::from(config.white_color_bottom).to_f32_array(),
            ];
            let vertex_colors_blacks: Vec<[f32; 4]> = vec![
                LinearRgba::from(config.black_color_top).to_f32_array(),
                LinearRgba::from(config.black_color_top).to_f32_array(),
                LinearRgba::from(config.black_color_bottom).to_f32_array(),
                LinearRgba::from(config.black_color_bottom).to_f32_array(),
            ];

            // mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, vertex_colors.clone()); // mesh.insert_attribute(Mesh::)
            // Insert the vertex colors as an attribute
            let mesh_handle: Handle<Mesh> = meshes.add(mesh);

            meshes
                .get_mut(&mesh_handle.clone())
                .unwrap()
                .insert_attribute(
                    Mesh::ATTRIBUTE_COLOR,
                    if notes_placement.blacks.contains(&(notes[i] as i8)) {
                        vertex_colors_blacks
                    } else {
                        vertex_colors
                    },
                );

            // note_meshes.note_meshes.push(&notes[i], mesh);
            // mesh.is_strong();
            // info!("{}", mesh.is_strong());
            // info!("{:?}", mesh);
            // info!("{:?}", meshes.ids().collect::<Vec<_>>());
            commands.spawn((
                MaterialMesh2dBundle {
                    mesh: meshes
                        .add(Ellipse {
                            half_size: Vec2::new(nn_width / 2. - 2., nn_width / 4.),
                            ..default()
                        })
                        .into(),
                    material: materials.add(
                        if notes_placement.blacks.contains(&(notes[i] as i8)) {
                            Color::from(config.black_color_top)
                        } else {
                            Color::from(config.white_color_top)
                        },
                    ),
                    transform: Transform::from_xyz(
                        notes_placement
                            .notes_position
                            .get(&(notes[i] as i8))
                            .unwrap() as &f32
                            * n_width
                            // / 88.
                            - res.width() / 2.
                            - 12 as f32 * n_width
                            + n_width / 2.
                            - note_offset.offset * n_width,
                        -res.height() / 2. + config.keyboard_height,
                        if notes_placement.blacks.contains(&(notes[i] as i8)) {
                            0.5
                        } else {
                            -0.5
                        },
                    ),

                    ..default()
                },
                NoteEdge {},
            ));
            commands.spawn((
                MaterialMesh2dBundle {
                    // mesh: meshes.add(Capsule2d::new(nn_width / 2., 15.)).into(),
                    mesh: mesh_handle.clone().into(),
                    material: materials.add(ColorMaterial::default()),
                    // material: materials.add(
                    //     if notes_placement.blacks.contains(&(notes[i] as i8)) {
                    //         Color::RED
                    //     } else {
                    //         Color::WHITE
                    //     },
                    // ),
                    transform: Transform::from_xyz(
                        notes_placement
                            .notes_position
                            .get(&(notes[i] as i8))
                            .unwrap() as &f32
                            * n_width
                            // / 88.
                            - res.width() / 2.
                            - 12 as f32 * n_width
                            + n_width / 2.
                            - note_offset.offset * n_width,
                        -res.height() / 2. + config.keyboard_height,
                        if notes_placement.blacks.contains(&(notes[i] as i8)) {
                            1.
                        } else {
                            0.
                        },
                    ),
                    ..default()
                },
                Note {
                    x: 0.,
                    y: 0.,
                    note_id: notes[i],
                    id: 0,
                },
            ));
            // note_meshes.note_handles.push((notes[i], mesh_handle));
        }
    }
    // transform_notes = transform_notes.into_iter().rev();
    for i in 0..active_notes.active_notes.len() {
        if notes.contains(&active_notes.active_notes[i]) {
            //grow note
            let mut n = 0;
            let mut nn = 0;
            for (_, note) in &mut transform_notes.iter_mut() {
                if note.note_id == active_notes.active_notes[i] {
                    n = nn;
                }
                nn += 1;
            }
            nn = 0;
            for (mut transform, _) in &mut transform_notes.iter_mut() {
                if nn == n {
                    transform.translation.y -= time.delta_seconds() * config.note_speed / 2.;
                    // transform.scale = Vec3::new(10., 1., 1.);transform.translation.y -= time.delta_seconds() * config.note_speed / 2.;
                    transform.scale = Vec3::new(
                        1.,
                        (time.delta_seconds() * config.note_speed) / 2. + transform.scale.y,
                        1.,
                    );
                    break;
                }
                nn += 1;
            }
        }
    }
    for i in 0..active_notes.active_notes.len() {
        if !notes.contains(&active_notes.active_notes[i]) {
            let nn_width = if notes_placement
                .blacks
                .contains(&(active_notes.active_notes[i] as i8))
            {
                res.width() / note_offset.whites_count * 0.694
            } else {
                res.width() / note_offset.whites_count - 2.
            };

            commands.spawn((
                MaterialMesh2dBundle {
                    mesh: meshes
                        .add(Ellipse {
                            // half_size: Vec2::new(nn_width / 2. - 2., 5.),
                            half_size: Vec2::new(nn_width / 2. - 2., nn_width / 4.),
                            ..default()
                        })
                        .into(),
                    material: materials.add(
                        if notes_placement
                            .blacks
                            .contains(&(active_notes.active_notes[i] as i8))
                        {
                            Color::from(config.black_color_bottom)
                        } else {
                            Color::from(config.white_color_bottom)
                        },
                    ),
                    transform: Transform::from_xyz(
                        notes_placement
                            .notes_position
                            .get(&(active_notes.active_notes[i] as i8))
                            .unwrap() as &f32
                            * n_width
                            // / 88.
                            - res.width() / 2.
                            - 12 as f32 * n_width
                            + n_width / 2.
                            - note_offset.offset * n_width,
                        -res.height() / 2. + config.keyboard_height,
                        if notes_placement
                            .blacks
                            .contains(&(active_notes.active_notes[i] as i8))
                        {
                            0.5
                        } else {
                            -0.5
                        },
                    ),

                    ..default()
                },
                NoteEdge {},
            ));
        }
    }
    active_notes.active_notes = notes.clone();
}

#[derive(Component, Debug)]
pub struct Note {
    pub x: f32,
    pub y: f32,
    pub note_id: i32,
    pub id: i32,
}
#[derive(Component, Debug)]
pub struct NoteEdge {
    // pub x: f32,
}
#[derive(Resource)]
pub struct ActiveNotes {
    active_notes: Vec<i32>,
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
    // pub vertex_colors: Vec<[f32; 4]>,
    // pub vertex_colors_blacks: Vec<[f32; 4]>,
    // pub vertex_colors_keyboard: Vec<[f32; 4]>,
    // pub vertex_colors_keyboard_blacks: Vec<[f32; 4]>,
    // pub vertex_colors: Vec<[f32; 4]>,
    // pub vertex_colors_blacks: Vec<[f32; 4]>,
}
#[derive(Resource)]
pub struct NoteMeshes {
    pub note_handles: Vec<(i32, Handle<Mesh>)>,
}
#[derive(Resource)]
pub struct KeyboardNoteMeshes {
    pub keyboard_handles: Vec<(i32, Handle<Mesh>)>,
}
#[derive(Resource, Debug)]
pub struct NotePlacemnt {
    pub notes_position: HashMap<i8, f32>,
    pub blacks: Vec<i8>,
    pub blacks2: Vec<i8>,
}
pub fn note_placement(mut notes_placement: ResMut<NotePlacemnt>) {
    let mut blacks = Vec::new();
    let mut n = 0;
    for i in 0..10 {
        for j in 0..12 {
            if [1, 3, 6, 8, 10].contains(&j) {
                blacks.push(i * 12 + j);
                notes_placement
                    .notes_position
                    .insert((i * 12 + j) as i8, n as f32 - 0.5);
            } else {
                notes_placement
                    .notes_position
                    .insert((i * 12 + j) as i8, n as f32);

                n += 1;
            }
        }
    }
    println!("{:?}  {:?}", blacks, notes_placement);

    notes_placement.blacks = blacks;

    let mut blacks2 = Vec::new();
    blacks2.push(1);
    for i in 0..=88 {
        let x = i % 12;
        if [1, 3, 6, 8, 10].contains(&x) {
            blacks2.push(i + 3);
        }
    }
    println!("{:?}", &blacks2);
    notes_placement.blacks2 = blacks2;
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
fn ui_config_system(
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

fn count_whites(start: i8, end: i8, blacks: &Vec<i8>) -> f32 {
    let mut count = 0f32;
    // if start==end{return 0f32;}
    for i in start..end {
        if !blacks.contains(&i) {
            count = count + 1.;
        }
    }
    println!("{} {} {}", start, end, count);
    count
}

pub fn draw_keyboard(
    config: &Configuration,
    mut commands: Commands,
    // mut active_notes: ResMut<ActiveNotes>,
    window: Query<&Window>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    notes_placement: Res<NotePlacemnt>,
    old_keys: Query<Entity, With<KeyboardElement>>,
    mut keyboard_note_meshes: ResMut<KeyboardNoteMeshes>,
    note_offset: &NoteOffset,
) {
    for entity in &old_keys {
        commands.entity(entity).despawn();
    }
    keyboard_note_meshes.keyboard_handles.clear();

    let res = &window.single().resolution;
    let n_width = res.width() / note_offset.whites_count;
    for i in config.starting_note..=config.ending_note {
        let is_white = if notes_placement.blacks.contains(&i) {
            false
        } else {
            true
        };
        let mesh = Rectangle::new(
            if is_white {
                n_width - 2.
            } else {
                n_width * 0.45
            },
            if is_white {
                config.keyboard_height
            } else {
                config.keyboard_height * 3. / 5.
            },
        );

        let white_colors: Vec<[f32; 4]> = vec![
            LinearRgba::from(config.keyboard_white_color).to_f32_array(),
            LinearRgba::from(config.keyboard_white_color).to_f32_array(),
            LinearRgba::from(config.keyboard_white_color).to_f32_array(),
            LinearRgba::from(config.keyboard_white_color).to_f32_array(),
        ];

        let black_colors: Vec<[f32; 4]> = vec![
            LinearRgba::from(config.keyboard_black_color).to_f32_array(),
            LinearRgba::from(config.keyboard_black_color).to_f32_array(),
            LinearRgba::from(config.keyboard_black_color).to_f32_array(),
            LinearRgba::from(config.keyboard_black_color).to_f32_array(),
        ];

        let mesh_handle: Handle<Mesh> = meshes.add(mesh);
        meshes
            .get_mut(&mesh_handle.clone())
            .unwrap()
            .insert_attribute(
                Mesh::ATTRIBUTE_COLOR,
                if notes_placement.blacks.contains(&(i as i8)) {
                    black_colors
                } else {
                    white_colors
                },
            );
        commands.spawn((
            MaterialMesh2dBundle {
                // mesh: meshes
                //     .add(Rectangle::new(
                //         if is_white {
                //             n_width - 2.
                //         } else {
                //             res.width() / 100.
                //         },
                //         if is_white {
                //             config.keyboard_height
                //         } else {
                //             config.keyboard_height * 3. / 5.
                //         },
                //     ))
                //     .into(),
                mesh: mesh_handle.clone().into(),

                // material: materials.add(material),
                material: materials.add(ColorMaterial::default()),

                transform: Transform::from_xyz(
                    notes_placement
                            .notes_position
                            .get(&(i as i8))
                            .unwrap() as &f32
                            * n_width
                            // / 88.
                            - res.width() / 2.
                            - 12 as f32 * n_width
                            -note_offset.offset*n_width
                        + n_width / 2.,
                    //    -res.height() / 2. + config.keyboard_height,
                    if is_white {
                        -res.height() / 2. + config.keyboard_height / 2.
                    } else {
                        -res.height() / 2. + config.keyboard_height * 0.694
                    },
                    if is_white { 1.2 } else { 1.5 },
                ),
                ..default()
            },
            KeyNote {
                active: false,
                id: i as u8,
                white: is_white,
            },
            KeyboardElement {},
        ));
        //clear before adding

        keyboard_note_meshes
            .keyboard_handles
            .push((i as i32, mesh_handle));
        commands.spawn((
            MaterialMesh2dBundle {
                mesh: meshes.add(Rectangle::new(res.width(), 6.)).into(),
                material: materials.add(ColorMaterial {
                    color: config.keyboard_felt_color.into(),
                    ..Default::default()
                }),
                transform: Transform::from_xyz(
                    0., //-res.width() / 2.,
                    -res.height() / 2. + config.keyboard_height,
                    2.,
                ),
                ..default()
            },
            KeyboardElement {},
        ));
    }
}
pub fn move_keyboard(
    mut keys: Query<(&mut Transform, &KeyNote)>,
    window: Query<&Window>,
    notes_placement: Res<NotePlacemnt>,
    config: Res<Configuration>,
) {
    for (mut transform, key_note) in &mut keys {
        let res = &window.single().resolution;
        let n_width = res.width() / 52.0;
        // let nn_width = n_width - 2.;

        // let nn_width = if notes_placement.blacks.contains(&(key_note.id as i8)) {
        //     res.width() / 72.0
        // } else {
        //     res.width() / 52.0 - 2.
        // };

        transform.translation = Vec3::new(
            notes_placement
                            .notes_position
                            .get(&(key_note.id as i8))
                            .unwrap() as &f32
                            * n_width
                            // / 88.
                            - res.width() / 2.
                            - 12 as f32 * n_width
                + n_width / 2.,
            //    -res.height() / 2. + config.keyboard_height,
            if key_note.white {
                -res.height() / 2. + config.keyboard_height / 2.
            } else {
                -res.height() / 2. + config.keyboard_height / (4. / 3.)
            },
            if notes_placement.blacks.contains(&(key_note.id as i8)) {
                1.5
            } else {
                1.2
            },
        );
    }
}
#[derive(Component, Debug)]
pub struct KeyNote {
    active: bool,
    white: bool,
    id: u8,
    // handle: Handle,
}
#[derive(Component, Debug)]
pub struct KeyboardElement {}

pub fn animate_keyboard(
    mut keys: Query<(&mut Transform, &mut KeyNote)>,
    active_notes: Res<ActiveNotes>,
    // materials: ResMut<Assets<StandardMaterial>>,
    keyboard_note_meshes: Res<KeyboardNoteMeshes>,
    mut meshes: ResMut<Assets<Mesh>>,
    config: Res<Configuration>,
    notes_placement: Res<NotePlacemnt>,
) {
    let white_colors: Vec<[f32; 4]> = vec![
        LinearRgba::from(config.keyboard_white_color).to_f32_array(),
        LinearRgba::from(config.keyboard_white_color).to_f32_array(),
        LinearRgba::from(config.keyboard_white_color).to_f32_array(),
        LinearRgba::from(config.keyboard_white_color).to_f32_array(),
    ];
    let white_active_colors: Vec<[f32; 4]> = vec![
        LinearRgba::from(config.keyboard_white_color_active).to_f32_array(),
        LinearRgba::from(config.keyboard_white_color_active).to_f32_array(),
        LinearRgba::from(config.keyboard_white_color_active).to_f32_array(),
        LinearRgba::from(config.keyboard_white_color_active).to_f32_array(),
    ];
    let black_colors: Vec<[f32; 4]> = vec![
        LinearRgba::from(config.keyboard_black_color).to_f32_array(),
        LinearRgba::from(config.keyboard_black_color).to_f32_array(),
        LinearRgba::from(config.keyboard_black_color).to_f32_array(),
        LinearRgba::from(config.keyboard_black_color).to_f32_array(),
    ];
    let black_active_colors: Vec<[f32; 4]> = vec![
        LinearRgba::from(config.keyboard_black_color_active).to_f32_array(),
        LinearRgba::from(config.keyboard_black_color_active).to_f32_array(),
        LinearRgba::from(config.keyboard_black_color_active).to_f32_array(),
        LinearRgba::from(config.keyboard_black_color_active).to_f32_array(),
    ];

    for (_transform, mut key_note) in &mut keys {
        if active_notes.active_notes.contains(&(key_note.id as i32)) {
            // println!("key{:?}", key_note);
            // for(id,mut handle)in keyboard_note_meshes.keyboard_handles{
            if !key_note.active {
                for (note, handle) in &mut keyboard_note_meshes.keyboard_handles.iter() {
                    if note == &(key_note.id as i32) {
                        let mesh = meshes.get_mut(handle).unwrap();
                        if notes_placement.blacks.contains(&(key_note.id as i8)) {
                            mesh.insert_attribute(
                                Mesh::ATTRIBUTE_COLOR,
                                black_active_colors.clone(),
                            );
                        } else {
                            mesh.insert_attribute(
                                Mesh::ATTRIBUTE_COLOR,
                                white_active_colors.clone(),
                            );
                        }
                        key_note.active = true;

                        // let material=materials.get_mut(handle).unwrap();
                        // println!("{:?}",material);
                    }
                    // for (note, handle) in &mut note_meshes.note_handles.iter() {
                    // info!("{:?},{:?}", note, handle);
                }
            }
        } else if key_note.active {
            // println!("shall not be active");
            for (note, handle) in &mut keyboard_note_meshes.keyboard_handles.iter() {
                if note == &(key_note.id as i32) {
                    let mesh = meshes.get_mut(handle).unwrap();
                    if notes_placement.blacks.contains(&(key_note.id as i8)) {
                        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, black_colors.clone());
                    } else {
                        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, white_colors.clone());
                    }
                    key_note.active = false;

                    // let material=materials.get_mut(handle).unwrap();
                    // println!("{:?}",material);
                }
                // for (note, handle) in &mut note_meshes.note_handles.iter() {
                // info!("{:?},{:?}", note, handle);
            }
        }
    }
}
