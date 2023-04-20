import {RouteObject, RouterProvider, createBrowserRouter} from 'react-router-dom';
import React                                              from 'react';
// @ts-expect-error
import {get, keys}                                        from 'website/sources/documents/[...path].mdx?meta';
import {Template}                                         from 'website/sources/template/Template';
import {Documentation}                                    from 'website/sources/template/pages/Documentation';

Error.stackTraceLimit = 100;

function documentationLoader(params: any) {
  return async () => ({
    MDXContent: await get(params).fetch(),
  });
}

const documentationRoutes = keys.map((params: any): RouteObject => ({
  path: `/${params.path}`,
  element: <Documentation/>,
  loader: documentationLoader(params),
  errorElement: <ErrorPage/>,
}));

const router = createBrowserRouter([
  ...documentationRoutes,
]);

function App() {
  return (
    <React.StrictMode>
      <Template>
        <RouterProvider router={router} />
      </Template>
    </React.StrictMode>
  );
}
