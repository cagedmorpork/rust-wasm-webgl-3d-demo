pub const SHADER: &str = r#"
attribute vec4 aPosition;
attribute float aY;
uniform mat4 uProjection;
varying lowp vec4 vColor;

void main() {
    gl_Position = uProjection * vec4(aPosition.x, aY, aPosition.z, 1.);

    if( aY > 0.0 ) {
        vColor = vec4(0.0, 5.0 * aY, 0.0, 1.0); // green value depends on position of y
    }
    else {
        vColor = vec4(-5.0 * aY, 0.0, 0.0, 1.0); // red value depends on position of -y
    }
    // vColor = vec4(0.5, 0.5, 0.8,1.0);
}

"#;
