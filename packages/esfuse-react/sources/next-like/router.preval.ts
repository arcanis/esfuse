import path                from 'path';

import {project, userData} from 'esfuse/context';
import {routeUtils}        from 'esfuse';

const appFolder = userData.pageFolder;

const [pages, layouts] = await Promise.all([
  project.glob(`**/page.*`, {cwd: appFolder}),
  project.glob(`**/layout.tsx`, {cwd: appFolder}),
]);

const layoutByFolder = new Map<string, number | null>(layouts.map((p, index) => {
  return [path.dirname(p), index] as const;
}));

function findLayoutForPage(p: string): number | null {
  const dir = path.dirname(p);
  if (dir === p)
    return null;

  const layout = layoutByFolder.get(dir);
  if (typeof layout !== `undefined`)
    return layout;

  const parentLayout = findLayoutForPage(dir);
  layoutByFolder.set(dir, parentLayout);

  return parentLayout;
}

const applyLayout = (layout: number | null, props: string) => layout !== null
  ? `<Layout_${layout} ${props}/>`
  : `<DefaultLayout ${props}/>`;

const makePage = (page: number, layout: number | null, info: ReturnType<typeof routeUtils[`serializeToReact`]>) => `{
  path: ${JSON.stringify(info.pattern)},
  errorElement: <ErrorPage/>,
  lazy: wrapLazy(Page_${page}, (route, children) => <RouteParams required={${JSON.stringify(info.required)}} splat={${JSON.stringify(info.splat)}}>
    ${applyLayout(layout, `route={route} children={children}`)}
  </RouteParams>),
}`;

const layoutCodeSegment = layouts.map((p, index) => `
import {Layout as Layout_${index}} from ${JSON.stringify(path.join(project.root, appFolder, p))};
`.trimStart()).join(``);

const pageCodeSegment = pages.map((p, index) => `
const Page_${index} = () => import(${JSON.stringify(path.join(project.root, appFolder, p))});
`.trimStart()).join(``);

const routeCodeSegment = pages.map((p, index) => {
  const segments = routeUtils.parseFilePattern(p);
  const layout = findLayoutForPage(p);
  const info = routeUtils.serializeToReact(segments);

  return makePage(index, layout, info);
}).join(`, `);

const script = `
import {ErrorPage, RouteParams}              from '@esfuse/react';
import {RouterProvider, createBrowserRouter} from 'react-router-dom';
import React                                 from 'react';

${layoutCodeSegment.trimEnd()}

${pageCodeSegment.trimEnd()}

function DefaultLayout({children}) {
  return <>{children}</>;
}

function wrapLazy(lazy, wrapper) {
  return async () => {
    const routeModule = await lazy();
    const Page = routeModule.Page ?? routeModule.default;

    return {
      loader: routeModule.loader,
      element: wrapper(routeModule, routeModule.element ?? <Page/>),
    };
  };
}

const router = createBrowserRouter([${routeCodeSegment}]);
console.log('router', [${routeCodeSegment}])

export function withRouter() {
  return () => (
    <React.StrictMode>
      <RouterProvider router={router} />
    </React.StrictMode>
  );
}
`.trimStart().replace(/\n{2,}/, `\n\n`);

// eslint-disable-next-line arca/no-default-export
export default {
  mimeType: `text/javascript`,
  source: script,
};
