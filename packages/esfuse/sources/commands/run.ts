import {Command, Option, UsageError} from 'clipanion';
import path                          from 'path';

import {Project}                     from 'esfuse/sources/Project';

export class RunCommand extends Command {
  static paths = [[`run`]];

  scriptPath = Option.String();

  async execute() {
    const app = new Project(process.cwd());

    try {
      const locator = app.locatorFromPath(path.resolve(this.scriptPath));
      if (!locator)
        throw new UsageError(`The provided script doesn't seem to be part of the project`);

      await app.run(locator);
    } finally {
      app.dispose();
    }
  }
}
