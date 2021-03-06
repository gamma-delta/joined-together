#version 100

precision highp float;
varying vec2 uv;

uniform float time;

/////////////// K.jpg's Re-oriented 8-Point BCC Noise (OpenSimplex2S) ////////////////
////////////////////// Output: vec4(dF/dx, dF/dy, dF/dz, value) //////////////////////

// Borrowed from Stefan Gustavson's noise code
vec4 permute(vec4 t) {
    return t * (t * 34.0 + 133.0);
}

// Gradient set is a normalized expanded rhombic dodecahedron
vec3 grad(float hash) {

    // Random vertex of a cube, +/- 1 each
    vec3 cube = mod(floor(hash / vec3(1.0, 2.0, 4.0)), 2.0) * 2.0 - 1.0;

    // Random edge of the three edges connected to that vertex
    // Also a cuboctahedral vertex
    // And corresponds to the face of its dual, the rhombic dodecahedron
    vec3 cuboct = cube;
    // == Edit to make this a constant index for webgl
    int hashFloor = int(hash / 16.0);
    if(hashFloor == 0) {
        cuboct.x = 0.0;
    } else if(hashFloor == 1) {
        cuboct.y = 0.0;
    } else if(hashFloor == 2) {
        cuboct.z = 0.0;
    }

    // In a funky way, pick one of the four points on the rhombic face
    float type = mod(floor(hash / 8.0), 2.0);
    vec3 rhomb = (1.0 - type) * cube + type * (cuboct + cross(cube, cuboct));

    // Expand it so that the new edges are the same length
    // as the existing ones
    vec3 grad = cuboct * 1.22474487139 + rhomb;

    // To make all gradients the same length, we only need to shorten the
    // second type of vector. We also put in the whole noise scale constant.
    // The compiler should reduce it into the existing floats. I think.
    grad *= (1.0 - 0.042942436724648037 * type) * 3.5946317686139184;

    return grad;
}

// BCC lattice split up into 2 cube lattices
vec4 openSimplex2SDerivativesPart(vec3 X) {
    vec3 b = floor(X);
    vec4 i4 = vec4(X - b, 2.5);

    // Pick between each pair of oppposite corners in the cube.
    vec3 v1 = b + floor(dot(i4, vec4(.25)));
    vec3 v2 = b + vec3(1, 0, 0) + vec3(-1, 1, 1) * floor(dot(i4, vec4(-.25, .25, .25, .35)));
    vec3 v3 = b + vec3(0, 1, 0) + vec3(1, -1, 1) * floor(dot(i4, vec4(.25, -.25, .25, .35)));
    vec3 v4 = b + vec3(0, 0, 1) + vec3(1, 1, -1) * floor(dot(i4, vec4(.25, .25, -.25, .35)));

    // Gradient hashes for the four vertices in this half-lattice.
    vec4 hashes = permute(mod(vec4(v1.x, v2.x, v3.x, v4.x), 289.0));
    hashes = permute(mod(hashes + vec4(v1.y, v2.y, v3.y, v4.y), 289.0));
    hashes = mod(permute(mod(hashes + vec4(v1.z, v2.z, v3.z, v4.z), 289.0)), 48.0);

    // Gradient extrapolations & kernel function
    vec3 d1 = X - v1;
    vec3 d2 = X - v2;
    vec3 d3 = X - v3;
    vec3 d4 = X - v4;
    vec4 a = max(0.75 - vec4(dot(d1, d1), dot(d2, d2), dot(d3, d3), dot(d4, d4)), 0.0);
    vec4 aa = a * a;
    vec4 aaaa = aa * aa;
    vec3 g1 = grad(hashes.x);
    vec3 g2 = grad(hashes.y);
    vec3 g3 = grad(hashes.z);
    vec3 g4 = grad(hashes.w);
    vec4 extrapolations = vec4(dot(d1, g1), dot(d2, g2), dot(d3, g3), dot(d4, g4));

    // Derivatives of the noise
    // == My note: This version of opengl doesn't actually have 4x3 matrices
    // so i extend them to 4x4 and then shorten it.
    const float matrixPad = 1.0;
    mat4 matrix1 = mat4(vec4(d1, matrixPad), vec4(d2, matrixPad), vec4(d3, matrixPad), vec4(d4, matrixPad));
    mat4 matrix2 = mat4(vec4(g1, matrixPad), vec4(g2, matrixPad), vec4(g3, matrixPad), vec4(g4, matrixPad));
    vec4 derivative = -8.0 * matrix1 * (aa * a * extrapolations) + matrix2 * aaaa;

    // Return it all as a vec4
    return vec4(derivative.xyz, dot(aaaa, extrapolations));
}

// Rotates domain, but preserve shape. Hides grid better in cardinal slices.
// Good for texturing 3D objects with lots of flat parts along cardinal planes.
vec4 openSimplex2SDerivatives_Classical(vec3 X) {
    X = dot(X, vec3(2.0 / 3.0)) - X;

    vec4 result = openSimplex2SDerivativesPart(X) + openSimplex2SDerivativesPart(X + 144.5);

    return vec4(dot(result.xyz, vec3(2.0 / 3.0)) - result.xyz, result.w);
}

// Gives X and Y a triangular alignment, and lets Z move up the main diagonal.
// Might be good for terrain, or a time varying X/Y plane. Z repeats.
vec4 openSimplex2SDerivatives_ImproveXYPlanes(vec3 X) {

    // Not a skew transform.
    mat3 orthonormalMap = mat3(0.788675134594813, -0.211324865405187, -0.577350269189626, -0.211324865405187, 0.788675134594813, -0.577350269189626, 0.577350269189626, 0.577350269189626, 0.577350269189626);

    X = orthonormalMap * X;
    vec4 result = openSimplex2SDerivativesPart(X) + openSimplex2SDerivativesPart(X + 144.5);

    return vec4(result.xyz * orthonormalMap, result.w);
}

//////////////////////////////// End noise code ////////////////////////////////

void main() {
    vec4 background = openSimplex2SDerivatives_Classical(vec3(uv.x * 4.0, uv.y * 3.0, 100.0 + time * 0.05));
    vec4 stars = openSimplex2SDerivatives_Classical(vec3(uv.x * 20.0 + 100.0, uv.y * 18.0 - 100.0, 5.0));
    vec4 starfields = openSimplex2SDerivatives_Classical(vec3(uv.x * 5.0 - 5000.0, uv.y * 5.0 - 2000.0, -20.0 - sin(time * 0.08)));
    vec4 noise = openSimplex2SDerivatives_Classical(vec3(uv.x * 100.0 + time * 0.1 * sin(time * 0.01), uv.y * 100.0 + time * 0.1 * cos(time * 0.009), 0.0));

    vec4 starSeed = openSimplex2SDerivatives_ImproveXYPlanes(vec3(0.0, uv.x, uv.y));
    bool hasStar = mod(dot(starSeed, starSeed), 0.5) < 0.01;

    // Output to screen
    vec4 backgroundCol = vec4(vec3(105.0 / 255.0, 36.0 / 255.0, 100.0 / 255.0) * background.w * 0.5, 1.0);
    vec4 starCol = vec4(vec3(stars.w * 0.5), 0.5) * starfields.w;
    gl_FragColor = backgroundCol + starCol + noise.w * 0.1 + background * 0.01 + vec4(0.9, 0.7, 1.0, 0.5) * float(int(hasStar)) * (starfields.w + stars.w);
}