import * as tsUtils from './utils/tsUtils';

export type ServerConfig = {
  pageFolder: string,
};

export type Config = {
  servers?: Record<string, ServerConfig>;
};

export type ConfigFn =
  () => tsUtils.RecursivePartial<Config>;

export const defaultConfig: () => Config = () => ({
});
