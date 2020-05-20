pub const GRID_SIZE: usize = 100;

pub const FIELD_OF_VIEW: f32 = 45. * std::f32::consts::PI / 180.;
pub const Z_FAR: f32 = 100.; // how far you can see before things are clipped
pub const Z_NEAR: f32 = 0.1; // clip things nearer than this to the camera
pub const Z_PLANE: f32 = -2.414213; // related to our 45 deg FOV. -1/tan(pi/8)
