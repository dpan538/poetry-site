use image::{Rgb, RgbImage};

const BAYER: [[f32; 4]; 4] = [
    [0., 8., 2., 10.],
    [12., 4., 14., 6.],
    [3., 11., 1., 9.],
    [15., 7., 13., 5.],
];

fn spread(bucket: &[[u8; 3]]) -> (u8, usize) {
    (0..3)
        .map(|channel| {
            let (lo, hi) = bucket.iter().fold((u8::MAX, 0), |(lo, hi), color| {
                (lo.min(color[channel]), hi.max(color[channel]))
            });
            (hi - lo, channel)
        })
        .max()
        .unwrap()
}

pub fn reduce(grid: &mut RgbImage, limit: usize) {
    let mut buckets = vec![grid.pixels().map(|pixel| pixel.0).collect::<Vec<_>>()];
    while buckets.len() < limit {
        let Some((index, (range, axis))) = buckets
            .iter()
            .enumerate()
            .filter(|(_, bucket)| bucket.len() > 1)
            .map(|(index, bucket)| (index, spread(bucket)))
            .max_by_key(|&(index, (range, _))| usize::from(range) * buckets[index].len().isqrt())
        else {
            break;
        };
        if range == 0 {
            break;
        }
        let bucket = &mut buckets[index];
        bucket.sort_unstable_by_key(|color| color[axis]);
        let right = bucket.split_off(bucket.len() / 2);
        buckets.push(right);
    }
    let palette: Vec<[u8; 3]> = buckets
        .iter()
        .map(|bucket| {
            std::array::from_fn(|channel| {
                (bucket
                    .iter()
                    .map(|color| usize::from(color[channel]))
                    .sum::<usize>()
                    / bucket.len()) as u8
            })
        })
        .collect();
    for (x, y, pixel) in grid.enumerate_pixels_mut() {
        let distance = |color: [u8; 3]| -> u32 {
            (0..3)
                .map(|i| {
                    let distance = i32::from(color[i]) - i32::from(pixel[i]);
                    (distance * distance) as u32 * [3, 5, 2][i]
                })
                .sum()
        };
        let mut nearest = (u32::MAX, 0);
        let mut next = nearest;
        for (index, &color) in palette.iter().enumerate() {
            let score = (distance(color), index);
            if score < nearest {
                next = nearest;
                nearest = score;
            } else if score < next {
                next = score;
            }
        }
        let dither = (BAYER[y as usize % 4][x as usize % 4] + 0.5) / 16.0;
        let fraction = nearest.0 as f32 / (nearest.0 as f32 + next.0 as f32).max(1.0);
        *pixel = Rgb(palette[if dither < fraction * 0.80 {
            next.1
        } else {
            nearest.1
        }]);
    }
}
