use nalgebra_glm::{Vec3, Vec4, Mat3, dot};
use crate::vertex::Vertex;
use crate::Uniforms;
use crate::fragment::Fragment;
use crate::color::Color;

/// Vertex shader that transforms vertices from object space to screen space.
///
/// This function performs the complete vertex transformation pipeline:
/// 1. Transforms position through model-view-projection matrices
/// 2. Performs perspective division
/// 3. Applies viewport transformation
/// 4. Transforms normals for lighting calculations
///
/// # Arguments
/// * `vertex` - Input vertex in object space
/// * `uniforms` - Transformation matrices and rendering parameters
///
/// # Returns
/// Transformed vertex with screen-space position and world-space normal
pub fn vertex_shader(vertex: &Vertex, uniforms: &Uniforms) -> Vertex {
    // Transform position to homogeneous coordinates
    let position = Vec4::new(
        vertex.position.x,
        vertex.position.y,
        vertex.position.z,
        1.0
    );
    
    // Apply model, view, and projection transformations
    let mut transformed = uniforms.projection_matrix 
        * uniforms.view_matrix 
        * uniforms.model_matrix 
        * position;

    // Perform perspective division
    let w = transformed.w;
    transformed /= w;

    // Apply viewport transformation
    let viewport_transformed = uniforms.viewport_matrix * transformed;
    
    let transformed_position = Vec3::new(
        viewport_transformed.x,
        viewport_transformed.y,
        viewport_transformed.z
    );

    // Transform normal using inverse transpose of model matrix
    let model_mat3 = Mat3::new(
        uniforms.model_matrix[0], uniforms.model_matrix[1], uniforms.model_matrix[2],
        uniforms.model_matrix[4], uniforms.model_matrix[5], uniforms.model_matrix[6],
        uniforms.model_matrix[8], uniforms.model_matrix[9], uniforms.model_matrix[10]
    );
    let normal_matrix = model_mat3.transpose().try_inverse().unwrap_or(Mat3::identity());

    let transformed_normal = normal_matrix * vertex.normal;

    // Create a new Vertex with transformed attributes
    Vertex {
        position: vertex.position,
        normal: vertex.normal,
        tex_coords: vertex.tex_coords,
        color: vertex.color,
        transformed_position,
        transformed_normal,
    }
}

// ============================================
// FRAGMENT SHADERS FOR CELESTIAL BODIES
// ============================================

/// Fragment shader for the Sun - Animated stellar surface with emission.
///
/// Creates a bright, dynamic star appearance with:
/// - Procedural noise for surface variation
/// - Animated convection patterns
/// - Yellow to orange-red color gradient
/// - High brightness values
///
/// # Arguments
/// * `fragment` - Fragment to shade
/// * `uniforms` - Rendering uniforms (unused but kept for consistency)
/// * `time` - Current simulation time for animation
///
/// # Returns
/// RGB color with emission
pub fn sun_shader(fragment: &Fragment, uniforms: &Uniforms, time: f32) -> Color {
    let world_pos = fragment.position;
    
    // Generate simplified procedural noise
    let noise1 = (world_pos.x * 3.0 + time).sin().abs();
    let noise2 = (world_pos.y * 3.0 - time * 0.5).cos().abs();
    
    // Create solar color palette
    let base_r = 255;
    let base_g = (220.0 - noise1 * 80.0) as u8;
    let base_b = (60.0 + noise2 * 40.0) as u8;
    
    Color::new(base_r, base_g, base_b)
}

/// Fragment shader for rocky planet (Earth/Mars-like).
///
/// Simulates a terrestrial planet with:
/// - Continental and oceanic regions
/// - Mountain ranges
/// - Directional lighting
/// - Terrain-based color variation
///
/// # Arguments
/// * `fragment` - Fragment to shade
/// * `uniforms` - Rendering uniforms (unused but kept for consistency)
/// * `normal` - Surface normal for lighting calculations
///
/// # Returns
/// RGB color representing terrain type with lighting
pub fn rocky_planet_shader(fragment: &Fragment, uniforms: &Uniforms, normal: Vec3) -> Color {
    let world_pos = fragment.position;
    
    // Generate multi-scale noise for terrain features
    let noise1 = ((world_pos.x * 3.0).sin() * (world_pos.y * 3.0).cos()).abs();
    let noise2 = ((world_pos.x * 8.0).cos() * (world_pos.y * 8.0).sin()).abs();
    
    // Define terrain color palette
    let land_color = Color::new(120, 100, 60);     // Brown landmass
    let ocean_color = Color::new(30, 60, 120);     // Blue ocean
    let mountain_color = Color::new(90, 85, 80);   // Gray mountains
    
    // Determine terrain type based on noise values
    let base_color = if noise1 > 0.5 {
        if noise2 > 0.7 {
            mountain_color
        } else {
            land_color
        }
    } else {
        ocean_color
    };
    
    // Apply basic directional lighting
    let light_dir = Vec3::new(-1.0, -1.0, -1.0).normalize();
    let intensity = dot(&normal, &light_dir).max(0.0);
    
    base_color * (0.4 + intensity * 0.6)
}

/// Fragment shader for volcanic planet (Venus-like).
///
/// Simulates a volcanically active world with:
/// - Molten lava flows with self-illumination
/// - Hot and cooled rock regions
/// - High surface temperature appearance
/// - Strong color contrast
///
/// # Arguments
/// * `fragment` - Fragment to shade
/// * `uniforms` - Rendering uniforms (unused but kept for consistency)
/// * `normal` - Surface normal for lighting calculations
///
/// # Returns
/// RGB color representing volcanic activity with emission
pub fn volcanic_planet_shader(fragment: &Fragment, uniforms: &Uniforms, normal: Vec3) -> Color {
    let world_pos = fragment.position;
    
    // Generate noise patterns for volcanic activity
    let noise1 = ((world_pos.x * 6.0).sin() * (world_pos.y * 6.0).cos()).abs();
    let noise2 = ((world_pos.x * 12.0).cos() * (world_pos.y * 12.0).sin()).abs();
    
    // Define volcanic color palette
    let lava_color = Color::new(255, 80, 0);       // Bright orange lava
    let rock_color = Color::new(60, 50, 50);       // Dark cooled rock
    let hot_rock_color = Color::new(180, 60, 30);  // Hot volcanic rock
    
    let base_color = if noise1 > 0.6 && noise2 > 0.6 {
        lava_color  // Active lava flows
    } else if noise1 > 0.4 {
        hot_rock_color
    } else {
        rock_color
    };
    
    // Apply directional lighting
    let light_dir = Vec3::new(-1.0, -1.0, -1.0).normalize();
    let intensity = dot(&normal, &light_dir).max(0.0);
    
    // Add self-illumination for lava regions
    let emission = if noise1 > 0.6 && noise2 > 0.6 { 0.3 } else { 0.0 };
    
    base_color * (0.3 + intensity * 0.5 + emission)
}

/// Fragment shader for gas giant (Jupiter-like).
///
/// Simulates a massive gas planet with:
/// - Horizontal atmospheric bands
/// - Storm systems and turbulence
/// - Animated cloud movement
/// - Color variation by latitude
///
/// # Arguments
/// * `fragment` - Fragment to shade
/// * `uniforms` - Rendering uniforms (unused but kept for consistency)
/// * `normal` - Surface normal for lighting calculations
/// * `time` - Current simulation time for animation
///
/// # Returns
/// RGB color representing atmospheric bands
pub fn gas_giant_jupiter_shader(fragment: &Fragment, uniforms: &Uniforms, normal: Vec3, time: f32) -> Color {
    let world_pos = fragment.position;
    
    // Generate animated horizontal bands
    let band_pos = world_pos.y * 4.0 + time * 0.3;
    let band_noise = (band_pos.sin() * 2.0).abs();
    
    // Add turbulence for storm systems
    let turbulence = ((world_pos.x * 10.0 + time * 0.5).sin() * (world_pos.y * 8.0).cos()).abs();
    
    // Define Jupiter color palette
    let orange_band = Color::new(200, 140, 100);
    let brown_band = Color::new(140, 100, 70);
    let light_band = Color::new(220, 200, 180);
    let storm_color = Color::new(180, 120, 100);  // Great Red Spot style
    
    // Select color based on band position
    let base_color = if band_noise > 1.5 {
        light_band
    } else if band_noise > 1.0 {
        orange_band
    } else {
        brown_band
    };
    
    // Add storm features
    let final_color = if turbulence > 0.75 && band_noise < 1.2 {
        storm_color
    } else {
        base_color
    };
    
    // Apply soft directional lighting
    let light_dir = Vec3::new(-1.0, -1.0, -1.0).normalize();
    let intensity = dot(&normal, &light_dir).max(0.0);
    
    final_color * (0.5 + intensity * 0.5)
}

/// Fragment shader for gas giant (Neptune-like).
///
/// Simulates an ice giant with:
/// - Subtle atmospheric layers
/// - Smooth color transitions
/// - Blue-cyan color palette
/// - Gentle cloud patterns
///
/// # Arguments
/// * `fragment` - Fragment to shade
/// * `uniforms` - Rendering uniforms (unused but kept for consistency)
/// * `normal` - Surface normal for lighting calculations
/// * `time` - Current simulation time for animation
///
/// # Returns
/// RGB color representing atmospheric layers
pub fn gas_giant_neptune_shader(fragment: &Fragment, uniforms: &Uniforms, normal: Vec3, time: f32) -> Color {
    let world_pos = fragment.position;
    
    // Generate subtle animated bands
    let band_pos = world_pos.y * 3.0 - time * 0.2;
    let band_noise = (band_pos.sin()).abs();
    
    // Add cloud patterns
    let cloud_noise = ((world_pos.x * 7.0).sin() * (world_pos.y * 7.0 + time * 0.3).cos()).abs();
    
    // Define blue-cyan color palette
    let deep_blue = Color::new(30, 80, 160);
    let light_blue = Color::new(60, 120, 200);
    let cyan = Color::new(80, 160, 220);
    
    // Select color based on atmospheric features
    let base_color = if cloud_noise > 0.6 {
        cyan
    } else if band_noise > 0.5 {
        light_blue
    } else {
        deep_blue
    };
    
    // Apply directional lighting
    let light_dir = Vec3::new(-1.0, -1.0, -1.0).normalize();
    let intensity = dot(&normal, &light_dir).max(0.0);
    
    base_color * (0.4 + intensity * 0.6)
}

/// Fragment shader for dwarf planet (Pluto-like).
///
/// Simulates a small icy body with:
/// - Frozen surface with high reflectivity
/// - Impact craters of varying sizes
/// - Gray-white color palette
/// - Surface roughness variation
///
/// # Arguments
/// * `fragment` - Fragment to shade
/// * `uniforms` - Rendering uniforms (unused but kept for consistency)
/// * `normal` - Surface normal for lighting calculations
///
/// # Returns
/// RGB color representing icy terrain with craters
pub fn dwarf_planet_shader(fragment: &Fragment, uniforms: &Uniforms, normal: Vec3) -> Color {
    let world_pos = fragment.position;
    
    // Generate multi-scale noise for surface features
    let noise1 = ((world_pos.x * 8.0).sin() * (world_pos.y * 8.0).cos()).abs();
    let noise2 = ((world_pos.x * 15.0).cos() * (world_pos.y * 15.0).sin()).abs();
    
    // Define icy surface palette
    let ice_white = Color::new(240, 240, 250);
    let ice_gray = Color::new(160, 160, 170);
    let dark_crater = Color::new(80, 80, 90);
    
    let base_color = if noise2 > 0.8 {
        dark_crater  // Deep impact craters
    } else if noise1 > 0.5 {
        ice_white
    } else {
        ice_gray
    };
    
    // Apply strong directional lighting
    let light_dir = Vec3::new(-1.0, -1.0, -1.0).normalize();
    let intensity = dot(&normal, &light_dir).max(0.0);
    
    base_color * (0.3 + intensity * 0.7)
}

/// Fragment shader for moon surface.
///
/// Simulates a cratered satellite with:
/// - Heavy impact cratering
/// - Grayscale color scheme
/// - High contrast surface features
/// - Strong directional lighting response
///
/// # Arguments
/// * `fragment` - Fragment to shade
/// * `uniforms` - Rendering uniforms (unused but kept for consistency)
/// * `normal` - Surface normal for lighting calculations
///
/// # Returns
/// RGB grayscale color representing lunar terrain
pub fn moon_shader(fragment: &Fragment, uniforms: &Uniforms, normal: Vec3) -> Color {
    let world_pos = fragment.position;
    
    // Generate crater patterns
    let crater_noise = ((world_pos.x * 12.0).sin() * (world_pos.y * 12.0).cos()).abs();
    let surface_noise = ((world_pos.x * 6.0).cos() * (world_pos.y * 6.0).sin()).abs();
    
    // Define lunar color palette (grayscale)
    let light_gray = Color::new(180, 180, 180);
    let dark_gray = Color::new(120, 120, 120);
    let crater_dark = Color::new(80, 80, 80);
    
    let base_color = if crater_noise > 0.75 {
        crater_dark
    } else if surface_noise > 0.5 {
        light_gray
    } else {
        dark_gray
    };
    
    // Apply strong directional lighting
    let light_dir = Vec3::new(-1.0, -1.0, -1.0).normalize();
    let intensity = dot(&normal, &light_dir).max(0.0);
    
    base_color * (0.2 + intensity * 0.8)
}

/// Fragment shader for planetary rings (Saturn-style).
///
/// Simulates ring systems with:
/// - Concentric band structure
/// - Gap regions (Cassini Division style)
/// - Semi-transparent appearance
/// - Distance-based color falloff
///
/// # Arguments
/// * `fragment` - Fragment to shade
/// * `uniforms` - Rendering uniforms (unused but kept for consistency)
/// * `normal` - Surface normal for lighting calculations
/// * `distance_from_center` - Radial distance for band calculation
///
/// # Returns
/// RGB color representing ring material or transparent gap
pub fn ring_shader(fragment: &Fragment, uniforms: &Uniforms, normal: Vec3, distance_from_center: f32) -> Color {
    // Generate concentric ring bands
    let band = (distance_from_center * 20.0).sin().abs();
    let gap = (distance_from_center * 40.0).cos().abs();
    
    // Define ring color palette
    let ring_light = Color::new(200, 180, 150);
    let ring_dark = Color::new(140, 120, 100);
    
    // Create gaps in ring structure
    if gap < 0.3 {
        Color::new(0, 0, 0)  // Transparent/dark gap
    } else if band > 0.6 {
        ring_light * 0.8
    } else {
        ring_dark * 0.6
    }
}