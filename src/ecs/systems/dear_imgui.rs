use super::Window as WinitWindow;
use imgui::*;
use imgui_winit_support::{HiDpiMode, WinitPlatform};

#[allow(dead_code)]
pub struct Imgui {
    pub imgui: Context,
    pub platform: WinitPlatform,
}

#[allow(dead_code)]
impl Imgui {
    pub fn new(window: &WinitWindow) -> Self {
        let mut imgui = Context::create();
        imgui.set_ini_filename(None);

        let mut platform = WinitPlatform::init(&mut imgui);
        platform.attach_window(imgui.io_mut(), &window.window, HiDpiMode::Default);

        let hidpi_factor = platform.hidpi_factor();
        let font_size = (13.0 * hidpi_factor) as f32;
        imgui.fonts().add_font(&[
            FontSource::DefaultFontData {
                config: Some(FontConfig {
                    size_pixels: font_size,
                    ..FontConfig::default()
                }),
            },
            FontSource::TtfData {
                config: None,
                data: include_bytes!("../../../resources/fonts/mplus-1p-regular.ttf"),
                size_pixels: font_size,
            },
        ]);
        imgui.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;
        imgui.fonts().build_rgba32_texture();

        Imgui { imgui, platform }
    }

    pub fn begin_frame<'a>(&mut self, window: &WinitWindow) -> UiHandler<'_> {
        self.platform
            .prepare_frame(self.imgui.io_mut(), &window.window)
            .expect("Failed to prepare a frame");
        let ui = self.imgui.frame();

        Window::new(&ui, im_str!("Hello world"))
            .size([300.0, 100.0], Condition::FirstUseEver)
            .build(|| {});

        UiHandler {
            ui,
            platform: &self.platform,
        }
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
