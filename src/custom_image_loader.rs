//! Ripped straight from https://github.com/bevyengine/bevy/blob/main/crates/bevy_render/src/texture/image_texture_loader.rs
//! Modified slightly though

use anyhow::Result;
use bevy::ecs::world::Mut;
use bevy::log::info;
use bevy::prelude::*;
use bevy::{
    asset::{AssetLoader, LoadContext, LoadedAsset},
    prelude::{App, FromWorld, Image, Plugin, World},
    render::{
        renderer::RenderDevice,
        texture::{CompressedImageFormats, ImageType, TextureError},
    },
    utils::BoxedFuture,
};
use std::path::Path;
use thiserror::Error;

pub struct CustomImageLoaderPlugin;

impl Plugin for CustomImageLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostStartup, add_asset_loader);
    }
}

fn add_asset_loader(world: &mut World) {
    world.resource_scope(|world, asset_server: Mut<AssetServer>| {
        asset_server.add_loader(GASImageTextureLoader::from_world(world));
    });
}

#[derive(Clone)]
pub struct GASImageTextureLoader {
    supported_compressed_formats: CompressedImageFormats,
}

const FILE_EXTENSIONS: &[&str] = &["png"];

impl AssetLoader for GASImageTextureLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<()>> {
        info!("Using that GAS!!");
        Box::pin(async move {
            // use the file extension for the image type
            let ext = load_context.path().extension().unwrap().to_str().unwrap();

            let dyn_img = Image::from_buffer(
                bytes,
                ImageType::Extension(ext),
                self.supported_compressed_formats,
                true,
            );

            // Return missing texture instead
            if dyn_img.is_err() {
                let path = Path::new("./assets/textures/missing_texture.png");
                let ext = path.extension().unwrap().to_str().unwrap();

                let dyn_img = Image::from_buffer(
                    bytes,
                    ImageType::Extension(ext),
                    self.supported_compressed_formats,
                    true,
                )
                .map_err(|err| GASFileTextureError {
                    error: err,
                    path: format!("{}", path.display()),
                })?;

                load_context.set_default_asset(LoadedAsset::new(dyn_img));
                return Ok(());
            }

            load_context.set_default_asset(LoadedAsset::new(dyn_img.unwrap()));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        FILE_EXTENSIONS
    }
}

impl FromWorld for GASImageTextureLoader {
    fn from_world(world: &mut World) -> Self {
        let supported_compressed_formats = match world.get_resource::<RenderDevice>() {
            Some(render_device) => CompressedImageFormats::from_features(render_device.features()),

            None => CompressedImageFormats::all(),
        };
        Self {
            supported_compressed_formats,
        }
    }
}

/// An error that occurs when loading a texture from a file.
#[derive(Error, Debug)]
pub struct GASFileTextureError {
    error: TextureError,
    path: String,
}
impl std::fmt::Display for GASFileTextureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(
            f,
            "Error reading image file {}: {}, this is an error in `bevy_render`.",
            self.path, self.error
        )
    }
}
