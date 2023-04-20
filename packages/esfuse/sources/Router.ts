import path            from 'path';
import {createRouter}  from 'radix3';

import {ServerConfig}  from 'esfuse/sources/Config';
import {Project}       from 'esfuse/sources/Project';
import * as nodeUtils  from 'esfuse/sources/utils/nodeUtils';
import * as routeUtils from 'esfuse/sources/utils/routeUtils';

export class Router {
  private router = createRouter<{
    path: string;
    required: Array<string>;
  }>();

  constructor(public project: Project, public server: ServerConfig) {
  }

  async init() {
    const pageFolder = path.join(this.project.root, this.server.pageFolder);

    const relPaths = await this.project.glob(`**/index.*`, {
      cwd: pageFolder,
    });

    let hasRootPattern = false;

    for (const relPath of relPaths) {
      const segments = routeUtils.parseFilePattern(relPath);
      if (segments.length === 0)
        hasRootPattern = true;

      if (segments.length === 0 || !segments[segments.length - 1].type.includes(`wildcard`))
        segments.push({type: `optional-wildcard`, name: `_`});

      const {
        pattern,
        required,
      } = routeUtils.serializeToRadix(segments);

      this.router.insert(pattern, {
        path: path.join(pageFolder, relPath),
        required,
      });
    }

    if (!hasRootPattern) {
      this.router.insert(`/**:_`, {
        path: require.resolve(`@esfuse/react/next-like`, {paths: [pageFolder]}),
        required: [],
      });
    }
  }

  lookup(pathname: string) {
    const match = this.router.lookup(pathname) ?? {
      path: require.resolve(`./pages/404.html`),
      required: [],
    };

    const script = match.path.endsWith(`.html`)
      ? null
      : match.path;

    const template = match.path.endsWith(`.html`)
      ? match.path
      : nodeUtils.findClosestFile(match.path, `index.html`) ?? require.resolve(`./pages/index.html`);

    return {
      template,
      script,
      params: match.params,
    };
  }
}
