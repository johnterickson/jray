mod vec3;
use vec3::*;

struct Point(Vec3);
struct Direction(Vec3);
struct Ray(Point, Direction);

fn main() {
    let camera_ray = Ray(
        Point(Vec3 {
            X: -10.0,
            Y: -10.0,
            Z: 0.0,
        }),
        Direction(
            Vec3 {
                X: 1.0,
                Y: 1.0,
                Z: 0.0,
            }
            .normalized(),
        ),
    );
    let camera_fov_degrees = 90.0;

    let imgx = 800;
    let imgy = 800;

    // Create a new ImgBuf with width: imgx and height: imgy
    let mut imgbuf = image::ImageBuffer::new(imgx, imgy);
    let center_x = imgx / 2;
    let center_y = imgy / 2;

    // Iterate over the coordinates and pixels of the image
    for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
        let i = (x + y) as f32;
        let r = ((i / 1.0 as f32) % 256.0) as u8;
        let g = ((i / 3.0 as f32) % 256.0) as u8;
        let b = ((i / 7.0 as f32) % 256.0) as u8;
        *pixel = image::Rgb([r, g, b]);
    }

    imgbuf.save("out.png").unwrap();
}
