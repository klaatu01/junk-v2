#import bevy_sprite::mesh2d_vertex_output::VertexOutput

struct MaterialUniforms {
    color: vec4<f32>,             // _Color
    outline_color: vec4<f32>,     // _OutlineColor
    outline_thickness: f32,       // _OutlineThickness
};

@group(2) @binding(0)
var<uniform> material: MaterialUniforms;

@group(2) @binding(1)
var main_texture: texture_2d<f32>;
@group(2) @binding(2)
var sampler_main: sampler;

@fragment
fn fragment(input: VertexOutput) -> @location(0) vec4<f32> {
    // Corrected access to UV coordinates
    var uv = input.uv;
    let current_sample = textureSample(main_texture, sampler_main, uv);
    let current_alpha = current_sample.a;

    if (current_alpha > 0.0) {
        // If the current pixel is opaque, render it with the main color
        return material.color * current_sample;
    } else {
        // Calculate the thickness in UV space
        let thickness = material.outline_thickness;

        // Define offsets for neighboring samples
        let offsets = array<vec2<f32>, 8>(
            vec2<f32>(0.0,  thickness), // Up
            vec2<f32>(0.0, -thickness), // Down
            vec2<f32>( thickness, 0.0), // Right
            vec2<f32>(-thickness, 0.0), // Left
            vec2<f32>( thickness,  thickness), // Up-Right
            vec2<f32>(-thickness,  thickness), // Up-Left
            vec2<f32>( thickness, -thickness), // Down-Right
            vec2<f32>(-thickness, -thickness)  // Down-Left
        );

        // Sample neighboring pixels
        var neighbor_alpha: f32 = 0.0;
        for (var i = 0u; i < 8u; i = i + 1u) {
            neighbor_alpha = neighbor_alpha + textureSample(main_texture, sampler_main, uv + offsets[i]).a;
        }

        if (neighbor_alpha > 0.0) {
            // If any neighbor is opaque, render the outline color
            return material.outline_color;
        }

        // Otherwise, render fully transparent
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }
}
