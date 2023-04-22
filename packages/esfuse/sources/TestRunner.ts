import expect         from 'expect';
import {style}        from 'term-strings';

import * as miscUtils from 'esfuse/sources/utils/miscUtils';

const SEP = ` ⁘ `;

const color = (colorName: string, text: string) => {
  return `${style.color.front(colorName)}${text}${style.color.front.out}`;
};

enum TestResultType {
  Skipped,
  Success,
  Failed,
}

type TestResult =
  | {sections: Array<string>, type: TestResultType.Skipped}
  | {sections: Array<string>, type: TestResultType.Success}
  | {sections: Array<string>, type: TestResultType.Failed, error: string, stack: string};

export type TestReport = Map<string, TestResult>;
export type FullReport = Map<string, TestReport>;

export class TestRunner {
  tests = new Map<string, {
    sections: Array<string>;
    fn: () => Promise<void>;
    result: TestResult | null;
  }>();

  currentTestSections: Array<string> = [];

  describeFn = (name: string, fn: () => void) => {
    this.currentTestSections.push(name);
    try {
      fn();
    } finally {
      this.currentTestSections.pop();
    }
  };

  itFn = (name: string, fn: () => Promise<void>) => {
    this.describeFn(`it ${name}`, () => {
      let id = this.currentTestSections.join(`\0`);
      while (this.tests.has(id))
        id += `+`;

      this.tests.set(id, {
        sections: [...this.currentTestSections],
        fn: this.makeTestRunner(id, fn),
        result: null,
      });
    });
  };

  makeTestRunner(id: string, fn: () => Promise<void>) {
    const wrapper = async () => {
      const testInfo = this.tests.get(id)!;
      try {
        await fn();
        testInfo.result = {sections: testInfo.sections, type: TestResultType.Success};
      } catch (error: any) {
        testInfo.result = {sections: testInfo.sections, type: TestResultType.Failed, error: error.toString(), stack: error.stack};
      }
    };

    return wrapper;
  }

  apply(ctx: any) {
    ctx.describe = this.describeFn;
    ctx.it = this.itFn;
    ctx.expect = expect;
  }

  async run(): Promise<TestReport> {
    await Promise.all([...this.tests.values()].map(({fn}) => {
      return fn();
    }));

    const results = new Map([...this.tests.entries()].map(([id, {result}]) => {
      return [id, result!];
    }));

    return results;
  }

  static renderTestReport(report: TestReport) {
    const lines: Array<string> = [];
    const testIds = miscUtils.sortMap(report.keys(), id => report.get(id)!.sections.join(``));

    let previousSection: string | null | undefined = undefined;

    for (const id of testIds) {
      const result = report.get(id)!;

      const leadingSection = result.sections.length > 1
        ? result.sections[0]
        : null;

      if (typeof previousSection === `undefined`)
        previousSection = leadingSection;

      if (leadingSection !== previousSection) {
        lines.push(``);
        previousSection = leadingSection;
      }

      const prefix = color(`grey`, result.sections.slice(0, -1).map(l => `${l}${SEP}`).join(``));
      const name = result.sections[result.sections.length - 1];

      switch (result.type) {
        case TestResultType.Skipped: {
          // Nothing to do
        } break;

        case TestResultType.Success: {
          lines.push(`    ${color(`green`, `✔`)}  ${prefix}${name}`);
        } break;

        case TestResultType.Failed: {
          lines.push(`    ${color(`red`, `✘`)}  ${prefix}${name}`);
        } break;
      }
    }

    return lines;
  }

  static renderFullReport(report: FullReport) {
    const lines: Array<string> = [];
    const fileNames = [...report.keys()].sort();
    const errors: Array<[string, string, string]> = [];

    let isFirst = true;

    for (const fileName of fileNames) {
      const testReport = report.get(fileName)!;
      if (testReport.size === 0)
        continue;

      if (!isFirst)
        lines.push(``);

      lines.push(`${style.underlined.in}${fileName}${style.underlined.out}`);
      lines.push(``);
      lines.push(...this.renderTestReport(testReport));

      for (const result of testReport.values())
        if (result.type === TestResultType.Failed)
          errors.push([result.sections.join(SEP), result.error, result.stack]);

      isFirst = false;
    }

    errors.sort((a, b) => {
      return a[0] < b[0] ? -1 : a[0] > b[0] ? +1 : 0;
    });

    for (const [name, error, stack] of errors) {
      lines.push(``);
      lines.push(color(`red`, name));
      lines.push(``);
      lines.push(error.replace(/^/gm, `  `));
      lines.push(``);
      lines.push(stack.replace(/((?! {4}at ).)*/s, ``));
    }

    return lines;
  }
}
