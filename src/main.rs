extern crate image;
extern crate num;

use num::complex::Complex;
use std::f32::consts::PI;
use std::str::FromStr;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let sep = ',';

    if args.len() == 4 {
        let path = args[0].clone();
        let bounds: (usize, usize) =
            parse_pair(&args[1], &sep).expect("error, invalid image dimention !!!");

        let (plane_ul, plane_br) = (
            parse_complex::<f64>(&args[2], &sep).expect("error invalid args !!!"),
            parse_complex::<f64>(&args[3], &sep).expect("error invalid args !!!"),
        );

        let mut buffer: Vec<u8> = vec![0; bounds.0 * bounds.1];
        render(&mut buffer, bounds, (plane_ul, plane_br));

        image::save_buffer(
            path,
            &buffer,
            bounds.0 as u32,
            bounds.1 as u32,
            image::ColorType::Rgb8,
        )
        .expect("error, can't create image !!!");
    } else {
        println!("Usage: [PATH] [IMG-DIM] [PLANE-DIM]");
        println!("Image Dimentions: [width, height]");
        println!("Complex Plane Dimentions: Top-Left[re, im] bottom-Right[re, im]");
    }
}

fn parse_pair<T: FromStr>(s: &str, sep: &char) -> Option<(T, T)> {
    match s.find(*sep) {
        Some(index) => match (T::from_str(&s[..index]), T::from_str(&s[index + 1..])) {
            (Ok(l), Ok(r)) => Some((l, r)),
            _ => None,
        },
        None => None,
    }
}

fn parse_complex<T: FromStr>(s: &str, sep: &char) -> Option<Complex<T>> {
    match parse_pair::<T>(&s, sep) {
        Some((re, im)) => Some(Complex::new(re, im)),
        _ => None,
    }
}

fn render(buffer: &mut Vec<u8>, bounds: (usize, usize), plane: (Complex<f64>, Complex<f64>)) {
    let tol = 0.0001;
    let mult = 15;
    let max_count = 255;
    let r1: Complex<f64> = Complex::new(1.0, 0.0);
    let r2: Complex<f64> = Complex::new(-0.5, (2.0 * PI / 3.0).sin() as f64);
    let r3: Complex<f64> = Complex::new(-0.5, -(2.0 * PI / 3.0).sin() as f64);

    for y in 0..bounds.1 {
        for x in 0..bounds.0 {
            let mut z = pixel_to_complex(bounds, (x, y), plane);
            let mut count = 0;
            while count < max_count
                && (z - r1).norm() >= tol
                && (z - r2).norm() >= tol
                && (z - r3).norm() >= tol
            {
                if z.norm() > 0.0 {
                    z = z - (z * z * z - 1.0) / (3.0 * z * z);
                }
                count += 1;
            }

            buffer[y * bounds.0 + x] = (255 - count * mult) as u8;
        }
    }
}

fn pixel_to_complex(
    bounds: (usize, usize),
    pixel: (usize, usize),
    plane: (Complex<f64>, Complex<f64>),
) -> Complex<f64> {
    let (w, h) = (plane.1.re - plane.0.re, plane.0.im - plane.1.im);
    Complex {
        re: plane.0.re + pixel.0 as f64 * w / bounds.0 as f64,
        im: plane.0.im - pixel.1 as f64 * h / bounds.1 as f64,
    }
}
