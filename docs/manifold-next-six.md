# Five approved atmospheric originals

The current local review retains **01, 02, 04, 05 and 06** at [localhost:4381](http://localhost:4381/?review=library-v7). Study 03 is withdrawn. These five have been approved and assigned to the local poem pages. Together with the preceding four, they form nine new independent originals. This batch has not been deployed.

| Number | Work | Scene | Continuous motion |
| --- | --- | --- | --- |
| 01 | Going Home | Vermilion bridge landing on a stone quay beside the far house, with a connecting shore path. | Advected dusk clouds, water, current reflections and drifting water lamps. |
| 02 | The Observer | Warm candlelit dinner, cream linen and white plates; mostly black exterior with black-coated figures beyond the window. | Flame shape and illumination, cast shadows, glass reflections, rising steam and falling rain. |
| 04 | Silver Tipped Shoes | Ray-marched silver mountains, moon and lake. | Whole cloud fields translate; water reflects moving clouds and shades fixed terrain along mirrored camera rays. |
| 05 | Sleeping In | Bright blue sky, white/ivory arcade, grey recesses, water courtyard and empty boat. | Blue water with sparse drifting pink beads and local reflections, cloud advection, boat drift, wake and architectural reflections. |
| 06 | Iron Nails | One large foreground torii with grounded stone bases, a smaller shrine hall lower on the distant terrace, and depth-separated snow peaks; no foreground trees or freestanding lamps. | Independent cloud field; architecture and mountains remain fixed. |

This revision changes scene construction rather than applying another global contrast adjustment. It adapts terrain/normals/occlusion from Alpine Dawn, cloud-field advection from Alpine Dawn/Lake Pichola, architectural light relationships from Gangaur Ghat and current-frame reflections from Lake Pichola. Local MIT source attribution is retained.

The renderer uses actual upstream font glyph masks rather than synthetic circles. Neutral inks remain separate from scene-specific colour. 01/02/04/05 use 255 × 150 desktop and 136 × 80 mobile cells. The retained 06 stays at 204 × 120 and 119 × 70. All preserve 1.7:1 framing and native device-pixel cell boundaries.

The first-minute timeline does not reset animation: time continues beyond60seconds. Visual review remains necessary; unit and browser checks establish behaviour, not aesthetic approval. See [the study README](../prototype/crates/studies/README.md) for implementation details.
