#![feature(macro_metavar_expr)]
#![warn(missing_docs)]

use custom_image_loader::{CustomImageLoaderPlugin, GASImageTextureLoader};
// Use a custom Tiny Allocator
use tcmalloc::TCMalloc;

#[global_allocator]
static GLOBAL: TCMalloc = TCMalloc;

use bevy::diagnostic::Diagnostics;
use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;
use bevy::utils::Duration;
use bevy_ecs_tilemap::TilemapPlugin;
use bevy_framepace::*;
use bevy_screen_diags::{FrameCounter, ScreenDiagsPlugin};
use bytesize::ByteSize;
use lazy_static::lazy_static;
use std::fmt::Write;
use sysinfo::SystemExt;

mod clamped;

mod camera;
use camera::move_camera;

mod tiles;
use tiles::SetupTilemapPlugin;

mod custom_image_loader;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugin(TilemapPlugin)
        .add_plugin(SetupTilemapPlugin)
        .add_plugin(CustomImageLoaderPlugin)
        .add_plugin(FramepacePlugin)
        .add_plugin(ScreenDiagsPlugin)
        .add_startup_system(startup)
        .add_system(move_camera)
        .add_system(update_fps_counter.run_if(on_timer(Duration::from_secs(1))))
        .run();
}

/// A struct to get track the camera's "velocity"
#[derive(Component)]
pub struct CameraComponent {
    /// Multiplier to the speed that the camera is moving at
    pub held_down_mult: f32,
}

fn startup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut frame_settings: ResMut<FramepaceSettings>,
) {
    frame_settings.limiter = Limiter::Auto;

    commands.spawn((
        Camera2dBundle::default(),
        CameraComponent {
            held_down_mult: 1.0,
        },
    ));
    let font = asset_server.load("fonts/FiraCode-Regular.ttf");
    let style = TextStyle {
        font: font.clone(),
        font_size: 30.0,
        color: Color::WHITE,
    };
    commands.spawn(TextBundle::from_sections([TextSection::from_style(
        TextStyle {
            font: asset_server.load("fonts/FiraCode-Regular.ttf"),
            font_size: 20.0,
            color: Color::WHITE,
        },
    )]));
}

lazy_static! {
    static ref SYSTEM: sysinfo::System = sysinfo::System::new_all();
}

fn update_fps_counter(
    mut text: Query<&mut Text>,
    diagnostics: Res<Diagnostics>,
    frame_counter: Res<FrameCounter>,
) {
    let mut text = text.single_mut();
    let value = &mut text.sections[0].value;
    value.clear();
    // Get the memory usage of the program
    let (physical, r#virtual_old) = if let Some(usage) = memory_stats::memory_stats() {
        (
            usage.physical_mem as u64 >> 10,
            usage.virtual_mem as u64 >> 10,
        )
    } else {
        (0, 0)
    };

    let ram_total = SYSTEM.total_memory();
    let swap_total = SYSTEM.total_swap();
    let physical = ByteSize::kb(physical);
    let r#virtual = ByteSize::kb(r#virtual_old);

    write!(
        value,
        "{:.0}fps\nMem: {} / {} {:.2}%\nSwap: {} / {} | {:.2}%",
        frame_counter.0,
        physical.to_string_as(true),
        ByteSize(ram_total).to_string_as(true),
        (physical.as_u64() as f32 / ram_total as f32) * 100.0,
        r#virtual.to_string_as(true),
        ByteSize(swap_total).to_string_as(true),
        (r#virtual.as_u64() as f32 / swap_total as f32) * 100.0
    )
    .unwrap();
}
