#version 450

void main(void) {
    float x = float((gl_VertexIndex & 1) << 2);
    float y = float((gl_VertexIndex & 2) << 1);
    gl_Position = vec4(x - 1.0, y - 1.0, 0, 1);
}
