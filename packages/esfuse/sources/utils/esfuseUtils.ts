import fs   from 'fs';
import os   from 'os';
import path from 'path';
import qs   from 'querystring';

const pkgJsonPath = eval(`require`).resolve(`esfuse/package.json`);
const esfuseDir = path.dirname(pkgJsonPath);
const fixturesDir = path.join(esfuseDir, `fixtures`);

export const emptySearchParams = new URLSearchParams();

export function getEsfusePath(p: string) {
  return path.join(esfuseDir, p);
}

function getFilePublicPath(p: string, {root}: {root: string}) {
  const appRel = path.relative(root, p);
  if (!appRel.startsWith(`../`))
    return `/_dev/app/${appRel}`;

  const homeRel = path.relative(os.homedir(), p);
  if (!homeRel.startsWith(`../`))
    return `/_dev/home/${homeRel}`;

  throw new Error(`Assertion failed: Couldn't locate the file in one of the known base directories (${p})`);
}

export function getPublicPath(p: string, opts: {root: string, query?: string}) {
  let queryPart = opts.query
    ? `?${opts.query}`
    : ``;

  const qsIndex = p.indexOf(`?`);
  if (qsIndex === -1)
    return `${getFilePublicPath(p, opts)}${queryPart}`;

  const left = p.slice(0, qsIndex);
  const right = p.slice(qsIndex + 1);

  queryPart += queryPart
    ? `&q=${encodeURIComponent(right)}`
    : `?q=${encodeURIComponent(right)}`;

  return `${getFilePublicPath(left, opts)}${queryPart}`;
}

function resolveFileFromPublicPath(p: string, {root}: {root: string}) {
  const appRel = path.relative(`/_dev/app`, p);
  if (!appRel.startsWith(`..`))
    return path.join(root, appRel);

  const homeRel = path.relative(`/_dev/home`, p);
  if (!homeRel.startsWith(`..`))
    return path.join(os.homedir(), homeRel);

  throw new Error(`Assertion failed: Couldn't locate the file in one of the known base directories (${p})`);
}

export function resolveFromPublicPath(p: string, {root}: {root: string}) {
  const qsIndex = p.indexOf(`?`);
  if (qsIndex === -1)
    return resolveFileFromPublicPath(p, {root});

  const left = p.slice(0, qsIndex);
  const right = p.slice(qsIndex + 1);
  const {q} = qs.parse(right);
  if (typeof q !== `string`)
    return resolveFileFromPublicPath(left, {root});

  return `${resolveFileFromPublicPath(left, {root})}?${decodeURIComponent(q)}`;
}

export function getFixtureDirForPath(p: string) {
  if (!p.startsWith(fixturesDir))
    return null;

  const sub = p.slice(fixturesDir.length);
  const match = sub.match(/^(?:\/([^/]*))/);
  if (!match)
    return null;

  const candidateDir = path.join(fixturesDir, match[1]);
  if (!fs.existsSync(path.join(candidateDir, `package.json`)))
    return null;

  return candidateDir;
}
