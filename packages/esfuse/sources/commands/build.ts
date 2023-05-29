import {Command, Option} from 'clipanion';

import {Project}         from 'esfuse/sources/Project';
import * as miscUtils    from 'esfuse/sources/utils/miscUtils';

export class BuildCommand extends Command {
  static paths = [[`build`]];

  name = Option.String({
    required: false,
  });

  async execute() {
    const app = new Project(process.cwd());

    const buildNames = typeof this.name === `undefined`
      ? Object.keys(app.config.builds ?? {})
      : [this.name];

    const buildPromises = buildNames.map(async buildName => {
      await app.build(buildName);
    });

    let results;
    try {
      results = await Promise.allSettled(buildPromises);
    } finally {
      app.dispose();
    }

    miscUtils.rethrowAllSettled(results);
  }
}
