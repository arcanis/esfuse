import {createElement} from 'react';
import {createRoot} from 'react-dom/client';

export function run(App: React.FunctionComponent) {
  const container = document.querySelector(`.root`);
  if (!container)
    throw new Error(`Failed to locate an element with the "root" class in the page`);

  const root = createRoot(container);
  root.render(createElement(App));
}
