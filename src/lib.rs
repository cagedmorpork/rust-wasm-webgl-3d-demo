extern crate wasm_bindgen;
use wasm_bindgen::prelude::*;
use web_sys::WebGlRenderingContext as GL;
use web_sys::*;

#[macro_use]
extern crate lazy_static;

mod app_state;
mod common_funcs;
mod constants;
mod gl_setup;
mod programs;
mod shaders;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
pub struct Client {
    gl: WebGlRenderingContext,
    program_color_2d: programs::Color2D,
    _program_color_2d_gradient: programs::Color2DGradient,
    program_graph_3d: programs::Graph3d,
}

#[wasm_bindgen]
impl Client {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        console_error_panic_hook::set_once();
        let gl = gl_setup::initialize_webgl_context().unwrap();
        Self {
            program_color_2d: programs::Color2D::new(&gl),
            _program_color_2d_gradient: programs::Color2DGradient::new(&gl),
            program_graph_3d: programs::Graph3d::new(&gl),
            gl: gl,
        }
    }

    pub fn update(&mut self, time: f32, height: f32, width: f32) -> Result<(), JsValue> {
        app_state::update_dynamic_data(time, height, width);
        Ok(())
    }

    pub fn render(&self) {
        self.gl.clear(GL::COLOR_BUFFER_BIT | GL::DEPTH_BUFFER_BIT);

        let cur_app_state = app_state::get_cur_state();

        self.program_color_2d.render(
            &self.gl,
            cur_app_state.control_bottom,
            cur_app_state.control_top,
            cur_app_state.control_left,
            cur_app_state.control_right,
            cur_app_state.canvas_height,
            cur_app_state.canvas_width,
        );
        // self.program_color_2d_gradient.render(
        //     &self.gl,
        //     cur_app_state.control_bottom + 30.,
        //     cur_app_state.control_top - 30.,
        //     cur_app_state.control_left + 30.,
        //     cur_app_state.control_right - 30.,
        //     cur_app_state.canvas_height,
        //     cur_app_state.canvas_width,
        // );
        self.program_graph_3d.render(
            &self.gl,
            cur_app_state.control_bottom,
            cur_app_state.control_top,
            cur_app_state.control_left,
            cur_app_state.control_right,
            cur_app_state.canvas_height,
            cur_app_state.canvas_width,
            cur_app_state.rotation_x_axis,
            cur_app_state.rotation_y_axis,
            &common_funcs::get_updated_3d_y_values(cur_app_state.time),
        );
    }
}
