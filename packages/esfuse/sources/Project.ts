import fs                      from 'fs';
import {getFiles}                                                                       from 'git-smart-project';
import debounce                from 'lodash/debounce';
import os                      from 'os';
import path                    from 'path';
import postcss from 'postcss';
import { Config, defaultConfig } from './Config';
import * as gitUtils from './utils/gitUtils';
import mergeWith from 'lodash/mergeWith';

import {
  OnTransformArgs,
  ProjectDefinition,
  bundle,
  getPathFromUrl,
  getUrlFromPath,
  resolveToPath,
  resolveToUrl,
  transform,
} from '@esfuse/compiler';

export type WatchEvent = {
  changes: Map<string, `added` | `removed` | `changed`>;
};

export type GlobOptions = {
  absolute?: boolean;
  cwd?: string;
};

export class Project {
  definition: ProjectDefinition;
  config = defaultConfig();

  constructor(public root: string) {
    this.definition = {
      root: this.root,
      namespaces: {
        [`ylc`]: path.join(root, `.yarn/cache`),
        [`ygc`]: path.join(os.homedir(), `.yarn/berry/cache`),
      },
    };

    const configPath = path.join(this.root, `esfuse.config.ts`);
    if (fs.existsSync(configPath)) {
      const {config} = require(configPath);

      mergeWith(this.config, config(), (left: any, right: any) => {
        return Array.isArray(left) ? right : undefined;
      }) as Config;
    }
  }

  watcher: fs.FSWatcher | null = null;
  watchListeners = new Set<(e: WatchEvent) => void>();

  watchEvents: Record<fs.WatchEventType, Set<string>> = {
    rename: new Set<string>(),
    change: new Set<string>(),
  };

  flushWatchEvents = debounce(() => {
    const changes: WatchEvent[`changes`] = new Map(
      [...this.watchEvents.rename].map(p => {
        return [p, fs.lstatSync(p, {throwIfNoEntry: false}) ? `added` : `removed`] as const;
      }),
    );

    for (const p of this.watchEvents.change)
      if (!changes.has(p))
        changes.set(p, `changed`);

    this.watchEvents.rename.clear();
    this.watchEvents.change.clear();

    const e: WatchEvent = {changes};
    //this.log(`watch`, e);

    for (const listener of this.watchListeners) {
      listener(e);
    }
  }, 100);

  watch(fn: (e: WatchEvent) => void) {
    let active = true;

    if (this.watchListeners.size === 0) {
      this.watcher = fs.watch(this.root, {
        recursive: true,
      }, (type, filename) => {
        this.watchEvents[type].add(filename);
        this.flushWatchEvents();
      });
    }

    const listener = fn.bind(null);
    this.watchListeners.add(listener);

    return () => {
      if (!active)
        return;

      this.watchListeners.delete(listener);
      active = false;

      if (this.watchListeners.size === 0) {
        this.watcher!.close();
        this.watcher = null;
      }
    };
  }

  async glob(opts?: GlobOptions): Promise<Array<string>>;
  async glob(pattern: string, opts?: GlobOptions): Promise<Array<string>>;
  async glob(arg1?: string | GlobOptions, arg2?: GlobOptions) {
    const pattern = typeof arg1 === `string` ? arg1 : undefined;
    const opts = (typeof arg1 === `string` ? arg2 : arg1) as GlobOptions;

    const {cwd = this.root, absolute = false} = opts;

    if (!fs.existsSync(cwd))
      throw new Error(`Cannot glob a folder that doesn't exist`);

    const git = gitUtils.createGitClient(cwd);
    const files = await getFiles(git, {pattern});

    return absolute
      ? files.map(p => path.join(cwd, p))
      : files;
  }

  pathToUrl(path: string) {
    return getUrlFromPath({
      project: this.definition,
      path,
    });
  }

  async resolveToPath(specifier: string, issuer: string = this.root) {
    return resolveToPath({
      project: this.definition,
      specifier,
      from: issuer,
    });
  }

  async resolveToUrl(specifier: string, issuer: string = this.root) {
    return resolveToUrl({
      project: this.definition,
      specifier,
      from: issuer,
    });
  }

  async transform(p: string, opts?: OnTransformArgs) {
    return transform({
      project: this.definition,
      file: p,
      opts,
    });
  }

  async devBundle(p: string) {
    if (p.endsWith(`/tailwind.config.js`))
      return this.tailwind(p);

    const res = await bundle({
      project: this.definition,
      entry: p,
    });

    let code = res.code;

    if (res.mimeType === `text/javascript`) {
      if (code)
        code += `\n`;

      const segments = Object.entries(res.errors).map(([id, error]) => {
        return `$esfuse$.define.error(${JSON.stringify(id)}, ${JSON.stringify(error, null, 4)});\n`;
      });

      segments.push(
        `$esfuse$.meta(${JSON.stringify({
          resolutions: res.resolutions,
        }, null, 4)});\n`,
      );

      code += segments.join(`\n`);
    }

    return {
      mimeType: res.mimeType,
      code,
    };
  }

  private async tailwind(url: string) {
    const physicalPath = getPathFromUrl({
      project: this.definition,
      url,
    });

    if (!physicalPath)
      throw new Error(`Assertion failed: Expected the tailwind configuration file path to be in the provided url`);

    const tailwindPath = await this.resolveToPath(
      `tailwindcss`,
      physicalPath,
    );

    const res = await postcss([
      require(tailwindPath)(physicalPath),
    ]).process(`@tailwind base;\n`);

    return {
      mimeType: `text/css`,
      code: res.css,
    };
  }
}
