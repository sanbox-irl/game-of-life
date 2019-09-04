use super::{Gameplay, Time, UserInput, Vec2, Window as WinitWindow};
use imgui::{Condition, Context, FontConfig, FontSource, ImGuiWindowFlags, Ui, Window};
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use std::collections::HashMap;

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

        imgui.fonts().add_font(&[FontSource::DefaultFontData {
            config: Some(FontConfig {
                size_pixels: (13.0 * platform.hidpi_factor()) as f32,
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
        io.mouse_pos = (mouse_input.mouse_position * self.platform.hidpi_factor() as f32).into();
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

        self.platform
            .attach_window(self.imgui.io_mut(), &window.window, HiDpiMode::Default);

        let ui = self.imgui.frame();

        UiHandler {
            ui,
            platform: &self.platform,
            size: window.get_window_size(),
            map: HashMap::new(),
        }
    }

    pub fn make_ui(ui_handler: &mut UiHandler<'_>, gameplay: &mut Gameplay) {
        let ui = &ui_handler.ui;

        if gameplay.show_ui == false {
            return;
        }

        // Auto-Increment World
        Window::new(ui, im_str!("Game of Life"))
            .size([300.0, 100.0], Condition::FirstUseEver)
            .position(
                [
                    ui_handler.size.x - ((ui_handler.size.x - PWS) / 2.0) - 300.0,
                    (ui_handler.size.y - PWH * 1.5) - 100.0,
                ],
                Condition::Always,
            )
            .flags(
                ImGuiWindowFlags::NoResize
                    | ImGuiWindowFlags::NoScrollbar
                    | ImGuiWindowFlags::NoTitleBar
                    | ImGuiWindowFlags::NoMove,
            )
            .build(|| {
                // Mode
                let do_auto_increment =
                    ui.radio_button_bool(im_str!("Automatically Increment"), gameplay.auto_increment);

                if gameplay.auto_increment {
                    let a = ui.push_item_width(80.0);
                    ui.slider_float(im_str!("Per Second"), &mut gameplay.increment_rate, 0.0, 25.0)
                        .build();
                    drop(a);

                    ui.same_line(175.0);

                    let play_pause = ui.button(
                        if gameplay.playing {
                            im_str!("Pause (space)")
                        } else {
                            im_str!("Play (space)")
                        },
                        [100.0, 20.0],
                    );
                    if play_pause {
                        gameplay.playing = !gameplay.playing;
                    }
                }
                let do_manual_increment =
                    ui.radio_button_bool(im_str!("Manual Increment"), !gameplay.auto_increment);

                if do_auto_increment || do_manual_increment {
                    gameplay.auto_increment = !gameplay.auto_increment;
                }
            });

        if gameplay.show_instructions {
            Window::new(ui, im_str!("Instructions"))
                .size([400.0, 200.0], Condition::FirstUseEver)
                .flags(ImGuiWindowFlags::NoResize)
                .position(
                    ((ui_handler.size / 2.0) - Vec2::new(200.0, 100.0)).into(),
                    Condition::Always,
                )
                .title_bar(false)
                .build(|| {
                    ui.text_wrapped(im_str!(
                        "INSTRUCTIONS:

CLICK on a cell to change it from LIVE to DEAD.
Use the MOUSE WHEEL to zoom in and out.

Click on a Prefab below, then on a cell,
to PASTE it into the world.

Press F2 to bring these instructions back.
Press F1 to hide all UI."
                    ));
                    // ui.same_line_with_spacing(0.0, 20.0);
                    ui.spacing();
                    ui.spacing();
                    ui.spacing();
                    ui.spacing();

                    ui.indent_by(150.0);
                    let pressed = ui.button(im_str!("Okay, got it"), [100.0, 25.0]);
                    if pressed {
                        gameplay.show_instructions = false;
                    }
                });
        }

        // PREFABS
        const PWS: f32 = 1000.0;
        const PWH: f32 = 75.0;
        const BUTTON: f32 = PWS / 7.0;
        Window::new(ui, im_str!("Prefabs"))
            .size([PWS, PWH], Condition::FirstUseEver)
            .position(
                [(ui_handler.size.x - PWS) / 2.0, (ui_handler.size.y - PWH * 1.5)],
                Condition::Always,
            )
            .flags(ImGuiWindowFlags::NoResize)
            .title_bar(false)
            .build(|| {
                let mut horizontal = PWS / 14.0;

                ui.spacing();
                ui.spacing();

                ui.same_line(horizontal);
                ui.button(im_str!("Glider"), [BUTTON, 50.0]);
                horizontal += BUTTON;

                ui.same_line(horizontal);
                ui.button(im_str!("Small Exploder"), [BUTTON, 50.0]);
                horizontal += BUTTON;

                ui.same_line(horizontal);
                ui.button(im_str!("Exploder"), [BUTTON, 50.0]);
                horizontal += BUTTON;

                ui.same_line(horizontal);
                ui.button(im_str!("Spaceship"), [BUTTON, 50.0]);
                horizontal += BUTTON;

                ui.same_line(horizontal);
                ui.button(im_str!("Tumbler"), [BUTTON, 50.0]);
                horizontal += BUTTON;

                ui.same_line(horizontal);
                ui.button(im_str!("Glider Gun"), [BUTTON, 50.0]);
                horizontal += BUTTON;
                horizontal += PWS / 14.0;

                ui.same_line(horizontal);
            });
    }

    pub fn make_debug_ui(ui_handler: &UiHandler<'_>, gameplay: &Gameplay, time: &Time) {
        let ui = &ui_handler.ui;
        if gameplay.show_debug {
            return;
        }

        // DEBUG
        Window::new(ui, im_str!("Debug Output"))
            .size([300.0, 80.0], Condition::FirstUseEver)
            .build(|| {
                ui.label_text(im_str!("Delta Time:"), &im_str!("{}", time.delta_time));
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
    pub size: Vec2,
    pub map: HashMap<&'static str, Vec2>,
}

impl<'a> UiHandler<'a> {
    pub fn prepare_draw(&self, window: &WinitWindow) {
        self.platform.prepare_render(&self.ui, &window.window);
    }
}
