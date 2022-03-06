pub struct AntiAliasing(Vec<(f64,f64)>);

impl AntiAliasing {
    pub fn create(n: u8) -> AntiAliasing {
        assert_ne!(0, n);
        let mut offsets = Vec::with_capacity(n as usize * n as usize);
        let n: i8 = n.try_into().unwrap();
        let delta = 1.0 / (n as f64);
        let start_offset = if n == 1 {
            0.0
        } else {
            0.5*(1.0 - delta)
        };
        for x in 0..n {
            let x = x as f64;
            for y in 0..n {
                let y = y as f64;
                offsets.push((x*delta - start_offset, y * delta - start_offset));
            }
        }

        for n in offsets.iter_mut().map(|(x,y)| [x,y]).flatten() {
            if n.abs() < 0.000000001 {
                *n = 0.0;
            }
        }

        AntiAliasing(offsets)
    }

    pub fn offsets(&self) -> &[(f64,f64)] {
        self.0.as_slice()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn add() {
        assert_eq!([(0.0,0.0)], AntiAliasing::create(1).offsets());
        assert_eq!([(-0.25, -0.25), (-0.25, 0.25), (0.25, -0.25), (0.25, 0.25)], AntiAliasing::create(2).offsets());
        let three = AntiAliasing::create(3);
        let three = three.offsets();
        assert_eq!(9, three.len());
        let avg_x = three.iter().map(|(x,_)| *x).sum::<f64>() / 9.0;
        let avg_y = three.iter().map(|(_,y)| *y).sum::<f64>() / 9.0;
        assert!(avg_x.abs() < 0.00001);
        assert!(avg_y.abs() < 0.00001);
    }
}