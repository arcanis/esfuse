import {Command, Option}        from 'clipanion';
import path                     from 'path';

import {Project}                from 'esfuse/sources/Project';
import {FullReport, TestRunner} from 'esfuse/sources/TestRunner';

export class TestCommand extends Command {
  static paths = [[`test`]];

  cwd = Option.String(`-C,--cwd`);
  testSuites = Option.Rest();

  fullReport: FullReport = new Map();

  async execute() {
    const app = new Project(process.cwd());

    try {
      const testSuites = this.testSuites.length > 0
        ? await Promise.all(this.testSuites.map(p => path.resolve(p)))
        : await app.glob(`**/*.test.*`, {absolute: true, cwd: this.cwd ?? process.cwd()});

      await Promise.all(testSuites.map(async p => {
        await this.registerSpecFile(app, p);
      }));

      const lines = TestRunner.renderFullReport(this.fullReport);
      this.context.stdout.write(lines.map(l => `${l}\n`).join(``));
    } finally {
      app.dispose();
    }
  }

  async registerSpecFile(app: Project, p: string) {
    const relativeP = path.relative(app.root, p);

    const runner = new TestRunner();

    await app.run(app.locatorFromPath(p)!, {
      contextify: ctx => runner.apply(ctx),
    });

    this.fullReport.set(relativeP, await runner.run());
  }
}
