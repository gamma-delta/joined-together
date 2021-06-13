// Finally, I'm free from that perlin noise.

#version 100

precision highp float;

varying vec2 uv;
varying vec4 color;

uniform sampler2D Texture;
// The XYZ component of this are the color.
// The W component is the progress.
// If it is more than 0, everything *darker* than it will be drawn
// If it is less than 0, everything *lighter* than it will be drawn.
uniform vec4 progress;
uniform int isPipe; // macroquad can't pass bools :(

void main() {
    vec4 outCol = texture2D(Texture, uv) * color;

    if(outCol.g == 0.0 && outCol.b == 0.0 && outCol.a != 0.0) {
        // special color time!
        bool drawSpecial = (progress.w > 0.0) ? (progress.w >= outCol.r) : (1.0 + progress.w <= outCol.r);
        if(drawSpecial) {
            outCol.xyz = progress.xyz;
        } else if(isPipe != 0) {
            // Display clear for pipes
            outCol.xyzw = vec4(0.0);
        } else {
            // Display the dark color for wires
            outCol.xyzw = vec4(20.0 / 255.0, 24.0 / 255.0, 46.0 / 255.0, 0.8);
        }
    }

    gl_FragColor = outCol;
}