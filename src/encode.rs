use crate::*;

pub fn encode(img: &Image) -> Vec<u8> {
    assert_eq!(
        img.pixels.len() % img.channels as u8 as usize,
        0,
        "the number of bytes of the image must be evenly divisible by the number of channels"
    );

    let mut res = Vec::new();
    header(img, &mut res);

    match img.channels {
        Channels::Rgb => encode_pixels::<3>(&img.pixels, &mut res),
        Channels::Rgba => encode_pixels::<4>(&img.pixels, &mut res),
    }

    res.extend(FOOTER);

    res
}

fn header(pic: &Image, res: &mut Vec<u8>) {
    res.extend(b"qoif");
    res.extend(pic.width.to_be_bytes());
    res.extend(pic.height.to_be_bytes());

    res.push(pic.channels as u8);
    res.push(pic.colorspace as u8);
}

fn encode_pixels<const SIZE: usize>(mut i: &[u8], v: &mut Vec<u8>) {
    let mut prev = [0; SIZE];

    if SIZE == 4 {
        prev[3] = 255;
    }

    let mut arr = [[0; SIZE]; 64];

    while !i.is_empty() {
        let curr: [u8; SIZE] = i[..SIZE].try_into().unwrap();

        if SIZE == 3 || curr[3] == prev[3] {
            if let Some((len, b)) = rle(&prev, i) {
                i = &i[len * SIZE..];
                v.push(b);
            } else if let Some(b) = index(&arr, i) {
                i = &i[SIZE..];
                v.push(b);
            } else if let Some(b) = diff(&prev, i) {
                i = &i[SIZE..];
                v.push(b);
            } else if let Some(b) = luma(&prev, i) {
                i = &i[SIZE..];
                v.extend(b);
            } else {
                full::<SIZE>(&prev, i, v);
            }
        } else {
            full::<SIZE>(&prev, i, v);
        }

        let hash = hash::<SIZE>(&curr);
        arr[hash] = curr;
        prev = curr;
    }
}

fn rle<const SIZE: usize>(prev: &[u8; SIZE], mut i: &[u8]) -> Option<(usize, u8)> {
    let mut len = 0;

    while len < 63 && !i.is_empty() && &i[..SIZE] == prev {
        len += 1;
        i = &i[SIZE..];
    }

    if len > 0 {
        Some((len * SIZE, (0b11 << 6) | (len as u8 - 1)))
    } else {
        None
    }
}

fn index<const SIZE: usize>(arr: &[[u8; SIZE]; 64], curr: &[u8]) -> Option<u8> {
    let hash = hash::<SIZE>(curr);
    if arr[hash] == curr[..SIZE] {
        Some(0b00 | hash as u8)
    } else {
        None
    }
}

fn diff<const SIZE: usize>(prev: &[u8; SIZE], curr: &[u8]) -> Option<u8> {
    if SIZE == 4 && prev[3] != curr[3] {
        return None;
    }

    let mut res = 0b01 << 6;

    for i in 0..3 {
        let diff = curr[i].wrapping_sub(prev[i]).wrapping_add(2);

        if diff > 3 {
            return None;
        }

        res |= diff << (i * 2);
    }

    Some(res)
}

fn luma<const SIZE: usize>(prev: &[u8; SIZE], curr: &[u8]) -> Option<[u8; 2]> {
    if SIZE == 4 && prev[3] != curr[3] {
        return None;
    }

    let dg = curr[1].wrapping_sub(prev[1]);
    let dr = curr[0].wrapping_sub(prev[0]).wrapping_sub(dg);
    let db = curr[2].wrapping_sub(prev[2]).wrapping_sub(dg);

    let dg = dg.wrapping_add(32);
    let dr = dr.wrapping_add(8);
    let db = db.wrapping_add(8);

    if dg > 63 || dr > 15 || db > 15 {
        return None;
    }

    Some([(0b10 << 6) | dg, (dr << 4) | db])
}

fn full<const SIZE: usize>(prev: &[u8; SIZE], i: &[u8], v: &mut Vec<u8>) {
    let (len, tag) = if SIZE == 4 && prev[3] != i[3] {
        (4, 0xff)
    } else {
        (3, 0xfe)
    };

    v.push(tag);

    i[..len].iter().for_each(|i| v.push(*i));
}
