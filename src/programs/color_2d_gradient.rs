use super::super::common_funcs as cf;
use js_sys::WebAssembly;
use wasm_bindgen::JsCast;
use web_sys::WebGlRenderingContext as GL;
use web_sys::*;

pub struct Color2DGradient {
    program: WebGlProgram,
    rect_vertices_buffer: WebGlBuffer,
    rect_vertices_indices_count: i32,
    u_color: WebGlUniformLocation,
    u_opacity: WebGlUniformLocation,
    u_transform: WebGlUniformLocation,
}

impl Color2DGradient {
    pub fn new(gl: &WebGlRenderingContext) -> Self {
        let program = cf::link_program(
            &gl,
            super::super::shaders::vertex::color_2d_gradient::SHADER,
            super::super::shaders::fragment::color_2d_gradient::SHADER,
        )
        .unwrap();

        // as a test program showing a rectangle, we will define the coordinates of the
        // two triangles forming the rectangle.
        // unlike before where we specified overlapping points, we omit repeats
        // instead, we alloc a new vec that informs the order
        // while it seems inefficient for this few shapes and overlaps
        // we are promised that when the graphics become complex in 3d, we
        // get a return on investment
        // nb: triangles are always specified ccw
        let vertices_rect: [f32; 8] = [0., 1., 0., 0., 1., 1., 1., 0.];
        let vertices_indices_rect: [u16; 6] = [0, 1, 2, 2, 1, 3];

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
        let buffer_rect = gl
            .create_buffer()
            .ok_or("failed to create buffer for vertices")
            .unwrap();
        gl.bind_buffer(GL::ARRAY_BUFFER, Some(&buffer_rect));
        gl.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &vert_array, GL::STATIC_DRAW);

        // ditto for the vertices indices. gotta expose'em to shaders
        let vertices_indices_memory_buffer = wasm_bindgen::memory()
            .dyn_into::<WebAssembly::Memory>()
            .unwrap()
            .buffer();
        let vertices_indices_location = vertices_indices_rect.as_ptr() as u32 / 2;
        let indices_array = js_sys::Uint16Array::new(&vertices_indices_memory_buffer).subarray(
            vertices_indices_location,
            vertices_indices_location + vertices_indices_rect.len() as u32,
        );
        let buffer_indices = gl.create_buffer().unwrap();
        gl.bind_buffer(GL::ELEMENT_ARRAY_BUFFER, Some(&buffer_indices));
        gl.buffer_data_with_array_buffer_view(
            GL::ELEMENT_ARRAY_BUFFER,
            &indices_array,
            GL::STATIC_DRAW,
        );

        Self {
            rect_vertices_buffer: buffer_rect,
            rect_vertices_indices_count: indices_array.length() as i32,
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
        gl.uniform4f(Some(&self.u_color), 0.5, 0., 0., 1.);

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
        // gl.draw_arrays(GL::TRIANGLES, offset, count); // can't use this with indices
        gl.draw_elements_with_i32(
            GL::TRIANGLES,
            self.rect_vertices_indices_count,
            GL::UNSIGNED_SHORT,
            offset,
        );
    }
}
