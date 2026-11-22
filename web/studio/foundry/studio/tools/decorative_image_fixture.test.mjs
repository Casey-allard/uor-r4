import assert from 'node:assert/strict';
import {test} from 'node:test';
import {isDecorativeImage} from './decorative_image_fixture.mjs';
const image='https://prism-unsplash.trinity-03d.workers.dev/image?src=https%3A%2F%2Fimages.unsplash.com%2Fphoto-fixture&w=1280&q=85&fm=avif';
test('decorative image fixtures recognize only the declared proxy image shape',()=>{
 assert.equal(isDecorativeImage(image),true);
 for(const address of [image.replace('/image?','/collect?'),image+'&telemetry=private',image+'&src=https://other.invalid',image.replace('images.unsplash.com','other.invalid'),image.replace('workers.dev','workers.dev.evil.invalid'),image.replace('1280','unbounded'),'not a URL','https://posthog.example/capture'])assert.equal(isDecorativeImage(address),false,address);
});
