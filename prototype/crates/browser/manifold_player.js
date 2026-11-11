const active = new WeakMap();
const moduleRetries = new Map();
let paintingModule;
let originalModule;
let atmosphereModule;

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

function loadOriginals() {
    return originalModule ??= importScene("/assets/manifold/originals.js").then(async (originals) => {
        const path = "/assets/manifold/originals_bg.wasm";
        let response = await fetch(path).catch(() => null);
        if (!response?.ok) {
            await new Promise(resolve => setTimeout(resolve, 180));
            response = await fetch(path, { cache: "reload" });
        }
        if (!response.ok) throw new Error("Original renderer unavailable");
        await originals.default({ module_or_path: response });
        return originals;
    }).catch((error) => {
        originalModule = undefined;
        throw error;
    });
}

function loadAtmospheres() {
    return atmosphereModule ??= importScene("/assets/manifold/atmospheres.js").then(async (module) => {
        const path = "/assets/manifold/atmospheres_bg.wasm";
        let response = await fetch(path).catch(() => null);
        if (!response?.ok) {
            await new Promise(resolve => setTimeout(resolve, 180));
            response = await fetch(path, { cache: "reload" });
        }
        if (!response.ok) throw new Error("Atmospheric renderer unavailable");
        await module.default({ module_or_path: response });
        return module;
    }).catch(error => {
        atmosphereModule = undefined;
        throw error;
    });
}

function mountAtmosphere(module, dialog) {
    const canvas = dialog.querySelector("canvas");
    const reduced = matchMedia("(prefers-reduced-motion: reduce)");
    let animation = module.mount_atmosphere(canvas, dialog.dataset.piece);
    let width = canvas.clientWidth, dpr = devicePixelRatio, resizeTask;
    const resize = new ResizeObserver(() => {
        if (Math.abs(canvas.clientWidth - width) < 0.5 && devicePixelRatio === dpr) return;
        clearTimeout(resizeTask);
        resizeTask = setTimeout(() => {
            const time = animation.elapsed();
            animation.free();
            animation = module.mount_atmosphere(canvas, dialog.dataset.piece);
            animation.seek(time);
            if (document.hidden || reduced.matches) animation.pause();
            width = canvas.clientWidth;
            dpr = devicePixelRatio;
        }, 100);
    });
    const visibility = () => document.hidden || reduced.matches ? animation.pause() : animation.play();
    resize.observe(canvas.parentElement);
    document.addEventListener("visibilitychange", visibility);
    visibility();
    return () => {
        clearTimeout(resizeTask);
        resize.disconnect();
        document.removeEventListener("visibilitychange", visibility);
        animation.free();
    };
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
        if (dialog.hasAttribute("data-atmosphere")) {
            const module = await loadAtmospheres();
            if (!dialog.open || active.get(dialog) !== playback) return;
            playback.unmount = mountAtmosphere(module, dialog);
            dialog.setAttribute("data-scene-ready", "");
            return;
        }
        if (dialog.hasAttribute("data-original")) {
            const originals = await loadOriginals();
            if (!dialog.open || active.get(dialog) !== playback) return;

            const animation = originals.mount_original(dialog.querySelector("canvas"), dialog.dataset.piece);
            playback.unmount = () => animation.free();
            dialog.setAttribute("data-scene-ready", "");
            return;
        }
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
