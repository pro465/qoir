use crate::*;

pub fn encode(img: &Image) -> Vec<u8> {
    let mut res = Vec::new();
    header(img, &mut res);

    match img.channels {
        Channels::Rgb => encode_pixels::<3>(&img.pixels, &mut res),
        Channels::Rgba => encode_pixels::<4>(&img.pixels, &mut res),
    }

    res.extend(FOOTER);

    dbg!(res)
}

fn header(pic: &Image, res: &mut Vec<u8>) {
    res.extend(b"qoif");
    res.extend(pic.width.to_be_bytes());
    res.extend(pic.height.to_be_bytes());

    res.push(pic.channels as u8);
    res.push(pic.colorspace as u8);
}

fn encode_pixels<const SIZE: usize>(mut i: &[Pixel], v: &mut Vec<u8>) {
    let mut prev = Pixel(0, 0, 0, 255);

    let mut arr = [Pixel(0, 0, 0, 0); 64];

    while !i.is_empty() {
        let mut curr = i[0];
        if SIZE == 3 {
            curr.3 = 255
        }

        if let Some(b) = index(&arr, curr) {
            i = &i[1..];
            v.push(b);
        } else if curr.3 == prev.3 {
            if let Some((len, b)) = rle(prev, i) {
                i = &i[len..];
                v.push(b);
            } else if let Some(b) = diff(prev, curr) {
                i = &i[1..];
                v.push(b);
            } else if let Some(b) = luma(prev, curr) {
                i = &i[1..];
                v.extend(b);
            } else {
                full::<3>(curr, v);
                i = &i[1..];
            }
        } else {
            full::<4>(curr, v);
            i = &i[1..];
        }

        let hash = hash(curr);
        arr[hash] = curr;
        prev = curr;
    }
}

fn rle(prev: Pixel, i: &[Pixel]) -> Option<(usize, u8)> {
    let mut len = 0;

    while len < i.len().min(63) && i[len] == prev {
        len += 1;
    }

    if len > 0 {
        Some((len, (0b11 << 6) | (len as u8 - 1)))
    } else {
        None
    }
}

fn index(arr: &[Pixel; 64], curr: Pixel) -> Option<u8> {
    let hash = hash(curr);
    if arr[hash] == curr {
        Some(0b00 | hash as u8)
    } else {
        None
    }
}

fn diff(prev: Pixel, curr: Pixel) -> Option<u8> {
    let mut res = 0b01 << 6;

    let mut f = |d, i| {
        let diff = wadd(d, 2);

        if diff > 3 {
            return None;
        }

        res |= diff << (4 - i * 2);
        Some(())
    };

    f(wsub(curr.0, prev.0), 0)?;
    f(wsub(curr.1, prev.1), 1)?;
    f(wsub(curr.2, prev.2), 2)?;

    Some(res)
}

fn luma(prev: Pixel, curr: Pixel) -> Option<[u8; 2]> {
    let dg = wsub(curr.1, prev.1);
    let dr = wsub(wsub(curr.0, prev.0), dg);
    let db = wsub(wsub(curr.2, prev.2), dg);

    let dg = wadd(dg, 32);
    let dr = wadd(dr, 8);
    let db = wadd(db, 8);

    if dg > 63 || dr > 15 || db > 15 {
        return None;
    }

    Some([(0b10 << 6) | dg, (dr << 4) | db])
}

fn full<const SIZE: usize>(i: Pixel, v: &mut Vec<u8>) {
    let tag = 0xfb + SIZE as u8;

    let mut f = |i| v.push(i);

    f(tag);
    f(i.0);
    f(i.1);
    f(i.2);

    if SIZE == 4 {
        f(i.3);
    }
}
