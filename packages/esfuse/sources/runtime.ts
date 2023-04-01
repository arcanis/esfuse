// Because this file is just transformed (not bundled), it can't
// import anything else.

type EsfuseResolutions = Record<string, string>;

type EsfuseRequireFn = ((p: string) => any) & {
  resolve(p: string): string;
};

type EsfuseModule = {
  exports: any;
};

type EsfuseModuleFactory = (
  module: EsfuseModule,
  exports: any,
  require: EsfuseRequireFn,
) => any;

type EsfuseModuleInfo = {
  resolutions: EsfuseResolutions;
  instances: Set<string> | null;
  factory: EsfuseModuleFactory;
  module: EsfuseModule | null;
};

interface EsfuseRuntime {
  modules: Map<string, EsfuseModuleInfo>;
  refreshRequests: Set<string>;

  meta(info: {resolutions: Record<string, EsfuseResolutions>}): void;

  import(p: string): Promise<any>;

  require: EsfuseRequireFn;
  requireWith: (resolution: EsfuseResolutions) => EsfuseRequireFn;

  define: ((p: string, fn: EsfuseModuleFactory, opts?: {physicalPath?: string}) => void) & {
    error: (p: string, error: any) => void;
  };
}

const badResolutions = Object.create(null);
const errors: Record<string, any> = Object.create(null);

let currentEntryPoint: string | null = null;

function upsertModuleInfo(p: string, factory: EsfuseModuleFactory) {
  const moduleInfo = $esfuse$.modules.get(p);
  if (typeof moduleInfo !== `undefined` && !$esfuse$.refreshRequests.has(p))
    return moduleInfo;

  const newModuleInfo: EsfuseModuleInfo = {
    resolutions: badResolutions,
    instances: null,
    module: null,
    factory,
  };

  $esfuse$.modules.set(p, newModuleInfo);
  return newModuleInfo;
}

const $esfuse$: EsfuseRuntime = {
  modules: new Map<string, EsfuseModuleInfo>(),
  refreshRequests: new Set<string>(),

  meta({resolutions}) {
    for (const [p, dependencies] of Object.entries(resolutions)) {
      const moduleInfo = $esfuse$.modules.get(p)!;
      moduleInfo.resolutions = dependencies;
    }
  },

  define: Object.assign((p: string, factory: EsfuseModuleFactory) => {
    upsertModuleInfo(p, factory);

    const physicalPath = resolveVirtual(p);
    if (physicalPath !== p && physicalPath.startsWith(`/_dev/file/`)) {
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
      const module: EsfuseModule = {exports: {}};
      moduleInfo.module = module;
      moduleInfo.factory(module, module.exports, $esfuse$.requireWith(moduleInfo.resolutions));
    }

    return moduleInfo.module.exports;
  }, {
    resolve: (p: string) => {
      return p;
    },
  }),

  requireWith: resolutions => {
    const resolve = (request: string) => {
      if (!Object.prototype.hasOwnProperty.call(resolutions, request))
        throw new Error(`Module not found: ${request}`);

      return resolutions[request];
    };

    return Object.assign((request: string) => {
      return $esfuse$.require(resolve(request));
    }, {resolve});
  },

  import: async p => {
    const moduleInfo = $esfuse$.modules.get(p);
    if (typeof moduleInfo !== `undefined`)
      return $esfuse$.require(p);

    const script = document.createElement(`script`);
    document.head.appendChild(script);

    const promise = new Promise(resolve => {
      script.addEventListener(`load`, () => {
        resolve($esfuse$.require(p));
      });
    });

    script.src = `${p}?type=bundle`;
    return promise;
  },
};

const NUMBER_REGEXP = /^[0-9]+$/;
const VIRTUAL_REGEXP = /^(\/(?:[^/]+\/)*?(?:\$\$virtual|__virtual__))((?:\/((?:[^/]+-)?[a-f0-9]+)(?:\/([^/]+))?)?((?:\/.*)?))$/;

function resolveVirtual(p: string): string {
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
