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

use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::{stdin, stdout, Write};
use std::sync::{Arc, Mutex};
// use std::thread::JoinHandle;
use bevy::window::WindowResized;

pub mod midi_flow;
pub mod ui_config;
pub mod keyboard;
pub mod config;
use keyboard::*;
use config::Configuration;



fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let args2 = args.clone();
    let _t1 = std::thread::spawn(move || match run(&args2) {
        Ok(_) => (),
        Err(err) => println!("Error: {}", err),
    });
    // println!("Hello, world!");
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ui_config::UiPlugin)
        .add_plugins(keyboard::KeyboardPlugin)
        .add_plugins(config::ConfigPlugin)
        .add_systems(Startup, setup)
        .insert_resource(ClearColor(Color::BLACK))
     
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

        .add_systems(Update, window_resize_system)
        // .add_systems(Update, ui_config_system)
        .add_systems(Startup, note_placement)
        .insert_resource(ActiveNotes {
            active_notes: Vec::new(),
        })
        //.add_systems(Startup, draw_keyboard)
        // .add_systems(Update, move_keyboard)
        .add_systems(Update, move_notes)
        .add_systems(Update, notes_spawner)
        .add_systems(Update, grow_notes)
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
    midi_flow::midi_flow_stream(args)
   
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

#[derive(Resource)]
pub struct NoteMeshes {
    pub note_handles: Vec<(i32, Handle<Mesh>)>,
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