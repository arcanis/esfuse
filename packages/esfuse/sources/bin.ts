import {runExit}    from 'clipanion';

import {DevCommand} from 'esfuse/sources/commands/dev';
import {RunCommand} from 'esfuse/sources/commands/run';

runExit({
  binaryLabel: `Esfuse`,
  binaryName: `yarn esfuse`,
}, [DevCommand, RunCommand]);
