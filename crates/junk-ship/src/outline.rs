// src/sprite_outline_material.rs

use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderRef};
use bevy::sprite::Material2d;

/// Custom material for rendering sprite outlines
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct SpriteOutlineMaterial {
    /// Tint color (_Color)
    #[uniform(0)]
    pub color: Vec4,

    /// Outline color (_OutlineColor)
    #[uniform(0)]
    pub outline_color: Vec4,

    /// Outline thickness in UV space (_OutlineThickness)
    #[uniform(0)]
    pub outline_thickness: f32,

    /// Main texture (_MainTex)
    #[texture(1)]
    #[sampler(2)]
    pub main_texture: Handle<Image>,
}

impl Material2d for SpriteOutlineMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/sprite_outline.wgsl".into()
    }

    fn alpha_mode(&self) -> bevy::sprite::AlphaMode2d {
        bevy::sprite::AlphaMode2d::Blend
    }
}
