extern crate crossbeam;
extern crate image;
extern crate num;

use num::complex::Complex;
use std::f32::consts::PI;
use std::str::FromStr;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let sep = ',';

    if args.len() == 4 {
        let threads = 7;
        let path = &args[0];
        let bounds: (usize, usize) =
            parse_pair(&args[1], &sep).expect("error, invalid image dimension !!!");

        let (plane_ul, plane_br) = (
            parse_complex::<f64>(&args[2], &sep).expect("error invalid args !!!"),
            parse_complex::<f64>(&args[3], &sep).expect("error invalid args !!!"),
        );

        let mut buffer: Vec<u8> = vec![0; bounds.0 * bounds.1];
        let row_per_chunk = bounds.1 / threads + 1;
        let chunks = buffer.chunks_mut(row_per_chunk * bounds.0);

        crossbeam::scope(|s| {
            for (i, c) in chunks.enumerate() {
                let chunk_bounds = (i * row_per_chunk, i * row_per_chunk + c.len() / bounds.0);
                s.spawn(move |_| render(c, chunk_bounds, bounds, (plane_ul, plane_br)));
            }
        })
        .expect("error, thread crashed !!!");

        image::save_buffer(
            path,
            &buffer,
            bounds.0 as u32,
            bounds.1 as u32,
            image::ColorType::L8,
        )
        .expect("error, can't create image !!!");
    } else {
        println!("Usage: [PATH] [x,y] [re,im]");
        println!("Image Dimensions: [width, height]");
        println!("Complex Plane: Top-Left[re, im] Bottom-Right[re, im]");
    }
}

/// parses a pair values seperated by a char into a tuple.
fn parse_pair<T: FromStr>(s: &str, sep: &char) -> Option<(T, T)> {
    match s.find(*sep) {
        Some(index) => match (T::from_str(&s[..index]), T::from_str(&s[index + 1..])) {
            (Ok(l), Ok(r)) => Some((l, r)),
            _ => None,
        },
        None => None,
    }
}

/// parses a re and im values seperated by char into complex.
fn parse_complex<T: FromStr>(s: &str, sep: &char) -> Option<Complex<T>> {
    match parse_pair::<T>(&s, sep) {
        Some((re, im)) => Some(Complex::new(re, im)),
        _ => None,
    }
}

/// maps pixel cordinate to complex plane.
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

/// renders fractal.
fn render(
    chunk: &mut [u8],
    chunk_bounds: (usize, usize),
    image_bounds: (usize, usize),
    plane_bounds: (Complex<f64>, Complex<f64>),
) {
    let tol = 0.0001;
    let mul = 15;
    let max_count = 255;
    let r1: Complex<f64> = Complex::new(1.0, 0.0);
    let r2: Complex<f64> = Complex::new(-0.5, (2.0 * PI / 3.0).sin() as f64);
    let r3: Complex<f64> = Complex::new(-0.5, -(2.0 * PI / 3.0).sin() as f64);

    for y in 0..(chunk_bounds.1 - chunk_bounds.0) {
        for x in 0..image_bounds.0 {
            let mut z = pixel_to_complex(image_bounds, (x, chunk_bounds.0 + y), plane_bounds);
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

            chunk[y * image_bounds.0 + x] = (255 - count * mul) as u8;
        }
    }
}
