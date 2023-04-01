use bevy::{
    prelude::{Color, Image},
    render::{
        render_resource::{Extent3d, TextureDimension, TextureFormat},
        texture::ImageSampler,
    },
};
use bytemuck::pod_align_to;
use rand::Rng;

pub struct BasicStarField {
    pub density: f32,
    pub star_color: Color,
}

impl Default for BasicStarField {
    fn default() -> Self {
        Self {
            density: 0.002,
            star_color: Color::WHITE,
        }
    }
}

impl BasicStarField {
    pub fn build(&self, extent: Extent3d) -> Image {
        let background_color = Color::rgba(0.0, 0.0, 0.0, 0.0).as_rgba_f32();
        let width = extent.width as usize;
        let height = extent.height as usize;
        let mut data = vec![background_color; width * height];

        // Now fill the image with stars
        let num_stars = (data.len() as f32 * self.density).clamp(0.0, data.len() as f32) as usize;
        // distribute them roughly evenly around a grid
        let rows = f32::sqrt((num_stars * height / width) as f32) as usize;
        let columns = f32::sqrt((num_stars * width / height) as f32) as usize;

        let r_step = height as usize / rows;
        let c_step = width as usize / columns;

        let mut rng = rand::thread_rng();

        for x in (0..height).step_by(c_step) {
            for y in (0..width).step_by(r_step) {
                // TODO: allow multiple colors
                let color = self.star_color.as_rgba_f32();
                let ix = rng.gen_range(y..y + r_step) * width + rng.gen_range(x..x + c_step);
                if ix >= data.len() {
                    continue;
                }
                data[ix] = color;
            }
        }
        let (head, body, tail) = pod_align_to(&data);
        assert!(head.is_empty());
        assert!(tail.is_empty());

        let mut image = Image::new_fill(
            extent,
            TextureDimension::D2,
            body,
            TextureFormat::Rgba32Float,
        );
        image.sampler_descriptor = ImageSampler::nearest();

        image
    }
}
