use bevy::{
    prelude::{Color, Handle, Vec2},
    text::{Font, TextSection, TextStyle},
};
use rand::{distributions::uniform::SampleUniform, Rng};

pub fn random_direction() -> Vec2 {
    let mut rng = rand::thread_rng();
    let mut dir = Vec2::ZERO;
    while dir.length_squared() == 0.0 {
        dir = Vec2::new(rng.gen_range(-1.0..=1.0), rng.gen_range(-1.0..=1.0));
    }
    dir.normalize()
}

pub fn random_range<T: SampleUniform + PartialOrd>(min: T, max: T) -> T {
    let mut rng = rand::thread_rng();
    rng.gen_range(min..=max)
}

pub fn random_in_circle(radius: f32) -> Vec2 {
    loop {
        let x = random_range(-1.0, 1.0);
        let y = random_range(-1.0, 1.0);

        if x * x + y * y <= 1.0 {
            return Vec2::new(x, y) * radius;
        }
    }
}

pub fn markup_to_text_sections(
    input: &str,
    font: Handle<Font>,
    font_size: f32,
    highlight_color: Color,
    normal_color: Color,
) -> Vec<TextSection> {
    let mut result: Vec<_> = Vec::new();
    let split = input.split('*');
    let normal_style = TextStyle {
        color: normal_color,
        font: font.clone(),
        font_size,
    };
    let highlight_style = TextStyle {
        color: highlight_color,
        font: font.clone(),
        font_size,
    };

    let mut highlight = false;
    for s in split {
        if highlight {
            result.push(TextSection {
                value: s.to_owned(),
                style: highlight_style.clone(),
            });
        } else {
            result.push(TextSection {
                value: s.to_owned(),
                style: normal_style.clone(),
            });
        }
        highlight = !highlight;
    }

    return result;
}
