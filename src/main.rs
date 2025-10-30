use nalgebra_glm::{Vec3, Mat4};
use minifb::{Key, Window, WindowOptions};
use std::time::Duration;
use std::f32::consts::PI;

mod framebuffer;
mod triangle;
mod line;
mod vertex;
mod obj;
mod color;
mod fragment;
mod shaders;

use framebuffer::Framebuffer;
use vertex::Vertex;
use obj::Obj;
use triangle::triangle;
use shaders::vertex_shader;
use color::Color;

pub struct Uniforms {
    model_matrix: Mat4,
    view_matrix: Mat4,
    projection_matrix: Mat4,
    viewport_matrix: Mat4,
}

fn create_model_matrix(translation: Vec3, scale: f32, rotation: Vec3) -> Mat4 {
    let (sin_x, cos_x) = rotation.x.sin_cos();
    let (sin_y, cos_y) = rotation.y.sin_cos();
    let (sin_z, cos_z) = rotation.z.sin_cos();

    let rotation_matrix_x = Mat4::new(
        1.0,  0.0,    0.0,   0.0,
        0.0,  cos_x, -sin_x, 0.0,
        0.0,  sin_x,  cos_x, 0.0,
        0.0,  0.0,    0.0,   1.0,
    );

    let rotation_matrix_y = Mat4::new(
        cos_y,  0.0,  sin_y, 0.0,
        0.0,    1.0,  0.0,   0.0,
        -sin_y, 0.0,  cos_y, 0.0,
        0.0,    0.0,  0.0,   1.0,
    );

    let rotation_matrix_z = Mat4::new(
        cos_z, -sin_z, 0.0, 0.0,
        sin_z,  cos_z, 0.0, 0.0,
        0.0,    0.0,  1.0, 0.0,
        0.0,    0.0,  0.0, 1.0,
    );

    let rotation_matrix = rotation_matrix_z * rotation_matrix_y * rotation_matrix_x;

    let translation_matrix = Mat4::new(
        1.0, 0.0, 0.0, translation.x,
        0.0, 1.0, 0.0, translation.y,
        0.0, 0.0, 1.0, translation.z,
        0.0, 0.0, 0.0, 1.0,
    );

    let scale_matrix = Mat4::new(
        scale, 0.0,   0.0,   0.0,
        0.0,   scale, 0.0,   0.0,
        0.0,   0.0,   scale, 0.0,
        0.0,   0.0,   0.0,   1.0,
    );

    translation_matrix * rotation_matrix * scale_matrix
}

fn create_view_matrix(eye: Vec3, center: Vec3, up: Vec3) -> Mat4 {
    nalgebra_glm::look_at(&eye, &center, &up)
}

fn create_perspective_matrix(fov: f32, aspect: f32, near: f32, far: f32) -> Mat4 {
    nalgebra_glm::perspective(fov, aspect, near, far)
}

fn create_viewport_matrix(width: f32, height: f32) -> Mat4 {
    Mat4::new(
        width / 2.0, 0.0, 0.0, width / 2.0,
        0.0, -height / 2.0, 0.0, height / 2.0,
        0.0, 0.0, 1.0, 0.0,
        0.0, 0.0, 0.0, 1.0
    )
}

fn render(framebuffer: &mut Framebuffer, uniforms: &Uniforms, vertex_array: &[Vertex], ship_color: Color) {
    let start_time = std::time::Instant::now();
    
    // Vertex Shader Stage
    let transformed_vertices: Vec<Vertex> = vertex_array
        .iter()
        .map(|vertex| vertex_shader(vertex, uniforms))
        .collect();

    let mut triangles_rendered = 0;
    let max_triangles = 5000; // Límite de seguridad

    // Procesar triángulos
    for i in (0..transformed_vertices.len()).step_by(3) {
        if i + 2 >= transformed_vertices.len() {
            break;
        }

        if triangles_rendered >= max_triangles {
            println!("⚠ Límite de triángulos alcanzado ({})", max_triangles);
            break;
        }

        let v0 = &transformed_vertices[i];
        let v1 = &transformed_vertices[i + 1];
        let v2 = &transformed_vertices[i + 2];

        // Backface culling
        let edge1 = v1.transformed_position - v0.transformed_position;
        let edge2 = v2.transformed_position - v0.transformed_position;
        let normal = Vec3::new(
            edge1.y * edge2.z - edge1.z * edge2.y,
            edge1.z * edge2.x - edge1.x * edge2.z,
            edge1.x * edge2.y - edge1.y * edge2.x,
        );
        
        if normal.z <= 0.0 {
            continue;
        }

        let fragments = triangle(v0, v1, v2);
        
        for fragment in fragments {
            let x = fragment.position.x as usize;
            let y = fragment.position.y as usize;
            
            if x < framebuffer.width && y < framebuffer.height {
                let lighting = (fragment.color.to_hex() & 0xFF) as f32 / 255.0;
                let final_color = ship_color * lighting.max(0.3);
                
                framebuffer.set_current_color(final_color.to_hex());
                framebuffer.point(x, y, fragment.depth);
            }
        }

        triangles_rendered += 1;
    }

    let elapsed = start_time.elapsed();
    if elapsed.as_millis() > 50 {
        println!("⚠ Frame lento: {}ms, {} triángulos", elapsed.as_millis(), triangles_rendered);
    }
}

fn main() {
    let window_width = 800;
    let window_height = 600;
    let framebuffer_width = 800;
    let framebuffer_height = 600;
    let frame_delay = Duration::from_millis(16);

    println!("Iniciando renderer...");

    let mut framebuffer = Framebuffer::new(framebuffer_width, framebuffer_height);
    
    let mut window = Window::new(
        "Rust Graphics - Nave Espacial",
        window_width,
        window_height,
        WindowOptions::default(),
    )
    .unwrap();

    window.set_position(500, 500);
    println!("✓ Ventana creada");

    framebuffer.set_background_color(0x1a1a2e);

    let mut translation = Vec3::new(0.0, 0.0, 0.0);
    let mut rotation = Vec3::new(0.0, 0.0, 0.0);
    let mut scale = 15.0f32;  // Escala reducida

    let ship_color = Color::new(255, 255, 0);

    println!("Cargando Nave.obj...");
    let obj = Obj::load("assets/Nave.obj").expect("Failed to load Nave.obj");
    let vertex_arrays = obj.get_vertex_array();
    println!("✓ Nave cargada: {} vértices, {} triángulos", 
        vertex_arrays.len(), 
        vertex_arrays.len() / 3
    );

    if vertex_arrays.len() > 30000 {
        println!("⚠ ADVERTENCIA: El modelo tiene muchos polígonos. El rendimiento será bajo.");
        println!("  Considera usar un modelo simplificado.");
    }

    // Cámara más alejada
    let mut eye = Vec3::new(0.0, 2.0, 8.0);
    let center = Vec3::new(0.0, 0.0, 0.0);
    let up = Vec3::new(0.0, 1.0, 0.0);
    let mut camera_distance = 8.0f32;

    let view_matrix = create_view_matrix(eye, center, up);
    let projection_matrix = create_perspective_matrix(
        60.0 * PI / 180.0,  // FOV más amplio
        framebuffer_width as f32 / framebuffer_height as f32,
        0.1,
        100.0
    );
    let viewport_matrix = create_viewport_matrix(
        framebuffer_width as f32,
        framebuffer_height as f32
    );

    let mut time = 0.0f32;
    let mut frame_count = 0;
    let mut last_fps_time = std::time::Instant::now();
    let mut auto_rotate = true;

    println!("\n=== CONTROLES ===");
    println!("  Flechas: Mover");
    println!("  A/S: Escala");
    println!("  Z/X: Zoom cámara (acercar/alejar)");
    println!("  Q/W/E/R/T/Y: Rotaciones");
    println!("  ESPACIO: Auto-rotación");
    println!("  ESC: Salir\n");

    println!("Renderizando...\n");

    while window.is_open() {
        let frame_start = std::time::Instant::now();

        if window.is_key_down(Key::Escape) {
            break;
        }

        if window.is_key_pressed(Key::Space, minifb::KeyRepeat::No) {
            auto_rotate = !auto_rotate;
        }

        handle_input(&window, &mut translation, &mut rotation, &mut scale, &mut camera_distance);

        // Actualizar posición de cámara
        eye = Vec3::new(0.0, 2.0, camera_distance);
        let view_matrix = create_view_matrix(eye, center, up);

        if auto_rotate {
            time += 0.015;
            rotation.y = time;
        }

        framebuffer.clear();

        let model_matrix = create_model_matrix(translation, scale, rotation);
        let uniforms = Uniforms {
            model_matrix,
            view_matrix,
            projection_matrix,
            viewport_matrix,
        };

        render(&mut framebuffer, &uniforms, &vertex_arrays, ship_color);

        window
            .update_with_buffer(&framebuffer.buffer, framebuffer_width, framebuffer_height)
            .unwrap();

        frame_count += 1;
        if last_fps_time.elapsed().as_secs() >= 1 {
            println!("FPS: {} | Escala: {:.0} | Zoom: {:.1} | Rot: ({:.1}, {:.1}, {:.1})", 
                frame_count, scale, camera_distance, rotation.x, rotation.y, rotation.z);
            frame_count = 0;
            last_fps_time = std::time::Instant::now();
        }

        let frame_time = frame_start.elapsed();
        if frame_time < frame_delay {
            std::thread::sleep(frame_delay - frame_time);
        }
    }

    println!("\n END");
}

fn handle_input(window: &Window, translation: &mut Vec3, rotation: &mut Vec3, scale: &mut f32, camera_distance: &mut f32) {
    let move_speed = 0.3;
    let rotation_speed = PI / 40.0;
    let scale_speed = 1.0;
    let zoom_speed = 0.3;

    if window.is_key_down(Key::Right) {
        translation.x += move_speed;
    }
    if window.is_key_down(Key::Left) {
        translation.x -= move_speed;
    }
    if window.is_key_down(Key::Up) {
        translation.y += move_speed;
    }
    if window.is_key_down(Key::Down) {
        translation.y -= move_speed;
    }
    
    // Zoom de cámara
    if window.is_key_down(Key::Z) {
        *camera_distance = (*camera_distance - zoom_speed).max(2.0);
    }
    if window.is_key_down(Key::X) {
        *camera_distance = (*camera_distance + zoom_speed).min(20.0);
    }
    
    // Escala del modelo
    if window.is_key_down(Key::S) {
        *scale += scale_speed;
    }
    if window.is_key_down(Key::A) {
        *scale = (*scale - scale_speed).max(1.0);
    }
    
    // Rotaciones
    if window.is_key_down(Key::Q) {
        rotation.x -= rotation_speed;
    }
    if window.is_key_down(Key::W) {
        rotation.x += rotation_speed;
    }
    if window.is_key_down(Key::E) {
        rotation.y -= rotation_speed;
    }
    if window.is_key_down(Key::R) {
        rotation.y += rotation_speed;
    }
    if window.is_key_down(Key::T) {
        rotation.z -= rotation_speed;
    }
    if window.is_key_down(Key::Y) {
        rotation.z += rotation_speed;
    }
}