import micromatch     from 'micromatch';
import path           from 'path';
import {Message}      from 'postcss';
import {Worker}       from 'worker_threads';

import {Project}      from 'esfuse/sources/Project';
import * as miscUtils from 'esfuse/sources/utils/miscUtils';
import * as nodeUtils from 'esfuse/sources/utils/nodeUtils';

const code = `
  const {createRequire} = require('module');
  const {parentPort, workerData} = require('worker_threads');

  const esfuseRequire = createRequire(workerData.esfusePath);
  const userRequire = createRequire(workerData.tailwindPath);

  const postcss = esfuseRequire('postcss');
  const tailwind = userRequire('tailwindcss');

  const compile = () => postcss([
    tailwind(workerData.tailwindPath),
  ]).process([
    '@tailwind base;\\n',
    '@tailwind components;\\n',
    '@tailwind utilities;\\n',
  ].join(''));

  parentPort.on('message', ({version}) => {
    compile().then(({messages, css}) => {
      parentPort.postMessage({
        type: 'complete',
        version,
        result: {
          messages,
          css,
        },
      });
    });
  });
`;

function makeFilter(root: string, messages: Array<Message>) {
  const exactMatches = new Set();
  const globMatches: Array<(p: string) => boolean> = [];

  for (const message of messages) {
    if (message.type === `dependency`) {
      exactMatches.add(path.relative(root, message.file));
    } else if (message.type === `dir-dependency`) {
      globMatches.push(micromatch.matcher(path.join(path.relative(root, message.dir), message.glob ?? `**`)));
    }
  }

  return (p: string) => {
    return exactMatches.has(p) || globMatches.some(matcher => matcher(p));
  };
}

export type Result = {
  messages: Array<Message>;
  css: string;
};

class Instance {
  worker: Worker;

  activeBuild: miscUtils.Deferred | null = null;
  pendingReads: Array<miscUtils.Deferred<string>> = [];

  version: number = 0;

  watchFilter: (p: string) => boolean = () => false;
  css: string = ``;

  constructor(public project: Project, public tailwindPath: string) {
    const locator = project.locatorFromUrl(
      path.posix.join(`/_dev/internal/tailwind`, path.relative(project.root, tailwindPath)),
    )!;

    this.worker = new Worker(code, {
      eval: true,
      workerData: {
        esfusePath: __filename,
        tailwindPath,
      },
    });

    this.worker.on(`error`, error => {
      console.log(error);
    });

    this.worker.on(`message`, ({version, result}) => {
      if (this.version !== version)
        return;

      this.activeBuild = null;

      this.watchFilter = makeFilter(this.project.root, result.messages);
      this.css = result.css;

      const pendingReads = this.pendingReads;
      this.pendingReads = [];

      for (const deferred of pendingReads)
        deferred.resolve(this.css);

      this.project.notifyUpdate(locator);
    });

    this.project.watch(e => {
      if ([...e.changes.keys()].some(p => typeof p === `string` && this.watchFilter(p))) {
        this.recompile();
      }
    });

    this.recompile();
  }

  async recompile() {
    this.activeBuild = miscUtils.makeDeferred();

    this.worker.postMessage({
      type: `recompile`,
      version: ++this.version,
    });
  }

  async read() {
    if (!this.activeBuild)
      return this.css;

    const deferred = miscUtils.makeDeferred<string>();
    this.pendingReads.push(deferred);

    return deferred.promise;
  }
}

export class Tailwind {
  instances = new Map<string, Instance>();

  constructor(public project: Project) {
  }

  async find(p: string) {
    let tailwindPath: string | null = null;
    try {
      tailwindPath = nodeUtils.findClosestFile(p, `tailwind.config.js`);
    } catch {}

    if (!tailwindPath)
      return null;

    return path.relative(this.project.root, tailwindPath);
  }

  async read(relativePath: string) {
    const fullPath = path.join(this.project.root, relativePath);

    const instance = miscUtils.getFactoryWithDefault(this.instances, fullPath, () => {
      return new Instance(this.project, fullPath);
    });

    return await instance.read();
  }
}
