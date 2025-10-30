use nalgebra_glm::{Vec3, Vec4, Mat3};
use crate::vertex::Vertex;
use crate::Uniforms;

pub fn vertex_shader(vertex: &Vertex, uniforms: &Uniforms) -> Vertex {
    // Transform position
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

    // Transform normal
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