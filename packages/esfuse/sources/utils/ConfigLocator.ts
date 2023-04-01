import fs             from 'fs';
import path           from 'path';

import * as miscUtils from './miscUtils';

export class ConfigLocator {
  caches = new Map<string, Map<string, boolean>>();

  constructor(public root: string) {
  }

  lookup(names: Array<string>, from: string) {
    const segments = from.split(`/`);

    const caches = names.map(name => {
      return miscUtils.getMapWithDefault(this.caches, name);
    });

    let t = segments.length - 1;
    while (t >= 0) {
      const dir = segments.slice(0, t).join(`/`);

      for (let u = 0; u < names.length; ++u) {
        const cache = caches[u];
        const name = names[u];

        const p = path.join(dir, name);

        let check = cache.get(p);
        if (typeof check === `undefined`)
          cache.set(p, check = !!fs.statSync(p, {throwIfNoEntry: false}));

        if (check) {
          return p;
        }
      }

      t -= 1;
    }

    return null;
  }
}
