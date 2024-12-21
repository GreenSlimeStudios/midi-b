use crate::*;
use bevy::sprite::MaterialMesh2dBundle;

pub struct KeyboardPlugin;

impl Plugin for KeyboardPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(KeyboardNoteMeshes {
            keyboard_handles: Vec::new(),
        })
        .add_systems(Update, animate_keyboard);
    }
}
#[derive(Resource)]
pub struct KeyboardNoteMeshes {
    pub keyboard_handles: Vec<(i32, Handle<Mesh>)>,
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

#[derive(Component, Debug)]
pub struct KeyNote {
    pub active: bool,
    pub white: bool,
    pub id: u8,
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
