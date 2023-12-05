use crate::prelude::*;

pub use player::ControllablePlayer;

mod player;
mod spawn_points;
mod thruster_block;

/// Plugin Group
pub struct PlayerPlugins;

impl PluginGroup for PlayerPlugins {
	fn build(self) -> PluginGroupBuilder {
		PluginGroupBuilder::start::<Self>()
			.add(player::PlayerPlugin)
			.add(thruster_block::ThrusterPlugin)
			.add(self::spawn_points::SpawnPointsPlugin)
			.build()
	}
}

pub use player::{PlayerBlueprint, PlayerMovement};
