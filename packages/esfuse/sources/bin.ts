import {runExit}     from 'clipanion';

import {DevCommand}  from './commands/dev';

runExit({
  binaryLabel: `Esfuse`,
  binaryName: `yarn esfuse`,
}, [DevCommand]);
