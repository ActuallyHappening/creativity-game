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
#![windows_subsystem = "windows"]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::type_complexity)]

mod blocks;
mod cameras;
mod global;
mod netcode;
mod players;
mod prelude;
mod states;
mod ui;
mod utils;
mod world;

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
				(BlueprintExpansion::Player, BlueprintExpansion::Thruster)
					.chain()
					.in_set(GlobalSystemSet::BlueprintExpansion),
				(
					GlobalSystemSet::BlueprintExpansion,
					GlobalSystemSet::PlayerMovement,
					GlobalSystemSet::PlayerThrusters,
					GlobalSystemSet::RawPhysics,
					GlobalSystemSet::GameLogic,
				)
					.chain(),
				(
					PhysicsSet::Prepare,
					PhysicsSet::StepSimulation,
					PhysicsSet::Sync,
				)
					.in_set(GlobalSystemSet::RawPhysics),
				ServerSet::Send.after(GlobalSystemSet::GameLogic),
			),
		);

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
			PhysicsPlugins::new(FixedUpdate),
			#[cfg(feature = "debug")]
			PhysicsDebugPlugin::default(),
			bevy_xpbd3d_parenting::PhysicsParentingPlugin,
			HanabiPlugin,
			ReplicationPlugins
				.build()
				.set(ServerPlugin::new(TickPolicy::Manual)),
			TimewarpPlugin::new(TimewarpConfig::new(
				GlobalSystemSet::GameLogic,
				GlobalSystemSet::GameLogic,
			)),
			crate::utils::scenes::HelperScene,
			bevy_starfield::StarfieldPlugin::default(),
		));
		// dep configuration
		app.insert_resource(Gravity(Vec3::ZERO));
		#[cfg(feature = "editor")]
		app.insert_resource(editor_controls());

		// game logic plugins
		app.add_plugins((
			self::netcode::NetcodePlugin,
			self::cameras::CameraPlugin,
			self::ui::UiPlugins,
			self::players::PlayerPlugins,
			self::blocks::BlockPlugins,
		));
		app.register_type::<BlockId>();

		// network replication
		app.replicate::<Position>().replicate::<Rotation>();
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