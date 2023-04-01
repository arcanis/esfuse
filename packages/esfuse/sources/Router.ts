import { createRouter } from 'radix3';
import { ServerConfig } from '.';
import {Project} from './Project';
import path from 'path';
import * as nodeUtils from './utils/nodeUtils';

const IDENTIFIER_REGEXP = /^[0-9a-zA-Z]+$/;

function parseFilePattern(path: string) {
  const fsPattern = path.replace(/((^|[\\/])index)?\.tsx$/, ``);
  const required = [];

  let pattern = `/`;
  let i = 0;

  const checkString = (str: string) => {
    if (!fsPattern.startsWith(str, i)) {
      throw new Error(`Parse error: Expected ${str}`);
    }

    return false;
  };

  const checkIsIdentifierCharacter = () => {
    if (!IDENTIFIER_REGEXP.test(fsPattern[i]))
      throw new Error(`Parse error: Invalid identifier`);

    return true;
  };

  const checkEol = () => {
    if (i >= fsPattern.length)
      throw new Error(`Parse error: Unexpected end of input`);

    return true;
  };

  while (i < fsPattern.length) {
    if (fsPattern[i] === `/` || fsPattern[i] === `\\`) {
      if (pattern[pattern.length - 1] !== `/`)
        pattern += `/`;

      i += 1;
    } else if (fsPattern[i] === `(`) {
      i += 1;
      while (checkEol() && fsPattern[i] !== `)`) {
        checkIsIdentifierCharacter();
        i += 1;
      }
      i += 1;
    } else if (fsPattern[i] === `[`) {
      i += 1;

      let optional = false;
      if (fsPattern[i] === `[`) {
        optional = true;
        i += 1;
      }

      if (fsPattern[i] === `.`) {
        checkString(`...`);
        pattern += `**`;
        i += 3;
      }

      let name = ``;
      while (checkEol() && fsPattern[i] !== `]`) {
        checkIsIdentifierCharacter();
        name += fsPattern[i];
        i += 1;
      }

      pattern += `:${name}`;

      if (optional) {
        checkString(`]]`);
        i += 2;
      } else {
        required.push(name);
        checkString(`]`);
        i += 1;
      }
    } else {
      checkIsIdentifierCharacter();
      pattern += fsPattern[i];
      i += 1;
    }
  }

  return {pattern, required};
}

/**
console.log(convertPattern("foo/bar")); // "/foo/bar"
console.log(convertPattern("abc/[val]")); // "/abc/:val"
console.log(convertPattern("qux/[val]/bar")); // "/qux/:val/bar"
console.log(convertPattern("foo/hello/[...val]")); // "/foo/hello/**:val"
console.log(convertPattern("foo/[[...val]]")); // "/foo/**:val"
console.log(convertPattern("[val1]/[[...val2]]")); // "/:val1/**:val2"
console.log(convertPattern("foo/[val1]/[[...val2]]")); // "/foo/:val1/**:val2"
console.log(convertPattern("(foo)/bar")); // "/bar"
console.log(convertPattern("(foo)/bar/[word]")); // "/bar/:word"
*/

export class Router {
  private router = createRouter<{
    path: string,
    required: string[],
  }>();

  constructor(public project: Project, public server: ServerConfig) {
  }

  async init() {
    const pageFolder = path.join(this.project.root, this.server.pageFolder);

    const relPaths = await this.project.glob(`**.tsx`, {
      cwd: pageFolder,
    });

    for (const relPath of relPaths) {
      const {pattern, required} = parseFilePattern(relPath);

      this.router.insert(pattern, {
        path: path.join(pageFolder, relPath),
        required,
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
