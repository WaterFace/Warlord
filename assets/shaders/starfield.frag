#version 450
layout(location = 0) in vec2 v_Uv;

layout(location = 0) out vec4 o_Target;

struct StarfieldMaterial {
    vec3 camera_position;
    float parallax_factor;
    vec3 resolution;
    float time;
};

layout(set = 1, binding = 0) uniform StarfieldMaterial material;

// layout(set = 1, binding = 0) uniform vec3 camera_position;
// layout(set = 1, binding = 1) uniform vec3 resolution;
// layout(set = 1, binding = 2) uniform float parallax_factor;
// layout(set = 1, binding = 3) uniform float time;


vec2 random2(vec2 c) {
    float j = 4096.0*sin(dot(c,vec2(17.0, 59.4)));
    vec2 r;
    r.y = fract(512.0*j);
    j *= .125;
    r.x = fract(512.0*j);
    return r-0.5;
}

vec3 color_temperature(float k) {
  k = k / 100.0;
  vec3 c = vec3(0.0);
  if (k <= 66.0) {
    c.r = 255.0;
    c.g = 99.4708025861 * log(k) - 161.1195681661;
  } else {
    c.r = k - 60.0;
    c.r = 329.698727446 * pow(c.r,-0.1332047592);

    c.g = k - 60.0;
    c.g = 288.1221695283 * pow(c.g,-0.0755148492);
  }

  if (k >= 66.0) {
    c.b = 255.0;
  } else if (k <= 19.0) {
    c.b = 0.0;
  } else {
    c.b = k - 10.0;
    c.b = 138.5177312231 * log(c.b) - 305.0447927307;
  }

  c = clamp(c, vec3(0.0), vec3(255.0)) / 255.0;
  return c;
}

float stars(vec2 uv, float scale, float cutoff) {
  uv = uv * scale;
  vec2 id = floor(uv);
  vec2 frac = fract(uv);

  float min_dist = 1.0;
  for (int y = -1; y <= 1; y++) {
    for (int x = -1; x <= 1; x++) {
      vec2 neighbor = vec2(float(x), float(y));
      vec2 point = random2(id + neighbor);

      vec2 diff = neighbor + point - frac;
      float dist = length(diff);

      min_dist = min(min_dist, dist);
    }
  }

  return smoothstep(cutoff, 1.0, 1.0 - min_dist);
}

void main() {
  vec2 uv = v_Uv - vec2(0.5) + material.parallax_factor * material.camera_position.xy * vec2(1.0, -1.0) / material.resolution.xy;
  uv.x *= material.resolution.x / material.resolution.y;

  float t = material.time * material.camera_position.z * material.resolution.z;

  vec3 color = vec3(0.0) + t;

  color += stars(uv, 50.0/1.3, 0.95) * color_temperature(10000.0) * 3.0;
  color += stars(uv, 25.0/1.3, 0.98) * color_temperature(60000.0) * 25.0;
  color += stars(uv, 75.0/1.3, 0.9) * color_temperature(6000.0) * 0.3;
  color += stars(uv, 5, 0.99) * color_temperature(800000.0) * 150.0;
  color += stars(uv, 4.5, 0.995) * vec3(0.9, 0.01, 0.5) * 150.0;
  color += stars(uv, 4.5, 0.995) * vec3(0.9, 0.01, 0.5) * 150.0;
  color += stars(uv, 4.6, 0.995) * vec3(0.7, 0.5, 0.3) * 150.0;

  o_Target = vec4(color, 1.0);
}
