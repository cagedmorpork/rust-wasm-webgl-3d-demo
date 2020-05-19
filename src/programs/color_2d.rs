use super::super::common_funcs as cf;
use js_sys::WebAssembly;
use wasm_bindgen::JsCast;
use web_sys::WebGlRenderingContext as GL;
use web_sys::*;

pub struct Color2D {
    program: WebGlProgram,
    rect_vertices_array_length: usize,
    rect_vertices_buffer: WebGlBuffer,
    u_color: WebGlUniformLocation,
    u_opacity: WebGlUniformLocation,
    u_transform: WebGlUniformLocation,
}

impl Color2D {
    pub fn new(gl: &WebGlRenderingContext) -> Self {
        let program = cf::link_program(
            &gl,
            super::super::shaders::vertex::color_2d::SHADER,
            super::super::shaders::fragment::color_2d::SHADER,
        )
        .unwrap();

        // as a test program showing a rectangle, we will define the coordinates of the
        // two triangles forming the rectangle. the same points are defined twice
        // deliberately at this time.
        // let vertices_rect: [f32; 12] = [0., 1., 0., 0., 1., 1., 1., 1., 0., 0., 1., 0.];
        let vertices_rect: [f32; 6] = [0., 1., 0., 0., 1., 1.];

        // we shall feed the vertices to the shader program. but how?
        // we have to allocate some memory and cast them...
        let memory_buffer = wasm_bindgen::memory()
            .dyn_into::<WebAssembly::Memory>()
            .unwrap()
            .buffer();
        let vertices_location = vertices_rect.as_ptr() as u32 / 4;
        let vert_array = js_sys::Float32Array::new(&memory_buffer).subarray(
            vertices_location,
            vertices_location + vertices_rect.len() as u32,
        );
        let buffer_rect = gl.create_buffer().ok_or("failed to create buffer").unwrap();
        gl.bind_buffer(GL::ARRAY_BUFFER, Some(&buffer_rect));
        gl.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &vert_array, GL::STATIC_DRAW);

        Self {
            rect_vertices_array_length: vertices_rect.len(),
            rect_vertices_buffer: buffer_rect,
            u_color: gl.get_uniform_location(&program, "uColor").unwrap(),
            u_opacity: gl.get_uniform_location(&program, "uOpacity").unwrap(),
            u_transform: gl.get_uniform_location(&program, "uTransform").unwrap(),
            program, // must be last as it takes over ownership of program
        }
    }

    pub fn render(
        &self,
        gl: &WebGlRenderingContext,
        bottom: f32,
        top: f32,
        left: f32,
        right: f32,
        canvas_height: f32,
        canvas_width: f32,
    ) {
        gl.use_program(Some(&self.program));
        gl.bind_buffer(GL::ARRAY_BUFFER, Some(&self.rect_vertices_buffer));
        gl.vertex_attrib_pointer_with_i32(0, 2, GL::FLOAT, false, 0, 0);
        // no attributes
        // size. elements per vertex attribute. we are doing 2d, so we have x,y
        // the vertices will contain float. (our shaders use vec4, which will auto fill zeros where not provided)
        // normalized?
        // stride
        // offset
        gl.enable_vertex_attrib_array(0);

        // color
        gl.uniform4f(Some(&self.u_color), 0., 0.5, 0.5, 0.2);

        // opacity
        gl.uniform1f(Some(&self.u_opacity), 1.);

        // translation
        let translation_matrix = cf::translation_matrix(
            2. * left / canvas_width - 1.,
            2. * bottom / canvas_height - 1.,
            0.,
        );

        // scaling
        let scale_matrix = cf::scaling_matrix(
            2. * (right - left) / canvas_width,
            2. * (top - bottom) / canvas_height,
            0.,
        );

        // combine scaling and translation
        let transform_matrix = cf::cross_multiply_matrix(&scale_matrix, &translation_matrix);
        gl.uniform_matrix4fv_with_f32_array(Some(&self.u_transform), false, &transform_matrix);

        // draw, given all the settings loaded above
        let offset = 0;
        let count = (self.rect_vertices_array_length / 2) as i32;
        gl.draw_arrays(GL::TRIANGLES, offset, count);
    }
}
