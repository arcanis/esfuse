import type {Project} from 'esfuse/sources/Project';

export type Context = {
  project: Project;
  userData: any;
};

declare global {
  const $esfuseContext$: Context;
}

export const project = $esfuseContext$.project;
export const userData = $esfuseContext$.userData;
