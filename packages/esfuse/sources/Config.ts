import * as tsUtils from 'esfuse/sources/utils/tsUtils';

export type PatternConfig = {
  distFolder?: string;
  sourceFolder?: string;
};

export type BuildConfig = {
  include?: Array<string>;
};

export type ServerConfig = {
  pageFolder: string | null;
};

export type Config = {
  patterns?: PatternConfig;
  builds?: Record<string, BuildConfig>;
  servers?: Record<string, ServerConfig>;
};

export type ConfigFn =
  () => tsUtils.RecursivePartial<Config>;

export const defaultConfig: () => Config = () => ({
});
