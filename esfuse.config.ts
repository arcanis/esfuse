import type {ConfigFn} from 'esfuse';

export const config: ConfigFn = () => ({
  patterns: {
    distFolder: `./packages/{}/dist`,
    sourceFolder: `./packages/{}`,
  },

  builds: {
    esfuse: {
      include: [`**/*.ts`],
    },
  },

  servers: {
    website: {
      pageFolder: `./website/app`,
    },
  },
});
