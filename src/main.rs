mod vec3;
use vec3::*;

struct Sphere(Point, f32);
struct Intersection {
    pub point: Point,
    pub object_normal: Direction,
}

trait Intersect {
    fn find_intersection(&self, r: &Ray) -> Option<Intersection>;
}

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
            Some(t)
        } else {
            let t0 = (discriminant.sqrt() - b) / (2.0 * a); 
            let t1 = (discriminant.sqrt() + b) / (-2.0 * a); 
            Some(if t0 < t1 {t0} else {t1})
        };

        t.map(|t| {
            let P = Point(O.0 + D.0*t);
            let normal = P - C;
            Intersection {
                point: P,
                object_normal: normal.normalized(),
            }
        })
    }
}

fn main() {
    let camera_ray = Ray::from_points(
        Point(Vec3(-2.0, 0.0, 0.0)),
        Point(Vec3(0.0, 0.0, 0.0)));
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
    let l = Point(Vec3(-2.0, -2.0, 1.0));
    let l_i = 100.0;
    let shininess = 3.0;

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

        
        let color = if let Some(i) = s.find_intersection(&pixel_ray) {
            let light_dir = (l - i.point).normalized();
            let c = i.object_normal.dot(&light_dir)*l_i;
            let c = c.clamp(0.0,255.0) as u8;
            [c,c,c]
        } else {
            [0u8,0,0]
        };
        *pixel = image::Rgb(color);
    }

    imgbuf.save("out.png").unwrap();
}
