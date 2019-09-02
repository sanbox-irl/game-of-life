use super::{Gameplay, UserInput, Window as WinitWindow};
use imgui::{Condition, Context, Direction, FontConfig, FontSource, ImGuiWindowFlags, Ui, Window};
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
        platform.attach_window(imgui.io_mut(), &window.window, HiDpiMode::Locked(1.0));

        let hidpi_factor = platform.hidpi_factor();
        let font_size = (26.0 * hidpi_factor) as f32;
        imgui.fonts().add_font(&[FontSource::DefaultFontData {
            config: Some(FontConfig {
                size_pixels: font_size,
                ..FontConfig::default()
            }),
        }]);

        Imgui { imgui, platform }
    }

    pub fn take_input(&mut self, user_input: &mut UserInput) {
        // Set them to false
        let io = self.imgui.io_mut();
        for i in 0..20 {
            io.keys_down[i] = false;
        }
        io.key_ctrl = false;
        io.key_shift = false;
        io.key_alt = false;
        io.key_super = false;

        // Held Keys
        for this_keycode in &user_input.kb_input.held_keys {
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

        let mouse_input = &user_input.mouse_input;
        io.mouse_pos = (mouse_input.mouse_position * 2.0).into();
        io.mouse_down = mouse_input.mouse_held;
        io.mouse_wheel = mouse_input.mouse_vertical_scroll_delta;

        for this_char in &user_input.kb_input.received_char {
            io.add_input_character(*this_char);
        }

        if io.mouse_down[0] && io.want_capture_mouse {
            user_input.mouse_input.mouse_input_taken = true;
        }
    }

    pub fn begin_frame<'a>(&mut self, window: &WinitWindow) -> UiHandler<'_> {
        self.platform
            .prepare_frame(self.imgui.io_mut(), &window.window)
            .expect("Failed to prepare a frame");
        let ui = self.imgui.frame();

        UiHandler {
            ui,
            platform: &self.platform,
        }
    }

    pub fn make_ui(ui_handler: &UiHandler<'_>, gameplay: &mut Gameplay) {
        let ui = &ui_handler.ui;

        // Auto-Increment World
        Window::new(ui, im_str!("Game of Life"))
            .size([800.0, 400.0], Condition::FirstUseEver)
            .flags(ImGuiWindowFlags::NoResize)
            .build(|| {
                // Mode
                let do_auto_increment =
                    ui.radio_button_bool(im_str!("Auto-Increment"), gameplay.auto_increment);
                let do_manual_increment =
                    ui.radio_button_bool(im_str!("Manual Increment"), !gameplay.auto_increment);

                if do_auto_increment || do_manual_increment {
                    gameplay.auto_increment = !gameplay.auto_increment;
                }

                if gameplay.auto_increment {
                    ui.separator();
                    ui.input_float(im_str!("Increments Per Second"), &mut gameplay.increment_rate)
                        .build();
                }
            });

        Window::new(ui, im_str!("Tools"))
            .size([1600.0, 100.0], Condition::FirstUseEver)
            .flags(ImGuiWindowFlags::NoResize)
            .title_bar(false)
            .build(|| {
                const VERT_SIZE: f32 = 82.0;
                const SPACE: f32 = 25.0;
                let mut horizontal = 550.0;
                ui.same_line(50.0);
                // Selection
                let change = ui.button(
                    &im_str!(
                        "Change to: {} Cell Mode",
                        if gameplay.single_selection {
                            "Multiple"
                        } else {
                            "Single"
                        }
                    ),
                    [500.0, VERT_SIZE],
                );

                if change {
                    gameplay.single_selection = !gameplay.single_selection;
                }

                horizontal += SPACE;
                ui.same_line(horizontal);

                // Tools
                horizontal += 200.0;
                let copy = ui.button(im_str!("Copy"), [200.0, VERT_SIZE]);
                horizontal += SPACE;
                ui.same_line(horizontal);

                horizontal += 200.0;
                let cut = ui.button(im_str!("Cut"), [200.0, VERT_SIZE]);
                horizontal += SPACE;
                ui.same_line(horizontal);

                horizontal += 200.0;
                let paste = ui.button(im_str!("Paste"), [200.0, VERT_SIZE]);
                horizontal += SPACE;
                ui.same_line(horizontal);

                horizontal += 300.0;
                let shapes = ui.button(im_str!("Copy Prefab"), [300.0, VERT_SIZE]);
            });

        Window::new(ui, im_str!("Instructions"))
            .size([500.0, 350.0], Condition::FirstUseEver)
            .flags(ImGuiWindowFlags::NoResize)
            .title_bar(false)
            .build(|| {
                ui.text_wrapped(im_str!(
                    "INSTRUCTIONS:
Use Enter to advance world.
Click and drag multi-selection to move.
Control-Z to undo user-action.
Control-B to undo auto-world advancement.
Press the X to remove these instructions. F2 returns them.
Press F1 to hide all UI."
                ));
            });

        Window::new(ui, im_str!("Prefabs"))
            .size([200.0, 1000.0], Condition::FirstUseEver)
            .flags(ImGuiWindowFlags::NoResize)
            .title_bar(false)
            .build(|| {
                ui.button(im_str!("Glider"), [200.0, 50.0]);
                ui.button(im_str!("Small Exploder"), [200.0, 50.0]);
                ui.button(im_str!("Exploder"), [200.0, 50.0]);
                ui.button(im_str!("Spaceship"), [200.0, 50.0]);
                ui.button(im_str!("Tumbler"), [200.0, 50.0]);
                ui.button(im_str!("Glider Gun"), [200.0, 50.0]);
            });
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
            Key::Insert => 9,
            Key::Delete => 10,
            Key::Back => 11,
            Key::Space => 12,
            Key::Return => 13,
            Key::Escape => 14,
            Key::A => 15,
            Key::C => 16,
            Key::V => 17,
            Key::X => 18,
            Key::Y => 19,
            Key::Z => 20,
            _ => return None,
        })
        /*
        io[Key::Tab] = VirtualKeyCode::Tab as _;
        io[Key::LeftArrow] = VirtualKeyCode::Left as _;
        io[Key::RightArrow] = VirtualKeyCode::Right as _;
        io[Key::UpArrow] = VirtualKeyCode::Up as _;
        io[Key::DownArrow] = VirtualKeyCode::Down as _;
        io[Key::PageUp] = VirtualKeyCode::PageUp as _;
        io[Key::PageDown] = VirtualKeyCode::PageDown as _;
        io[Key::Home] = VirtualKeyCode::Home as _;
        io[Key::End] = VirtualKeyCode::End as _;
        io[Key::Insert] = VirtualKeyCode::Insert as _;
        io[Key::Delete] = VirtualKeyCode::Delete as _;
        io[Key::Backspace] = VirtualKeyCode::Back as _;
        io[Key::Space] = VirtualKeyCode::Space as _;
        io[Key::Enter] = VirtualKeyCode::Return as _;
        io[Key::Escape] = VirtualKeyCode::Escape as _;
        io[Key::A] = VirtualKeyCode::A as _;
        io[Key::C] = VirtualKeyCode::C as _;
        io[Key::V] = VirtualKeyCode::V as _;
        io[Key::X] = VirtualKeyCode::X as _;
        io[Key::Y] = VirtualKeyCode::Y as _;
        io[Key::Z] = VirtualKeyCode::Z as _;

        */
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
