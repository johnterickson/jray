mod vec3;
use std::ops::{Add, Mul};

use vec3::*;

#[derive(Copy, Clone)]
struct Color(f32,f32,f32);

const RED: Color = Color(1.0,0.0,0.0);
const GREEN: Color = Color(0.0,1.0,0.0);
const BLUE: Color = Color(0.0,0.0,1.0);
const WHITE: Color  = Color(1.0,1.0,1.0);
const BLACK: Color  = Color(0.0,0.0,0.0);


impl Add<Color> for Color {
    type Output = Self;

    fn add(self, rhs: Color) -> Self::Output {
        Color(
            self.0 + rhs.0,
            self.1 + rhs.1,
            self.2 + rhs.2,
        )
    }
}

impl Mul<Color> for f32 {
    type Output = Color;

    fn mul(self, rhs: Color) -> Self::Output {
        Color(rhs.0 * self, rhs.1 * self, rhs.2 * self)
    }
}

#[derive(Copy, Clone)]
struct Material {
    pub diffuse_color: Color,
    pub specular_color: Color,
    pub shininess: f32,
}

struct Light {
    pub position: Point,
    pub color: Color,
    pub intensity: f32,
}

struct Intersection {
    pub distance: f32,
    pub point: Point,
    pub normal: Direction,
    pub material: Material,
}

trait Intersect {
    fn find_intersection(&self, r: &Ray) -> Option<Intersection>;
}

struct Sphere(Point, f32, Material);

impl Intersect for Sphere {
    #![allow(non_snake_case)]
    // https://www.scratchapixel.com/lessons/3d-basic-rendering/minimal-ray-tracer-rendering-simple-shapes/ray-sphere-intersection
    fn find_intersection(&self, r: &Ray) -> Option<Intersection> {
        let O = r.0;
        let D = r.1;
        let C = self.0;
        let R = self.1;

        let O_minus_C = O - C;

        let a = 1.0;
        let b = 2.0 * D.dot(&O_minus_C);
        let c = O_minus_C.dot(&O_minus_C) - R*R;

        let discriminant = b*b - 4.0*a*c;
        let t = if discriminant < 0.0 {
            return None;
        } else if discriminant == 0.0 {
            let t = b / (-2.0 * a);
            if t >= 0.0 {
                Some(t)
            } else {
                None
            }
        } else {
            let t0 = (discriminant.sqrt() - b) / (2.0 * a); 
            let t1 = (discriminant.sqrt() + b) / (-2.0 * a);
            if t0 < 0.0 {
                if t1 < 0.0 {
                    None
                } else {
                    Some(t1)
                }
            } else {
                if t1 < 0.0 {
                    Some(t0)
                } else {
                    Some(f32::min(t0, t1))
                }
            }
        };

        t.map(|t| {
            let P = Point(O.0 + D.0*t);
            let normal = P - C;
            Intersection {
                distance: t,
                point: P,
                normal: normal.normalized(),
                material: self.2,
            }
        })
    }
}

struct Camera {
    ray: Ray,
    up: Direction,
    w_fov_degrees: f32,
}

struct Scene {
    camera: Camera,
    imgx: u32,
    imgy: u32,
    shapes: Vec<Box<dyn Intersect>>,
    lights: Vec<Light>,
}

impl Scene {
    fn render(&self, path: &str) {
        let camera_right = self.camera.ray.1.cross(&self.camera.up);
        // println!("camera ray:{:?} right:{:?} up:{:?}", &camera_ray, &camera_right, &camera_up);
    
        let camera_w_fov_radians: f32 = self.camera.w_fov_degrees.to_radians();
        let caemra_h_fov_radians = camera_w_fov_radians * (self.imgy as f32) / (self.imgx as f32);
        
        // Create a new ImgBuf with width: imgx and height: imgy
        let mut imgbuf = image::ImageBuffer::new(self.imgx, self.imgy);
        
        let center_x = self.imgx as f32 / 2.0;
        let center_y = self.imgy as f32 / 2.0;
    
        // Iterate over the coordinates and pixels of the image
        for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
            let radians_x = ((x as f32) - center_x) / (self.imgx as f32) * camera_w_fov_radians;
            let radians_y = (center_y - (y as f32))  / (self.imgy as f32) * caemra_h_fov_radians;
            let pixel_dir = 
                self.camera.ray.1.0 + 
                camera_right.0 * radians_x +
                self.camera.up.0 * radians_y;
            let pixel_dir = Direction(pixel_dir.normalized());
            let pixel_ray = Ray(self.camera.ray.0, pixel_dir);
    
            let intersections = self.shapes.iter().filter_map(|s| s.find_intersection(&pixel_ray));
            let closest = intersections
                .min_by(|i1,i2| i1.distance.partial_cmp(&i2.distance).unwrap());
    
            let mut color = BLACK;
            if let Some(i) = closest {
                for l in &self.lights {
                    let light_dir = l.position - i.point;
                    let light_distance = light_dir.0.magnitude();
                    let apparent_brightness = 1.0/light_distance*light_distance;
                    let light_dir = light_dir.normalized();
                    let diffuse = i.normal.dot(&light_dir)*l.intensity;
                    let light_reflect = light_dir.reflect(&i.normal);
                    let specular = l.intensity*light_reflect.dot(&pixel_dir).clamp(0.0, 1.0).powf(i.material.shininess);
                    let c = Color(
                        apparent_brightness * l.color.0 * (diffuse * i.material.diffuse_color.0 + specular * i.material.specular_color.0),
                        apparent_brightness * l.color.1 * (diffuse * i.material.diffuse_color.1 + specular * i.material.specular_color.1),
                        apparent_brightness * l.color.2 * (diffuse * i.material.diffuse_color.2 + specular * i.material.specular_color.2),
                    );
                    color = color + c;
                }
            }
            *pixel = image::Rgb([
                (color.0 * 256.0).clamp(0.0,255.0) as u8,
                (color.1 * 256.0).clamp(0.0,255.0) as u8,
                (color.2 * 256.0).clamp(0.0,255.0) as u8,
                ]);
        }
    
        imgbuf.save(path).unwrap();
    }
}

fn main() {
    let shapes: Vec<Box<dyn Intersect>> = vec![
        Box::new(Sphere(
            Point(Vec3(0.0,0.0,0.0)),
            0.7,
            Material{ 
                diffuse_color: BLUE,
                specular_color: 0.5 * WHITE,
                shininess: 50.0
            })),
        Box::new(Sphere(
            Point(Vec3(-0.7,0.7,-1.0)),
            1.0,
            Material{ 
                diffuse_color: RED,
                specular_color: 0.5 * WHITE,
                shininess: 50.0
            })),
        Box::new(Sphere(
            Point(Vec3(1.0,-1.0,1.0)),
            0.5,
            Material{ 
                diffuse_color: GREEN,
                specular_color: 0.5 * WHITE,
                shininess: 50.0
            })),
        Box::new(Sphere(
            Point(Vec3(10.0,0.0,-110.0)),
            100.0,
            Material{ 
                diffuse_color: WHITE,
                specular_color: 0.5 * WHITE,
                shininess: 50.0
            })),
    ];

    let lights = vec![
        Light {
            position: Point(Vec3(-2.0, -2.0, 1.0)),
            color: WHITE,
            intensity: 1.0,
        },
        Light {
            position: Point(Vec3(-2.0, 2.0, 1.0)),
            color: RED,
            intensity: 0.5,
        },
    ];

    let scene = Scene {
        camera: Camera {
            ray: Ray::from_points(
                Point(Vec3(-10.0, 0.0, 0.0)),
                Point(Vec3(0.0, 0.0, 0.0))),
            up: Direction(Vec3(0.0, 0.0, 1.0)),
            w_fov_degrees: 90.0,
        },
        imgx: 800,
        imgy: 800,
        shapes,
        lights,
    };

    scene.render("out.png");
}
