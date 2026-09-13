use bevy::prelude::*;

/// Component marker for the back button in menus.
#[derive(Component)]
pub struct BackButton;

/// Component marker holding the save file name for a list item in the load menu.
#[derive(Component)]
pub struct SaveHouseListItem(pub String);

/// Component marker holding the save file name for the delete button in the load menu.
#[derive(Component)]
pub struct DeleteSaveButton(pub String);

/// Component marker for the root node of the in-game HUD.
#[derive(Component)]
pub struct GameHudRoot;