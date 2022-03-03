mod vec3;
use vec3::*;

mod color;
use color::*;

mod sphere;
use sphere::*;

#[derive(Copy, Clone)]
pub struct Material {
    pub diffuse_color: Color,
    pub specular_color: Color,
    pub shininess: f64,
}

struct Light {
    pub position: Point,
    pub color: Color,
    pub intensity: f64,
}

struct Intersection {
    pub distance: f64,
    pub point: Point,
    pub normal: Direction,
    pub material: Material,
}

trait Intersect {
    fn find_intersection(&self, r: &Ray) -> Option<Intersection>;
}

struct Camera {
    ray: Ray,
    up: Direction,
    w_fov_degrees: f64,
}

struct Scene {
    camera: Camera,
    imgx: u32,
    imgy: u32,
    shapes: Vec<Box<dyn Intersect>>,
    lights: Vec<Light>,
}

impl Scene {
    fn closest_intersection(&self, ray: &Ray) -> Option<Intersection> {
        let intersections = self.shapes.iter().filter_map(|s| s.find_intersection(&ray));
        let closest = intersections.min_by(|i1, i2| i1.distance.partial_cmp(&i2.distance).unwrap());
        if let Some(ref closest) = closest {
            assert!(closest.distance >= 0.0);
        }
        closest
    }

    fn render_ray(&self, ray: &Ray) -> Color {
        let mut color = BLACK;
        if let Some(i) = self.closest_intersection(&ray) {
            color = color + Color(0.05, 0.05, 0.05); // ambient

            let slightly_off_surface = Point(i.point.0 + i.normal.0 * 0.00001);
            for l in &self.lights {
                let light_dir = l.position - i.point;
                let ray_to_light = Ray(slightly_off_surface, light_dir.normalized());
                if let Some(_shadow) = self.closest_intersection(&ray_to_light) {
                    continue;
                }
                let light_distance = light_dir.0.magnitude();
                let apparent_brightness = l.intensity / light_distance * light_distance;
                assert!(apparent_brightness >= 0.0);
                let light_dir = light_dir.normalized();
                let diffuse = apparent_brightness * i.normal.dot(&light_dir).clamp(0.0, 1.0);
                assert!(diffuse >= 0.0);
                let light_reflect = light_dir.reflect(&i.normal);
                let specular = apparent_brightness
                    * light_reflect
                        .dot(&ray.1)
                        .clamp(0.0, 1.0)
                        .powf(i.material.shininess);
                assert!(specular >= 0.0);

                let c = l.color
                    * (diffuse * i.material.diffuse_color + specular * i.material.specular_color);
                assert!(c.0 >= 0.0);
                assert!(c.1 >= 0.0);
                assert!(c.2 >= 0.0);
                color = color + c;
            }
        }

        color
    }

    fn render(&self, path: &str) {
        let camera_right = self.camera.ray.1.cross(&self.camera.up);
        // println!("camera ray:{:?} right:{:?} up:{:?}", &camera_ray, &camera_right, &camera_up);

        let camera_w_fov_radians: f64 = self.camera.w_fov_degrees.to_radians();
        let caemra_h_fov_radians = camera_w_fov_radians * (self.imgy as f64) / (self.imgx as f64);

        // Create a new ImgBuf with width: imgx and height: imgy
        let mut imgbuf = image::ImageBuffer::new(self.imgx, self.imgy);

        let center_x = self.imgx as f64 / 2.0;
        let center_y = self.imgy as f64 / 2.0;

        // Iterate over the coordinates and pixels of the image
        for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
            let mut color = BLACK;

            let mut count = 0;
            for xx in [-0.33, 0.0, 0.33] {
                for yy in [-0.33, 0.0, 0.33] {
                    let x = xx + x as f64;
                    let y = yy + y as f64;
                    let radians_x = (x - center_x) / (self.imgx as f64) * camera_w_fov_radians;
                    let radians_y = (center_y - y) / (self.imgy as f64) * caemra_h_fov_radians;
                    let pixel_dir = self.camera.ray.1 .0
                        + camera_right.0 * radians_x
                        + self.camera.up.0 * radians_y;
                    let pixel_dir = Direction(pixel_dir.normalized());
                    let pixel_ray = Ray(self.camera.ray.0, pixel_dir);

                    color += self.render_ray(&pixel_ray);
                    count += 1;
                }
            }

            color *= 1.0 / count as f64;

            *pixel = image::Rgb(color.to_rgb());
        }

        imgbuf.save(path).unwrap();
    }
}

fn main() {
    let shapes: Vec<Box<dyn Intersect>> = vec![
        Box::new(Sphere(
            Point(Vec3(0.0, 0.0, 0.0)),
            0.7,
            Material {
                diffuse_color: BLUE,
                specular_color: 0.5 * WHITE,
                shininess: 50.0,
            },
        )),
        Box::new(Sphere(
            Point(Vec3(-0.7, 0.7, -1.0)),
            1.0,
            Material {
                diffuse_color: RED,
                specular_color: 0.0 * WHITE,
                shininess: 50.0,
            },
        )),
        Box::new(Sphere(
            Point(Vec3(1.0, -1.0, 1.0)),
            0.5,
            Material {
                diffuse_color: GREEN,
                specular_color: 0.5 * WHITE,
                shininess: 50.0,
            },
        )),
        Box::new(Sphere(
            Point(Vec3(10.0, 0.0, -102.5)),
            100.0,
            Material {
                diffuse_color: WHITE,
                specular_color: 0.5 * WHITE,
                shininess: 50.0,
            },
        )),
    ];

    let lights = vec![
        Light {
            position: Point(Vec3(-2.0, -2.0, 2.0)),
            color: WHITE,
            intensity: 2.0,
        },
        Light {
            position: Point(Vec3(-2.0, 3.0, 0.7)),
            color: WHITE,
            intensity: 0.5,
        },
    ];

    let scene = Scene {
        camera: Camera {
            ray: Ray::from_points(Point(Vec3(-10.0, 0.0, 0.0)), Point(Vec3(0.0, 0.0, 0.0))),
            up: Direction(Vec3(0.0, 0.0, 1.0)),
            w_fov_degrees: 20.0,
        },
        imgx: 800,
        imgy: 800,
        shapes,
        lights,
    };

    scene.render("out.png");
}
