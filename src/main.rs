use bevy::{
    prelude::*,
    sprite::collide_aabb::{collide, Collision},
};
use rand::{seq::SliceRandom, thread_rng};

// ブロックの数
const BLOCK_NUM: f32 = 11.;

const BLOCK_SIZE: Vec2 = Vec2::new(600. / (BLOCK_NUM + 2.), 600. / (BLOCK_NUM + 2.));
// プレイヤーの初期位置
const PLAYER1_INITIAL_POSITION: Vec2 = Vec2::new(
    -BLOCK_SIZE.x * (BLOCK_NUM + 1.) / 2.,
    BLOCK_SIZE.y * (BLOCK_NUM + 1.) / 2.,
);
const PLAYER2_INITIAL_POSITION: Vec2 = Vec2::new(
    BLOCK_SIZE.x * (BLOCK_NUM + 1.) / 2.,
    -BLOCK_SIZE.y * (BLOCK_NUM + 1.) / 2.,
);

const PLAYER_SIZE: Vec2 = Vec2::new(BLOCK_SIZE.x / 2.0, BLOCK_SIZE.y / 2.0);
const PLAYER_SPEED: f32 = 4000. / BLOCK_NUM;

const ITEM_NUM: usize = 7;

const SCOREBOARD_TEXT_PADDING: Val = Val::Px(8.0);
const SCOREBOARD_FONT_SIZE: f32 = 40.0;

const TEXT_COLOR: Color = Color::WHITE;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, States, Component)]
enum Status {
    #[default]
    Shuffle,
    Disabled,
}

#[derive(Component, Clone, Copy)]
enum ButtonAction {
    Shuffle,
    ItemNumUp,
    ItemNumDown,
    BlockNumUp,
    BlockNumDown,
}

#[derive(Component)]
enum ScoreboardSection {
    P1,
    P2,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "labyrinth".into(),
                resolution: (1200., 900.).into(),
                ..default()
            }),
            ..default()
        }))
        .add_state::<Status>()
        .add_event::<ScoreEvent>()
        .init_resource::<ScoreBoard>()
        .init_resource::<ItemPosition>()
        .init_resource::<Game>()
        .add_systems(Startup, (setup, labyrinth_setup))
        .add_systems(OnEnter(Status::Shuffle), labyrinth_setup)
        .add_systems(OnExit(Status::Shuffle), create_item)
        .add_systems(
            Update,
            (
                move_player1,
                move_player2,
                wall_collision,
                item_collision::<Player1>,
                item_collision::<Player2>,
            )
                .run_if(in_state(Status::Disabled)),
        )
        .add_systems(Update, (button_system, score_board_update))
        .run();
}

#[derive(Resource)]
struct Game {
    item_num: usize,
    block_num: f32,
    block_size: Vec2,
    player_size: Vec2,
    player_speed: f32,
    player1_init_pos: Vec2,
    player2_init_pos: Vec2,
}

#[derive(Resource, Default)]
struct ScoreBoard {
    player1: usize,
    player2: usize,
}

#[derive(Event, Default)]
struct ScoreEvent;

#[derive(Component)]
struct Player1;

#[derive(Component)]
struct Player2;

#[derive(Component)]
struct Block;

#[derive(Resource, Default)]
struct ItemPosition(Vec<Vec2>);

#[derive(Component, Debug)]
enum Item {
    Item,
}

#[derive(Bundle)]
struct ItemBundle {
    sprite_bundle: SpriteBundle,
    item: Item,
}

#[derive(Bundle)]
struct BlockBundle {
    sprite_bundle: SpriteBundle,
    block: Block,
}

impl BlockBundle {
    // 新しいブロックを作る
    fn new(translation: Vec2, size: Vec2) -> BlockBundle {
        BlockBundle {
            sprite_bundle: SpriteBundle {
                transform: Transform {
                    translation: translation.extend(0.0),
                    scale: size.extend(0.0),
                    ..default()
                },
                ..default()
            },
            block: Block,
        }
    }
}

impl ItemBundle {
    // 新しいアイテムを作る
    fn new(item: Item, translation: Vec2, size: Vec2) -> ItemBundle {
        ItemBundle {
            sprite_bundle: SpriteBundle {
                transform: Transform {
                    translation: translation.extend(0.0),
                    scale: size.extend(0.0),
                    ..default()
                },
                sprite: Sprite {
                    color: item.color(),
                    ..default()
                },
                ..default()
            },
            item,
        }
    }
}

impl Item {
    const fn color(&self) -> Color {
        match self {
            Item::Item => Color::BLUE,
        }
    }
}

impl Default for Game {
    fn default() -> Self {
        Game {
            item_num: ITEM_NUM,
            block_num: BLOCK_NUM,
            block_size: BLOCK_SIZE,
            player_size: PLAYER_SIZE,
            player_speed: PLAYER_SPEED,
            player1_init_pos: PLAYER1_INITIAL_POSITION,
            player2_init_pos: PLAYER2_INITIAL_POSITION,
        }
    }
}

impl Game {
    const WALL_MAX: f32 = 600.0;

    fn init(&mut self) {
        if self.block_num < 1. {
            self.block_num = 1.;
        }

        let block_x = Self::WALL_MAX / (self.block_num + 2.0);
        let block_y = Self::WALL_MAX / (self.block_num + 2.0);

        self.block_size = Vec2::new(block_x, block_y);
        self.player_size = Vec2::new(block_x / 2.0, block_y / 2.0);
        self.player_speed = ((4000. / self.block_num) + (4000. / self.block_num)) / 2.;
        self.player1_init_pos = Vec2::new(
            ((self.block_num + 1.) / 2.) * -block_x,
            ((self.block_num + 1.) / 2.) * block_y,
        );
        self.player2_init_pos = Vec2::new(
            ((self.block_num + 1.) / 2.) * block_x,
            ((self.block_num + 1.) / 2.) * -block_y,
        );
    }

    fn wall_adjustment(&self, x: f32, y: f32) -> Vec2 {
        let x = self.block_size.x * (x - 2.)
            - self.block_size.x * (self.block_num / 2.)
            - self.block_size.x / 2.;
        let y = self.block_size.y * (y - 2.)
            - self.block_size.y * (self.block_num / 2.)
            - self.block_size.y / 2.;
        Vec2::new(x, y)
    }
    fn block_adjustment(&self, x: f32, y: f32) -> Vec2 {
        let x = self.block_size.x * x
            - self.block_size.x * (self.block_num / 2.)
            - self.block_size.x / 2.;
        let y = self.block_size.y * y
            - self.block_size.y * (self.block_num / 2.)
            - self.block_size.y / 2.;
        Vec2::new(x, y)
    }
}

const SETTING_SECTION: [(&str, (ButtonAction, &str), (ButtonAction, &str)); 2] = [
    (
        "ItemNum: ",
        (ButtonAction::ItemNumDown, "<"),
        (ButtonAction::ItemNumUp, ">"),
    ),
    (
        "BlockNum: ",
        (ButtonAction::BlockNumDown, "<"),
        (ButtonAction::BlockNumUp, ">"),
    ),
];

fn setup(mut commands: Commands, game: Res<Game>) {
    commands.spawn(Camera2dBundle::default());

    let text_style = TextStyle {
        font_size: SCOREBOARD_FONT_SIZE,
        color: TEXT_COLOR,
        ..default()
    };

    commands.spawn((
        TextBundle::from_sections([
            TextSection::new("p1: ", text_style.clone()),
            TextSection::new("0", text_style.clone()),
        ])
        .with_style(Style {
            position_type: PositionType::Absolute,
            justify_self: JustifySelf::End,
            top: SCOREBOARD_TEXT_PADDING,
            left: SCOREBOARD_TEXT_PADDING,
            ..default()
        }),
        ScoreboardSection::P1,
    ));
    commands.spawn((
        TextBundle::from_sections([
            TextSection::new("p2: ", text_style.clone()),
            TextSection::new("0", text_style),
        ])
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: SCOREBOARD_TEXT_PADDING,
            right: SCOREBOARD_TEXT_PADDING,
            ..default()
        }),
        ScoreboardSection::P2,
    ));

    let button_style = Style {
        width: Val::Px(50.),
        height: Val::Px(40.),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        margin: UiRect::horizontal(Val::Px(8.)),
        ..default()
    };
    let text_style = TextStyle {
        font_size: 40.,
        color: Color::BLACK,
        ..default()
    };

    commands
        .spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                align_self: AlignSelf::FlexEnd,
                justify_self: JustifySelf::End,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                right: Val::Px(10.),
                bottom: Val::Px(10.),
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            for section in SETTING_SECTION {
                parent.spawn(TextBundle::from_sections([
                    TextSection::new(
                        section.0,
                        TextStyle {
                            font_size: 35.,
                            color: Color::WHITE,
                            ..default()
                        },
                    ),
                    TextSection::new(
                        if section.0 == "ItemNum: " {
                            game.item_num.to_string()
                        } else {
                            game.block_num.to_string()
                        },
                        TextStyle {
                            font_size: 35.,
                            color: Color::WHITE,
                            ..default()
                        },
                    ),
                ]));
                parent
                    .spawn(NodeBundle {
                        style: Style {
                            padding: UiRect::vertical(Val::Px(6.)),
                            ..default()
                        },
                        ..default()
                    })
                    .with_children(|parent| {
                        parent
                            .spawn((
                                ButtonBundle {
                                    style: button_style.clone(),
                                    ..default()
                                },
                                section.1 .0,
                            ))
                            .with_children(|parent| {
                                parent.spawn(TextBundle::from_section(
                                    section.1 .1,
                                    text_style.clone(),
                                ));
                            });
                        parent
                            .spawn((
                                ButtonBundle {
                                    style: button_style.clone(),
                                    ..default()
                                },
                                section.2 .0,
                            ))
                            .with_children(|parent| {
                                parent.spawn(TextBundle::from_section(
                                    section.2 .1,
                                    text_style.clone(),
                                ));
                            });
                    });
            }
            parent
                .spawn((
                    ButtonBundle {
                        style: Style {
                            width: Val::Px(230.),
                            height: Val::Px(60.),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                        ..default()
                    },
                    ButtonAction::Shuffle,
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section("Shuffle", text_style));
                });
        });
}

// アイテムを作る
fn create_item(mut commands: Commands, item_position: Res<ItemPosition>, game: Res<Game>) {
    for i in 0..game.item_num {
        commands.spawn(ItemBundle::new(
            Item::Item,
            item_position.0[i],
            game.player_size,
        ));
    }
}

fn labyrinth_setup(
    mut commands: Commands,
    entity_query: Query<Entity, Or<(With<Player1>, With<Player2>, With<Block>, With<Item>)>>,
    mut game: ResMut<Game>,
    mut resource_item_pos: ResMut<ItemPosition>,
    mut state: ResMut<NextState<Status>>,
) {
    game.init();

    for entity in &entity_query {
        commands.entity(entity).despawn();
    }

    commands.spawn((
        SpriteBundle {
            transform: Transform {
                translation: game.player1_init_pos.extend(0.0),
                scale: game.player_size.extend(0.0),
                ..default()
            },
            sprite: Sprite {
                color: Color::RED,
                ..default()
            },
            ..default()
        },
        Player1,
    ));
    commands.spawn((
        SpriteBundle {
            transform: Transform {
                translation: game.player2_init_pos.extend(0.0),
                scale: game.player_size.extend(0.0),
                ..default()
            },
            sprite: Sprite {
                color: Color::GREEN,
                ..default()
            },
            ..default()
        },
        Player2,
    ));

    let block_num = game.block_num as i32;

    // 壁
    for y in 1..=block_num + 4 {
        for x in 1..=block_num + 4 {
            if y == 1 || x == 1 || y == block_num + 4 || x == block_num + 4 {
                // 座標をいい感じの位置にするための計算
                let pos = game.wall_adjustment(x as f32, y as f32);

                commands.spawn(BlockBundle::new(pos, game.block_size));
            }
        }
    }

    let mut item_position: Vec<Vec2> = Vec::new();
    let mut block_position: Vec<Vec2> = Vec::new();

    for y in (1..=block_num).step_by(2) {
        for x in (1..=block_num).step_by(2) {
            // 座標をいい感じの位置にするための計算
            let pos = game.block_adjustment(x as f32, y as f32);

            // 棒
            commands.spawn(BlockBundle::new(pos, game.block_size));

            // 倒した棒の位置
            let block_pos = knock_down_the(
                &mut item_position,
                pos.x,
                pos.y,
                game.block_size.x,
                game.block_size.y,
            );

            block_position.push(block_pos);

            commands.spawn(BlockBundle::new(block_pos, game.block_size));
        }
    }

    // ブロックがある位置にアイテムが作られないように
    for block in block_position {
        overlapping_remove(&mut item_position, block);
    }

    // アイテムをシャッフル
    item_position.shuffle(&mut thread_rng());
    // アイテムの位置を更新する
    resource_item_pos.0 = item_position;

    // ステータスの変更
    state.set(Status::Disabled);
}

// valueと重複する値を消去する
#[inline]
fn overlapping_remove(vec: &mut Vec<Vec2>, value: Vec2) {
    // 重複する要素があるインデックスを取得し、削除
    // ※ as_ivec2()メソッドを使用してf32をi32にしないと判定漏れが発生する
    if let Some(index) = vec.iter().position(|&i| i.as_ivec2() == value.as_ivec2()) {
        vec.remove(index);
    }
}

//倒した棒の座標
#[inline]
fn knock_down_the(vec: &mut Vec<Vec2>, x: f32, y: f32, block_x: f32, block_y: f32) -> Vec2 {
    // 十字の方向
    let mut cross_array: Vec<Vec2> = vec![
        Vec2::new(x, y + block_y),
        Vec2::new(x + block_x, y),
        Vec2::new(x, y - block_y),
        Vec2::new(x - block_x, y),
    ];
    // 斜めの座標
    let slantings_array: [Vec2; 4] = [
        Vec2::new(x + block_x, y + block_y),
        Vec2::new(x + block_x, y - block_y),
        Vec2::new(x - block_x, y + block_y),
        Vec2::new(x - block_x, y - block_y),
    ];

    cross_array.shuffle(&mut thread_rng());

    let block = cross_array.pop().expect("pop エラー");

    for value in cross_array {
        vec.push(value);
    }
    for value in slantings_array {
        vec.push(value);
    }

    block
}

// プレイヤーを動かす
fn move_player1(
    mut player_query: Query<&mut Transform, With<Player1>>,
    key: Res<Input<KeyCode>>,
    game: Res<Game>,
    time: Res<Time>,
) {
    let mut transform = player_query.single_mut();

    // 方向
    let mut direction_x = 0.0;
    let mut direction_y = 0.0;

    if key.pressed(KeyCode::W) {
        direction_y += 1.0
    }
    if key.pressed(KeyCode::S) {
        direction_y -= 1.0;
    }
    if key.pressed(KeyCode::D) {
        direction_x += 1.0
    }
    if key.pressed(KeyCode::A) {
        direction_x -= 1.0;
    }

    // 座標を更新
    transform.translation.x += direction_x * game.player_speed * time.delta_seconds();
    transform.translation.y += direction_y * game.player_speed * time.delta_seconds();
}

fn move_player2(
    mut player_query: Query<&mut Transform, With<Player2>>,
    key: Res<Input<KeyCode>>,
    game: Res<Game>,
    time: Res<Time>,
) {
    let mut transform = player_query.single_mut();

    // 方向
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

    // 座標を更新
    transform.translation.x += direction_x * game.player_speed * time.delta_seconds();
    transform.translation.y += direction_y * game.player_speed * time.delta_seconds();
}

// TODO: 別の方法で実装したい
// 今のままだと引っ掛かりがある
// 壁の判定
fn wall_collision(
    mut player_query: Query<&mut Transform, (Or<(With<Player1>, With<Player2>)>, Without<Block>)>,
    block_query: Query<&Transform, With<Block>>,
    game: Res<Game>,
    time: Res<Time>,
) {
    for mut player_transform in &mut player_query {
        for transform in &block_query {
            let collision = collide(
                transform.translation,
                transform.scale.xy(),
                player_transform.translation,
                player_transform.scale.xy(),
            );
            // 衝突したなら
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

                player_transform.translation.x +=
                    direction_x * game.player_speed * time.delta_seconds();
                player_transform.translation.y +=
                    direction_y * game.player_speed * time.delta_seconds();
            }
        }
    }
}

// アイテムの判定
fn item_collision<T>(
    mut commands: Commands,
    player_query: Query<&Transform, With<T>>,
    item_query: Query<(Entity, &Transform, &Item), With<Item>>,
    mut score_board: ResMut<ScoreBoard>,
    mut event: EventWriter<ScoreEvent>,
) where
    T: Component + PlayerMethod,
{
    let player_transform = player_query.single();

    for (entity, transform, _) in &item_query {
        let collision = collide(
            player_transform.translation,
            player_transform.scale.truncate(),
            transform.translation,
            transform.scale.truncate(),
        );
        if collision.is_some() {
            match T::section() {
                ScoreboardSection::P1 => score_board.player1 += 1,
                ScoreboardSection::P2 => score_board.player2 += 1,
            }

            event.send_default();

            commands.entity(entity).despawn();
        }
    }
}

// スコアボードの更新
fn score_board_update(
    score_board: Res<ScoreBoard>,
    mut query: Query<(&mut Text, &ScoreboardSection)>,
    mut event: EventReader<ScoreEvent>,
) {
    if !event.is_empty() {
        event.clear();

        for (mut text, section) in &mut query {
            text.sections[1].value = match section {
                ScoreboardSection::P1 => score_board.player1.to_string(),
                ScoreboardSection::P2 => score_board.player2.to_string(),
            }
        }
    }
}

fn button_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &ButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut text_query: Query<&mut Text>,
    mut game: ResMut<Game>,
    mut status: ResMut<NextState<Status>>,
) {
    for (interaction, mut background, action) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                match *action {
                    ButtonAction::Shuffle => status.set(Status::Shuffle),
                    ButtonAction::ItemNumUp => game.item_num += 1,
                    ButtonAction::ItemNumDown => game.item_num -= 1,
                    ButtonAction::BlockNumUp => game.block_num += 2.,
                    ButtonAction::BlockNumDown => game.block_num -= 2.,
                };
                for mut text in &mut text_query {
                    match text.sections[0].value.as_str() {
                        "ItemNum: " => text.sections[1].value = game.item_num.to_string(),
                        "BlockNum: " => text.sections[1].value = game.block_num.to_string(),
                        _ => (),
                    }
                }
            }
            Interaction::Hovered => background.0 = Color::GREEN,
            Interaction::None => background.0 = Color::WHITE,
        }
    }
}

trait PlayerMethod {
    // どのスコアを変更するればいいかを返す
    fn section() -> ScoreboardSection;
}

impl PlayerMethod for Player1 {
    fn section() -> ScoreboardSection {
        ScoreboardSection::P1
    }
}

impl PlayerMethod for Player2 {
    fn section() -> ScoreboardSection {
        ScoreboardSection::P2
    }
}
