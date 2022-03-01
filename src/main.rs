mod vec3;
use std::{ops::Sub, cmp::{self, min}};

use vec3::*;

struct Sphere(Point, f32);

trait Intersect {
    fn find_intersection(&self, r: &Ray) -> Option<f32>;
}

impl Intersect for Sphere {
    // https://www.scratchapixel.com/lessons/3d-basic-rendering/minimal-ray-tracer-rendering-simple-shapes/ray-sphere-intersection
    fn find_intersection(&self, r: &Ray) -> Option<f32> {
        let O = r.0;
        let D = r.1;
        let C = self.0;
        let R = self.1;

        let O_minus_C = O - C;

        let a = 1.0;
        let b = 2.0 * D.dot(&O_minus_C);
        let c = O_minus_C.dot(&O_minus_C) - R*R;

        let discriminant = b*b - 4.0*a*c;
        if discriminant < 0.0 {
            return None;
        } else if discriminant == 0.0 {
            let t = b / (-2.0 * a);
            Some(t)
        } else {
            let t0 = (discriminant.sqrt() - b) / (2.0 * a); 
            let t1 = (discriminant.sqrt() + b) / (-2.0 * a); 
            Some(if t0 < t1 {t0} else {t1})
        }
    }
}

fn main() {
    let camera_ray = Ray(
        Point(Vec3(-2.0, 0.0, 0.0)),
        Direction(Vec3(1.0, 0.0, 0.0).normalized()),
    );
    let camera_up = Direction(Vec3(0.0, 0.0, 1.0));
    let camera_right = camera_ray.1.cross(&camera_up);
    println!("camera ray:{:?} right:{:?} up:{:?}",
        &camera_ray, &camera_right, &camera_up);

    let imgx = 800;
    let imgy = 800;

    let camera_w_fov_degrees: f32 = 90.0;
    let camera_w_fov_radians: f32 = camera_w_fov_degrees.to_radians();
    let caemra_h_fov_radians = camera_w_fov_radians * (imgy as f32) / (imgx as f32);
    

    // Create a new ImgBuf with width: imgx and height: imgy
    let mut imgbuf = image::ImageBuffer::new(imgx, imgy);
    
    let center_x = imgx as f32 / 2.0;
    let center_y = imgy as f32 / 2.0;

    let s = Sphere(Point(Vec3(0.0,0.0,0.0)), 1.0);

    // Iterate over the coordinates and pixels of the image
    for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
        let radians_x = ((x as f32) - center_x) / (imgx as f32) * camera_w_fov_radians;
        let radians_y = (center_y - (y as f32))  / (imgy as f32) * caemra_h_fov_radians;
        let pixel_dir = 
            camera_ray.1.0 + 
            camera_right.0 * radians_x +
            camera_up.0 * radians_y;
        let pixel_dir = Direction(pixel_dir.normalized());
        let pixel_ray = Ray(camera_ray.0, pixel_dir);

        let color = if let Some(t) = s.find_intersection(&pixel_ray) {
            let c = (100.0*t).clamp(0.0,255.0) as u8;
            [c,c,c]
        } else {
            [0u8,0,0]
        };
        *pixel = image::Rgb(color);
    }

    imgbuf.save("out.png").unwrap();
}
