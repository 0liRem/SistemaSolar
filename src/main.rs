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
use shaders::*;
use color::Color;

pub struct Uniforms {
    model_matrix: Mat4,
    view_matrix: Mat4,
    projection_matrix: Mat4,
    viewport_matrix: Mat4,
    time: f32,
}

#[derive(Clone, Copy)]
enum CelestialBodyType {
    Sun,
    RockyPlanet1,
    RockyPlanet2,
    GasGiantJupiter,
    GasGiantNeptune,
    DwarfPlanet,
    Moon,
    Ring,
    Ship,
}

struct CelestialBody {
    body_type: CelestialBodyType,
    position: Vec3,
    rotation: Vec3,
    scale: f32,
    orbit_radius: f32,
    orbit_speed: f32,
    orbit_angle: f32,
    rotation_speed: f32,
    has_moons: bool,
    has_rings: bool,
}

impl CelestialBody {
    fn new(
        body_type: CelestialBodyType,
        orbit_radius: f32,
        orbit_speed: f32,
        scale: f32,
        rotation_speed: f32,
    ) -> Self {
        CelestialBody {
            body_type,
            position: Vec3::new(0.0, 0.0, 0.0),
            rotation: Vec3::new(0.0, 0.0, 0.0),
            scale,
            orbit_radius,
            orbit_speed,
            orbit_angle: 0.0,
            rotation_speed,
            has_moons: false,
            has_rings: false,
        }
    }

    fn update(&mut self, time: f32) {
        self.orbit_angle += self.orbit_speed * 0.01;
        
        let eccentricity = 0.1;
        let semi_major_axis = self.orbit_radius;
        let semi_minor_axis = semi_major_axis * (1.0 - eccentricity);
        
        self.position.x = semi_major_axis * self.orbit_angle.cos();
        self.position.z = semi_minor_axis * self.orbit_angle.sin();
        self.position.y = 0.0;
        
        self.rotation.y += self.rotation_speed * 0.01;
    }
}

struct Camera {
    eye: Vec3,
    center: Vec3,
    up: Vec3,
    distance: f32,
    angle: f32,
    height: f32,
    orbit_speed: f32,
}

impl Camera {
    fn new() -> Self {
        Camera {
            eye: Vec3::new(0.0, 50.0, 100.0),
            center: Vec3::new(0.0, 0.0, 0.0),
            up: Vec3::new(0.0, 1.0, 0.0),
            distance: 100.0,
            angle: 0.0,
            height: 50.0,
            orbit_speed: 0.0,
        }
    }

    fn update_orbital(&mut self, time: f32) {
        self.angle += self.orbit_speed * 0.01;
        self.eye.x = self.distance * self.angle.cos();
        self.eye.z = self.distance * self.angle.sin();
        self.eye.y = self.height;
    }

    fn follow_ship(&mut self, ship_pos: Vec3) {
        let offset = Vec3::new(0.0, 20.0, 40.0);
        self.eye = ship_pos + offset;
        self.center = ship_pos;
    }

    fn set_position(&mut self, eye: Vec3, center: Vec3) {
        self.eye = eye;
        self.center = center;
    }
}

struct ViewPoint {
    name: &'static str,
    ship_position: Vec3,
    camera_eye: Vec3,
    camera_center: Vec3,
}

impl ViewPoint {
    const VIEWPOINTS: [ViewPoint; 4] = [
        ViewPoint {
            name: "Overview",
            ship_position: Vec3::new(0.0, 150.0, 300.0),
            camera_eye: Vec3::new(0.0, 180.0, 320.0),
            camera_center: Vec3::new(0.0, 0.0, 0.0),
        },
        ViewPoint {
            name: "Inner System",
            ship_position: Vec3::new(80.0, 40.0, 80.0),
            camera_eye: Vec3::new(80.0, 60.0, 100.0),
            camera_center: Vec3::new(0.0, 0.0, 0.0),
        },
        ViewPoint {
            name: "Gas Giants",
            ship_position: Vec3::new(180.0, 50.0, 0.0),
            camera_eye: Vec3::new(200.0, 80.0, 50.0),
            camera_center: Vec3::new(150.0, 0.0, 0.0),
        },
        ViewPoint {
            name: "Outer Rim",
            ship_position: Vec3::new(250.0, 30.0, 250.0),
            camera_eye: Vec3::new(270.0, 70.0, 270.0),
            camera_center: Vec3::new(200.0, 0.0, 200.0),
        },
    ];
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

fn render_body(
    framebuffer: &mut Framebuffer,
    uniforms: &Uniforms,
    vertex_array: &[Vertex],
    body_type: CelestialBodyType,
) {
    let transformed_vertices: Vec<Vertex> = vertex_array
        .iter()
        .map(|vertex| vertex_shader(vertex, uniforms))
        .collect();

    let mut triangles_processed = 0;
    let max_triangles = match body_type {
        CelestialBodyType::Sun => 1000,
        CelestialBodyType::GasGiantJupiter | CelestialBodyType::GasGiantNeptune => 800,
        CelestialBodyType::RockyPlanet1 | CelestialBodyType::RockyPlanet2 => 600,
        CelestialBodyType::Ship => 500,
        CelestialBodyType::Moon => 300,
        CelestialBodyType::DwarfPlanet => 400,
        CelestialBodyType::Ring => 400,
    };

    for i in (0..transformed_vertices.len()).step_by(3) {
        if i + 2 >= transformed_vertices.len() || triangles_processed >= max_triangles {
            break;
        }

        let v0 = &transformed_vertices[i];
        let v1 = &transformed_vertices[i + 1];
        let v2 = &transformed_vertices[i + 2];

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

        let in_frustum = [v0, v1, v2].iter().any(|v| {
            v.transformed_position.x >= 0.0 && v.transformed_position.x < framebuffer.width as f32 &&
            v.transformed_position.y >= 0.0 && v.transformed_position.y < framebuffer.height as f32
        });

        if !in_frustum {
            continue;
        }

        let fragments = triangle(v0, v1, v2);
        
        let use_simple_shader = matches!(body_type, 
            CelestialBodyType::Moon | 
            CelestialBodyType::DwarfPlanet |
            CelestialBodyType::Ship
        );

        for fragment in fragments {
            let x = fragment.position.x as usize;
            let y = fragment.position.y as usize;
            
            if x < framebuffer.width && y < framebuffer.height {
                let final_color = if use_simple_shader {
                    match body_type {
                        CelestialBodyType::Moon => {
                            Color::new(140, 140, 140)
                        },
                        CelestialBodyType::DwarfPlanet => {
                            Color::new(200, 200, 210)
                        },
                        CelestialBodyType::Ship => {
                            Color::new(255, 255, 0)
                        },
                        _ => Color::new(255, 255, 255),
                    }
                } else {
                    match body_type {
                        CelestialBodyType::Sun => {
                            sun_shader(&fragment, uniforms, uniforms.time)
                        },
                        CelestialBodyType::RockyPlanet1 => {
                            rocky_planet_shader(&fragment, uniforms, v0.transformed_normal)
                        },
                        CelestialBodyType::RockyPlanet2 => {
                            volcanic_planet_shader(&fragment, uniforms, v0.transformed_normal)
                        },
                        CelestialBodyType::GasGiantJupiter => {
                            gas_giant_jupiter_shader(&fragment, uniforms, v0.transformed_normal, uniforms.time)
                        },
                        CelestialBodyType::GasGiantNeptune => {
                            gas_giant_neptune_shader(&fragment, uniforms, v0.transformed_normal, uniforms.time)
                        },
                        CelestialBodyType::Ring => {
                            let distance = (fragment.position.x.powi(2) + fragment.position.y.powi(2)).sqrt();
                            ring_shader(&fragment, uniforms, v0.transformed_normal, distance)
                        },
                        _ => Color::new(255, 255, 255),
                    }
                };
                
                framebuffer.set_current_color(final_color.to_hex());
                framebuffer.point(x, y, fragment.depth);
            }
        }

        triangles_processed += 1;
    }
}

fn main() {
    let window_width = 1200;
    let window_height = 800;
    let framebuffer_width = 800;
    let framebuffer_height = 600;
    let frame_delay = Duration::from_millis(16);

    println!("Initializing Solar System Simulation...");

    let mut framebuffer = Framebuffer::new(framebuffer_width, framebuffer_height);
    
    let mut window = Window::new(
        "Solar System - Rust Renderer",
        window_width,
        window_height,
        WindowOptions::default(),
    )
    .unwrap();

    window.set_position(100, 100);
    framebuffer.set_background_color(0x000008);

    println!("Loading 3D models...");
    let sphere = Obj::load("assets/esfera.obj").expect("Failed to load esfera.obj");
    let sphere_vertices = sphere.get_vertex_array();
    
    let ship_obj = Obj::load("assets/Nave.obj").expect("Failed to load Nave.obj");
    let ship_vertices = ship_obj.get_vertex_array();
    
    let moon_obj = Obj::load("assets/luna.obj").expect("Failed to load luna.obj");
    let moon_vertices = moon_obj.get_vertex_array();
    
    let ring_obj = Obj::load("assets/anillo.obj").expect("Failed to load anillo.obj");
    let ring_vertices = ring_obj.get_vertex_array();
    
    println!("Models loaded successfully");

    let mut sun = CelestialBody::new(CelestialBodyType::Sun, 0.0, 0.0, 20.0, 0.5);
    
    let mut earth = CelestialBody::new(CelestialBodyType::RockyPlanet1, 80.0, 1.0, 5.0, 2.0);
    earth.has_moons = true;
    
    let mut venus = CelestialBody::new(CelestialBodyType::RockyPlanet2, 60.0, 1.5, 4.5, 1.5);
    
    let mut jupiter = CelestialBody::new(CelestialBodyType::GasGiantJupiter, 150.0, 0.5, 12.0, 3.0);
    jupiter.has_rings = true;
    
    let mut neptune = CelestialBody::new(CelestialBodyType::GasGiantNeptune, 200.0, 0.3, 10.0, 2.5);
    
    let mut pluto = CelestialBody::new(CelestialBodyType::DwarfPlanet, 250.0, 0.2, 3.0, 1.0);

    let mut ship_pos = Vec3::new(100.0, 10.0, 0.0);
    let mut ship_rotation = Vec3::new(0.0, 0.0, 0.0);
    let ship_scale = 2.0;

    let mut camera = Camera::new();
    camera.orbit_speed = 0.5;
    let mut camera_mode = 0;
    let mut current_viewpoint = 0;

    let projection_matrix = create_perspective_matrix(
        60.0 * PI / 180.0,
        framebuffer_width as f32 / framebuffer_height as f32,
        0.1,
        1000.0
    );
    let viewport_matrix = create_viewport_matrix(
        framebuffer_width as f32,
        framebuffer_height as f32
    );

    let mut time = 0.0f32;
    let mut frame_count = 0;
    let mut last_fps_time = std::time::Instant::now();

    println!("\n=== CONTROLS ===");
    println!("WASD: Move spaceship");
    println!("Q/E: Rotate spaceship");
    println!("Arrow keys: Move free camera");
    println!("1/2/3: Change camera mode (Orbital/Follow/Free)");
    println!("V: Cycle through viewpoints (Fast Travel)");
    println!("+/-: Camera orbit speed");
    println!("ESC: Exit\n");

    println!("Solar System simulation started\n");

    while window.is_open() {
        if window.is_key_down(Key::Escape) {
            break;
        }

        handle_input(
            &window, 
            &mut ship_pos, 
            &mut ship_rotation, 
            &mut camera, 
            &mut camera_mode,
            &mut current_viewpoint
        );

        time += 0.0006;

        sun.update(time);
        earth.update(time);
        venus.update(time);
        jupiter.update(time);
        neptune.update(time);
        pluto.update(time);

        match camera_mode {
            0 => camera.update_orbital(time),
            1 => camera.follow_ship(ship_pos),
            _ => {},
        }

        let view_matrix = create_view_matrix(camera.eye, camera.center, camera.up);

        framebuffer.clear();

        let model_matrix = create_model_matrix(sun.position, sun.scale, sun.rotation);
        let uniforms = Uniforms {
            model_matrix,
            view_matrix,
            projection_matrix,
            viewport_matrix,
            time,
        };
        render_body(&mut framebuffer, &uniforms, &sphere_vertices, sun.body_type);

        for body in [&earth, &venus, &jupiter, &neptune, &pluto].iter() {
            let model_matrix = create_model_matrix(body.position, body.scale, body.rotation);
            let uniforms = Uniforms {
                model_matrix,
                view_matrix,
                projection_matrix,
                viewport_matrix,
                time,
            };
            render_body(&mut framebuffer, &uniforms, &sphere_vertices, body.body_type);

            if body.has_moons {
                let moon_offset = Vec3::new(
                    (time * 2.0).cos() * 15.0,
                    0.0,
                    (time * 2.0).sin() * 15.0
                );
                let moon_pos = body.position + moon_offset;
                let moon_model = create_model_matrix(moon_pos, 2.0, Vec3::new(0.0, time, 0.0));
                let moon_uniforms = Uniforms {
                    model_matrix: moon_model,
                    view_matrix,
                    projection_matrix,
                    viewport_matrix,
                    time,
                };
                render_body(&mut framebuffer, &moon_uniforms, &moon_vertices, CelestialBodyType::Moon);
            }

            if body.has_rings {
                let ring_rotation = Vec3::new(PI / 2.2, body.rotation.y, 0.0);
                let ring_model = create_model_matrix(body.position, body.scale * 0.9, ring_rotation);
                let ring_uniforms = Uniforms {
                    model_matrix: ring_model,
                    view_matrix,
                    projection_matrix,
                    viewport_matrix,
                    time,
                };
                render_body(&mut framebuffer, &ring_uniforms, &ring_vertices, CelestialBodyType::Ring);
            }
        }

        let ship_model = create_model_matrix(ship_pos, ship_scale, ship_rotation);
        let ship_uniforms = Uniforms {
            model_matrix: ship_model,
            view_matrix,
            projection_matrix,
            viewport_matrix,
            time,
        };
        render_body(&mut framebuffer, &ship_uniforms, &ship_vertices, CelestialBodyType::Ship);

        window
            .update_with_buffer(&framebuffer.buffer, framebuffer_width, framebuffer_height)
            .unwrap();

        frame_count += 1;
        if last_fps_time.elapsed().as_secs() >= 1 {
            println!("FPS: {} | Ship: ({:.1}, {:.1}, {:.1}) | Camera: {} | Viewpoint: {}", 
                frame_count, 
                ship_pos.x, 
                ship_pos.y, 
                ship_pos.z, 
                camera_mode,
                ViewPoint::VIEWPOINTS[current_viewpoint].name
            );
            frame_count = 0;
            last_fps_time = std::time::Instant::now();
        }

        std::thread::sleep(frame_delay.saturating_sub(last_fps_time.elapsed()));
    }

    println!("\nSolar System simulation closed");
}

fn handle_input(
    window: &Window,
    ship_pos: &mut Vec3,
    ship_rotation: &mut Vec3,
    camera: &mut Camera,
    camera_mode: &mut i32,
    current_viewpoint: &mut usize,
) {
    let speed = 1.0;
    let rot_speed = 0.05;

    if window.is_key_down(Key::W) {
        ship_pos.z -= speed * ship_rotation.y.cos();
        ship_pos.x -= speed * ship_rotation.y.sin();
    }
    if window.is_key_down(Key::S) {
        ship_pos.z += speed * ship_rotation.y.cos();
        ship_pos.x += speed * ship_rotation.y.sin();
    }
    if window.is_key_down(Key::A) {
        ship_pos.x -= speed * ship_rotation.y.cos();
        ship_pos.z += speed * ship_rotation.y.sin();
    }
    if window.is_key_down(Key::D) {
        ship_pos.x += speed * ship_rotation.y.cos();
        ship_pos.z -= speed * ship_rotation.y.sin();
    }
    if window.is_key_down(Key::Q) {
        ship_rotation.y -= rot_speed;
    }
    if window.is_key_down(Key::E) {
        ship_rotation.y += rot_speed;
    }

    if window.is_key_pressed(Key::Key1, minifb::KeyRepeat::No) {
        *camera_mode = 0;
        println!("Camera: Orbital mode");
    }
    if window.is_key_pressed(Key::Key2, minifb::KeyRepeat::No) {
        *camera_mode = 1;
        println!("Camera: Follow ship mode");
    }
    if window.is_key_pressed(Key::Key3, minifb::KeyRepeat::No) {
        *camera_mode = 2;
        println!("Camera: Free mode");
    }

    if window.is_key_pressed(Key::V, minifb::KeyRepeat::No) {
        *current_viewpoint = (*current_viewpoint + 1) % ViewPoint::VIEWPOINTS.len();
        let vp = &ViewPoint::VIEWPOINTS[*current_viewpoint];
        *ship_pos = vp.ship_position;
        camera.set_position(vp.camera_eye, vp.camera_center);
        *camera_mode = 2;
        println!("Fast Travel to: {}", vp.name);
    }

    if window.is_key_down(Key::Equal) {
        camera.orbit_speed += 0.003;
    }
    if window.is_key_down(Key::Minus) {
        camera.orbit_speed -= 0.003;
    }

    if *camera_mode == 2 {
        if window.is_key_down(Key::Up) {
            camera.eye.y += 1.0;
        }
        if window.is_key_down(Key::Down) {
            camera.eye.y -= 1.0;
        }
        if window.is_key_down(Key::Left) {
            camera.distance -= 2.0;
        }
        if window.is_key_down(Key::Right) {
            camera.distance += 2.0;
        }
    }
}