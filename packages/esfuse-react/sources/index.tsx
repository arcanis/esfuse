import {Root, createRoot}                         from 'react-dom/client';
import {Params, useParams}                        from 'react-router-dom';
import {createContext, createElement, useContext} from 'react';

export {ErrorPage} from './ErrorPage';

const RouteParamsContext = createContext<Readonly<Params<string>> | null>(null);

export function RouteParams({required, splat, children}: {required: Array<string>, splat: string, children: React.ReactNode}) {
  const params = useParams();

  return (
    <RouteParamsContext.Provider value={params}>
      {children}
    </RouteParamsContext.Provider>
  );
}

export function useRouteParams() {
  return useContext(RouteParamsContext);
}

let root: Root | undefined;

export function run(module: NodeModule, App: React.FunctionComponent) {
  if (typeof root === `undefined`) {
    const container = document.querySelector(`.root`);
    if (!container)
      throw new Error(`Failed to locate an element with the "root" class in the page`);

    root = createRoot(container);
  }

  root.render(createElement(App));
}
