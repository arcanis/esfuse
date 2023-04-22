import {Expect} from 'expect';


export * from './Config';
export * from './Project';

export * as errorUtils from './utils/errorUtils';
export * as routeUtils from './utils/routeUtils';

declare global {
  var expect: Expect;

  function describe(section: string, fn: () => void): void;
  function it(name: string, fn: () => Promise<void>): void;
}
