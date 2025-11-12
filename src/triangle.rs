use nalgebra_glm::{Vec3, dot};
use crate::fragment::Fragment;
use crate::vertex::Vertex;
use crate::color::Color;

pub fn triangle(v1: &Vertex, v2: &Vertex, v3: &Vertex) -> Vec<Fragment> {
  let mut fragments = Vec::new();
  let (a, b, c) = (v1.transformed_position, v2.transformed_position, v3.transformed_position);

  // Calcular bounding box
  let (min_x, min_y, max_x, max_y) = calculate_bounding_box(&a, &b, &c);

  // Verificar que el bounding box es válido
  if min_x > max_x || min_y > max_y {
    return fragments;
  }

  // Límite de seguridad más estricto
  let width = (max_x - min_x) as usize;
  let height = (max_y - min_y) as usize;
  
  // Reducir límite para mejorar rendimiento
  if width > 1000 || height > 1000 {
    return fragments;
  }

  let light_dir = Vec3::new(0.0, 0.0, -1.0);
  let triangle_area = edge_function(&a, &b, &c);

  // Si el área es casi cero, el triángulo es degenerado
  if triangle_area.abs() < 0.001 {
    return fragments;
  }

  let inv_area = 1.0 / triangle_area; // Pre-calcular inverso

  // Pre-calcular normales transformadas
  let n1 = v1.transformed_normal;
  let n2 = v2.transformed_normal;
  let n3 = v3.transformed_normal;

  // Optimización: iterar con step para reducir fragmentos
  let step = 1; // Cambiar a 2 si necesitas más rendimiento
  
  // Iterar sobre cada pixel en el bounding box
  for y in (min_y..=max_y).step_by(step) {
    for x in (min_x..=max_x).step_by(step) {
      let point = Vec3::new(x as f32 + 0.5, y as f32 + 0.5, 0.0);

      // Calcular coordenadas baricéntricas (optimizado)
      let w1 = edge_function(&b, &c, &point) * inv_area;
      let w2 = edge_function(&c, &a, &point) * inv_area;
      let w3 = edge_function(&a, &b, &point) * inv_area;

      // Verificar si el punto está dentro del triángulo
      // Optimización: verificar rango con una sola comparación
      if w1 >= -0.01 && w1 <= 1.01 && 
         w2 >= -0.01 && w2 <= 1.01 &&
         w3 >= -0.01 && w3 <= 1.01 {
        
        // Interpolar normal (optimizado)
        let normal = (n1 * w1 + n2 * w2 + n3 * w3).normalize();

        // Calcular intensidad de iluminación
        let intensity = dot(&normal, &light_dir).max(0.0);

        // Color base gris con iluminación - más simple
        let base_intensity = (intensity * 200.0 + 55.0) as u8;
        let lit_color = Color::new(base_intensity, base_intensity, base_intensity);

        // Interpolar profundidad
        let depth = a.z * w1 + b.z * w2 + c.z * w3;

        fragments.push(Fragment::new(x as f32, y as f32, lit_color, depth));
      }
    }
  }

  fragments
}

#[inline(always)]
fn calculate_bounding_box(v1: &Vec3, v2: &Vec3, v3: &Vec3) -> (i32, i32, i32, i32) {
    let min_x = v1.x.min(v2.x).min(v3.x).floor() as i32;
    let min_y = v1.y.min(v2.y).min(v3.y).floor() as i32;
    let max_x = v1.x.max(v2.x).max(v3.x).ceil() as i32;
    let max_y = v1.y.max(v2.y).max(v3.y).ceil() as i32;

    (min_x, min_y, max_x, max_y)
}

#[inline(always)]
fn edge_function(a: &Vec3, b: &Vec3, c: &Vec3) -> f32 {
    (c.x - a.x) * (b.y - a.y) - (c.y - a.y) * (b.x - a.x)
}