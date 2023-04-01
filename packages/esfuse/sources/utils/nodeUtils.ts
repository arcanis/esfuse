import fs               from 'fs';
import path             from 'path';
import {Readable}       from 'stream';

import * as esfuseUtils from './esfuseUtils';

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

export async function fetchDirectoryListing(baseP: string) {
  const listing: Array<fs.Dirent> = [];

  async function walk(p: string) {
    const entries = await fs.promises.readdir(path.join(baseP, p), {
      withFileTypes: true,
    });

    for (const entry of entries) {
      entry.name = path.join(p, entry.name);
      if (entry.isDirectory()) {
        await walk(entry.name);
      } else {
        listing.push(entry);
      }
    }
  }

  await walk(`.`);
  return listing;
}

export function getPackageDirForPath(p: string) {
  const fixtureDir = esfuseUtils.getFixtureDirForPath(p);
  if (fixtureDir)
    return fixtureDir;

  const parts = p.split(`/`);

  const lastNmIndex = parts.lastIndexOf(`node_modules`);
  if (lastNmIndex === -1)
    return null;

  const packageDirIndex = parts[lastNmIndex + 1]?.[0] === `@`
    ? lastNmIndex + 3
    : lastNmIndex + 2;

  if (packageDirIndex >= parts.length)
    return null;

  return parts.slice(0, packageDirIndex).join(`/`);
}

const PACKAGE_NAME_REGEXP = /^(@[^/]*\/)?[^/]*/;

export function getPackageNameForSpecifier(p: string) {
  const match = p.match(PACKAGE_NAME_REGEXP);
  return match![0];
}

export function isPathSpecifier(p: string) {
  const packageName = getPackageNameForSpecifier(p);
  return packageName === `.` || packageName === `..`;
}

export function saferRelative(root: string, target: string) {
  const rel = path.relative(root, target);

  return rel !== `.` && !rel.startsWith(`../`) ? `./${rel}` : rel;
}
