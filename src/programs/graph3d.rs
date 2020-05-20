use super::super::common_funcs as cf;
use super::super::constants::*;
use js_sys::WebAssembly;
use wasm_bindgen::JsCast;
use web_sys::WebGlRenderingContext as GL;
use web_sys::*;

// from shader, we know we will need these
// attribute vec4 aPosition;
// uniform mat4 uProjection;
// varying lowp vec4 vColor;
pub struct Graph3d {
    pub program: WebGlProgram,
    pub position_buffer: WebGlBuffer,
    pub indices_buffer: WebGlBuffer,
    pub index_count: i32,
    pub y_buffer: WebGlBuffer,
    pub normals_buffer: WebGlBuffer,
    pub u_normals_rotation: WebGlUniformLocation,
    pub u_opacity: WebGlUniformLocation,
    pub u_projection: WebGlUniformLocation,
}

impl Graph3d {
    pub fn new(gl: &WebGlRenderingContext) -> Self {
        let program = cf::link_program(
            &gl,
            super::super::shaders::vertex::graph_3d::SHADER,
            super::super::shaders::fragment::varying_color_from_vertex::SHADER,
        )
        .unwrap();

        // define the grid
        let positions_and_indices = cf::get_position_grid_n_by_n(GRID_SIZE);
        // memory binding for grid location
        let memory_buffer = wasm_bindgen::memory()
            .dyn_into::<WebAssembly::Memory>()
            .unwrap()
            .buffer();
        let vertices_location = positions_and_indices.0.as_ptr() as u32 / 4;
        let vert_array = js_sys::Float32Array::new(&memory_buffer).subarray(
            vertices_location,
            vertices_location + positions_and_indices.0.len() as u32,
        );
        let buffer_position = gl
            .create_buffer()
            .ok_or("failed to create buffer for grid")
            .unwrap();
        gl.bind_buffer(GL::ARRAY_BUFFER, Some(&buffer_position));
        gl.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &vert_array, GL::STATIC_DRAW);

        // memory binding for grid indices
        let indices_memory_buffer = wasm_bindgen::memory()
            .dyn_into::<WebAssembly::Memory>()
            .unwrap()
            .buffer();
        let indices_location = positions_and_indices.1.as_ptr() as u32 / 2;
        let indices_array = js_sys::Uint16Array::new(&indices_memory_buffer).subarray(
            indices_location,
            indices_location + positions_and_indices.1.len() as u32,
        );
        let buffer_indices = gl
            .create_buffer()
            .ok_or("failed to create buffer for grid indices")
            .unwrap();
        gl.bind_buffer(GL::ARRAY_BUFFER, Some(&buffer_indices));
        gl.buffer_data_with_array_buffer_view(
            GL::ELEMENT_ARRAY_BUFFER,
            &indices_array,
            GL::STATIC_DRAW,
        );

        Self {
            // color_buffer: gl
            //     .create_buffer()
            //     .ok_or("failed to create color buffer")
            //     .unwrap(),
            // rect_vertices_buffer: buffer_rect,
            // rect_vertices_indices_count: indices_array.length() as i32,
            u_opacity: gl.get_uniform_location(&program, "uOpacity").unwrap(),
            u_projection: gl.get_uniform_location(&program, "uProjection").unwrap(),
            u_normals_rotation: gl
                .get_uniform_location(&program, "uNormalsRotation")
                .unwrap(),
            program, // must be last as it takes over ownership of program

            position_buffer: buffer_position,
            indices_buffer: buffer_indices,
            index_count: indices_array.length() as i32,
            y_buffer: gl
                .create_buffer()
                .ok_or("failed to create y buffer")
                .unwrap(),

            normals_buffer: gl
                .create_buffer()
                .ok_or("failed to create normals buffer")
                .unwrap(),
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
        rotation_angle_x_axis: f32,
        rotation_angle_y_axis: f32,
        y_vals: &Vec<f32>,
    ) {
        gl.use_program(Some(&self.program));

        let projection_and_rotation_matrices = cf::get_3d_projection_matrix_and_rotation(
            bottom,
            top,
            left,
            right,
            canvas_height,
            canvas_width,
            rotation_angle_x_axis,
            rotation_angle_y_axis,
        );

        gl.uniform_matrix4fv_with_f32_array(
            Some(&self.u_projection),
            false,
            &projection_and_rotation_matrices.projection,
        );

        // normals
        gl.uniform_matrix4fv_with_f32_array(
            Some(&self.u_normals_rotation),
            false,
            &projection_and_rotation_matrices.normals_rotation,
        );

        // opacity
        gl.uniform1f(Some(&self.u_opacity), 1.);
        // position
        gl.bind_buffer(GL::ARRAY_BUFFER, Some(&self.position_buffer));
        gl.vertex_attrib_pointer_with_i32(0, 3, GL::FLOAT, false, 0, 0);
        gl.enable_vertex_attrib_array(0);

        // y buffer
        gl.bind_buffer(GL::ARRAY_BUFFER, Some(&self.y_buffer));
        gl.vertex_attrib_pointer_with_i32(1, 1, GL::FLOAT, false, 0, 0);
        gl.enable_vertex_attrib_array(1);

        // y dynamic draw
        let y_memory_buffer = wasm_bindgen::memory()
            .dyn_into::<WebAssembly::Memory>()
            .unwrap()
            .buffer();

        let y_location = y_vals.as_ptr() as u32 / 4;
        let y_array = js_sys::Float32Array::new(&y_memory_buffer)
            .subarray(y_location, y_location + y_vals.len() as u32);
        gl.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &y_array, GL::DYNAMIC_DRAW);

        // normals
        gl.bind_buffer(GL::ARRAY_BUFFER, Some(&self.normals_buffer));
        gl.vertex_attrib_pointer_with_i32(2, 3, GL::FLOAT, false, 0, 0);
        gl.enable_vertex_attrib_array(2);

        // normals dynamic draw
        let normals_vals = cf::get_grid_normals(super::super::constants::GRID_SIZE, &y_vals);
        let normals_memory_buffer = wasm_bindgen::memory()
            .dyn_into::<WebAssembly::Memory>()
            .unwrap()
            .buffer();

        let normals_location = normals_vals.as_ptr() as u32 / 4;
        let normals_array = js_sys::Float32Array::new(&normals_memory_buffer).subarray(
            normals_location,
            normals_location + normals_vals.len() as u32,
        );
        gl.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &normals_array, GL::DYNAMIC_DRAW);

        gl.draw_elements_with_i32(GL::TRIANGLES, self.index_count, GL::UNSIGNED_SHORT, 0);
    }
}
