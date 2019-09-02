use super::{MouseInput, Window as WinitWindow};
use imgui::{Context, FontConfig, FontSource, Ui};
use imgui_winit_support::{HiDpiMode, WinitPlatform};

#[allow(dead_code)]
pub struct Imgui {
    pub imgui: Context,
    pub platform: WinitPlatform,
}

#[allow(dead_code)]
use winit::VirtualKeyCode as Key;
impl Imgui {
    pub fn new(window: &WinitWindow) -> Self {
        let mut imgui = Context::create();
        imgui.set_ini_filename(None);

        let mut platform = WinitPlatform::init(&mut imgui);
        platform.attach_window(imgui.io_mut(), &window.window, HiDpiMode::Default);

        let hidpi_factor = platform.hidpi_factor();
        let font_size = (26.0 * hidpi_factor) as f32;
        imgui.fonts().add_font(&[FontSource::DefaultFontData {
            config: Some(FontConfig {
                size_pixels: font_size,
                ..FontConfig::default()
            }),
        }]);
        imgui.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;
        imgui.fonts().build_rgba32_texture();

        Imgui { imgui, platform }
    }

    pub fn take_input(&mut self, held_keys: &[Key], mouse_input: &MouseInput) {
        // Set them to false
        let io = self.imgui.io_mut();
        for i in 0..18 {
            io.keys_down[i] = false;
        }
        io.key_ctrl = false;
        io.key_shift = false;
        io.key_alt = false;
        io.key_super = false;

        // Held Keys
        for this_keycode in held_keys {
            if let Some(index) = Self::convert_vk_to_imgui_key(this_keycode) {
                io.keys_down[index] = true;
            } else {
                match this_keycode {
                    Key::LControl | Key::RControl => io.key_ctrl = true,
                    Key::LShift | Key::RShift => io.key_shift = true,
                    Key::LAlt | Key::RAlt => io.key_alt = true,
                    Key::LWin | Key::RWin => io.key_super = true,
                    _ => {}
                }
            }
        }

        io.mouse_pos = mouse_input.mouse_position.into();
        io.mouse_down = mouse_input.mouse_held;
        io.mouse_wheel = mouse_input.mouse_vertical_scroll_delta;
    }

    pub fn begin_frame<'a>(&mut self, window: &WinitWindow) -> UiHandler<'_> {
        self.platform
            .prepare_frame(self.imgui.io_mut(), &window.window)
            .expect("Failed to prepare a frame");
        let ui = self.imgui.frame();
        let mut okay = false;
        ui.show_demo_window(&mut okay);

        UiHandler {
            ui,
            platform: &self.platform,
        }
    }

    fn convert_vk_to_imgui_key(key: &Key) -> Option<usize> {
        Some(match key {
            Key::Tab => 0,
            Key::Left => 1,
            Key::Right => 2,
            Key::Up => 3,
            Key::Down => 4,
            Key::PageUp => 5,
            Key::PageDown => 6,
            Key::Home => 7,
            Key::End => 8,
            Key::Delete => 9,
            Key::Back => 10,
            Key::Return => 11,
            Key::Escape => 12,
            Key::A => 13,
            Key::C => 14,
            Key::V => 15,
            Key::X => 16,
            Key::Y => 17,
            Key::Z => 18,
            _ => return None,
        })
    }
}

pub struct UiHandler<'a> {
    pub ui: Ui<'a>,
    pub platform: &'a WinitPlatform,
}

impl<'a> UiHandler<'a> {
    pub fn prepare_draw(&self, window: &WinitWindow) {
        self.platform.prepare_render(&self.ui, &window.window);
    }
}
