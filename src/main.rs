use rayon::prelude::*;

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
    pub opacity: f64,
}

struct Light {
    pub point: Point,
    pub color: Color,
    pub radius: f64, 
    pub intensity: f64,
}

struct Intersection {
    pub distance: f64,
    pub point: Point,
    pub normal: Direction,
    pub material: Material,
}

enum Shape {
    Sphere(Sphere)
}

impl Intersect for Shape {
    fn find_intersection(&self, r: &Ray) -> Option<Intersection> {
        match self {
            Shape::Sphere(s) => s.find_intersection(r)
        }
    }
}

trait Intersect {
    fn find_intersection(&self, r: &Ray) -> Option<Intersection>;
}

struct Camera {
    ray: Ray,
    up: Direction,
    w_fov_degrees: f64,
}

struct AntiAliasing(Vec<(f64,f64)>);
impl AntiAliasing {
    fn create(n: u8) -> AntiAliasing {
        let mut offsets = Vec::with_capacity(n as usize * n as usize);
        offsets.push((0.0, 0.0));

        let n: i8 = n.try_into().unwrap();

        let delta = 1.0 / (n as f64 + 1.0);
        for x in 1..n {
            let x = x as f64;
            for y in 1..n {
                let y = y as f64;
                offsets.push((x * delta, y * delta));
                offsets.push((x * delta, -1.0 *y * delta));
                offsets.push((-1.0 * x * delta, y * delta));
                offsets.push((-1.0 * x * delta, -1.0 * y * delta));
            }
        }

        AntiAliasing(offsets)
    }

    fn offsets(&self) -> &[(f64,f64)] {
        self.0.as_slice()
    }
}

struct Scene {
    camera: Camera,
    imgx: u32,
    imgy: u32,
    shapes: Vec<Shape>,
    lights: Vec<Light>,
}

impl Scene {
    fn closest_intersection(&self, ray: &Ray) -> Option<(&Shape,Intersection)> {
        let intersections = self.shapes.iter()
            .filter_map(|s| s.find_intersection(&ray).map(|i| (s,i)));
        let closest = intersections.min_by(|(_,i1), (_,i2)| {
            i1.distance.partial_cmp(&i2.distance).unwrap()
        });
        if let Some((_, i)) = &closest {
            assert!(i.distance >= 0.0);
        }
        closest
    }

    fn light_positions(center_ray: &Ray, light_radius: f64, count: usize) -> Vec<Point> {
        let mut positions = Vec::with_capacity(count);
        positions.push(center_ray.0);
        if light_radius > 0.0 {
            let up = Direction(Vec3(0.0, 0.0, 1.0));
            let right = center_ray.1.cross(&up).normalized();

            let revolutions_in_spiral = 2.0;
            
            for i in 0..count {
                let scaler = (i as f64) / (count as f64);
                let theta = revolutions_in_spiral * scaler * 2.0 * std::f64::consts::PI;
                let spiral_radius = scaler * light_radius;
                let up = spiral_radius * up;
                let right = spiral_radius * right;
                positions.push(center_ray.0 + theta.cos() * right + theta.sin() * up);
            }
        }
        positions
    }

    fn render_ray(&self, ray: &Ray) -> Color {
        let mut color = BLACK;
        if let Some( (_, i)) = self.closest_intersection(&ray) {
            color += Color(0.1, 0.1, 0.1); // ambient

            let slightly_off_surface = Point(i.point.0 + i.normal.0 * 0.00001);
            for l in &self.lights {
                let mut shaded = 0;
                let mut total = 0;
                let center_ray = Ray(l.point, i.point - l.point);
                let light_positions = Self::light_positions(&center_ray, l.radius, 20);
                for light_position in light_positions {
                    let light_dir = light_position - i.point;
                    let ray_to_light = Ray(slightly_off_surface, light_dir.normalized());
                    if let Some(_shadow) = self.closest_intersection(&ray_to_light) {
                        shaded += 1;
                    }
                    total += 1;
                }

                if shaded == total {
                    continue;
                }

                let unblocked = 1.0 - shaded as f64 / total as f64;
                let light_dir = l.point - i.point;
                let light_distance = light_dir.0.magnitude();
                let apparent_brightness = unblocked * l.intensity / light_distance * light_distance;
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
                color += c;
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

        let aa = AntiAliasing::create(2);

        // Iterate over the coordinates and pixels of the image
        let mut pixels: Vec<_> = imgbuf.enumerate_pixels_mut().collect();
        pixels//.iter_mut()
              .par_iter_mut()
              .for_each(|(x, y, pixel)| {
                let mut color = BLACK;

                let mut count = 0;
                for (xx, yy) in aa.offsets() {
                    let x = xx + *x as f64;
                    let y = yy + *y as f64;
                    let radians_x = (x - center_x) / (self.imgx as f64) * camera_w_fov_radians;
                    let radians_y = (center_y - y) / (self.imgy as f64) * caemra_h_fov_radians;
                    let pixel_dir = self.camera.ray.1.0
                        + camera_right.0 * radians_x
                        + self.camera.up.0 * radians_y;
                    let pixel_dir = Direction(pixel_dir.normalized());
                    let pixel_ray = Ray(self.camera.ray.0, pixel_dir);

                    color += self.render_ray(&pixel_ray);
                    count += 1;
                }
    
                color *= 1.0 / count as f64;
    
                **pixel = image::Rgb(color.to_rgb());
              });

        imgbuf.save(path).unwrap();
    }
}

fn main() {
    let shapes: Vec<Shape> = vec![
        Shape::Sphere(Sphere(
            Point(Vec3(0.0, 0.0, 0.0)),
            0.7,
            Material {
                diffuse_color: BLUE,
                specular_color: 1.0 * WHITE,
                shininess: 50.0,
                opacity: 1.0,
            },
        )),
        Shape::Sphere(Sphere(
            Point(Vec3(-0.7, 0.7, -1.0)),
            1.0,
            Material {
                diffuse_color: RED,
                specular_color: 0.0 * WHITE,
                shininess: 50.0,
                opacity: 0.1,
            },
        )),
        Shape::Sphere(Sphere(
            Point(Vec3(1.0, -1.0, 1.0)),
            0.5,
            Material {
                diffuse_color: GREEN,
                specular_color: 0.5 * WHITE,
                shininess: 50.0,
                opacity: 1.0,
            },
        )),
        Shape::Sphere(Sphere(
            Point(Vec3(10.0, 0.0, -102.5)),
            100.0,
            Material {
                diffuse_color: WHITE,
                specular_color: 0.5 * WHITE,
                shininess: 50.0,
                opacity: 1.0,
            },
        )),
    ];

    let lights = vec![
        Light {
            point: Point(Vec3(-2.0, 1.0, 0.7)),
            color: WHITE,
            intensity: 0.9,
            radius: 0.1,
        },
        Light {
            point: Point(Vec3(-2.0, -2.0, 2.0)),
            color: WHITE,
            intensity: 0.7,
            radius: 0.05,
        },
    ];

    let scene = Scene {
        camera: Camera {
            ray: Ray::from_points(Point(Vec3(-10.0, 0.0, 0.0)), Point(Vec3(0.0, 0.0, 0.0))),
            up: Direction(Vec3(0.0, 0.0, 1.0)),
            w_fov_degrees: 50.0,
        },
        imgx: 1000,
        imgy: 1000,
        shapes,
        lights,
    };

    scene.render("out.png");
}
