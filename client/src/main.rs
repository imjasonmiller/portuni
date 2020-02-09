mod state;
mod system;
mod utils;

mod cobs_buffer;
mod compass;
mod config;
mod transceiver;

use state::app::App;

use amethyst::{
    core::transform::TransformBundle,
    input::{InputBundle, StringBindings},
    prelude::*,
    renderer::{
        plugins::{RenderPbr3D, RenderToWindow},
        types::DefaultBackend,
        RenderingBundle,
    },
    ui::{RenderUi, UiBundle},
    utils::application_root_dir,
};

// Move this to a seperate entity creation system
use amethyst::{
    assets::{AssetPrefab, PrefabData, PrefabLoaderSystemDesc, ProgressCounter},
    core::transform::Transform,
    derive::PrefabData,
    ecs::prelude::{Component, DenseVecStorage},
    ecs::Entity,
    gltf::{GltfSceneAsset, GltfSceneFormat, GltfSceneLoaderSystemDesc},
    utils::tag::Tag,
    Error,
};
use serde::{self, Deserialize, Serialize};

// The tag
#[derive(Clone, Serialize, Deserialize)]
pub struct DroneMarker;

impl Component for DroneMarker {
    type Storage = DenseVecStorage<Self>;
}

#[derive(Default, Deserialize, Serialize, PrefabData)]
#[serde(default)]
pub struct ScenePrefabData {
    transform: Option<Transform>,
    gltf: Option<AssetPrefab<GltfSceneAsset, GltfSceneFormat>>,
    tag: Option<Tag<DroneMarker>>,
}

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    let app_root = application_root_dir()?;
    let assets_dir = app_root.join("assets");
    let display_path = app_root.join("config").join("display.ron");

    let app_data = GameDataBuilder::default()
        .with_bundle(TransformBundle::new())?
        .with_system_desc(
            PrefabLoaderSystemDesc::<ScenePrefabData>::default(),
            "scene_loader",
            &[],
        )
        .with_system_desc(
            GltfSceneLoaderSystemDesc::default(),
            "gltf_loader",
            &["scene_loader"], // This is important so that entity instantiation is performed in a single frame.
        )
        .with_bundle(UiBundle::<StringBindings>::new())?
        .with_system_desc(
            system::ui::UiEventHandlerSystemDesc::default(),
            "ui_event_handler",
            &[],
        )
        .with_system_desc(
            system::transceiver::TransceiverCodecSystem::new(),
            "transceiver_codec",
            &[],
        )
        .with_bundle(InputBundle::<StringBindings>::new())?
        .with_bundle(
            RenderingBundle::<DefaultBackend>::new()
                .with_plugin(RenderToWindow::from_config_path(display_path)?.with_clear([
                    // Linear colorspace
                    f32::powf((30.0 / 255.0 + 0.055) / 1.055, 2.4), // R
                    f32::powf((30.0 / 255.0 + 0.055) / 1.055, 2.4), // G
                    f32::powf((30.0 / 255.0 + 0.055) / 1.055, 2.4), // B
                    1.0,                                            // A
                ]))
                .with_plugin(RenderUi::default())
                .with_plugin(RenderPbr3D::default()),
        )?;

    let mut app = Application::new(assets_dir, App::default(), app_data)?;
    app.run();

    Ok(())
}
