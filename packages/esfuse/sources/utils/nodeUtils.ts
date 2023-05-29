import fs             from 'fs';
import {escapeRegExp} from 'lodash';
import path           from 'path';
import {Readable}     from 'stream';

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

export function getRegExpFromPath(pattern: string) {
  const normalized = path.normalize(pattern);

  const parts = normalized.split(`/`).map(segment => {
    return segment === `{}` ? `[^\\\\/]+` : escapeRegExp(segment);
  });

  if (parts[parts.length - 1] !== ``)
    parts.push(``);

  const distFolderRegExpStr = parts.join(escapeRegExp(path.sep));
  const distRegExp = new RegExp(`^${distFolderRegExpStr}`);

  return distRegExp;
}
