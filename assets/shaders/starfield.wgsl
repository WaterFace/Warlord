#import noisy_bevy::prelude

struct StarfieldMaterial {
    // fields for the noise that forms the stars
    scale: f32,
    ramp_cutoff: f32,
    octaves: i32,
    lacunarity: f32,
    gain: f32,

    // fields for the brightness noise
    brightness_scale: f32,
    brightness_octaves: i32,
    brightness_lacunarity: f32,
    brightness_gain: f32,

    // scalar for the final brightness
    brightness: f32,

    // parallax parameters
    parallax_factor: f32,
    camera_position: vec2<f32>,
    resolution: vec2<f32>,
};

@group(1) @binding(0)
var<uniform> material: StarfieldMaterial;

@fragment
fn fragment(
    #import bevy_pbr::mesh_vertex_output
    ) -> @location(0) vec4<f32> {

    // Parameters for hot-reloading. These should be encoded
    // in the component's values once they're tuned
    // let scale = 5.0;
    // let cutoff = 0.0;
    // let octaves = 3;
    // let lacunarity = 2.1;
    // let gain = 0.5;

    // let m_scale = 30.0;
    // let m_octaves = 1;
    // let m_lacunarity = 2.5;
    // let m_gain = 1.0;

    // let brightness = 0.3;

    let seed = 
        material.scale 
        * material.ramp_cutoff 
        * f32(material.octaves)
        * material.lacunarity 
        * material.gain 
        * material.brightness_scale 
        * f32(material.brightness_octaves)
        * material.brightness_lacunarity 
        * material.brightness_gain 
        * material.brightness;

    let white = vec4(1.0, 1.0, 1.0, 1.0);
    let black = vec4(0.0, 0.0, 0.0, 0.0);
    let noise = fbm_simplex_2d_seeded(
        uv * material.scale + material.camera_position * vec2(1.0, -1.0) * material.parallax_factor, 
        material.octaves, 
        material.lacunarity, 
        material.gain,
        seed);

    let t = max((noise - material.ramp_cutoff) / (1.0 - material.ramp_cutoff), 0.0);

    let noise2 = fbm_simplex_2d_seeded(
        uv * material.brightness_scale + material.camera_position * vec2(1.0, -1.0) * material.parallax_factor, 
        material.brightness_octaves, 
        material.brightness_lacunarity, 
        material.brightness_gain,
        seed);

    let c2 = noise2 * white + (1.0 - noise2) * black;
    let c = t * white + (1.0-t) * black;
    // return c2;
    // return c * c2 * brightness;
    return vec4(c.xyz * material.brightness, c.w);
}
