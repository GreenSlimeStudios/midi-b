// use bevy::core_pipeline::clear_color::ClearColorConfig;
extern crate midir;

use bevy::prelude::*;
use midir::{Ignore, MidiInput};
use std::error::Error;
use std::io::{stdin, stdout, Write};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

// use midir::{Ignore, MidiInput};
use std::fs;
use std::fs::File;

use std::time;

const NOTE_SPEED: f32 = 200.;
const NOTE_WIDTH: f32 = 15.;

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
        .add_systems(Startup, setup)
        .insert_resource(ActiveNotes {
            active_notes: Vec::new(),
        })
        .add_systems(Update, move_notes)
        .add_systems(Update, notes_spawner)
        .run();
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

    let whitelisted_inputs_u8: Vec<u8> = whitelisted_inputs
        .into_iter()
        .map(|x| x.parse().unwrap())
        .collect();

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
                    }
                } else {
                    display_board(&active_notes);
                    // println!("{:?}",active_notes);
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
    for i in 21..=108 {
        if act_notes.contains(&(i as i32)) {
            print!("X");
        } else {
            print!(" ");
        }
    }
    println!();
    // write_notes_to_file(act_notes);
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
    commands.spawn(Camera2dBundle {
        // camera_2d: Camera2d {
        //     clear_color: ClearColorConfig::Custom(Color::GREEN),
        // },
        ..default()
    });
    // commands.spawn((
    //     SpriteBundle {
    //         sprite: Sprite {
    //             custom_size: Some(Vec2::new(20.0, 100.0)),
    //             ..default()
    //         },
    //         transform: Transform::from_xyz(50., 0., 0.),
    //         ..default()
    //     },
    //     Note {
    //         x: 0.,
    //         y: 0.,
    //         note_id: 46,
    //         id: 0,
    //     },
    // ));
}

fn move_notes(mut notes: Query<(&mut Transform, &Note)>, time: Res<Time>) {
    for (mut transform, _) in &mut notes {
        transform.translation.y += NOTE_SPEED * time.delta_seconds();
    }
}
fn notes_spawner(
    mut commands: Commands,
    mut active_notes: ResMut<ActiveNotes>,
    window: Query<&Window>,
    mut transform_notes: Query<(&mut Transform, &Note, &mut Sprite)>,
    time: Res<Time>,
) {
    let contents = fs::read_to_string("info.txt").expect("Something went wrong reading the file");
    let mut notes_string: Vec<&str> = contents.split("\n").collect();
    notes_string.pop();
    // println!("{:?}",contents);

    let notes: Vec<i32>;

    if !notes_string.is_empty() {
        notes = notes_string
            .iter()
            .map(|x| x.parse::<i32>().unwrap())
            .collect();
    } else {
        notes = Vec::new();
    }

    for i in 0..notes.len() {
        if !active_notes.active_notes.contains(&notes[i]) {
            commands.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        custom_size: Some(Vec2::new(NOTE_WIDTH, 1.0)),
                        ..default()
                    },
                    transform: Transform::from_xyz(
                        notes[i] as f32 * NOTE_WIDTH - window.single().resolution.width() / 2.,
                        -window.single().resolution.height() / 2.,
                        0.,
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
        }
    }

    // transform_notes = transform_notes.into_iter().rev();
    for i in 0..active_notes.active_notes.len() {
        if notes.contains(&active_notes.active_notes[i]) {
            //grow note
            let mut n = 0;
            let mut nn = 0;
            for (mut transform, note, mut sprite) in &mut transform_notes.iter_mut() {
                if note.note_id == active_notes.active_notes[i] {
                    n = nn;
                }
                nn += 1;
            }
            nn = 0;
            for (mut transform, note, mut sprite) in &mut transform_notes.iter_mut() {
                if nn == n {
                    sprite.custom_size = Some(Vec2 {
                        x: sprite.custom_size.unwrap().x,
                        y: sprite.custom_size.unwrap().y + time.delta_seconds() * NOTE_SPEED,
                    });
                    transform.translation.y -= time.delta_seconds() * NOTE_SPEED / 2.;
                    break;
                }
                nn += 1;
            }
        }
    }
    active_notes.active_notes = notes.clone();
}

#[derive(Component)]
pub struct Note {
    pub x: f32,
    pub y: f32,
    pub note_id: i32,
    pub id: i32,
}
#[derive(Resource)]
pub struct ActiveNotes {
    active_notes: Vec<i32>,
}