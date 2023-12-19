use bevy::{
    prelude::*,
    sprite::collide_aabb::{collide, Collision},
};
use rand::{thread_rng, Rng};

const PLAYER_INITIAL_POSITION: Vec3 = Vec3::new(-300., 300., 0.0);
const PLAYER_SIZE: Vec3 = Vec3::new(WALL_SIZE.x / 2.0, WALL_SIZE.y / 2.0, 0.0);
const PLAYER_SPEED: f32 = 4000. / BLOCK_NUM_X as f32;

const BLOCK_NUM_X: i32 = 41;
const BLOCK_NUM_Y: i32 = 41;
const WALL_SIZE: Vec3 = Vec3::new(
    600. / (BLOCK_NUM_X as f32 + 2.),
    600. / (BLOCK_NUM_Y as f32 + 2.),
    0.0,
);

//
const _: () = assert!(BLOCK_NUM_X % 2 != 0);
const _: () = assert!(BLOCK_NUM_Y % 2 != 0);

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, States, Component)]
enum Status {
    Shuffle,
    #[default]
    Disabled,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "labyrinth".into(),
                resolution: (900., 900.).into(),
                ..default()
            }),
            ..default()
        }))
        .add_state::<Status>()
        .add_systems(Update, bevy::window::close_on_esc)
        .add_systems(Startup, (setup, labyrinth_setup))
        .add_systems(OnEnter(Status::Shuffle), labyrinth_setup)
        .add_systems(
            Update,
            (move_player, wall_collision, button_system).run_if(in_state(Status::Disabled)),
        )
        .run();
}

#[derive(Component, Debug)]
struct Player;

#[derive(Component)]
struct Block;

#[derive(Bundle)]
struct BlockBundle {
    sprite_bundle: SpriteBundle,
    block: Block,
}

impl BlockBundle {
    fn new(translation: Vec2) -> BlockBundle {
        BlockBundle {
            sprite_bundle: SpriteBundle {
                transform: Transform {
                    translation: translation.extend(0.0),
                    scale: WALL_SIZE,
                    ..default()
                },
                ..default()
            },
            block: Block,
        }
    }
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle {
        camera: Camera { ..default() },
        ..default()
    });

    commands
        .spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                position_type: PositionType::Absolute,
                left: Val::Px(7.),
                top: Val::Px(7.),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent.spawn((
                ButtonBundle {
                    style: Style {
                        width: Val::Px(240.),
                        height: Val::Px(68.),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    ..default()
                },
                Status::Shuffle,
            ));
        });
}

fn labyrinth_setup(
    mut commands: Commands,
    entity_query: Query<Entity, Or<(With<Block>, With<Player>)>>,
    mut state: ResMut<NextState<Status>>,
) {
    for entity in &entity_query {
        commands.entity(entity).despawn();
    }

    commands.spawn((
        SpriteBundle {
            transform: Transform {
                translation: PLAYER_INITIAL_POSITION,
                scale: PLAYER_SIZE,
                ..default()
            },
            sprite: Sprite {
                color: Color::RED,
                ..default()
            },
            ..default()
        },
        Player,
    ));

    // 壁
    for y in 1..=BLOCK_NUM_Y + 4 {
        for x in 1..=BLOCK_NUM_X + 4 {
            if y == 1 || x == 1 || y == BLOCK_NUM_Y + 4 || x == BLOCK_NUM_X + 4 {
                let x = WALL_SIZE.x * (x as f32 - 2.)
                    - WALL_SIZE.x * (BLOCK_NUM_X as f32 / 2.)
                    - WALL_SIZE.x / 2.;
                let y = WALL_SIZE.y * (y as f32 - 2.)
                    - WALL_SIZE.y * (BLOCK_NUM_Y as f32 / 2.)
                    - WALL_SIZE.y / 2.;

                commands.spawn(BlockBundle::new(Vec2::new(x, y)));
            }
        }
    }

    for y in (1..=BLOCK_NUM_Y).step_by(2) {
        for x in (1..=BLOCK_NUM_X).step_by(2) {
            let x = WALL_SIZE.x * (x as f32)
                - WALL_SIZE.x * (BLOCK_NUM_X as f32 / 2.)
                - WALL_SIZE.x / 2.;
            let y = WALL_SIZE.y * (y as f32)
                - WALL_SIZE.y * (BLOCK_NUM_Y as f32 / 2.)
                - WALL_SIZE.y / 2.;

            // 棒
            commands.spawn(BlockBundle::new(Vec2::new(x, y)));
            // 倒した棒
            commands.spawn(BlockBundle::new(knock_down_the(x, y)));
        }
    }

    state.set(Status::Disabled);
}

//倒した棒の座標
fn knock_down_the(x: f32, y: f32) -> Vec2 {
    // ランダムな4方向
    let direction = thread_rng().gen_range(0..4);

    match direction {
        0 => Vec2::new(x, y + WALL_SIZE.y),
        1 => Vec2::new(x + WALL_SIZE.x, y),
        2 => Vec2::new(x, y - WALL_SIZE.y),
        3 => Vec2::new(x - WALL_SIZE.x, y),
        _ => panic!("random error!?!?!"),
    }
}

// プレイヤーを動かす
fn move_player(
    mut player_query: Query<&mut Transform, With<Player>>,
    key: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    let mut player_transform = player_query.single_mut();

    let mut direction_x = 0.0;
    let mut direction_y = 0.0;

    if key.pressed(KeyCode::Up) {
        direction_y += 1.0
    }
    if key.pressed(KeyCode::Down) {
        direction_y -= 1.0;
    }
    if key.pressed(KeyCode::Right) {
        direction_x += 1.0
    }
    if key.pressed(KeyCode::Left) {
        direction_x -= 1.0;
    }

    player_transform.translation.x =
        player_transform.translation.x + direction_x * PLAYER_SPEED * time.delta_seconds();
    player_transform.translation.y =
        player_transform.translation.y + direction_y * PLAYER_SPEED * time.delta_seconds();
}

// 壁の判定
fn wall_collision(
    mut player_query: Query<&mut Transform, (With<Player>, Without<Block>)>,
    block_query: Query<&Transform, With<Block>>,
    time: Res<Time>,
) {
    let mut player_transform = player_query.single_mut();

    for transform in &block_query {
        let collision = collide(
            transform.translation,
            transform.scale.truncate(),
            player_transform.translation,
            player_transform.scale.truncate(),
        );
        if let Some(collision) = collision {
            let mut direction_x = 0.0;
            let mut direction_y = 0.0;

            match collision {
                Collision::Top => direction_y -= 1.0,
                Collision::Bottom => direction_y += 1.0,
                Collision::Right => direction_x -= 1.0,
                Collision::Left => direction_x += 1.0,
                _ => (),
            }

            let new_player_position_x =
                player_transform.translation.x + direction_x * PLAYER_SPEED * time.delta_seconds();
            let new_player_position_y =
                player_transform.translation.y + direction_y * PLAYER_SPEED * time.delta_seconds();

            player_transform.translation.x = new_player_position_x;
            player_transform.translation.y = new_player_position_y;
        }
    }
}

fn button_system(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<Button>)>,
    mut status: ResMut<NextState<Status>>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            status.set(Status::Shuffle);
        }
    }
}
