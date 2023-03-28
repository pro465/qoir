use std::iter::zip;

const PRIMES: [usize; 4] = [3, 5, 7, 11];
const FOOTER: [u8; 8] = [0, 0, 0, 0, 0, 0, 0, 1];

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Channels {
    Rgb = 3,
    Rgba = 4,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColorSpace {
    LinearAlpha,
    AllLinear,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Image {
    pixels: Vec<Pixel>,
    width: u32,
    height: u32,
    channels: Channels,
    colorspace: ColorSpace,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pixel(u8, u8, u8, u8);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DecodeError {
    pub loc: usize,
}

pub mod decode;
pub mod encode;

use std::fs;
use std::io::Result as IoRes;
use std::path::Path;

pub fn load<P: AsRef<Path>>(p: P) -> IoRes<Result<Image, DecodeError>> {
    Ok(decode::decode(&fs::read(p)?))
}

pub fn store<P: AsRef<Path>>(img: &Image, p: P) -> IoRes<()> {
    fs::write(p, encode::encode(img))
}

fn hash(c: Pixel) -> usize {
    let curr = [c.0, c.1, c.2, c.3];

    zip(&curr, PRIMES)
        .map(|(c, p): (&u8, _)| *c as usize * p)
        .sum::<usize>()
        % 64
}

fn wadd(a: u8, b: u8) -> u8 {
    a.wrapping_add(b)
}

fn wsub(a: u8, b: u8) -> u8 {
    a.wrapping_sub(b)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {
        let inp = include_bytes!("lib.rs");

        let img = Image {
            pixels: inp
                .chunks_exact(3)
                .map(|i| Pixel(i[0], i[1], i[2], 255))
                .collect(),
            width: 3,
            height: inp.len() as u32 / 9,
            channels: Channels::Rgb,
            colorspace: ColorSpace::AllLinear,
        };
        let out = encode::encode(&img);
        let dec = decode::decode(&out).unwrap();

        assert_eq!(img.pixels, dec.pixels);
    }
}
