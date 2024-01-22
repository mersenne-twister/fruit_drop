// Fruit-Drop, written by Iris, based off https://nerdyteachers.com/PICO-8/Bitesize_Games/?tutorial=2.
// Made in bevy, which uses ECS:
// entites are instantiated objects, components are structs (rust structs are effectively classes,
// implementing encapsulation, interfaces, (no implementation inheritance), polymorphism, etc)
// that derive the 'Component' attribute, and systems are functions. see https://bevyengine.org/ for details.
//
// Known issues:
// * is really janky with it's size. currently does not have a fixed aspect ratio. due to this
// we have to disable fullscreen mode and window resizing.
// * uses the rolling stones mouth, which probably isn't good for copyright
// * when the player loses, the game goes into an infinite loop. solution: use states to
// pause game, then slap some GAME OVER text on the screen when the player loses

#![allow(clippy::type_complexity)]

use bevy::{prelude::*, time::common_conditions, utils::Duration, window::EnabledButtons};
use rand::Rng;

#[derive(Component)]
struct Player;

#[derive(Resource)]
struct PlayerInput {
    move_left: bool,
    move_right: bool,
}
impl PlayerInput {
    fn default() -> Self {
        Self {
            move_left: false,
            move_right: false,
        }
    }
}

/// Component to hold the score. This gets inserted into the text entity.
#[derive(Component)]
struct Score {
    score: u32,
}
impl Score {
    fn default() -> Self {
        Self { score: 0 }
    }
}

#[derive(Event)]
struct ScoreEvent;

#[derive(Event)]
struct GameOverEvent;

#[derive(Component)]
struct Fruit;

/// indicates the number of different fruit sprites. uses "fruit[sprite_num].png" naming.
const NUM_FRUIT: usize = 6;

fn main() {
    App::new() //initialize our window settings
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Fruit Drop".into(),
                resolution: (750., 750.).into(),
                resizable: false,
                enabled_buttons: EnabledButtons {
                    maximize: false,
                    ..default()
                },
                ..default()
            }),
            ..default()
        }))
        .insert_resource(Time::<Fixed>::from_seconds(0.017)) // unified timestep, used here
        // to move at a constant speed (FixedUpdate)
        .insert_resource(PlayerInput::default())
        .add_event::<GameOverEvent>()
        .add_event::<ScoreEvent>()
        .add_systems(Startup, setup)
        .add_systems( // these get run constantly
            Update,
            (
                get_input,
                score_increase.after(fruit_movement),
                game_over.after(fruit_movement),
            ),
        )
        .add_systems(
            FixedUpdate,
            ( // these get run every 0.017 seconds
                player_movement.after(get_input),
                fruit_movement.after(player_movement),
            ),
        )
        .add_systems(
            Update, //this gets run every second
            spawn_fruit.run_if(common_conditions::on_timer(Duration::from_secs(1))),
        )
        .run();
}

/// initialise cameras and sprites
fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    commands
        .spawn(SpriteBundle {
            texture: asset_server.load("mouth.png"),
            sprite: Sprite {
                custom_size: Some(Vec2 { x: 64., y: 64. }),
                ..default()
            },
            transform: Transform {
                translation: Vec3 {
                    // indexed from the middle of the screen
                    x: 0.,
                    y: -100.,
                    z: 2.,
                },
                ..default()
            },
            ..default()
        })
        .insert(Player);

    commands
        .spawn(Text2dBundle { // the text to show the score
            text: Text {
                sections: vec![TextSection::new(
                    String::from("Score: "),
                    TextStyle {
                        font_size: 30.,
                        color: Color::rgb(1., 1., 1.),
                        ..default()
                    },
                )],
                ..default()
            },
            transform: Transform {
                translation: Vec3 {
                    x: -305.,
                    y: 345.,
                    z: 3.,
                },
                ..default()
            },
            ..default()
        })
        .insert(Score::default());

    commands.spawn(SpriteBundle { // this holds the floor.
        sprite: Sprite {
            color: Color::rgb(0.5, 0.8, 0.3),
            custom_size: Some(Vec2 { x: 750., y: 275. }),
            ..default()
        },
        transform: Transform {
            translation: Vec3 {
                x: 0.,
                y: -270.,
                z: 1.,
            },
            ..default()
        },
        ..default()
    });
}

/// spawns a fruit entity at slightly above the top of the screen
fn spawn_fruit(mut commands: Commands, asset_server: Res<AssetServer>) {
    let fruit_num = rand::thread_rng().gen_range(1..=NUM_FRUIT);

    commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2 { x: 64., y: 64. }),
                ..default()
            },
            // texture: asset_server.load(format!("fruit{}", fruit_num)),
            texture: asset_server.load(format!("fruit{}.png", fruit_num)),
            transform: Transform {
                translation: Vec3 {
                    x: rand::thread_rng().gen_range(-350.0..=350.0),
                    y: 385., //start above screen -385
                    z: 1.,
                },
                ..default()
            },
            ..default()
        })
        .insert(Fruit);
}

/// get player input and put it into the input resource
fn get_input(input: Res<Input<KeyCode>>, mut player_input: ResMut<PlayerInput>) {
    player_input.move_left = input.pressed(KeyCode::Left);

    player_input.move_right = input.pressed(KeyCode::Right);
}

/// use the input to move left or right
fn player_movement(
    mut player: Query<&mut Transform, With<Player>>,
    player_input: Res<PlayerInput>,
) {
    let mut player = player.iter_mut().next().unwrap(); //change to if let?

    if player_input.move_left && player.translation.x > -340. {
        player.translation.x -= 7.5;
    }
    // separate ifs so pressing both will cancel out
    if player_input.move_right && player.translation.x < 340. {
        player.translation.x += 7.5;
    }
}

/// if the player has scored, update the visible score
fn score_increase(
    mut score_reader: EventReader<ScoreEvent>,
    mut score: Query<(&mut Text, &mut Score)>,
) {
    if score_reader.read().next().is_some() {
        let (mut text, mut score) = score.iter_mut().next().unwrap();

        score.score += 1;

        text.sections[0].value = format!("Score: {}", score.score);
    }
}

/// move the fruit downwards, and check for collision with the player or ground
fn fruit_movement(
    mut commands: Commands,
    mut fruits: Query<(&mut Transform, Entity), (With<Fruit>, Without<Player>)>,
    player: Query<&Transform, (With<Player>, Without<Fruit>)>,
    score: Query<&Score>,
    mut game_over_writer: EventWriter<GameOverEvent>,
    mut score_writer: EventWriter<ScoreEvent>,
) {
    //increase fruit speed based on score
    let fruit_speed = 2. + ((score.iter().next().unwrap().score as f32) * 0.03);

    let player = player.iter().next().unwrap();

    for (mut fruit, fruit_ent) in fruits.iter_mut() {
        if fruit.translation.y < -100. {
            game_over_writer.send(GameOverEvent);
        }

        if (fruit.translation.y < -55.)
            && (fruit.translation.x > (player.translation.x - 50.))
            && (fruit.translation.x < (player.translation.x + 50.))
        {
            score_writer.send(ScoreEvent);
            commands.entity(fruit_ent).despawn();
        }

        fruit.translation.y -= fruit_speed;
    }
}

/// put the game into an infinite loop to stop the game
fn game_over(mut game_over_reader: EventReader<GameOverEvent>) {
    if game_over_reader.read().next().is_some() {
        loop {}
    }
}
