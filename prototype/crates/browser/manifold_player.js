const active = new WeakMap();
const moduleRetries = new Map();
let paintingModule;

async function importScene(path) {
    for (let attempt = 0; ; attempt++) {
        const retry = moduleRetries.get(path) ?? 0;
        try {
            return await import(retry ? `${path}?retry=${retry}` : path);
        } catch (error) {
            moduleRetries.set(path, retry + 1);
            if (attempt === 1) throw error;
            await new Promise(resolve => setTimeout(resolve, 180));
        }
    }
}

function loadPaintings() {
    return paintingModule ??= importScene("/assets/manifold/paintings.js").then(async (paintings) => {
        const path = "/assets/manifold/paintings_bg.wasm";
        let response = await fetch(path).catch(() => null);
        if (!response?.ok) {
            await new Promise(resolve => setTimeout(resolve, 180));
            response = await fetch(path, { cache: "reload" });
        }
        if (!response.ok) throw new Error("Painting renderer unavailable");
        await paintings.default({ module_or_path: response });
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
            importScene("/assets/manifold/mount.js"),
            importScene(`/assets/manifold/pieces/${dialog.dataset.piece}.js`),
        ]);
        if (!dialog.open || active.get(dialog) !== playback) return;

        const { cols, rows, cell = 2 } = piece.meta;
        const canvas = dialog.querySelector("canvas");
        canvas.parentElement.style.setProperty("--scene-ratio", `${cols} / ${rows * cell}`);
        playback.unmount = player.mount(canvas, piece);
        dialog.setAttribute("data-scene-ready", "");
    } catch (error) {
        if (dialog.open && active.get(dialog) === playback) {
            dialog.setAttribute("data-scene-error", "");
            dialog.removeAttribute("data-scene-ready");
            dialog.querySelector("[data-manifold-playback]").hidden = true;
            console.warn("Could not load the Manifold scene", error);
        }
    }
}
