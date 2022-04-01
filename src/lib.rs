use std::iter::zip;

const PRIMES: [usize; 4] = [3, 5, 7, 11];
const FOOTER: [u8; 8] = [0, 0, 0, 0, 0, 0, 0, 1];

#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum Channels {
    Rgb = 3,
    Rgba = 4,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum ColorSpace {
    LinearAlpha,
    AllLinear,
}

#[derive(Clone, Debug)]
pub struct Image {
    pixels: Vec<u8>,
    width: u32,
    height: u32,
    channels: Channels,
    colorspace: ColorSpace,
}

pub mod decode;
pub mod encode;

use std::fs;
use std::io::Result;
use std::path::Path;

pub fn load<P: AsRef<Path>>(p: P) -> Result<Image> {
    Ok(decode::decode(&fs::read(p)?))
}

pub fn store<P: AsRef<Path>>(img: &Image, p: P) -> Result<()> {
    fs::write(p, encode::encode(img))
}

fn hash<const SIZE: usize>(curr: &[u8]) -> usize {
    zip(&curr[..SIZE], PRIMES)
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
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
