// #![deny(clippy::arithmetic_side_effects)]
// #![warn(
// 	clippy::all,
// 	// clippy::restriction,
// 	clippy::pedantic,
// 	clippy::nursery,
// 	clippy::cargo
// )]
// #![allow(
// 	clippy::module_name_repetitions,
// 	clippy::needless_pass_by_value,
// 	clippy::too_many_arguments,
// 	clippy::wildcard_imports,
// 	clippy::use_self
// )]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::type_complexity)]

mod blocks;
mod cameras;
mod global;
mod netcode;
mod physics;
mod players;
mod prelude;
mod states;
mod ui;
mod utils;
// mod world;

use bevy::ecs::schedule::{LogLevel, ScheduleBuildSettings};
#[allow(unused_imports)]
use bevy_mod_picking::{
	prelude::{DebugPickingPlugin, DefaultHighlightingPlugin},
	DefaultPickingPlugins,
};
use bevy_replicon::ReplicationPlugins;

use crate::prelude::*;

pub struct MainPlugin;

impl Plugin for MainPlugin {
	fn build(&self, app: &mut App) {
		use bevy_screen_diagnostics::{ScreenDiagnosticsPlugin, ScreenFrameDiagnosticsPlugin};

		info!("MainPlugin initializing ...");
		app.add_systems(Startup, || info!("Startup running"));

		// global system set configuration
		app.configure_sets(
			FixedUpdate,
			(
				(
					GlobalSystemSet::WorldCreation,
					GlobalSystemSet::PlayerMovement,
					GlobalSystemSet::RawPhysics,
					GlobalSystemSet::ExecuteGameLogic,
					GlobalSystemSet::BlueprintExpansion,
				)
					.chain(),
				(
					PhysicsSet::Prepare,
					PhysicsSet::StepSimulation,
					PhysicsSet::Sync,
				)
					.in_set(GlobalSystemSet::RawPhysics),
				ServerSet::Send.after(GlobalSystemSet::ExecuteGameLogic),
			),
		);
		// Set up the physics schedule, the schedule that advances the physics simulation
		app.edit_schedule(GameLogic, |schedule| {
			schedule
				// .set_executor_kind(ExecutorKind::SingleThreaded)
				.set_build_settings(ScheduleBuildSettings {
					ambiguity_detection: LogLevel::Error,
					..default()
				});
		});
		// todo: make deterministic
		// app.edit_schedule(FixedUpdate, |schedule| {
		// 	schedule
		// 		// .set_executor_kind(ExecutorKind::SingleThreaded)
		// 		.set_build_settings(ScheduleBuildSettings {
		// 			ambiguity_detection: LogLevel::Error,
		// 			..default()
		// 		});
		// });
		app.add_systems(
			FixedUpdate,
			run_game_logic.in_set(GlobalSystemSet::ExecuteGameLogic),
		);
		fn run_game_logic(world: &mut World) {
			// trace!("Running Game Logic");
			world.try_run_schedule(GameLogic).ok();
		}

		// spawn initial light
		app.add_systems(Startup, |mut commands: Commands| {
			// commands.spawn(DirectionalLightBundle {
			// 	directional_light: DirectionalLight {
			// 		shadows_enabled: true,
			// 		..default()
			// 	},
			// 	..default()
			// });

			commands.insert_resource(AmbientLight {
				color: Color::WHITE,
				brightness: 0.1,
			});
		});

		// will take cli inputs, or default to start menu
		// app.add_state::<GlobalGameStates>();
		let state;
		if std::env::args().len() > 1 {
			info!("Using options provided by CLI");
			state = GlobalGameStates::InGame;
			app.insert_resource(self::netcode::NetcodeConfig::parse());
		} else {
			state = GlobalGameStates::StartMenu;
		}
		app
			.insert_resource::<State<GlobalGameStates>>(State::new(state))
			.init_resource::<NextState<GlobalGameStates>>()
			.add_systems(
				StateTransition,
				(
					bevy::ecs::schedule::run_enter_schedule::<GlobalGameStates>.run_if(run_once()),
					apply_state_transition::<GlobalGameStates>,
				)
					.chain(),
			);

		let picking_plugins = DefaultPickingPlugins
			.build()
			// .disable::<DebugPickingPlugin>()
			.disable::<DefaultHighlightingPlugin>();
		#[cfg(not(feature = "debug"))]
		let picking_plugins = picking_plugins.disable::<DebugPickingPlugin>();

		// dep plugins
		app.add_plugins((
			#[cfg(feature = "editor")]
			bevy_editor_pls::EditorPlugin::default(),
			ScreenDiagnosticsPlugin::default(),
			ScreenFrameDiagnosticsPlugin,
			picking_plugins,
			physics::PhysicsPlugin,
			HanabiPlugin,
			ReplicationPlugins.build().set(ServerPlugin {
				tick_policy: TickPolicy::Manual,
				update_timeout: Duration::from_secs(5),
			}),
			TimewarpPlugin::new(TimewarpConfig::new(
				GlobalSystemSet::ExecuteGameLogic,
				GlobalSystemSet::ExecuteGameLogic,
			)),
			// crate::utils::scenes::HelperScene,
		));

		// personally built projects
		app.add_plugins(bevy_starfield::StarfieldPlugin::default());

		// dep configuration
		#[cfg(feature = "editor")]
		app.insert_resource(editor_controls());

		// game logic plugins
		app.add_plugins((
			global::blueprints::BlueprintsPlugin,
			self::netcode::NetcodePlugin,
			self::cameras::CameraPlugin,
			self::ui::UiPlugins,
			self::players::PlayerPlugins,
			self::blocks::BlockPlugins,
		));
		app.register_type::<BlockId>();

		// general network replication
		replicate_marked!(app, Transform);
	}
}

#[cfg(feature = "editor")]
fn editor_controls() -> bevy_editor_pls::controls::EditorControls {
	use bevy_editor_pls::controls;
	use bevy_editor_pls::controls::EditorControls;

	let mut editor_controls = EditorControls::default_bindings();
	editor_controls.unbind(controls::Action::PlayPauseEditor);

	editor_controls.insert(
		controls::Action::PlayPauseEditor,
		controls::Binding {
			input: controls::UserInput::Single(controls::Button::Keyboard(KeyCode::Backslash)),
			conditions: vec![controls::BindingCondition::ListeningForText(false)],
		},
	);

	editor_controls
}
