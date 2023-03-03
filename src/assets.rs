use bevy::prelude::*;

pub struct GameAssetsPlugin;

impl Plugin for GameAssetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system_to_stage(StartupStage::PreStartup, asset_loading);
    }
}
pub struct AudioHandles {
    pub drop_002: Handle<AudioSource>,
    pub drop_003: Handle<AudioSource>,
    pub drop_004: Handle<AudioSource>,
}
pub struct ImageHandles {
    pub ball: Handle<Image>,
    pub paddle: Handle<Image>,
    pub brick_blue: Handle<Image>,
    pub brick_green: Handle<Image>,
    pub brick_red: Handle<Image>,
    pub brick_yellow: Handle<Image>,
    pub brick_orange: Handle<Image>,
    pub brick_cyan: Handle<Image>,
    pub brick_light_green: Handle<Image>,
    pub background: Handle<Image>,
}
#[derive(Resource)]
pub struct GameAssets {
    pub audio: AudioHandles,
    pub image: ImageHandles,
}

fn asset_loading(mut commands: Commands, assets: Res<AssetServer>) {
    commands.insert_resource(GameAssets {
        audio: AudioHandles {
            drop_002: assets.load("sound/drop_002.ogg"),
            drop_003: assets.load("sound/drop_003.ogg"),
            drop_004: assets.load("sound/drop_004.ogg"),
        },
        image: ImageHandles {
            ball: assets.load("images/ball.png"),
            paddle: assets.load("images/paddle.png"),
            brick_blue: assets.load("images/brick_blue.png"),
            brick_green: assets.load("images/brick_green.png"),
            brick_red: assets.load("images/brick_red.png"),
            brick_yellow: assets.load("images/brick_yellow.png"),
            brick_orange: assets.load("images/brick_orange.png"),
            brick_cyan: assets.load("images/brick_cyan.png"),
            brick_light_green: assets.load("images/brick_light_green.png"),
            background: assets.load("images/background.jpeg"),
        },
    });
}
