import {runExit}      from 'clipanion';

import {BuildCommand} from 'esfuse/sources/commands/build';
import {DevCommand}   from 'esfuse/sources/commands/dev';
import {RunCommand}   from 'esfuse/sources/commands/run';
import {TestCommand}  from 'esfuse/sources/commands/test';

runExit({
  binaryLabel: `Esfuse`,
  binaryName: `yarn esfuse`,
}, [BuildCommand, DevCommand, RunCommand, TestCommand]);
