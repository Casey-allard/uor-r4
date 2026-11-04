import { describe, expect, it } from 'vitest';
import source from '../../index.html?raw';

function factory(apiConfig: unknown, attachments: unknown[], recalledImages = 0) {
    const from = source.indexOf('function isNativeLoopback');
    const to = source.indexOf('async function sendViaApi', from);
    expect(from).toBeGreaterThan(0);
    expect(to).toBeGreaterThan(from);
    const create = new Function('app', 'window', 'decomposeNativeImage', 'imageUrlPart', `${source.slice(from, to)} return apiMessages;`);
    return create(
        { apiConfig, abortController: null },
        { __studioUi: { getAttachments: () => attachments, getImageRecallCount: () => recalledImages } },
        async ({ image }: { image: { decomposition: unknown } }) => image.decomposition,
        (image: { dataUrl: string }) => ({ type: 'image_url', image_url: { url: image.dataUrl } }),
    ) as (conversation: { messages: { role: string; content: string }[] }) => { role: string; content: unknown }[];
}

const canonicalHex = '5234494d4730303166697874757265';
const image = {
    name: 'pixel.png',
    kind: 'image',
    content: '',
    dataUrl: 'data:image/png;base64,preview',
    decomposition: {
        type: 'uor_r4_image',
        schema: 'uor-r4.image-decomposition/1',
        canonical_hex: canonicalHex,
        source_blake3: 'a'.repeat(64),
        width: 1,
        height: 1,
        grid: { columns: 1, rows: 1 },
        decomposition_id: 'r4img-test',
    },
};

describe('API image content parts', () => {
    it('appends only canonical R4 parts for native loopback', async () => {
        const apiMessages = factory({ id: 'native-loopback', ephemeral: true }, [image]);
        const messages = apiMessages({ messages: [{ role: 'user', content: 'Hi' }] });
        expect(await messages[0].content).toEqual([
            { type: 'text', text: 'Hi' },
            image.decomposition,
        ]);
    });

    it('uses ordinary image_url only for custom providers', async () => {
        const apiMessages = factory({ id: 'custom', ephemeral: false }, [image]);
        const messages = apiMessages({ messages: [{ role: 'user', content: 'Hi' }] });
        expect(await messages[0].content).toEqual([
            { type: 'text', text: 'Hi' },
            { type: 'image_url', image_url: { url: image.dataUrl } },
        ]);
    });

    it('uses ordinary image_url for a reserved ID without the ephemeral native flag', async () => {
        const apiMessages = factory({ id: 'native-loopback', ephemeral: false }, [image]);
        const messages = apiMessages({ messages: [{ role: 'user', content: 'Hi' }] });
        expect(await messages[0].content).toEqual([
            { type: 'text', text: 'Hi' },
            { type: 'image_url', image_url: { url: image.dataUrl } },
        ]);
    });

    it('allows text-only sends with recalled metadata and no current images', async () => {
        const apiMessages = factory({ id: 'native-loopback', ephemeral: true }, [], 1);
        const messages = apiMessages({ messages: [{ role: 'user', content: 'Hi' }] });
        expect(await messages[0].content).toBe('Hi');
    });

    it('allows a send after recalled metadata is cleared', async () => {
        const apiMessages = factory({ id: 'native-loopback', ephemeral: true }, [], 0);
        const messages = apiMessages({ messages: [{ role: 'user', content: 'Hi' }] });
        expect(await messages[0].content).toBe('Hi');
    });
});
