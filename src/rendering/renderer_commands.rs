use super::{Entity, Vec2, GameColors};

pub struct RendererCommands<'a> {
    pub game_world_draw_commands: Option<GameWorldDrawCommands<'a>>,
    pub imgui_draw_commands: Option<ImGuiDrawCommands<'a>>,
}

pub struct GameWorldDrawCommands<'a> {
    pub entities: &'a mut [Vec<Entity>],
    pub game_colors: &'a GameColors,
    pub camera_position: &'a Vec2,
    pub camera_scale: f32,
    pub aspect_ratio: f32,
}

pub struct ImGuiDrawCommands<'a> {
    pub draw_data: &'a imgui::DrawData,
    pub imgui_dimensions: Vec2,
}
