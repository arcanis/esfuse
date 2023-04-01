import type {ConfigFn} from 'esfuse';

export const config: ConfigFn = () => ({
  servers: {
    website: {
      pageFolder: `./website/pages`,
    }
  },
});
