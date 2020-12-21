#version 450
#extension GL_ARB_separate_shader_objects : enable
#extension GL_EXT_multiview : require

// IO stuff
layout(location = 0) in vec3 fragPos;
layout(location = 0) out vec4 outColor;

layout(binding = 1) uniform Animation {
    float anim;
};

layout(binding = 0) uniform CameraUbo {
    mat4 camera[2];
};

struct SDF {
    float dist;
    vec3 color;
};

SDF cube(vec3 pos, vec3 origin, vec3 color, float side) {
    vec3 pt = pos - origin;
    return SDF(
        distance(pt, clamp(vec3(-side), pt, vec3(side))),
        color
    );
}

SDF sdf_min(SDF a, SDF b) {
    if (a.dist <= b.dist) {
        return a;
    } else {
        return b;
    }
}

float rand(vec2 co){
    return fract(sin(dot(co.xy ,vec2(12.9898,78.233))) * 43758.5453);
}

SDF scene(vec3 pos) {
    ivec2 cell = ivec2(pos.xz);
    pos.xz = fract(pos.xz);
    float m = rand(vec2(cell));

    const vec3 orange = vec3(0.995,0.467,0.002);
    const vec3 blue = 1. - orange;
    const vec3 white = vec3(1.);
    const vec3 black = vec3(0.);

    vec3 color = mix(mix(orange, white, m), mix(-white * 2., blue, m), m);

    return sdf_min(
        cube(pos + vec3(0., 2., 0.), vec3(0.), color, 0.9),
        cube(pos - vec3(0., 6. + 1. * cos(m), 0.), vec3(0.), color, 0.9)
    );
}

const float CLIP_NEAR = 0.1; // Near clipping sphere
const float CLIP_FAR = 1000.; // Far clipping sphere
const int MAX_STEPS = 50; // Maximum sphere steps
const float HIT_THRESHOLD = 0.01; // Minimum distance considered a hit
const vec3 BACKGROUND = vec3(0.); // Backgroudn color

void main() {
    mat4 cam_inv = inverse(camera[gl_ViewIndex]);
    vec3 origin = (cam_inv * vec4(vec3(0.), 1.)).xyz;
    vec3 ray_out = (cam_inv * vec4(fragPos.x, fragPos.y, -1., 1.)).xyz;
    vec3 unit_ray = normalize(ray_out - origin);

	vec3 color = BACKGROUND;
    vec3 pos = origin + unit_ray * CLIP_NEAR;
    for (int i = 0; i < MAX_STEPS; i++) {
        SDF hit = scene(pos);

        if (hit.dist < HIT_THRESHOLD) {
            color = hit.color;
            break;
        }

        if (hit.dist > CLIP_FAR) {
            color = BACKGROUND;
            break;
        }

        pos += unit_ray * hit.dist;
    }

    outColor = vec4(color, 1.0);
}
