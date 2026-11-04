import { afterEach, describe, expect, it, vi } from 'vitest';
// @ts-expect-error The plain-JavaScript image controller has no declaration file.
import { decodeImageFile, isImageFile, readImageFiles, IMAGE_LABEL } from '../../assets/js/image_input.js';
// @ts-expect-error The plain-JavaScript API has no declaration file.
import { decomposeNativeImage } from '../../assets/js/studio_api.js';
// @ts-expect-error The plain-JavaScript composer controller has no declaration file.
import { readTextFiles } from '../../assets/js/composer_input.js';

function canvasFixture(width = 256, height = 128) {
    const context = {
        clearRect: vi.fn(),
        drawImage: vi.fn(),
        getImageData: vi.fn(() => ({ data: new Uint8ClampedArray(width * height * 4), width, height })),
    };
    const canvas = {
        width: 0,
        height: 0,
        getContext: vi.fn(() => context),
        toDataURL: vi.fn(() => 'data:image/png;base64,preview'),
    };
    const original = document.createElement.bind(document);
    vi.spyOn(document, 'createElement').mockImplementation((tag: string) => tag === 'canvas' ? canvas as unknown as HTMLCanvasElement : original(tag));
    return { canvas, context };
}

afterEach(() => {
    vi.restoreAllMocks();
    vi.unstubAllGlobals();
});

describe('image file decoding and composer intake', () => {
    it('downsamples proportionally, bounds RGBA, and closes the bitmap', async () => {
        const { context } = canvasFixture();
        const bitmap = { width: 512, height: 256, close: vi.fn() };
        vi.stubGlobal('createImageBitmap', vi.fn().mockResolvedValue(bitmap));
        const file = new File([new Uint8Array(4)], 'photo.png', { type: 'image/png' });
        const image = await decodeImageFile(file);
        expect(image.width).toBe(256);
        expect(image.height).toBe(128);
        expect(image.rgba).toHaveLength(256 * 128 * 4);
        expect(image.dataUrl).toBe('data:image/png;base64,preview');
        expect(context.drawImage).toHaveBeenCalledWith(bitmap, 0, 0, 256, 128);
        expect(bitmap.close).toHaveBeenCalledOnce();
        expect(IMAGE_LABEL).toBe('R4 patch input — experimental; not visual understanding');
    });

    it('does not upscale tiny images and mixes text with images without sending', async () => {
        canvasFixture(1, 1);
        const bitmap = { width: 1, height: 1, close: vi.fn() };
        vi.stubGlobal('createImageBitmap', vi.fn().mockResolvedValue(bitmap));
        const text = await readTextFiles([new File(['hello'], 'note.txt', { type: 'text/plain' })]);
        const images = await readImageFiles([new File([new Uint8Array(4)], 'pixel.png', { type: 'image/png' })]);
        const result = { attachments: [...text.attachments, ...images.attachments], errors: [...text.errors, ...images.errors] };
        expect(result.errors).toEqual([]);
        expect(result.attachments).toHaveLength(2);
        expect(result.attachments[0].kind).toBe('text');
        expect(result.attachments[1].kind).toBe('image');
        expect(result.attachments[1].grid).toEqual({ columns: 1, rows: 1 });
        expect(result.attachments[1].rgba).toHaveLength(4);
        expect(isImageFile(new File(['x'], 'x.svg', { type: 'image/svg+xml' }))).toBe(false);
    });

    it('uses up to a 2x2 grid for larger and odd-dimension images', async () => {
        canvasFixture();
        vi.stubGlobal('createImageBitmap', vi.fn().mockResolvedValue({ width: 512, height: 256, close: vi.fn() }));
        const large = await readImageFiles([new File([new Uint8Array(4)], 'large.png', { type: 'image/png' })]);
        expect(large.errors).toEqual([]);
        expect(large.attachments[0].grid).toEqual({ columns: 2, rows: 2 });
        canvasFixture(3, 3);
        vi.stubGlobal('createImageBitmap', vi.fn().mockResolvedValue({ width: 3, height: 3, close: vi.fn() }));
        const odd = await readImageFiles([new File([new Uint8Array(4)], 'odd.png', { type: 'image/png' })]);
        expect(odd.errors).toEqual([]);
        expect(odd.attachments[0].grid).toEqual({ columns: 2, rows: 2 });
        canvasFixture(1, 3);
        vi.stubGlobal('createImageBitmap', vi.fn().mockResolvedValue({ width: 1, height: 3, close: vi.fn() }));
        const tall = await readImageFiles([new File([new Uint8Array(4)], 'tall.png', { type: 'image/png' })]);
        expect(tall.errors).toEqual([]);
        expect(tall.attachments[0].grid).toEqual({ columns: 1, rows: 2 });
        canvasFixture(3, 1);
        vi.stubGlobal('createImageBitmap', vi.fn().mockResolvedValue({ width: 3, height: 1, close: vi.fn() }));
        const wide = await readImageFiles([new File([new Uint8Array(4)], 'wide.png', { type: 'image/png' })]);
        expect(wide.errors).toEqual([]);
        expect(wide.attachments[0].grid).toEqual({ columns: 2, rows: 1 });
    });

    it('rejects huge decoded source dimensions before drawing or canvas allocation', async () => {
        const { context, canvas } = canvasFixture(1, 1);
        const bitmap = { width: 10000, height: 10000, close: vi.fn() };
        vi.stubGlobal('createImageBitmap', vi.fn().mockResolvedValue(bitmap));
        await expect(decodeImageFile(new File([new Uint8Array(4)], 'huge-dimensions.png', { type: 'image/png' }))).rejects.toThrow('source dimensions');
        expect(context.drawImage).not.toHaveBeenCalled();
        expect(canvas.getContext).not.toHaveBeenCalled();
        expect(bitmap.close).toHaveBeenCalledOnce();
    });

    it('refuses oversized input and revokes fallback object URLs on decode failure', async () => {
        const oversized = await readImageFiles([{ name: 'huge.png', type: 'image/png', size: 4 * 1024 * 1024 + 1 }], { maxFileBytes: 4 * 1024 * 1024 });
        expect(oversized.attachments).toEqual([]);
        expect(oversized.errors.join(' ')).toContain('4194304-byte image limit');
        vi.stubGlobal('createImageBitmap', undefined);
        vi.stubGlobal('Image', class {
            onload: (() => void) | null = null;
            onerror: ((error: Error) => void) | null = null;
            set src(_value: string) { this.onerror?.(new Error('broken image')); }
        });
        const create = vi.fn(() => 'blob:fixture');
        const revoke = vi.fn();
        vi.stubGlobal('URL', Object.assign(URL, { createObjectURL: create, revokeObjectURL: revoke }));
        await expect(decodeImageFile(new File(['bad'], 'bad.png', { type: 'image/png' }))).rejects.toThrow('The browser could not decode this image.');
        expect(revoke).toHaveBeenCalledWith('blob:fixture');
    });
});

describe('native image decomposition transport', () => {
    it('posts exact RGBA query/body and validates canonical bytes and issued headers', async () => {
        const canonical = new TextEncoder().encode('R4IMG001fixture');
        const headers = {
            'X-UOR-Image-Schema': 'uor-r4.image-decomposition/1',
            'X-UOR-Source-Blake3': 'a'.repeat(64),
            'X-UOR-Image-Width': '1',
            'X-UOR-Image-Height': '1',
            'X-UOR-Grid-Columns': '1',
            'X-UOR-Grid-Rows': '1',
            'X-UOR-Patch-Count': '1',
            'X-UOR-Decomposition-Id': 'r4img-test',
        };
        const fetchMock = vi.fn().mockResolvedValue(new Response(canonical.buffer as ArrayBuffer, { status: 200, headers }));
        vi.stubGlobal('fetch', fetchMock);
        const rgba = new Uint8Array([12, 20, 28, 255]);
        const result = await decomposeNativeImage({
            config: { baseUrl: 'http://127.0.0.1:8765/v1/', apiKey: 'secret' },
            image: { rgba, width: 1, height: 1 },
            grid: { columns: 1, rows: 1 },
        });
        expect(result).toEqual({
            type: 'uor_r4_image',
            schema: 'uor-r4.image-decomposition/1',
            canonical_hex: '5234494d4730303166697874757265',
            source_blake3: 'a'.repeat(64),
            width: 1,
            height: 1,
            grid: { columns: 1, rows: 1 },
            decomposition_id: 'r4img-test',
        });
        const [url, init] = fetchMock.mock.calls[0];
        expect(url).toBe('http://127.0.0.1:8765/v1/images/decompose?width=1&height=1&columns=1&rows=1');
        expect(init.method).toBe('POST');
        expect(init.headers).toMatchObject({
            'Content-Type': 'application/octet-stream',
            Accept: 'application/octet-stream',
            Authorization: 'Bearer secret',
        });
        expect(init.headers).not.toHaveProperty('X-UOR-Width');
        expect(init.body).toBe(rgba);
    });

    it('surfaces native error envelopes and rejects invalid success responses', async () => {
        const input = { config: { baseUrl: 'http://localhost/v1' }, image: { rgba: new Uint8Array(4), width: 1, height: 1 }, grid: { columns: 1, rows: 1 } };
        vi.stubGlobal('fetch', vi.fn().mockResolvedValue(new Response(JSON.stringify({ error: { message: 'dimension refused' } }), { status: 400 })));
        await expect(decomposeNativeImage(input)).rejects.toThrow('dimension refused');
        vi.stubGlobal('fetch', vi.fn().mockResolvedValue(new Response(new Uint8Array([0]), { status: 200 })));
        await expect(decomposeNativeImage(input)).rejects.toThrow('invalid decomposition');
    });
});
