use super::constants::*;
use nalgebra::Perspective3;
use web_sys::WebGlRenderingContext as GL;
use web_sys::*;

pub fn link_program(
    gl: &WebGlRenderingContext,
    vertex_shader_source: &str,
    fragment_shader_source: &str,
) -> Result<WebGlProgram, String> {
    let program = gl
        .create_program()
        .ok_or_else(|| String::from("Error creating gl program"))?;
    let vertex_shader = compile_shader(&gl, GL::VERTEX_SHADER, vertex_shader_source).unwrap();
    let fragment_shader = compile_shader(&gl, GL::FRAGMENT_SHADER, fragment_shader_source).unwrap();

    gl.attach_shader(&program, &vertex_shader);
    gl.attach_shader(&program, &fragment_shader);
    gl.link_program(&program);

    if gl
        .get_program_parameter(&program, WebGlRenderingContext::LINK_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(program)
    } else {
        Err(gl
            .get_program_info_log(&program)
            .unwrap_or_else(|| String::from("Unknown error creating program object")))
    }
}

fn compile_shader(
    gl: &WebGlRenderingContext,
    shader_type: u32,
    source: &str,
) -> Result<WebGlShader, String> {
    let shader = gl
        .create_shader(shader_type)
        .ok_or_else(|| String::from("Error creating shader"))?;
    gl.shader_source(&shader, source);
    gl.compile_shader(&shader);

    if gl
        .get_shader_parameter(&shader, WebGlRenderingContext::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(shader)
    } else {
        Err(gl
            .get_shader_info_log(&shader)
            .unwrap_or_else(|| String::from("Unable to get shader info log")))
    }
}

// matrix! returns 1d array that acts as a 4x4 matrix. this can be used for 3d later
pub fn translation_matrix(tx: f32, ty: f32, tz: f32) -> [f32; 16] {
    let mut protag = [0.; 16];

    protag[0] = 1.;
    protag[5] = 1.;
    protag[10] = 1.;
    protag[15] = 1.;

    protag[12] = tx;
    protag[13] = ty;
    protag[14] = tz;

    protag
}

pub fn scaling_matrix(sx: f32, sy: f32, sz: f32) -> [f32; 16] {
    let mut protag = [0.; 16];

    protag[0] = sx;
    protag[5] = sy;
    protag[10] = sz;
    protag[15] = 1.;

    protag
}

pub fn cross_multiply_matrix(a: &[f32; 16], b: &[f32; 16]) -> [f32; 16] {
    let mut protag = [0.; 16];

    protag[0] = a[0] * b[0] + a[1] * b[4] + a[2] * b[8] + a[3] * b[12];
    protag[1] = a[0] * b[1] + a[1] * b[5] + a[2] * b[9] + a[3] * b[13];
    protag[2] = a[0] * b[2] + a[1] * b[6] + a[2] * b[10] + a[3] * b[14];
    protag[3] = a[0] * b[3] + a[1] * b[7] + a[2] * b[11] + a[3] * b[15];

    protag[4] = a[4] * b[0] + a[5] * b[4] + a[6] * b[8] + a[7] * b[12];
    protag[5] = a[4] * b[1] + a[5] * b[5] + a[6] * b[9] + a[7] * b[13];
    protag[6] = a[4] * b[2] + a[5] * b[6] + a[6] * b[10] + a[7] * b[14];
    protag[7] = a[4] * b[3] + a[5] * b[7] + a[6] * b[11] + a[7] * b[15];

    protag[8] = a[8] * b[0] + a[9] * b[4] + a[10] * b[8] + a[11] * b[12];
    protag[9] = a[8] * b[1] + a[9] * b[5] + a[10] * b[9] + a[11] * b[13];
    protag[10] = a[8] * b[2] + a[9] * b[6] + a[10] * b[10] + a[11] * b[14];
    protag[11] = a[8] * b[3] + a[9] * b[7] + a[10] * b[11] + a[11] * b[15];

    protag[12] = a[12] * b[0] + a[13] * b[4] + a[14] * b[8] + a[15] * b[12];
    protag[13] = a[12] * b[1] + a[13] * b[5] + a[14] * b[9] + a[15] * b[13];
    protag[14] = a[12] * b[2] + a[13] * b[6] + a[14] * b[10] + a[15] * b[14];
    protag[15] = a[12] * b[3] + a[13] * b[7] + a[14] * b[11] + a[15] * b[15];

    protag
}

// ==== this is for grid creation ==== //
// objective: to draw lots of rectangles on the x-z plane, using a bunch of triangles
// these triangles'/rectangles' vertices will be held in a vecf32.
// (reminder, for 3d, 3 elements in the vec define a single vertex.)
// as before, to avoid duplicating the same points for overlapping triangle vertices, we will also define a indices vec,
// (reminder,  elements per triangle)
// return a tuple containing the coordinates for the grid, and the indices which tell webgl how to step through the coords vec.
pub fn get_position_grid_n_by_n(n: usize) -> (Vec<f32>, Vec<u16>) {
    // a 2x2 grid is 4 rectangles, which have 9 vertices, which are 18 x,y coords.
    // a 3x3 grid is 9 rects, which have 16 vertices, which are 32 coords.
    let n_plus_one = n + 1;
    let mut positions: Vec<f32> = vec![0.; 3 * n_plus_one * n_plus_one];
    let mut indices: Vec<u16> = vec![0; 6 * n * n];

    let graph_layout_width: f32 = 2.; // derived from webgl's clip space, which goes from -1 to +1
    let square_size: f32 = graph_layout_width / n as f32;

    // construct the positions and indices!
    // imagine the grid. we are moving from one corner of a grid box to the next
    for z in 0..n_plus_one {
        for x in 0..n_plus_one {
            let start_pos_i = 3 * (z * n_plus_one + x); // basically offset. first loop 0, second loop 3, etc
            positions[start_pos_i + 0] = -1. + (x as f32) * square_size; // the left most grid box starts at the imaginary clip space coords of -1
            positions[start_pos_i + 1] = 0.; // the grid is drawn on the x-z plane, so y-coord is always 0
            positions[start_pos_i + 2] = -1. + (z as f32) * square_size; // the top most grid box starts at clip space coords of -1

            // **** at this point, we have 1 position coords defined **** //

            // defining all indices necessary to draw the TWO triangles to the bottom right of current corner of grid box
            // |\   \‾|
            // |_\   \|
            // remember, ccw
            if z < n && x < n {
                let start_index_i = 6 * (z * n + x); // basically an offset

                let vertex_index_top_left = (z * n_plus_one + x) as u16;
                let vertex_index_btm_left = vertex_index_top_left + n_plus_one as u16;
                let vertex_index_top_right = vertex_index_top_left + 1;
                let vertex_index_btm_right = vertex_index_btm_left + 1;

                indices[start_index_i + 0] = vertex_index_top_left;
                indices[start_index_i + 1] = vertex_index_btm_left;
                indices[start_index_i + 2] = vertex_index_btm_right;
                indices[start_index_i + 3] = vertex_index_top_left;
                indices[start_index_i + 4] = vertex_index_btm_right;
                indices[start_index_i + 5] = vertex_index_top_right;
            }
        }
    }

    (positions, indices)
}

// ==== function to get 3D perspective projection matrix ==== //
pub fn get_3d_projection_matrix(
    bottom: f32,
    top: f32,
    left: f32,
    right: f32,
    canvas_height: f32,
    canvas_width: f32,
    rotation_angle_x_axis: f32,
    rotation_angle_y_axis: f32,
) -> [f32; 16] {
    // ---- rotation matrix ---- //
    #[rustfmt::skip]
    let rotate_x_axis: [f32;16] = [
        1., 0., 0., 0., //
        0., rotation_angle_x_axis.cos(), -rotation_angle_x_axis.sin(), 0., //
        0., rotation_angle_x_axis.sin(), rotation_angle_x_axis.cos(), 0., //
        0., 0., 0., 1. //
    ];

    #[rustfmt::skip]
    let rotate_y_axis: [f32;16] = [
        rotation_angle_y_axis.cos(), 0., rotation_angle_y_axis.sin(), 0.,
        0., 1., 0., 0.,
        -rotation_angle_y_axis.sin(), 0., rotation_angle_y_axis.cos(), 0.,
        0., 0., 0., 1.,
    ];

    let rotation_matrix = cross_multiply_matrix(&rotate_x_axis, &rotate_y_axis);

    // ---- calculate aspect ratio ---- //
    let aspect: f32 = canvas_width / canvas_height;
    let scale_x = (right - left) / canvas_width;
    let scale_y = (top - bottom) / canvas_height;
    let scale = scale_y;

    let translation_matrix: [f32; 16] = translation_matrix(
        -1. + scale_x + 2. * left / canvas_width,
        -1. + scale_y + 2. * bottom / canvas_height,
        Z_PLANE,
    );
    let scale_matrix: [f32; 16] = scaling_matrix(scale, scale, 0.);
    let rotation_scale = cross_multiply_matrix(&rotation_matrix, &scale_matrix);
    let combined_transform = cross_multiply_matrix(&rotation_scale, &translation_matrix);
    let perspective_matrix_tmp: Perspective3<f32> =
        Perspective3::new(aspect, FIELD_OF_VIEW, Z_NEAR, Z_FAR);
    let mut perspective: [f32; 16] = [0.; 16];
    perspective.copy_from_slice(perspective_matrix_tmp.as_matrix().as_slice());

    cross_multiply_matrix(&combined_transform, &perspective)
}

// ==== y values! ==== //
pub fn get_updated_3d_y_values(cur_time: f32) -> Vec<f32> {
    let point_count_per_row = GRID_SIZE + 1;
    let mut y_vals: Vec<f32> = vec![0.; point_count_per_row * point_count_per_row];

    // to draw a sine wave
    let half_grid: f32 = point_count_per_row as f32 / 2.;
    let frequency_scale: f32 = 10. * std::f32::consts::PI;
    let y_scale = 0.15;

    for z in 0..point_count_per_row {
        for x in 0..point_count_per_row {
            let use_y_index = z * point_count_per_row + x;
            let scaled_x = frequency_scale * (x as f32 - half_grid) / half_grid;
            let scaled_z = frequency_scale * (z as f32 - half_grid) / half_grid;
            y_vals[use_y_index] =
                y_scale * ((scaled_x * scaled_x + scaled_z * scaled_z).sqrt()).sin();
            // pythagorean
        }
    }

    y_vals
}
