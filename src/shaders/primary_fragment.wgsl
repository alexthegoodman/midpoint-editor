struct FragmentInput {
    @location(0) normal: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) color: vec3<f32>
};

struct LightUniforms {
    direction: vec3<f32>,
    color: vec3<f32>,
    ambient_intensity: f32,
    time_of_day: f32,
};

@group(2) @binding(0) var t_diffuse: texture_2d_array<f32>;
@group(2) @binding(1) var s_diffuse: sampler;
@group(2) @binding(2) var<uniform> renderMode: i32;
@group(3) @binding(0) var<uniform> light: LightUniforms;

// @fragment
// fn main(in: FragmentInput) -> @location(0) vec4<f32> {
//     let tiling_factor: f32 = 10.0;
//     let tiled_tex_coords = fract(in.tex_coords * tiling_factor);

//     let primary = textureSample(t_diffuse, s_diffuse, tiled_tex_coords, 0);
//     let primary_mask = textureSample(t_diffuse, s_diffuse, in.tex_coords, 1).r;
//     let rockmap = textureSample(t_diffuse, s_diffuse, tiled_tex_coords, 2);
//     let rockmap_mask = textureSample(t_diffuse, s_diffuse, in.tex_coords, 3).r;
//     let soil = textureSample(t_diffuse, s_diffuse, tiled_tex_coords, 4);
//     let soil_mask = textureSample(t_diffuse, s_diffuse, in.tex_coords, 5).r;
    
//     // Normalize masks
//     let total_mask = primary_mask + rockmap_mask + soil_mask;
//     let primary_weight = primary_mask / max(total_mask, 0.001);
//     let rockmap_weight = rockmap_mask / max(total_mask, 0.001);
//     let soil_weight = soil_mask / max(total_mask, 0.001);

//     // Blend textures based on normalized weights
//     let final_color = primary.rgb * primary_weight + 
//                       rockmap.rgb * rockmap_weight + 
//                       soil.rgb * soil_weight;
    
//     if (renderMode == 1) { // Assume 1 means rendering texture
//         return vec4<f32>(final_color, 1.0); // Texture rendering
//     } else {
//         return vec4(in.color, 1.0); // Color mode
//     }
// }

@fragment
fn main(in: FragmentInput) -> @location(0) vec4<f32> {
    let tiling_factor: f32 = 10.0;
    let tiled_tex_coords = fract(in.tex_coords * tiling_factor);

    let primary = textureSample(t_diffuse, s_diffuse, tiled_tex_coords, 0);
    let primary_mask = textureSample(t_diffuse, s_diffuse, in.tex_coords, 1).r;
    let rockmap = textureSample(t_diffuse, s_diffuse, tiled_tex_coords, 2);
    let rockmap_mask = textureSample(t_diffuse, s_diffuse, in.tex_coords, 3).r;
    let soil = textureSample(t_diffuse, s_diffuse, tiled_tex_coords, 4);
    let soil_mask = textureSample(t_diffuse, s_diffuse, in.tex_coords, 5).r;
    
    // Normalize masks
    let total_mask = primary_mask + rockmap_mask + soil_mask;
    let primary_weight = primary_mask / max(total_mask, 0.001);
    let rockmap_weight = rockmap_mask / max(total_mask, 0.001);
    let soil_weight = soil_mask / max(total_mask, 0.001);

    let base_color = primary.rgb * primary_weight + 
                    rockmap.rgb * rockmap_weight + 
                    soil.rgb * soil_weight;

    // Normalize the normal (it might have been interpolated)
    let N = normalize(in.normal);
    let L = normalize(-light.direction); // Light direction (pointing from surface to light)
    
    // Calculate diffuse lighting
    let diffuse_strength = max(dot(N, L), 0.0);
    
    // Sunrise color temperature (warm oranges)
    let sunrise_color = vec3<f32>(1.0, 0.7, 0.4);
    
    // Blend between sunrise color and regular sunlight based on time_of_day
    let light_color = mix(
        sunrise_color,
        light.color,
        smoothstep(0.15, 0.3, light.time_of_day)  // Smooth transition during sunrise
    );
    
    // Combine ambient and diffuse lighting
    let ambient = light.ambient_intensity * light_color;
    let diffuse = diffuse_strength * light_color;
    
    let final_color = base_color * (ambient + diffuse);

    if (renderMode == 1) {
        return vec4<f32>(final_color, 1.0);
    } else if (renderMode == 2) {
        let reg_primary = textureSample(t_diffuse, s_diffuse, in.tex_coords, 0);

        return vec4<f32>(reg_primary.rgb, 1.0);
    } else {
        return vec4<f32>(in.color * (ambient + diffuse), 1.0);
    }
}