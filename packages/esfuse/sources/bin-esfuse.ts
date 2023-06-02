// eslint-disable-next-line arca/import-absolutes
import {keys, get} from './commands/[command].ts';
import {runExit}   from 'clipanion';

runExit({
  binaryLabel: `Esfuse`,
  binaryName: `yarn esfuse`,
}, keys.flatMap((key: string) => Object.values(get(key))));
