use crate::*;

#[allow(non_snake_case)]
pub fn find_intersection(center: Point, radius: f64, r: &Ray) -> Option<Intersection> {
    // https://www.scratchapixel.com/lessons/3d-basic-rendering/minimal-ray-tracer-rendering-simple-shapes/ray-sphere-intersection

    let O = r.0;
    let D = r.1;
    let C = center;
    let R = radius;

    let O_minus_C = O - C;

    let a = 1.0;
    let b = 2.0 * D.dot(&O_minus_C);
    let c = O_minus_C.dot(&O_minus_C) - R * R;

    let discriminant = b * b - 4.0 * a * c;
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
                Some(f64::min(t0, t1))
            }
        }
    };

    t.map(|t| {
        let P = Point(O.0 + D.0 * t);
        let normal = P - C;
        Intersection {
            distance: t,
            point: P,
            surface_normal: normal.normalized(),
        }
    })
}
