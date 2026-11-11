// Golden outputs evaluated from bundled ascii.rest Kyoto Dusk, W=200.
// Copyright (c) 2026 bas3line; MIT notice retained in ../ascii.rest-LICENSE.txt.
struct ReferenceCase { name: &'static str, rgb: Color, floor: f32, fade: f32, glyphs: [u8; 16], inks: [Color; 4] }
const REFERENCE_CASES: &[ReferenceCase] = &[
ReferenceCase { name: "black", rgb: [0.0, 0.0, 0.0], floor: 0.0, fade: 1.0, glyphs: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], inks: [[0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0]] },
ReferenceCase { name: "charcoal", rgb: [0.12, 0.12, 0.12], floor: 0.02, fade: 1.0, glyphs: [0, 0, 0, 0, 1, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0], inks: [[0.3, 0.3, 0.3], [0.707249876, 0.707249876, 0.707249876], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0]] },
ReferenceCase { name: "middle_grey", rgb: [0.3, 0.3, 0.3], floor: 0.04, fade: 1.0, glyphs: [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1], inks: [[0.0, 0.0, 0.0], [1.0, 1.0, 1.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0]] },
ReferenceCase { name: "light_grey", rgb: [0.7, 0.7, 0.7], floor: 0.04, fade: 1.0, glyphs: [2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2], inks: [[0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [1.0, 1.0, 1.0], [0.0, 0.0, 0.0]] },
ReferenceCase { name: "white", rgb: [1.0, 1.0, 1.0], floor: 0.04, fade: 1.0, glyphs: [3, 3, 2, 3, 3, 2, 3, 3, 3, 3, 2, 3, 3, 3, 3, 3], inks: [[0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [1.0, 1.0, 1.0], [0.9864242876, 0.9864242876, 0.9864242876]] },
ReferenceCase { name: "blue", rgb: [0.1, 0.2, 0.6], floor: 0.04, fade: 1.0, glyphs: [2, 2, 1, 2, 2, 1, 2, 2, 2, 2, 1, 2, 2, 2, 2, 2], inks: [[0.0, 0.0, 0.0], [0.1633986928, 0.3333333333, 1.0], [0.1633986928, 0.3333333333, 1.0], [0.0, 0.0, 0.0]] },
ReferenceCase { name: "flame", rgb: [1.0, 0.6, 0.2], floor: 0.35, fade: 1.0, glyphs: [3, 3, 2, 3, 3, 3, 3, 3, 3, 3, 2, 3, 3, 3, 3, 3], inks: [[0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [1.0, 0.6581898723, 0.2193966241], [1.0, 0.6581898723, 0.2193966241]] },
ReferenceCase { name: "half_fade_grey", rgb: [0.7, 0.7, 0.7], floor: 0.04, fade: 0.5, glyphs: [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1], inks: [[0.0, 0.0, 0.0], [1.0, 1.0, 1.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0]] },
];
