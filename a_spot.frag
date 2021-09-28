#ifdef GL_ES
precision mediump float;
#endif
uniform vec2 u_resolution;
uniform vec2 u_mouse;
uniform float u_time;

vec3 pixel(vec2 pos) {
    vec2 st = ((pos * 2. - 1.)/min(u_resolution.x, u_resolution.y));
    vec2 q = vec2(0.);
    vec3 g = vec3(0.);
    const int steps = 249;
    for (int i = 0; i < steps; i++) { 
        float m = float(i);
    	q += cos(st * vec2(2. - m, 1.720 * m) + q.yx);
        q = q + dot(q, vec2(u_time / 5.,u_time / 10.));
        float s = float(i)/float(steps);
        if (smoothstep(0., 0.5, abs(q.x) + abs(q.y)) < 0.1) {
            g += mix(
            vec3(0.948,0.383,1.000) * 1., 
            vec3(0.259,0.608,1.000) * 3., 
            s) * 3.;
        }
    }
    return vec3(g);
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
    gl_FragColor = vec4(color, 1.);
}
