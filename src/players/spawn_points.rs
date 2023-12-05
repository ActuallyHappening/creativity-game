use crate::prelude::*;

use self::components::SpawnPoint;

pub struct SpawnPointsPlugin;

impl Plugin for SpawnPointsPlugin {
	fn build(&self, app: &mut App) {
		app
			.register_type::<SpawnPoint>()
			.add_systems(PostProcessCollisions, Self::filter_non_occupied_collisions)
			.add_systems(WorldCreation, Self::creation_spawn_points);
	}
}

mod systems {
	use crate::{players::ControllablePlayer, prelude::*};

	use super::{
		blueprint::SpawnPointBlueprint, bundle::SpawnPointBundle, components::SpawnPoint,
		SpawnPointsPlugin,
	};

	impl SpawnPointsPlugin {
		pub(super) fn filter_non_occupied_collisions(
			mut collisions: ResMut<Collisions>,
			players: Query<&ControllablePlayer>,
			spawn_points: Query<&SpawnPoint>,
		) {
			collisions.retain(|contacts| {
				let check = |e1: Entity, e2: Entity| -> bool {
					if let Ok(player) = players.get(e1) {
						if let Ok(spawn_point) = spawn_points.get(e2) {
							match spawn_point.get_id() {
								None => false,
								Some(id) => {
									// e1 is player, e2 is spawn point, and spawn point is occupied
									// if the spawn point is occupied by the player, then ignore the collision
									if id == player.get_id() {
										// reject self-collision
										true
									} else {
										false
									}
								}
							}
						} else {
							false
						}
					} else {
						false
					}
				};

				!(check(contacts.entity1, contacts.entity2) || check(contacts.entity2, contacts.entity1))
			});
		}

		pub(super) fn creation_spawn_points(mut commands: Commands, mut mma: MMA) {
			let spawn_points: Vec<SpawnPointBundle> = (0..8)
				.map(|n| {
					// 8 corners of a cube
					let pos: Vec3 = vec3_polar(n as f32 * TAU / 8., n as f32 * TAU / 8.);
					let transform = Transform::from_translation(pos * 10.);
					let blueprint = SpawnPointBlueprint::new(transform, None);
					SpawnPointBundle::stamp_from_blueprint(&blueprint, &mut mma)
				})
				.collect();
			commands.spawn_batch(spawn_points);
		}
	}
}

mod blueprint {
	use crate::prelude::*;

	#[derive(Debug, Component, Reflect, Clone, Serialize, Deserialize)]
	pub struct SpawnPointBlueprint {
		pub at: Transform,
		pub size: f32,

		#[reflect(ignore)]
		pub initial_occupation: Option<ClientId>,
	}

	impl SpawnPointBlueprint {
		pub fn new(at: Transform, initially_occupied: Option<ClientId>) -> Self {
			Self {
				at,
				initial_occupation: initially_occupied,
				..default()
			}
		}
	}

	impl Default for SpawnPointBlueprint {
		fn default() -> Self {
			Self {
				at: Transform::default(),
				size: 3.0,
				initial_occupation: None,
			}
		}
	}
}

mod bundle {
	use crate::prelude::*;

	use super::{blueprint::SpawnPointBlueprint, components::SpawnPoint};

	#[derive(Bundle)]
	pub struct SpawnPointBundle {
		pbr: PbrBundle,
		name: Name,
		spawn_point: SpawnPoint,
		rigid_body: RigidBody,
		collider: AsyncCollider,
	}

	impl FromBlueprint for SpawnPointBundle {
		type Blueprint = SpawnPointBlueprint;

		fn stamp_from_blueprint(
			SpawnPointBlueprint {
				at,
				size,
				initial_occupation,
			}: &Self::Blueprint,
			mma: &mut MMA,
		) -> Self {
			SpawnPointBundle {
				pbr: PbrBundle {
					transform: *at,
					mesh: mma.meshs.add(
						shape::UVSphere {
							radius: *size,
							..default()
						}
						.into(),
					),
					material: mma.mats.add(StandardMaterial {
						base_color: Color::BLUE,
						emissive: Color::BLUE,
						specular_transmission: 0.7,
						thickness: 0.7,
						ior: 1.33,
						// attenuation_distance: 0.0,
						// attenuation_color: (),
						// normal_map_texture: (),
						// flip_normal_map_y: (),
						// alpha_mode: (),
						// opaque_render_method: (),
						..default()
					}),
					..default()
				},
				name: Name::new("SpawnPoint"),
				spawn_point: SpawnPoint::new(*initial_occupation),
				rigid_body: RigidBody::Dynamic,
				collider: AsyncCollider::default(),
			}
		}
	}
}

mod components {
	use crate::prelude::*;

	#[derive(Component, Debug, Reflect)]
	pub(super) struct SpawnPoint {
		/// Player that occupies this spawn point.
		#[reflect(ignore)]
		player_occupation: Option<ClientId>,
	}

	impl SpawnPoint {
		pub(super) fn new(occupation: Option<ClientId>) -> Self {
			Self {
				player_occupation: occupation,
			}
		}

		pub(super) fn get_id(&self) -> Option<ClientId> {
			self.player_occupation
		}
	}
}
