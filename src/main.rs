use rayon::prelude::*;

mod vec3;
use vec3::*;

mod aa;
use aa::*;

mod color;
use color::*;

mod sphere;

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

#[derive(Debug)]
pub struct Intersection {
    pub distance: f64,
    pub point: Point,
    pub surface_normal: Direction,
}

enum Shape {
    Sphere{
        center: Point, 
        radius: f64,
    },
    Plane {
        point: Point,
        normal: Direction,
    }
}

impl Shape {
    fn find_intersection(&self, r: &Ray) -> Option<Intersection> {
        match self {
            Shape::Plane { point, normal } => {
                // https://en.wikipedia.org/wiki/Line%E2%80%93plane_intersection#Algebraic_form
                let p0 = *point;
                let n = normal;
                let l0 = r.0;
                let l = r.1;

                let l_dot_n = l.dot(n);

                if l_dot_n == 0.0 {
                    return None;
                }

                let distance = (p0 - l0).dot(n) / l_dot_n;

                if distance <= 0.0 {
                    return None;
                }

                let point = l0 + l*distance;
                Some(Intersection {
                    distance,
                    point,
                    surface_normal: *normal,
                })
            },
            Shape::Sphere { center, radius } => {
                sphere::find_intersection(*center, *radius, r)
            },
        }
    }
}

struct Object {
    pub material: Material,
    pub shape: Shape,
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
    objects: Vec<Object>,
    lights: Vec<Light>,
}

impl Scene {
    fn closest_intersection(&self, ray: &Ray) -> Option<(&Object,Intersection)> {
        let intersections = self.objects.iter()
            .filter_map(|o| o.shape.find_intersection(&ray).map(|i| (o,i)));
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
        if let Some( (object, i)) = self.closest_intersection(&ray) {
            color += Color(0.1, 0.1, 0.1); // ambient

            let slightly_off_surface = Point(i.point.0 + i.surface_normal.0 * 0.00001);
            for l in &self.lights {
                let mut shaded = 0;
                let mut total = 0;
                let center_ray = Ray(l.point, i.point - l.point);
                let light_positions = Self::light_positions(&center_ray, l.radius, 1);
                for light_position in light_positions {
                    let light_dir = light_position - i.point;
                    let light_distance = light_dir.0.magnitude();
                    let ray_to_light = Ray(slightly_off_surface, light_dir.normalized());
                    if let Some((_shadow_obj, shadow_i)) = self.closest_intersection(&ray_to_light) {
                        // intersection with shadow object happens ...
                        if shadow_i.distance > light_distance {
                            // ... behind the light
                        } else {
                            // ... between the light and the target
                            shaded += 1;
                        }
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
                let diffuse = apparent_brightness * i.surface_normal.dot(&light_dir).clamp(0.0, 1.0);
                assert!(diffuse >= 0.0);
                let light_reflect = light_dir.reflect(&i.surface_normal);
                let specular = apparent_brightness
                    * light_reflect
                        .dot(&ray.1)
                        .clamp(0.0, 1.0)
                        .powf(object.material.shininess);
                assert!(specular >= 0.0);

                let c = l.color
                    * (diffuse * object.material.diffuse_color + specular * object.material.specular_color);
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
              //.par_iter_mut()
              .iter_mut()
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
    let shapes: Vec<_> = vec![
        Object {
            shape: Shape::Sphere {
                center: Point(Vec3(0.0, 0.0, 0.0)),
                radius: 0.7,
            },
            material: Material {
                diffuse_color: BLUE,
                specular_color: 1.0 * WHITE,
                shininess: 50.0,
                opacity: 1.0,
            },
        },
        Object {
            shape: Shape::Sphere {
                center: Point(Vec3(-0.7, 0.7, -1.0)),
                radius: 1.0,
            },
            material: Material {
                diffuse_color: RED,
                specular_color: 0.0 * WHITE,
                shininess: 50.0,
                opacity: 0.1,
            },
        },
        Object {
            shape: Shape::Sphere {
                center: Point(Vec3(1.0, -1.0, 1.0)),
                radius: 0.5,
            },
            material: Material {
                diffuse_color: GREEN,
                specular_color: 0.5 * WHITE,
                shininess: 50.0,
                opacity: 1.0,
            },
        },
        Object {
            shape: Shape::Plane {
                point: Point(Vec3(0.0, 0.0, -5.0)),
                normal: Direction(Vec3(0.0,0.0,1.0)),
            },
            material: Material {
                diffuse_color: WHITE,
                specular_color: 0.5 * WHITE,
                shininess: 50.0,
                opacity: 1.0,
            },
        }
    ];

    let lights = vec![
        Light {
            point: Point(Vec3(-2.0, 1.0, 0.7)),
            color: WHITE,
            intensity: 0.9,
            radius: 0.05,
        },
        // Light {
        //     point: Point(Vec3(-2.0, -2.0, 2.0)),
        //     color: WHITE,
        //     intensity: 0.7,
        //     radius: 0.05,
        // },
    ];

    let scene = Scene {
        camera: Camera {
            ray: Ray::from_points(Point(Vec3(-10.0, 0.0, 0.0)), Point(Vec3(0.0, 0.0, 0.0))),
            up: Direction(Vec3(0.0, 0.0, 1.0)),
            w_fov_degrees: 50.0,
        },
        imgx: 1000,
        imgy: 1000,
        objects: shapes,
        lights,
    };

    scene.render("out.png");
}
