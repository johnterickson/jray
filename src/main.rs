use rayon::prelude::*;

mod vec3;
use smallvec::*;
use vec3::*;

mod aa;
use aa::*;

mod color;
use color::*;

mod sphere;

#[derive(Copy, Clone, Debug)]
pub struct Material {
    pub diffuse_color: Color,
    pub specular_color: Color,
    pub shininess: f64,
    pub reflectivity: f64,
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

#[derive(Debug)]
enum Shape {
    Sphere { center: Point, radius: f64 },
    Plane { point: Point, normal: Direction },
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

                let point = l0 + l * distance;
                Some(Intersection {
                    distance,
                    point,
                    surface_normal: *normal,
                })
            }
            Shape::Sphere { center, radius } => sphere::find_intersection(*center, *radius, r),
        }
    }
}

#[derive(Debug)]
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
    fn closest_intersection(&self, ray: &Ray) -> Option<(&Object, Intersection)> {
        let intersections = self
            .objects
            .iter()
            .filter_map(|o| o.shape.find_intersection(&ray).map(|i| (o, i)));
        let closest =
            intersections.min_by(|(_, i1), (_, i2)| i1.distance.partial_cmp(&i2.distance).unwrap());
        if let Some((_, i)) = &closest {
            assert!(i.distance >= 0.0);
        }
        closest
    }

    fn light_positions(
        center_ray: &Ray,
        light_radius: f64,
        count: usize,
        positions: &mut SmallVec<[Point; Self::MAX_LIGHT_POINTS]>,
    ) {
        positions.clear();
        positions.push(center_ray.0);
        if light_radius > 0.0 {
            let up = Direction(Vec3([0.0, 0.0, 1.0]));
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
    }

    const MAX_LIGHT_POINTS: usize = 10;

    #[inline(never)]
    fn render_ray(&self, ray: &Ray, mut recursion_limit: usize) -> Color {
        let mut color = BLACK;

        if recursion_limit == 0 {
            return color;
        }

        recursion_limit -= 1;

        let light_points = 1;

        assert!(light_points <= Self::MAX_LIGHT_POINTS);

        let mut light_positions: SmallVec<[_; Self::MAX_LIGHT_POINTS]> =
            smallvec![Point::origin(); Self::MAX_LIGHT_POINTS];

        if let Some((object, i)) = self.closest_intersection(&ray) {
            color += Color(0.0, 0.0, 0.0); // ambient

            // lighting and shadows
            let slightly_off_surface = Point(i.point.0 + i.surface_normal.0 * 0.001);
            for l in &self.lights {
                let mut shaded = 0;
                let mut total = 0;
                let center_ray = Ray(l.point, i.point - l.point);
                Self::light_positions(&center_ray, l.radius, light_points, &mut light_positions);
                for light_position in &light_positions {
                    let light_dir = *light_position - i.point;
                    let light_distance = light_dir.0.magnitude();
                    let ray_to_light = Ray(slightly_off_surface, light_dir.normalized());
                    if let Some((_shadow_obj, shadow_i)) = self.closest_intersection(&ray_to_light)
                    {
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
                let light_dir = i.point - l.point;
                let light_distance = light_dir.0.magnitude();
                let apparent_brightness = unblocked * l.intensity / light_distance * light_distance;
                assert!(apparent_brightness >= 0.0);
                let light_dir = light_dir.normalized();
                let dir_to_light = -1.0 * light_dir;
                let diffuse = i.surface_normal.dot(&dir_to_light).clamp(0.0, 1.0);
                assert!(diffuse >= 0.0);
                let light_reflect = light_dir.reflect(&i.surface_normal);
                let light_reflect = -1.0 * light_reflect;
                let specular = light_reflect
                    .dot(&ray.1)
                    .clamp(0.0, 1.0)
                    .powf(object.material.shininess);
                assert!(specular >= 0.0);

                let c = l.color
                    * apparent_brightness
                    * (diffuse * object.material.diffuse_color
                        + specular * object.material.specular_color);
                assert!(c.0 >= 0.0);
                assert!(c.1 >= 0.0);
                assert!(c.2 >= 0.0);
                color += c;
            }

            // reflection
            if object.material.reflectivity > 0.0 {
                let reflected_dir = ray.1.reflect(&i.surface_normal).normalized();
                let reflected_ray = Ray(slightly_off_surface, reflected_dir);
                let reflected_color = self.render_ray(&reflected_ray, recursion_limit);
                if reflected_color != BLACK {
                    // dbg!(&ray);
                    // dbg!(&object);
                    // dbg!(&i);
                    // dbg!(reflected_dir);
                    // dbg!(reflected_color);
                    color += object.material.reflectivity * reflected_color;
                    // assert!(false);
                }
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

        let aa = AntiAliasing::create(3);

        // Iterate over the coordinates and pixels of the image
        let mut pixels: Vec<_> = imgbuf.enumerate_pixels_mut().collect();
        pixels //.iter_mut()
            // .par_iter_mut()
            .iter_mut()
            .for_each(|(x, y, pixel)| {
                let mut color = BLACK;

                let mut count = 0;
                for (xx, yy) in aa.offsets() {
                    let x = xx + *x as f64;
                    let y = yy + *y as f64;
                    let radians_x = (x - center_x) / (self.imgx as f64) * camera_w_fov_radians;
                    let radians_y = (center_y - y) / (self.imgy as f64) * caemra_h_fov_radians;
                    let pixel_dir = self.camera.ray.1 .0
                        + camera_right.0 * radians_x
                        + self.camera.up.0 * radians_y;
                    let pixel_dir = Direction(pixel_dir.normalized());
                    let pixel_ray = Ray(self.camera.ray.0, pixel_dir);

                    color += self.render_ray(&pixel_ray, 1);
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
                center: Point(Vec3([0.0, 0.0, 0.0])),
                radius: 0.7,
            },
            material: Material {
                diffuse_color: BLUE,
                specular_color: 1.0 * WHITE,
                shininess: 50.0,
                reflectivity: 0.0,
            },
        },
        Object {
            shape: Shape::Sphere {
                center: Point(Vec3([-0.7, 0.7, -1.0])),
                radius: 1.0,
            },
            material: Material {
                diffuse_color: RED,
                specular_color: 0.0 * WHITE,
                shininess: 50.0,
                reflectivity: 0.0,
            },
        },
        Object {
            shape: Shape::Sphere {
                center: Point(Vec3([1.0, -1.0, 1.0])),
                radius: 0.5,
            },
            material: Material {
                diffuse_color: GREEN,
                specular_color: 0.5 * WHITE,
                shininess: 50.0,
                reflectivity: 0.0,
            },
        },
        Object {
            shape: Shape::Plane {
                point: Point(Vec3([0.0, 0.0, -10.0])),
                normal: Direction(Vec3([0.0, 0.0, 1.0])),
            },
            material: Material {
                diffuse_color: 0.1 * WHITE,
                specular_color: BLACK,
                shininess: 50.0,
                reflectivity: 0.7,
            },
        },
        Object {
            shape: Shape::Plane {
                point: Point(Vec3([0.0, 0.0, 10.0])),
                normal: Direction(Vec3([0.0, 0.0, -1.0])),
            },
            material: Material {
                diffuse_color: 0.1 * WHITE,
                specular_color: 0.1 * WHITE,
                shininess: 50.0,
                reflectivity: 0.7,
            },
        },
        Object {
            shape: Shape::Plane {
                point: Point(Vec3([10.0, 0.0, 0.0])),
                normal: Direction(Vec3([-1.0, 0.0, 0.0])),
            },
            material: Material {
                diffuse_color: 0.1 * WHITE,
                specular_color: 0.1 * WHITE,
                shininess: 50.0,
                reflectivity: 0.7,
            },
        },
        Object {
            shape: Shape::Plane {
                point: Point(Vec3([-10.0, 0.0, 0.0])),
                normal: Direction(Vec3([1.0, 0.0, 0.0])),
            },
            material: Material {
                diffuse_color: 0.1 * WHITE,
                specular_color: 0.1 * WHITE,
                shininess: 50.0,
                reflectivity: 0.7,
            },
        },
        Object {
            shape: Shape::Plane {
                point: Point(Vec3([0.0, 10.0, 0.0])),
                normal: Direction(Vec3([0.0, -1.0, 0.0])),
            },
            material: Material {
                diffuse_color: 0.1 * WHITE,
                specular_color: 0.1 * WHITE,
                shininess: 50.0,
                reflectivity: 0.7,
            },
        },
        Object {
            shape: Shape::Plane {
                point: Point(Vec3([0.0, -10.0, 0.0])),
                normal: Direction(Vec3([0.0, 1.0, 0.0])),
            },
            material: Material {
                diffuse_color: 0.1 * WHITE,
                specular_color: 0.5 * WHITE,
                shininess: 50.0,
                reflectivity: 0.7,
            },
        },
    ];

    let lights = vec![
        Light {
            point: Point(Vec3([-2.0, 1.0, 0.7])),
            color: WHITE,
            intensity: 0.6,
            radius: 0.05,
        },
        Light {
            point: Point(Vec3([-2.0, -2.0, 2.0])),
            color: WHITE,
            intensity: 0.7,
            radius: 0.05,
        },
    ];

    let scene = Scene {
        camera: Camera {
            ray: Ray::from_points(Point(Vec3([-4.9, 3.0, 3.0])), Point(Vec3([0.0, 0.0, 0.0]))),
            up: Direction(Vec3([0.0, 0.0, 1.0])),
            w_fov_degrees: 90.0,
        },
        imgx: 800,
        imgy: 800,
        objects: shapes,
        lights,
    };

    scene.render("out.png");
}
