use bevy::prelude::*;



#[derive(Resource, Default)]
pub struct GameResources {
    pub in_editor: bool,
    pub mouse_grabbed: bool,
    pub building: bool,
}
