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

    match img.channels {
        Channels::Rgb => decode_pixels::<3>(encoded, &mut img.pixels)?,
        Channels::Rgba => decode_pixels::<4>(encoded, &mut img.pixels)?,
    }

    Ok(img)
}

fn decode_pixels<const SIZE: usize>(
    mut encoded: &[u8],
    res: &mut Vec<u8>,
) -> Result<(), DecodeError> {
    let mut loc = 14;
    let mut prev = [0; SIZE];
    let mut arr = [[0; SIZE]; 64];

    while encoded != FOOTER && encoded.len() > FOOTER.len() {
        let (s, last) = match decode_pixel(&arr, &prev, encoded, res) {
            Ok(x) => x,
            _ => return Err(DecodeError { loc }),
        };

        encoded = &encoded[s..];
        loc += s;

        arr[hash::<SIZE>(&last)] = last;
        prev = last;
    }

    if encoded != FOOTER {
        Err(DecodeError { loc })
    } else {
        Ok(())
    }
}

fn decode_pixel<const SIZE: usize>(
    arr: &[[u8; SIZE]; 64],
    prev: &[u8; SIZE],
    i: &[u8],
    v: &mut Vec<u8>,
) -> Result<(usize, [u8; SIZE]), ()> {
    if i[0] == 255 {
        if SIZE != 4 || i.len() < 5 {
            return Err(());
        }

        let res = i[1..5].try_into().unwrap();

        v.extend(res);

        Ok((5, res))
    } else if i[0] == 254 {
        let mut res = [0; SIZE];

        if i.len() < 4 {
            return Err(());
        }

        res[0] = i[1];
        res[1] = i[2];
        res[2] = i[3];

        if SIZE == 4 {
            res[3] = prev[3];
        }

        v.extend(res);

        Ok((4, res))
    } else {
        let tag = i[0] >> 6;
        let f = [decode_idx, decode_diff, decode_luma, decode_run];

        Ok((f[usize::from(tag)])(arr, prev, i, v))
    }
}

fn decode_idx<const SIZE: usize>(
    arr: &[[u8; SIZE]; 64],
    _prev: &[u8; SIZE],
    i: &[u8],
    v: &mut Vec<u8>,
) -> (usize, [u8; SIZE]) {
    let idx = i[0] & 0x3f;
    let res = arr[idx as usize];

    v.extend(res);

    (1, res)
}

fn decode_diff<const SIZE: usize>(
    _arr: &[[u8; SIZE]; 64],
    prev: &[u8; SIZE],
    i: &[u8],
    v: &mut Vec<u8>,
) -> (usize, [u8; SIZE]) {
    let mut res = [0; SIZE];
    let curr = i[0] & 0x3f;

    if SIZE == 4 {
        res[3] = prev[3];
    }

    res[0] = wsub(wadd(prev[0], curr >> 4), 2);
    res[1] = wsub(wadd(prev[1], (curr >> 2) & 3), 2);
    res[2] = wsub(wadd(prev[2], curr & 3), 2);

    v.extend(res);

    (1, res)
}

fn decode_luma<const SIZE: usize>(
    _arr: &[[u8; SIZE]; 64],
    prev: &[u8; SIZE],
    i: &[u8],
    v: &mut Vec<u8>,
) -> (usize, [u8; SIZE]) {
    let curr = &i[..2];
    let dg = wsub(curr[0] & 0x3f, 32);
    let dr = wsub(curr[1] >> 4, 8);
    let db = wsub(curr[1] & 0xf, 8);

    let dr = wadd(dr, dg);
    let db = wadd(db, dg);

    let mut res = [0; SIZE];
    if SIZE == 4 {
        res[3] = prev[3];
    }

    res[0] = wadd(prev[0], dr);
    res[1] = wadd(prev[1], dg);
    res[2] = wadd(prev[2], db);

    v.extend(res);

    (2, res)
}

fn decode_run<const SIZE: usize>(
    _arr: &[[u8; SIZE]; 64],
    prev: &[u8; SIZE],
    i: &[u8],
    v: &mut Vec<u8>,
) -> (usize, [u8; SIZE]) {
    for _ in 0..(i[0] & 0x3f) + 1 {
        v.extend(prev);
    }

    (1, *prev)
}
