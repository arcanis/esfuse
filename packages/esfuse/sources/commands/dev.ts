import {Command, UsageError}     from 'clipanion';

import {Project}     from '../Project';
import {Server}      from '../Server';
import * as logUtils from '../utils/logUtils';

export class DevCommand extends Command {
  static paths = [[`dev`]];

  async execute() {
    const app = new Project(process.cwd());
    const servers = Object.entries(app.config.servers ?? {});

    if (servers.length === 0)
      throw new UsageError(`No servers have been configured in your Esfuse configuration file`);

    for (const [name, config] of servers) {
      const server = new Server(app, config);
      const info = await server.listen();

      this.context.stdout.write(`Server ${name} is listening on http://localhost:${info.port}!\n`);
      this.context.stdout.write(`Application root: ${app.root}\n`);
    }

    return new Promise<void>(() => {});
  }
}
