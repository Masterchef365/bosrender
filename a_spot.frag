#ifdef GL_ES
precision mediump float;
#endif
uniform vec2 u_resolution;
uniform vec2 u_mouse;
uniform float u_time;

vec3 hsv2rgb(vec3 c) {
  vec4 K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
  vec3 p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
  return c.z * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), c.y);
}

vec3 pixel(vec2 pos) {
    vec2 st = ((pos * 2. - 1.)/min(u_resolution.x, u_resolution.y));
    vec2 q = vec2(0.);
    float g = 0.;
    const int steps = 80;
    for (int i = 0; i < steps; i++) { 
        float m = float(i);
    	q += cos(st * vec2(0.280 - m, 1.720 * m) + q.yx);
        q = q + dot(q, vec2(fract(u_time / 50.) * 1.5 - .5, -1.192));
        float s = float(i)/float(steps);
        if (smoothstep(0., 0.5, abs(q.x) + abs(q.y)) < 0.1) {
            g += 1.;
        }
    }
    return vec3(normalize(q).y, 1.,g);
}

void main() {
    const int AA_DIVS = 2;
    const int AA_WIDTH = AA_DIVS*2+1;
    vec3 color = vec3(0.);
 	for (int x = -AA_DIVS; x <= AA_DIVS; x++) {
        for (int y = -AA_DIVS; y <= AA_DIVS; y++) {
        	vec2 off = vec2(x, y) / float(AA_WIDTH);
            color += pixel(off + gl_FragCoord.xy);
        }
    }
    color /= float(AA_WIDTH*AA_WIDTH);
    color = hsv2rgb(color);
    gl_FragColor = vec4(color, 1.);
}

