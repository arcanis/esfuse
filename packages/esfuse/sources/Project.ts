import fs                      from 'fs';
import {getFiles}              from 'git-smart-project';
import debounce                from 'lodash/debounce';
import mergeWith               from 'lodash/mergeWith';
import {createRequire}         from 'module';
import os                      from 'os';
import path                    from 'path';
import * as t                  from 'typanion';
import vm                      from 'vm';

import {Config, defaultConfig} from 'esfuse/sources/Config';
import {Tailwind}              from 'esfuse/sources/Tailwind';
import {Context}               from 'esfuse/sources/context';
import * as gitUtils           from 'esfuse/sources/utils/gitUtils';
import * as miscUtils          from 'esfuse/sources/utils/miscUtils';

import {
  FetchResult,
  ModuleLocator,
  OnBundleOpts,
  OnFetchArgs,
  OnResolveArgs,
  OnTransformOpts,
  ProjectHandle,
  ResolutionKind,
  ResolveResult,
} from '@esfuse/compiler';

export type WatchEvent = {
  changes: Map<string | ModuleLocator, `added` | `removed` | `changed`>;
};

export type GlobOptions = {
  absolute?: boolean;
  cwd?: string;
};

export function extractResult<T, E>(res: {value?: T, error?: E}) {
  return res as {
    value: T;
    error: null;
  } | {
    value: null;
    error: E;
  };
}

export class Project {
  handle: ProjectHandle;
  config = defaultConfig();
  tailwind: Tailwind;

  constructor(public root: string) {
    this.handle = ProjectHandle.create({
      root: this.root,
      namespaces: {
        [`ylc`]: path.join(root, `.yarn/cache`),
        [`ygc`]: path.join(os.homedir(), `.yarn/berry/cache`),
      },
      onResolve: [{
        regexp: `\\[`,
        cb: miscUtils.withErrorLogging(async args => {
          return await this.onDynamicResolve(args);
        }),
      }],
      onFetch: [{
        regexp: `^/_dev/internal/tailwind`,
        cb: miscUtils.withErrorLogging(async args => {
          return await this.tailwindHandler(args);
        }),
      }, {
        regexp: `\\.preval\\.[^/]+$`,
        cb: miscUtils.withErrorLogging(async args => {
          return await this.prevalHandler(args);
        }),
      }, {
        regexp: `\\[`,
        cb: miscUtils.withErrorLogging(async args => {
          return await this.onDynamicFetch(args);
        }),
      }],
    });

    const configPath = path.join(this.root, `esfuse.config.ts`);
    if (fs.existsSync(configPath)) {
      const {config} = require(configPath);

      mergeWith(this.config, config(), (left: any, right: any) => {
        return Array.isArray(left) ? right : undefined;
      }) as Config;
    }

    this.tailwind = new Tailwind(this);
  }

  dispose() {
    this.handle.dispose();
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
    for (const listener of this.watchListeners) {
      listener(e);
    }
  }, 100);

  notifyUpdate(locator: ModuleLocator) {
    const e: WatchEvent = {changes: new Map([[locator, `added`]])};
    for (const listener of this.watchListeners) {
      listener(e);
    }
  }

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

    const {cwd = this.root, absolute = false} = opts ?? {};

    if (!fs.existsSync(cwd))
      throw new Error(`Cannot glob a folder that doesn't exist`);

    const git = gitUtils.createGitClient(cwd);
    const files = await getFiles(git, {pattern});

    return absolute
      ? files.map(p => path.join(cwd, p))
      : files;
  }

  pathFromLocator(locator: ModuleLocator) {
    return this.handle.getPathFromLocator({
      locator,
    });
  }

  locatorFromPath(path: string) {
    return this.handle.getLocatorFromPath(path);
  }

  locatorFromUrl(url: string) {
    return this.handle.getLocatorFromUrl(url);
  }

  async resolveToPath(request: string, issuer?: ModuleLocator) {
    return extractResult(await this.handle.resolve({
      kind: ResolutionKind.ImportDeclaration,
      request,
      issuer,
      opts: {
        forceParams: [],
        userData: {},
      },
    }));
  }

  async transform(locator: ModuleLocator, opts?: OnTransformOpts) {
    return extractResult(await this.handle.transform({
      locator,
      opts: {
        userData: {},
        ...opts,
        swc: {
          promisifyBody: false,
          useEsfuseRuntime: false,
          ...opts?.swc,
        },
      },
    }));
  }

  async transformByPath(path: string, opts?: OnTransformOpts) {
    const locator = this.locatorFromPath(path);
    if (!locator)
      throw new Error(`This path doesn't map to an acceptable locator`);

    return this.transform(locator, opts);
  }

  async run(locator: ModuleLocator, opts: {userData?: any, contextify?: (ctx: any) => void} = {}): Promise<unknown> {
    const res = await this.bundle(locator, {
      promisifyEntryPoint: true,
      requireOnLoad: true,
      traverseVendors: false,
      userData: opts.userData ?? {},
    });

    if (res.value!.mimeType !== `text/javascript`)
      throw new Error(`Only JavaScript files can be run`);

    const $esfuseContext$: Context = {
      project: this,
      userData: opts.userData,
    };

    const ctx = vm.createContext(Object.create(globalThis));

    ctx.$esfuseContext$ = $esfuseContext$;
    ctx.exports = {};
    ctx.module = {exports: ctx.exports};
    ctx.require = createRequire(__filename);

    opts.contextify?.(ctx);

    vm.runInContext(res.value!.code, ctx);

    return await ctx.module.exports;
  }

  async tailwindHandler(args: OnFetchArgs): Promise<FetchResult | undefined> {
    const subPath = path.posix.relative(`/_dev/internal/tailwind`, args.locator.url.replace(/\?.*/, ``));

    return {
      value: {
        locator: args.locator,
        mimeType: `text/css`,
        source: await this.tailwind.read(subPath),
      },
      dependencies: [],
    };
  }

  async prevalHandler(args: OnFetchArgs): Promise<FetchResult | undefined> {
    if (args.locator.params.some(({name}) => name === `skip-preval`))
      return undefined;

    const mod = await this.run({
      ...args.locator,
      params: [
        ...args.locator.params,
        {name: `skip-preval`, value: ``},
      ],
    }, {
      userData: args.opts.userData,
    }) as any;

    if (typeof mod.default === `undefined`)
      return this.prevalData(args, mod);

    const keys = Object.keys(mod);
    if (keys.length !== 1)
      throw new Error(`Preval files must either have a default export or named exports, but not both (except for types)`);

    return this.prevalSource(args, mod.default);
  }

  async prevalData(args: OnFetchArgs, data: any): Promise<FetchResult> {
    const source = Object.entries(data).map(([key, value]) => {
      return `export const ${key} = ${JSON.stringify(value)};\n`;
    }).join(`\n`);

    return {
      value: {
        locator: args.locator,
        mimeType: `text/javascript`,
        source,
      },
      dependencies: [],
    };
  }

  async prevalSource(args: OnFetchArgs, spec: unknown): Promise<FetchResult> {
    t.assertWithErrors(spec, t.isObject({
      mimeType: t.isString(),
      source: t.isString(),
    }));

    return {
      value: {
        locator: args.locator,
        mimeType: spec.mimeType,
        source: spec.source,
      },
      dependencies: [],
    };
  }

  async bundle(locator: ModuleLocator, opts: Partial<OnBundleOpts> = {}) {
    return extractResult(await this.handle.bundle({
      locator,
      opts: {
        onlyEntryPoint: false,
        promisifyEntryPoint: false,
        requireOnLoad: false,
        runtime: this.locatorFromPath(require.resolve(`./runtimes/base.ts`))!,
        traverseVendors: true,
        userData: {},
        ...opts,
      },
    }));
  }

  private async onDynamicResolve(args: OnResolveArgs): Promise<ResolveResult> {
    const issuerPath = args.issuer
      ? this.pathFromLocator(args.issuer)
      : path.isAbsolute(args.request)
        ? args.request
        : null;

    if (issuerPath === null) {
      return {
        error: {
          diagnostics: [{
            message: `Cannot use dynamic imports from files without physical paths`,
            highlights: [],
          }],
        },
        dependencies: [],
      };
    }

    const dynamicType = args.kind === ResolutionKind.DynamicImport
      ? `lazy`
      : `eager`;

    const absoluteDynamicTarget = path.resolve(path.dirname(issuerPath), args.request);
    const nsQualifiedDynamicTarget = this.handle.getNsQualifiedFromPath(absoluteDynamicTarget)!;

    return {
      value: {
        locator: this.locatorFromUrl(path.posix.join(`/_dev/internal`, dynamicType, nsQualifiedDynamicTarget))!,
      },
      dependencies: [],
    };
  }

  private async onDynamicFetch(args: OnFetchArgs): Promise<FetchResult> {
    const firstSegmentIndex = args.locator.specifier.indexOf(`/`);
    const dynamicType = args.locator.specifier.slice(0, firstSegmentIndex);

    const specifier = this.handle.getPathFromNsQualified(args.locator.specifier.slice(firstSegmentIndex + 1))!;
    const eager = dynamicType === `eager`;

    // Used to query the entries from the filesystem
    let globPattern = ``;
    // Used to extract the variables from the entries
    let regexpRawPattern = ``;
    // List of variables; used to find the right branch
    const variables: Array<string> = [];

    let currentIndex = 0;
    let resolvedRelativeTo = `/`;

    // catches all [foo] and [...bar] tags inside the glob pattern to replace
    // them with path-aware capture groups (similar to Next.js' args, for instance)
    for (const match of specifier.matchAll(/\[(\.\.\.)?([a-z0-9]+)\]/gi)) {
      const isFirstMatch = currentIndex === 0;

      const prefix = specifier.slice(currentIndex, match.index);
      currentIndex = match.index! + match[0].length;

      if (isFirstMatch) {
        const slashIndex = prefix.lastIndexOf(`/`);
        const [left, right] =
          slashIndex !== -1
            ? [prefix.slice(0, slashIndex), prefix.slice(slashIndex + 1)]
            : [``, prefix];

        resolvedRelativeTo = path.resolve(resolvedRelativeTo, left);

        globPattern += right;
        regexpRawPattern += right;
      } else {
        globPattern += prefix;
        regexpRawPattern += prefix;
      }

      const captureGroup = !variables.includes(match[2])
        ? `?<${match[2]}>`
        : `?:`;

      globPattern += match[1] ? `**` : `*`;
      regexpRawPattern += match[1]
        ? `(?:(${captureGroup}[^/]+(/[^/]+)*))?`
        : `(${captureGroup}[^/]+)`;

      variables.push(match[2]);
    }

    const suffix = specifier.slice(currentIndex);
    globPattern += suffix;
    regexpRawPattern += suffix;

    const entries = await this.glob(globPattern, {
      absolute: true,
      cwd: resolvedRelativeTo,
    });

    if (entries.length === 0)
      throw new Error(`No entries matched the specified glob pattern\n\nPattern: ${args.locator.specifier}\nGlob:    ${globPattern}\nCwd:     ${resolvedRelativeTo}`);

    const regexpPattern = new RegExp(`^${regexpRawPattern}$`);

    const imports = [];
    const importKeys = [];
    const cases = [];

    for (const [key, entry] of entries.entries()) {
      const entryKey = JSON.stringify(`${entry}`/*args.suffix*/);

      const relPath = path.relative(resolvedRelativeTo, entry);
      const matches = relPath.match(regexpPattern);
      if (!matches)
        throw new Error(`Assertion failed: The generated regexp failed to parse the glob pattern\n\nPattern: ${args.locator.specifier}\nGlob:    ${globPattern}\nCwd:     ${resolvedRelativeTo}\nPath:    ${relPath}\nRegexp:   ${regexpPattern}`);

      if (eager) {
        imports.push(
          `import * as _${key} from ${entryKey};\n`,
        );
      }

      const access = eager
        ? `_${key}`
        : `require(${entryKey})`;

      const importKey = JSON.stringify(
        matches.groups,
        Object.keys(matches.groups!).sort(),
      );

      importKeys.push(matches.groups);

      cases.push(`    case ${JSON.stringify(importKey)}: return ${access};\n`);
    }

    const source = [
      ...imports,
      `\n`,
      `export ${eager ? `` : `async `}function ${eager ? `get` : `fetch`}(vars) {\n`,
      `  const key = JSON.stringify(vars, Object.keys(vars).sort());\n`,
      `  switch (key) {\n`,
      ...cases,
      `    default: throw new Error("Entry not found: " + key);\n`,
      `  }\n`,
      `}\n`,
      ...eager ? [
        `\n`,
        `export async function fetch(vars) {\n`,
        `  return get(vars);\n`,
        `}\n`,
      ] : [],
      `\n`,
      `export const keys = ${JSON.stringify(importKeys, null, 2)};\n`,
    ].join(``);

    return {
      value: {
        locator: args.locator,
        mimeType: `text/javascript`,
        source,
      },
      dependencies: [],
    };
  }
}
