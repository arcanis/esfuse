import fs         from 'fs';
import path       from 'path';
import {Readable} from 'stream';

export function findClosestFile(p: string, name: string) {
  let next = path.dirname(p);
  let current = next;

  do {
    current = next;
    next = path.dirname(current);

    const match = path.join(current, name);
    if (fs.existsSync(match)) {
      return match;
    }
  } while (next !== current);

  return null;
}

export function consumeStream(stream: Readable) {
  const segments: Array<Buffer> = [];

  stream.on(`data`, chunk => {
    segments.push(chunk);
  });

  return new Promise<Buffer>((resolve, reject) => {
    stream.on(`error`, reject);
    stream.on(`close`, () => {
      resolve(Buffer.concat(segments));
    });
  });
}
