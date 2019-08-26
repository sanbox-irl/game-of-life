use imgui::*;

#[allow(dead_code)]
pub struct Imgui {}

#[allow(dead_code)]
impl Imgui {
    pub fn test() {
        let mut imgui = Context::create();
        imgui.set_ini_filename(None);

        // Window::new(im_str!("Hello Dear ImGUI"));
    }
}
