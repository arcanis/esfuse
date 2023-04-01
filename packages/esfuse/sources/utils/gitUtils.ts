import {execFile}  from 'child_process';
import {promisify} from 'util';

const spawnP = promisify(execFile);

export function createGitClient(cwd: string) {
  return async (...args: Array<string>) => spawnP(`git`, args, {
    cwd,
    encoding: `utf8`,
  });
}
