import {npath, PortablePath, ppath, xfs} from '@yarnpkg/fslib';
import {execFile}                        from 'child_process';
import {createRequire}                   from 'module';
import path                              from 'path';
import {promisify}                       from 'util';
import vm                                from 'vm';

import {Project}                         from 'esfuse/sources/Project';
import {Request, Server}                 from 'esfuse/sources/Server';

declare module 'vm' {
  class SourceTextModule {
    constructor(source: string, opts: {context: vm.Context});
    link(fn: (specifier: string, referencingModule: any) => Promise<SourceTextModule>): Promise<void>;
    evaluate(): Promise<any>;
    namespace: any;
  }
}

export function makeTestApplication() {
  return new Project(path.dirname(__dirname));
}

export async function makeAppRunner(app: Project) {
  const server = new Server(app, {
    pageFolder: null,
  });

  let runtime: EsfuseRuntime;

  const query = async (req: Request) => {
    const res = await server.handle(req);
    if (typeof res.code !== `undefined` && res.code !== 200)
      throw new Error(`Server answered status code ${res.code}: ${res.body}`);

    const ctx = vm.createContext(Object.create(globalThis));
    ctx.$esfuse$ = undefined;
    ctx.exports = {};
    ctx.module = {exports: ctx.exports};
    ctx.require = require;

    vm.runInContext(res.body!.toString(), ctx);

    runtime = ctx.$esfuse$;

    return ctx.module.exports as Record<string, any>;
  };

  const get = async (url: string) => {
    return query({method: `GET`, url: new URL(url, `https://example.org`), body: null});
  };

  const getDefinedModules = () => {
    return [...runtime.modules.keys()].sort();
  };

  const getEvaluatedModules = () => {
    return getDefinedModules().filter(n => runtime.modules.get(n)!.module !== null);
  };

  return {
    runtime: {
      getDefinedModules,
      getEvaluatedModules,
    },
    query: {
      get,
    },
  };
}

export async function installProjectLayout(layout: Record<string, string>, fn: (tmpDir: PortablePath) => Promise<void>) {
  await xfs.mktempPromise(async tmpDir => {
    await execvp(`git`, [`init`], {
      cwd: tmpDir,
    });

    for (const [relP, content] of Object.entries(layout)) {
      const p = ppath.join(tmpDir, relP as PortablePath);

      await xfs.mkdirPromise(ppath.dirname(p), {recursive: true});
      await xfs.writeFilePromise(p, content);
    }

    try {
      await fn(tmpDir);
    } catch (e: any) {
      xfs.detachTemp(tmpDir);
      e.message += ` (in ${tmpDir})`;
      throw e;
    }
  });
}

const execFileP = promisify(execFile);

export async function execvp(file: string, args: Array<string>, {cwd}: {cwd: PortablePath}) {
  return await execFileP(file, args, {
    cwd: npath.fromPortablePath(cwd),
    env: {
      ...process.env,
      NODE_OPTIONS: ``,
    },
  });
}
