use bevy::prelude::*;

use crate::components;


#[derive(Default)]
pub struct SnakeSegments(pub Vec<Entity>);

#[derive(Default)]
pub struct LastTailPosition(pub Option<components::Position>);

pub struct MoveTimer(pub Timer);

#[derive(Default)]
pub struct Score(pub u32);
