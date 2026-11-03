import test from 'node:test';
import assert from 'node:assert/strict';
import {
    downsampleDimensions,
    isImageFile,
    readImageFiles,
} from './image_input.js';

function file(name, type, size = 8) {
    return { name, type, size };
}

function decoded(width, height) {
    return {
        width,
        height,
        rgba: new Uint8ClampedArray(width * height * 4).fill(127),
        dataUrl: 'data:image/png;base64,AA==',
    };
}

test('image dimensions preserve aspect ratio and never upscale', () => {
    assert.deepEqual(downsampleDimensions(1024, 512), { width: 256, height: 128 });
    assert.deepEqual(downsampleDimensions(1, 1000), { width: 1, height: 256 });
    assert.deepEqual(downsampleDimensions(100, 50), { width: 100, height: 50 });
    assert.throws(() => downsampleDimensions(0, 1), /positive/);
});

test('image MIME and extensions are bounded to raster formats', () => {
    assert.equal(isImageFile(file('photo.png', 'image/png')), true);
    assert.equal(isImageFile(file('photo', 'image/jpeg')), true);
    assert.equal(isImageFile(file('photo.webp', '')), true);
    assert.equal(isImageFile(file('diagram.svg', 'image/svg+xml')), false);
    assert.equal(isImageFile(file('notes.txt', 'text/plain')), false);
    assert.equal(isImageFile(file('photo.png', 'image/svg+xml')), false);
    assert.equal(isImageFile(file('photo.png', 'text/plain')), false);
    assert.equal(isImageFile(file('photo.heic', 'image/heic')), false);
});

test('image reading retains memory-only RGBA and bounded preview metadata', async () => {
    const result = await readImageFiles([file('photo.png', 'image/png', 12)], {
        decodeImage: async () => decoded(2, 1),
        maxDimension: 256,
    });
    assert.deepEqual(result.errors, []);
    assert.equal(result.attachments.length, 1);
    const image = result.attachments[0];
    assert.equal(image.kind, 'image');
    assert.equal(image.content, '');
    assert.equal(image.size, 12);
    assert.equal(image.width, 2);
    assert.equal(image.height, 1);
    assert.equal(image.rgba.length, 8);
    assert.equal(image.dataUrl, 'data:image/png;base64,AA==');
});

test('image failures are loud and leave no partial attachment', async () => {
    const oversized = await readImageFiles([file('huge.png', 'image/png', 20)], {
        maxFileBytes: 10,
        decodeImage: async () => decoded(1, 1),
    });
    assert.equal(oversized.attachments.length, 0);
    assert.match(oversized.errors[0], /huge\.png.*10-byte/);

    const malformed = await readImageFiles([file('bad.png', 'image/png')], {
        decodeImage: async () => ({ width: 2, height: 2, rgba: new Uint8Array(3), dataUrl: 'x' }),
    });
    assert.equal(malformed.attachments.length, 0);
    assert.match(malformed.errors[0], /RGBA/);

    const unsupported = await readImageFiles([file('notes.txt', 'text/plain')], {
        decodeImage: async () => decoded(1, 1),
    });
    assert.equal(unsupported.attachments.length, 0);
    assert.match(unsupported.errors[0], /not a supported image/);
});

test('duplicate image replacement counts retained bytes correctly', async () => {
    const existingAttachments = [{ name: 'photo.png', kind: 'image', size: 8 }];
    const result = await readImageFiles([file('photo.png', 'image/png', 8)], {
        existingAttachments,
        maxFileBytes: 8,
        maxTotalBytes: 8,
        decodeImage: async () => decoded(1, 1),
    });
    assert.deepEqual(result.errors, []);
    assert.equal(result.attachments.length, 1);
});
