use crate::*;

pub fn decode(encoded: &[u8]) -> Result<Image, DecodeError> {
    if &encoded[..4] != b"qoif" {
        return Err(DecodeError { loc: 0 });
    }

    if encoded.len() <= 14 {
        return Err(DecodeError {
            loc: encoded.len() - 1,
        });
    }

    let mut img = Image {
        pixels: Vec::new(),
        width: u32::from_be_bytes(encoded[4..8].try_into().unwrap()),
        height: u32::from_be_bytes(encoded[8..12].try_into().unwrap()),
        channels: match encoded[12] {
            3 => Channels::Rgb,
            4 => Channels::Rgba,

            _ => return Err(DecodeError { loc: 12 }),
        },
        colorspace: match encoded[13] {
            0 => ColorSpace::LinearAlpha,
            1 => ColorSpace::AllLinear,

            _ => return Err(DecodeError { loc: 13 }),
        },
    };

    let encoded = &encoded[14..];

    decode_pixels(encoded, &mut img.pixels)?;

    Ok(img)
}

fn decode_pixels(mut encoded: &[u8], res: &mut Vec<Pixel>) -> Result<(), DecodeError> {
    let mut loc = 14;
    let mut prev = Pixel(0, 0, 0, 255);
    let mut arr = [Pixel(0, 0, 0, 0); 64];

    while encoded != FOOTER && encoded.len() > FOOTER.len() {
        let (s, last) = match decode_pixel(&arr, prev, encoded, res) {
            Ok(x) => x,
            _ => return Err(DecodeError { loc }),
        };

        encoded = &encoded[s..];
        loc += s;

        arr[hash(last)] = last;
        prev = last;
    }

    if encoded != FOOTER {
        Err(DecodeError { loc })
    } else {
        Ok(())
    }
}

fn decode_pixel(
    arr: &[Pixel; 64],
    prev: Pixel,
    i: &[u8],
    v: &mut Vec<Pixel>,
) -> Result<(usize, Pixel), ()> {
    if i[0] == 255 {
        if i.len() < 5 {
            return Err(());
        }

        let res = Pixel(i[1], i[2], i[3], i[4]);

        v.push(res);

        Ok((5, res))
    } else if i[0] == 254 {
        if i.len() < 4 {
            return Err(());
        }

        let res = Pixel(i[1], i[2], i[3], prev.3);

        v.push(res);

        Ok((4, res))
    } else {
        let tag = i[0] >> 6;
        let f = [decode_idx, decode_diff, decode_luma, decode_run];

        Ok((f[usize::from(tag)])(arr, prev, i, v))
    }
}

fn decode_idx(arr: &[Pixel; 64], _prev: Pixel, i: &[u8], v: &mut Vec<Pixel>) -> (usize, Pixel) {
    let idx = i[0] & 0x3f;
    let res = arr[idx as usize];

    v.push(res);

    (1, res)
}

fn decode_diff(_arr: &[Pixel; 64], prev: Pixel, i: &[u8], v: &mut Vec<Pixel>) -> (usize, Pixel) {
    let curr = i[0] & 0x3f;

    let res = Pixel(
        wsub(wadd(prev.0, curr >> 4), 2),
        wsub(wadd(prev.1, (curr >> 2) & 3), 2),
        wsub(wadd(prev.2, curr & 3), 2),
        prev.3,
    );

    v.push(res);

    (1, res)
}

fn decode_luma(_arr: &[Pixel; 64], prev: Pixel, i: &[u8], v: &mut Vec<Pixel>) -> (usize, Pixel) {
    let curr = &i[..2];
    let dg = wsub(curr[0] & 0x3f, 32);
    let dr = wsub(curr[1] >> 4, 8);
    let db = wsub(curr[1] & 0xf, 8);

    let dr = wadd(dr, dg);
    let db = wadd(db, dg);

    let res = Pixel(wadd(prev.0, dr), wadd(prev.1, dg), wadd(prev.2, db), prev.3);

    v.push(res);

    (2, res)
}

fn decode_run(_arr: &[Pixel; 64], prev: Pixel, i: &[u8], v: &mut Vec<Pixel>) -> (usize, Pixel) {
    for _ in 0..(i[0] & 0x3f) + 1 {
        v.push(prev);
    }

    (1, prev)
}
