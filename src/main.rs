mod components;
mod resources;
mod types;

use bevy::prelude::*;
use bevy::ecs::schedule::ShouldRun;
use rand::prelude::random;


const WINDOW_WIDTH: f32  = 500.;
const WINDOW_HEIGHT: f32 = 500.;

const ARENA_WIDTH: u32  = 10;
const ARENA_HEIGHT: u32 = 10;

const CLEAR_COLOR: Color    = Color::rgb(0.04, 0.04, 0.04);
const SNAKE_HEAD_COLOR: Color    = Color::rgb(0.0, 0.7, 0.0);
const SNAKE_SEGMENT_COLOR: Color = Color::rgb(0.0, 0.3, 0.0);
const FOOD_COLOR: Color          = Color::rgb(0.8, 0.0, 0.0);

const MOVE_TIMER_DURATION_SECONDS_INITIAL: f32           = 0.3;
const MOVE_TIMER_DURATION_SECONDS_DECREASE_PER_FOOD: f32 = 0.01;
const MOVE_TIMER_DURATION_SECONDS_MINIMUM: f32           = 0.1;


fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "Snake!".to_string(),
            width: WINDOW_WIDTH,
            height: WINDOW_HEIGHT,
            ..Default::default()
        })
        .insert_resource(ClearColor(CLEAR_COLOR))
        .add_startup_system(setup)
        .add_startup_system(new_game)
        .add_system(
            snake_movement_input
                .label(types::SnakeMovement::Input)
                .before(types::SnakeMovement::Movement),
        )
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(run_with_timer)
                .with_system(snake_movement.label(types::SnakeMovement::Movement))
                .with_system(
                    snake_eating
                        .label(types::SnakeMovement::Eating)
                        .after(types::SnakeMovement::Movement),
                )
                .with_system(
                    snake_growth
                        .label(types::SnakeMovement::Growth)
                        .after(types::SnakeMovement::Eating),
                ),
        )
        .add_system(game_over.after(types::SnakeMovement::Movement))
        .add_system_set_to_stage(
            CoreStage::PostUpdate,
            SystemSet::new()
                .with_system(position_translation)
                .with_system(size_scaling)
                .with_system(set_score),
        )
        .add_system(food_spawn)
        .add_event::<types::GrowthEvent>()
        .add_event::<types::FoodEvent>()
        .add_event::<types::GameOverEvent>()
        .insert_resource(resources::LastTailPosition::default())
        .insert_resource(resources::SnakeSegments::default())
        .insert_resource(resources::MoveTimer(Timer::from_seconds(MOVE_TIMER_DURATION_SECONDS_INITIAL, true)))
        .insert_resource(resources::Score::default())
        .add_plugins(DefaultPlugins)
        .run();
}

fn set_score(
    score: Res<resources::Score>,
    mut texts: Query<&mut Text, With<components::ScoreText>>,
) {
    let s = format!("{}", score.0);
    for mut text in texts.iter_mut() {
        text.sections[0].value = s.clone();
    }
}

fn run_with_timer(
    time: Res<Time>,
    mut move_timer: ResMut<resources::MoveTimer>,
) -> ShouldRun {
    move_timer.0.tick(time.delta());
    if move_timer.0.just_finished() {
        return ShouldRun::Yes;
    }
    else {
        return ShouldRun::No;
    }
}

fn setup(
    mut commands: Commands,
    mut fonts: ResMut<Assets<Font>>,
) {
    let font_bytes = include_bytes!("../assets/fonts/OpenSans-Regular.ttf");
    let font = Font::try_from_bytes(font_bytes.to_vec()).unwrap();
    let font_handle = fonts.add(font);

    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    commands.spawn_bundle(UiCameraBundle::default());

    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                position_type: PositionType::Absolute,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::FlexEnd,
                ..Default::default()
            },
            color: UiColor(Color::NONE),
            ..Default::default()
        })
        .with_children(|parent| {
            parent
                .spawn_bundle(TextBundle {
                    text: Text::with_section(
                        "",
                        TextStyle {
                            font: font_handle,
                            font_size: 300.0,
                            color: Color::Rgba{
                                red: 0.8,
                                blue: 0.8,
                                green: 0.8,
                                alpha: 0.05,
                            },
                        },
                        TextAlignment {
                            vertical: VerticalAlign::Center,
                            horizontal: HorizontalAlign::Center,
                        },
                    ),
                    style: Style {
                        align_self: AlignSelf::Center,
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .insert(components::ScoreText);
        });
}

fn new_game(
    mut commands: Commands,
    mut segments: ResMut<resources::SnakeSegments>,
    mut food_writer: EventWriter<types::FoodEvent>,
    mut score: ResMut<resources::Score>,
    mut move_timer: ResMut<resources::MoveTimer>,
) {
    score.0 = 0;
    move_timer.0.set_duration(bevy::utils::Duration::from_secs_f32(MOVE_TIMER_DURATION_SECONDS_INITIAL));

    segments.0 = vec![
        commands
            .spawn_bundle(SpriteBundle {
                sprite: Sprite {
                    color: SNAKE_HEAD_COLOR,
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(components::SnakeHead {
                direction: types::Direction::Up,
            })
            .insert(components::SnakeSegment)
            .insert(components::Position { x: 5, y: 5 })
            .insert(components::Size::square(0.8))
            .id(),
        spawn_segment(commands, components::Position { x: 5, y: 4 }),
    ];

    food_writer.send(types::FoodEvent);
}

fn spawn_segment(mut commands: Commands, position: components::Position) -> Entity {
    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: SNAKE_SEGMENT_COLOR,
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(components::SnakeSegment)
        .insert(position)
        .insert(components::Size::square(0.65))
        .id()
}

fn snake_movement_input(keyboard_input: Res<Input<KeyCode>>, mut heads: Query<&mut components::SnakeHead>) {
    if let Some(mut head) = heads.iter_mut().next() {
        let dir: types::Direction = if keyboard_input.pressed(KeyCode::Left) {
            types::Direction::Left
        } else if keyboard_input.pressed(KeyCode::Down) {
            types::Direction::Down
        } else if keyboard_input.pressed(KeyCode::Up) {
            types::Direction::Up
        } else if keyboard_input.pressed(KeyCode::Right) {
            types::Direction::Right
        } else {
            head.direction
        };
        if dir != head.direction.opposite() {
            head.direction = dir;
        }
    }
}

fn snake_movement(
    segments: ResMut<resources::SnakeSegments>,
    mut heads: Query<(Entity, &components::SnakeHead)>,
    mut positions: Query<&mut components::Position>,
    mut last_tail_position: ResMut<resources::LastTailPosition>,
    mut game_over_writer: EventWriter<types::GameOverEvent>,
) {
    if let Some((head_entity, head)) = heads.iter_mut().next() {
        let segment_positions = segments
            .0
            .iter()
            .map(|e| *positions.get_mut(*e).unwrap())
            .collect::<Vec<components::Position>>();
        let mut head_pos = positions.get_mut(head_entity).unwrap();
        match &head.direction {
            types::Direction::Left => {
                head_pos.x -= 1;
            }
            types::Direction::Right => {
                head_pos.x += 1;
            }
            types::Direction::Up => {
                head_pos.y += 1;
            }
            types::Direction::Down => {
                head_pos.y -= 1;
            }
        };
        if head_pos.x < 0
            || head_pos.y < 0
            || head_pos.x as u32 >= ARENA_WIDTH
            || head_pos.y as u32 >= ARENA_HEIGHT
        {
            game_over_writer.send(types::GameOverEvent);
        }
        if segment_positions.contains(&head_pos) {
            game_over_writer.send(types::GameOverEvent);
        }
        segment_positions
            .iter()
            .zip(segments.0.iter().skip(1))
            .for_each(|(pos, segment)| {
                *positions.get_mut(*segment).unwrap() = *pos;
            });
        last_tail_position.0 = Some(*segment_positions.last().unwrap());
    }
}

fn size_scaling(windows: Res<Windows>, mut q: Query<(&components::Size, &mut Transform)>) {
    let window = windows.get_primary().unwrap();
    for (sprite_size, mut transform) in q.iter_mut() {
        transform.scale = Vec3::new(
            sprite_size.width / ARENA_WIDTH as f32 * window.width() as f32,
            sprite_size.height / ARENA_HEIGHT as f32 * window.height() as f32,
            1.0,
        );
    }
}

fn position_translation(windows: Res<Windows>, mut q: Query<(&components::Position, &mut Transform)>) {
    fn convert(pos: f32, bound_window: f32, bound_game: f32) -> f32 {
        let tile_size = bound_window / bound_game;
        pos / bound_game * bound_window - (bound_window / 2.) + (tile_size / 2.)
    }
    let window = windows.get_primary().unwrap();
    for (pos, mut transform) in q.iter_mut() {
        transform.translation = Vec3::new(
            convert(pos.x as f32, window.width() as f32, ARENA_WIDTH as f32),
            convert(pos.y as f32, window.height() as f32, ARENA_HEIGHT as f32),
            0.0,
        );
    }
}

fn food_spawn(
    mut commands: Commands,
    segment_positions: Query<&components::Position, With<components::SnakeSegment>>,
    food_positions: Query<&components::Position, With<components::Food>>,
    mut food_reader: EventReader<types::FoodEvent>,
) {
    if food_reader.iter().next().is_some() {
        let mut position = components::Position{x: 0, y: 0};
        loop {
            // Create new random position
            position.x = (random::<f32>() * ARENA_WIDTH as f32) as i32;
            position.y = (random::<f32>() * ARENA_HEIGHT as f32) as i32;

            // Check against segments
            if segment_positions.iter().any(|&p| p.x == position.x && p.y == position.y) {
                continue
            }

            // Check against food
            if food_positions.iter().any(|&p| p.x == position.x && p.y == position.y) {
                continue
            }

            // Found an unused position
            break
        }

        commands
            .spawn_bundle(SpriteBundle {
                sprite: Sprite {
                    color: FOOD_COLOR,
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(components::Food)
            .insert(position)
            .insert(components::Size::square(0.8));
    }
}

fn snake_eating(
    mut commands: Commands,
    mut growth_writer: EventWriter<types::GrowthEvent>,
    mut food_writer: EventWriter<types::FoodEvent>,
    mut score: ResMut<resources::Score>,
    mut move_timer: ResMut<resources::MoveTimer>,
    food_positions: Query<(Entity, &components::Position), With<components::Food>>,
    head_positions: Query<&components::Position, With<components::SnakeHead>>,
) {
    for head_pos in head_positions.iter() {
        for (ent, food_pos) in food_positions.iter() {
            if food_pos == head_pos {
                commands.entity(ent).despawn();
                growth_writer.send(types::GrowthEvent);
                food_writer.send(types::FoodEvent);
                score.0 += 1;

                let mut seconds = MOVE_TIMER_DURATION_SECONDS_INITIAL - (score.0 as f32 * MOVE_TIMER_DURATION_SECONDS_DECREASE_PER_FOOD);
                if seconds < MOVE_TIMER_DURATION_SECONDS_MINIMUM {
                    seconds = MOVE_TIMER_DURATION_SECONDS_MINIMUM
                }
                move_timer.0.set_duration(bevy::utils::Duration::from_secs_f32(seconds));

                return;
            }
        }
    }
}

fn snake_growth(
    commands: Commands,
    last_tail_position: Res<resources::LastTailPosition>,
    mut segments: ResMut<resources::SnakeSegments>,
    mut growth_reader: EventReader<types::GrowthEvent>,
) {
    if growth_reader.iter().next().is_some() {
        segments
            .0
            .push(spawn_segment(commands, last_tail_position.0.unwrap()));
    }
}

fn game_over(
    mut commands: Commands,
    mut reader: EventReader<types::GameOverEvent>,
    food: Query<Entity, With<components::Food>>,
    segment_entities: Query<Entity, With<components::SnakeSegment>>,
    segments: ResMut<resources::SnakeSegments>,
    food_writer: EventWriter<types::FoodEvent>,
    score: ResMut<resources::Score>,
    move_timer: ResMut<resources::MoveTimer>,
) {
    if reader.iter().next().is_some() {
        for entity in food.iter().chain(segment_entities.iter()) {
            commands.entity(entity).despawn();
        }
        new_game(commands, segments, food_writer, score, move_timer);
    }
}
