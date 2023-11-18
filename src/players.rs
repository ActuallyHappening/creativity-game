use crate::prelude::*;

mod blocks;

/// Plugin Group
pub struct PlayerPlugins;

impl PluginGroup for PlayerPlugins {
	fn build(self) -> PluginGroupBuilder {
		PluginGroupBuilder::start::<Self>()
			.add(player::PlayerPlugin)
			.build()
	}
}

pub use player::PlayerBlueprint;

mod player {
	use super::blocks::{BlockBlueprint, BlockId, StructureBlock};
	use super::thruster::ThrusterBlock;
	use crate::players::blocks::StructureBlockBundle;
	use crate::prelude::*;

	pub struct PlayerPlugin;
	impl Plugin for PlayerPlugin {
		fn build(&self, app: &mut App) {
			app
				.replicate::<PlayerBlueprint>()
				.add_systems(
					FixedUpdate,
					Self::handle_spawn_player_blueprints
						.in_set(GlobalSystemSet::BlueprintExpansion("player")),
				);
		}
	}

	/// Sent as an event to all clients, then expanded into a full player bundle
	#[derive(Component, Serialize, Deserialize, Clone, Debug)]
	pub struct PlayerBlueprint {
		network_id: ClientId,
		transform: Transform,
		structure_children: Vec<BlockBlueprint<StructureBlock>>,
		thruster_children: Vec<BlockBlueprint<ThrusterBlock>>,
	}

	impl PlayerBlueprint {
		pub fn default_at(network_id: ClientId, transform: Transform) -> Self {
			PlayerBlueprint {
				network_id,
				transform,
				structure_children: vec![BlockBlueprint::new(StructureBlock::Aluminum, IVec3::ZERO)],
				thruster_children: vec![],
			}
		}
	}

	/// Parent entity of a player.
	/// Doesn't actually have its own mesh
	#[derive(Bundle)]
	struct PlayerBundle {
		spatial: SpatialBundle,
		replication: Replication,
		collider: AsyncCollider,
		name: Name,
		controllable_player: ControllablePlayer,
	}

	#[derive(Component)]
	struct ControllablePlayer {
		network_id: ClientId,
		movement_input: HashMap<BlockId, f32>,
	}

	impl PlayerPlugin {
		fn handle_spawn_player_blueprints(
			player_blueprints: Query<(Entity, &PlayerBlueprint), Added<PlayerBlueprint>>,
			mut commands: Commands,
			mut mma: MMA,
		) {
			for (player, player_blueprint) in player_blueprints.iter() {
				debug!(
					"Expanding player blueprint for {:?}",
					player_blueprint.network_id
				);
				commands.entity(player)
					.insert(PlayerBundle::stamp_from_blueprint(
						player_blueprint,
						&mut mma,
					))
					.with_children(|parent| {
						for blueprint in &player_blueprint.structure_children {
							parent.spawn(StructureBlockBundle::stamp_from_blueprint(
								blueprint, &mut mma,
							));
						}
					});
			}
		}
	}

	impl FromBlueprint for PlayerBundle {
		type Blueprint = PlayerBlueprint;

		fn stamp_from_blueprint(
			PlayerBlueprint {
				network_id,
				transform,
				..
			}: &PlayerBlueprint,
			_mma: &mut MMA,
		) -> Self {
			Self {
				spatial: SpatialBundle {
					transform: *transform,
					..default()
				},
				name: Name::new(format!("Player {}", network_id)),
				controllable_player: ControllablePlayer {
					network_id: *network_id,
					movement_input: Default::default(),
				},
				collider: AsyncCollider(ComputedCollider::ConvexHull),
				replication: Replication,
			}
		}
	}
}

mod thruster {
	use crate::prelude::*;

	/// Will spawn a particle emitter as a child
	#[derive(Debug, Serialize, Deserialize, Clone)]
	pub struct ThrusterBlock;
}
