export const IMAGE_LABEL = 'R4 patch input — experimental; not visual understanding';
const MAX_IMAGE_DIMENSION = 256;
const MAX_SOURCE_DIMENSION = 4096;
const DEFAULT_MAX_IMAGE_BYTES = 8 * 1024 * 1024;
const DEFAULT_MAX_TOTAL_IMAGE_BYTES = 16 * 1024 * 1024;
const IMAGE_MIME_TYPES = new Set([
    'image/avif',
    'image/bmp',
    'image/gif',
    'image/jpeg',
    'image/png',
    'image/webp',
]);
const IMAGE_EXTENSIONS = new Set(['png', 'jpg', 'jpeg', 'webp', 'gif', 'bmp', 'avif']);

function extension(name) {
    const parts = String(name || '').toLowerCase().split('.');
    return parts.length > 1 ? parts[parts.length - 1] : '';
}

function positiveLimit(value, fallback) {
    return Number.isFinite(Number(value)) && Number(value) > 0 ? Math.floor(Number(value)) : fallback;
}

export function isImageFile(file) {
    const type = String(file && file.type || '').toLowerCase();
    if (type) return IMAGE_MIME_TYPES.has(type);
    return IMAGE_EXTENSIONS.has(extension(file && file.name));
}

export function downsampleDimensions(width, height, maxDimension = MAX_IMAGE_DIMENSION) {
    const sourceWidth = Number(width);
    const sourceHeight = Number(height);
    const bound = positiveLimit(maxDimension, MAX_IMAGE_DIMENSION);
    if (!Number.isInteger(sourceWidth) || !Number.isInteger(sourceHeight) || sourceWidth <= 0 || sourceHeight <= 0) {
        throw new Error('Image dimensions must be positive integers.');
    }
    const scale = Math.min(1, bound / Math.max(sourceWidth, sourceHeight));
    return {
        width: Math.max(1, Math.round(sourceWidth * scale)),
        height: Math.max(1, Math.round(sourceHeight * scale)),
    };
}

async function loadImageElement(file) {
    if (typeof document === 'undefined' || typeof document.createElement !== 'function') {
        throw new Error('Image decoding requires a browser document.');
    }
    const url = URL.createObjectURL(file);
    try {
        const image = new Image();
        image.decoding = 'async';
        await new Promise((resolve, reject) => {
            image.onload = resolve;
            image.onerror = () => reject(new Error('The browser could not decode this image.'));
            image.src = url;
        });
        return { source: image, close: () => URL.revokeObjectURL(url) };
    } catch (error) {
        URL.revokeObjectURL(url);
        throw error;
    }
}

export async function decodeImageFile(file, options = {}) {
    const maxDimension = positiveLimit(options.maxDimension, MAX_IMAGE_DIMENSION);
    let decoded;
    if (typeof createImageBitmap === 'function') {
        decoded = { source: await createImageBitmap(file), close: null };
    } else {
        decoded = await loadImageElement(file);
    }
    try {
        const maxSourceDimension = positiveLimit(options.maxSourceDimension, MAX_SOURCE_DIMENSION);
        const sourceWidth = Number(decoded.source.naturalWidth || decoded.source.width);
        const sourceHeight = Number(decoded.source.naturalHeight || decoded.source.height);
        if (sourceWidth > maxSourceDimension || sourceHeight > maxSourceDimension) {
            throw new Error('Image source dimensions exceed the ' + maxSourceDimension + '-pixel per-side limit.');
        }
        const dimensions = downsampleDimensions(sourceWidth, sourceHeight, maxDimension);
        const canvas = document.createElement('canvas');
        canvas.width = dimensions.width;
        canvas.height = dimensions.height;
        const context = canvas.getContext('2d', { alpha: true, willReadFrequently: true });
        if (!context) throw new Error('Canvas 2D context is unavailable.');
        context.clearRect(0, 0, canvas.width, canvas.height);
        context.drawImage(decoded.source, 0, 0, canvas.width, canvas.height);
        const pixels = context.getImageData(0, 0, canvas.width, canvas.height);
        return {
            width: pixels.width,
            height: pixels.height,
            rgba: new Uint8Array(pixels.data),
            dataUrl: canvas.toDataURL('image/png'),
        };
    } finally {
        if (typeof decoded.source.close === 'function') decoded.source.close();
        if (typeof decoded.close === 'function') decoded.close();
    }
}

export async function readImageFiles(files, options = {}) {
    const maxFileBytes = positiveLimit(options.maxFileBytes, DEFAULT_MAX_IMAGE_BYTES);
    const maxTotalBytes = positiveLimit(options.maxTotalBytes, DEFAULT_MAX_TOTAL_IMAGE_BYTES);
    const maxDimension = positiveLimit(options.maxDimension, MAX_IMAGE_DIMENSION);
    const decodeImage = typeof options.decodeImage === 'function' ? options.decodeImage : decodeImageFile;
    const retained = new Map();
    for (const entry of options.existingAttachments || []) {
        if (!entry || entry.kind !== 'image') continue;
        retained.set(String(entry.name || 'image'), Number(entry.size) || 0);
    }
    let totalBytes = Array.from(retained.values()).reduce((sum, size) => sum + size, 0);
    const attachments = [];
    const errors = [];
    for (const file of Array.from(files || [])) {
        const name = String(file && file.name || 'image');
        const size = Number(file && file.size || 0);
        if (!isImageFile(file)) {
            errors.push(name + ' is not a supported image; it was not attached.');
            continue;
        }
        if (size > maxFileBytes) {
            errors.push(name + ' exceeds the ' + maxFileBytes + '-byte image limit (' + size + ' bytes).');
            continue;
        }
        const nextTotal = totalBytes - (retained.get(name) || 0) + size;
        if (nextTotal > maxTotalBytes) {
            errors.push(name + ' exceeds the total image limit (' + maxTotalBytes + ' bytes).');
            continue;
        }
        let image;
        try {
            image = await decodeImage(file, { maxDimension });
        } catch (error) {
            errors.push(name + ' could not be decoded: ' + String(error && error.message || error));
            continue;
        }
        const width = Number(image && image.width);
        const height = Number(image && image.height);
        const rgba = image && image.rgba;
        if (!Number.isInteger(width) || !Number.isInteger(height) || width <= 0 || height <= 0 || width > maxDimension || height > maxDimension) {
            errors.push(name + ' decoded to invalid or oversized dimensions.');
            continue;
        }
        if (!rgba || Number(rgba.length) !== width * height * 4) {
            errors.push(name + ' did not decode to exact RGBA bytes.');
            continue;
        }
        const dataUrl = typeof image.dataUrl === 'string' ? image.dataUrl : '';
        if (!dataUrl.startsWith('data:image/')) {
            errors.push(name + ' has no bounded image preview.');
            continue;
        }
        attachments.push({
            name,
            kind: 'image',
            content: '',
            source: 'file',
            size,
            width,
            height,
            rgba: rgba instanceof Uint8Array ? rgba : new Uint8Array(rgba),
            dataUrl,
            grid: { columns: Math.min(2, width), rows: Math.min(2, height) },
        });
        totalBytes = nextTotal;
        retained.set(name, size);
    }
    return { attachments, errors };
}
