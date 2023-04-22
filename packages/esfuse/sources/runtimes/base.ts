// Because this file is just transformed (not bundled), it can't
// import anything else but types.

type EsfuseRequireFn = ((p: string) => any) & {
  resolve(p: string): string;
  import(p: string): Promise<any>;
};

type EsfuseHotApi = {
  accept: (fn?: () => void) => void;
};

type EsfuseModule = {
  id: string;
  exports: any;
  hot: EsfuseHotApi;
};

type EsfuseModuleFactory = (
  module: EsfuseModule,
  exports: any,
  require: EsfuseRequireFn,
  filename: string | null,
  dirname: string | null,
) => any;

type EsfuseMetaInfo = Record<string, {
  error: Record<string, any>;
} | {
  error: null;
  path: string | null;
  resolutions: Record<string, string>;
}>;

type EsfuseModuleInfo = {
  instances: Set<string> | null;
  factory: EsfuseModuleFactory;
  module: EsfuseModule | null;
};

class EsfuseCompilationError extends Error {
  constructor(public error: any) {
    super();
  }
}

interface EsfuseRuntime {
  CompilationError: typeof EsfuseCompilationError;

  log(msg: string, ...args: Array<any>): void;

  modules: Map<string, EsfuseModuleInfo>;
  refreshRequests: Set<string>;

  meta(info: EsfuseMetaInfo): void;

  define: ((p: string, fn: EsfuseModuleFactory, opts?: {physicalPath?: string}) => void) & {
    error: (p: string, error: any) => void;
  };

  require: EsfuseRequireFn;
  instantiate: (p: string, fn: () => void) => void;
}

const meta: EsfuseMetaInfo = {};

function upsertModuleInfo(p: string, factory: EsfuseModuleFactory) {
  const moduleInfo = $esfuse$.modules.get(p);
  if (typeof moduleInfo !== `undefined` && !$esfuse$.refreshRequests.has(p))
    return moduleInfo;

  const newModuleInfo: EsfuseModuleInfo = {
    instances: null,
    module: null,
    factory,
  };

  $esfuse$.modules.set(p, newModuleInfo);
  return newModuleInfo;
}

const $esfuse$: EsfuseRuntime = {
  CompilationError: EsfuseCompilationError,

  modules: new Map<string, EsfuseModuleInfo>(),
  refreshRequests: new Set<string>(),

  log(msg, ...extra) {
    console.log(msg, ...extra);
  },

  meta(newMeta) {
    Object.assign(meta, newMeta);
  },

  define: Object.assign((p: string, factory: EsfuseModuleFactory) => {
    upsertModuleInfo(p, factory);

    const physicalPath = resolveVirtual(p);
    if (physicalPath !== p && physicalPath.startsWith(`/_dev/`)) {
      const physicalModuleInfo = upsertModuleInfo(physicalPath, () => {
        throw new Error(`Physical paths with virtual instances cannot be directly instantiated`);
      });

      physicalModuleInfo.instances ??= new Set();
      physicalModuleInfo.instances.add(p);
    }
  }, {
    error: (p: string, error: any) => {
      $esfuse$.define(p, () => {
        throw {p, error};
      });
    },
  }),

  require: Object.assign((p: string) => {
    const moduleInfo = $esfuse$.modules.get(p);
    if (typeof moduleInfo === `undefined`)
      throw new Error(`Module not found: ${p}`);

    if (moduleInfo.module === null) {
      const module: EsfuseModule = {id: p, exports: {}, hot: {accept: () => {}}};
      moduleInfo.module = module;

      const moduleMeta = meta[p];
      if (moduleMeta.error)
        throw new Error(`Assertion failed: Cannot resolve from a module that failed to load`);

      const resolveFn = (request: string) => {
        if (request.startsWith(`/_dev/`))
          return request;

        if (!Object.prototype.hasOwnProperty.call(meta, p))
          throw new Error(`Assertion failed: Cannot resolve from a module that doesn't exist`);

        if (!Object.prototype.hasOwnProperty.call(moduleMeta.resolutions, request))
          throw new Error(`Module not found: ${request}`);

        const paths = moduleMeta.path
          ? [moduleMeta.path]
          : [];

        return moduleMeta.resolutions[request] ?? require.resolve(request, {paths});
      };

      const requireFn = (request: string) => {
        const resolution = resolveFn(request);

        return resolution.startsWith(`/_dev/`)
          ? $esfuse$.require(resolution)
          : require(request);
      };

      const importFn = (request: string) => {
        const resolution = resolveFn(request);

        return resolution.startsWith(`/_dev/`)
          ? $esfuse$.require.import(resolution)
          : import(request);
      };

      const requireApi = Object.assign(requireFn, {
        import: importFn,
        resolve: resolveFn,
      });

      const filename = moduleMeta.path;
      const dirname = moduleMeta.path?.replace(/[\\/][^\\/]*$/, ``) ?? null;

      $esfuse$.instantiate(module.id, () => {
        moduleInfo.factory(module, module.exports, requireApi, filename, dirname);
      });
    }

    return moduleInfo.module.exports;
  }, {
    resolve: (p: string) => {
      return p;
    },
    import: (p: string) => {
      const moduleInfo = $esfuse$.modules.get(p);
      if (typeof moduleInfo !== `undefined`)
        return Promise.resolve().then(() => $esfuse$.require(p));

      const script = document.createElement(`script`);
      document.head.appendChild(script);

      const promise = new Promise(resolve => {
        script.addEventListener(`load`, () => {
          resolve({default: $esfuse$.require(p)});
        });
      });

      script.src = p;
      return promise;
    },
  }),

  instantiate(p, fn) {
    // This function does nothing interesting, but is a hook point that the
    // hmr implementation can use to track component definition

    fn();
  },
};

const NUMBER_REGEXP = /^[0-9]+$/;
const VIRTUAL_REGEXP = /^(\/(?:[^/]+\/)*?(?:\$\$virtual|__virtual__))((?:\/((?:[^/]+-)?[a-f0-9]+)(?:\/([^/]+))?)?((?:\/.*)?))$/;

function resolveVirtual(p: string): string {
  p = p.replace(/\?.*/, ``);

  while (true) {
    const match = p.match(VIRTUAL_REGEXP);
    if (!match || (!match[3] && match[5]))
      return p;

    const target = match[1].replace(/\/[^/]+\/*$/, ``);
    if (!match[3] || !match[4])
      return target;

    const isnum = NUMBER_REGEXP.test(match[4]);
    if (!isnum)
      return p;

    const depth = Number(match[4]);
    const backstep = `../`.repeat(depth);
    const subpath = (match[5] || `.`);

    p = new URL(`${target}/${backstep}/${subpath}`, `https://example.org`).pathname.replace(/\/{2,}/, `/`);
  }
}
