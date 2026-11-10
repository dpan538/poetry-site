const active = new WeakMap();
let paintingModule;

function loadPaintings() {
    return paintingModule ??= import("/assets/manifold/paintings.js").then(async (paintings) => {
        await paintings.default({ module_or_path: "/assets/manifold/paintings_bg.wasm" });
        return paintings;
    }).catch((error) => {
        paintingModule = undefined;
        throw error;
    });
}

export function stopManifold(dialog) {
    const playback = active.get(dialog);
    active.delete(dialog);
    playback?.unmount?.();
}

export async function playManifold(dialog) {
    stopManifold(dialog);
    const playback = {};
    active.set(dialog, playback);
    try {
        if (dialog.hasAttribute("data-painting")) {
            const paintings = await loadPaintings();
            if (!dialog.open || active.get(dialog) !== playback) return;

            const animation = paintings.mount_painting(dialog.querySelector("canvas"), dialog.dataset.piece);
            playback.unmount = () => animation.free();
            dialog.setAttribute("data-scene-ready", "");
            return;
        }
        const [player, piece] = await Promise.all([
            import("/assets/manifold/mount.js"),
            import(`/assets/manifold/pieces/${dialog.dataset.piece}.js`),
        ]);
        if (!dialog.open || active.get(dialog) !== playback) return;

        const { cols, rows, cell = 2 } = piece.meta;
        const canvas = dialog.querySelector("canvas");
        canvas.parentElement.style.setProperty("--scene-ratio", `${cols} / ${rows * cell}`);
        playback.unmount = player.mount(canvas, piece);
        dialog.setAttribute("data-scene-ready", "");
    } catch (error) {
        if (active.get(dialog) === playback) {
            console.error("Could not load the Manifold scene", error);
        }
    }
}
