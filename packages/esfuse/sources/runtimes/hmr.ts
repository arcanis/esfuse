import ReactRefresh from 'react-refresh/runtime';

declare global {
  interface Window {
    $RefreshReg$: (type: string, id: string) => any;
    $RefreshSig$: () => (type: any) => any;
  }

  interface EsfuseRuntime {
    refresh?(): void;
  }
}

$esfuse$.instantiate = (id, fn) => {
  var prevRefreshReg = window.$RefreshReg$;
  var prevRefreshSig = window.$RefreshSig$;

  window.$RefreshReg$ = (type, id) => {
    const fullId = `${module.id} ${id}`;
    ReactRefresh.register(type, fullId);
  };

  // @ts-expect-error: The DefinitelyTyped types declare createSignatureFunctionForTransform as returning void 🤔
  window.$RefreshSig$ = ReactRefresh.createSignatureFunctionForTransform;

  try {
    fn();
  } finally {
    window.$RefreshReg$ = prevRefreshReg;
    window.$RefreshSig$ = prevRefreshSig;
  }
};

$esfuse$.refresh = () => {
  ReactRefresh.performReactRefresh();
};

ReactRefresh.injectIntoGlobalHook(window);

window.$RefreshReg$ = () => {};
window.$RefreshSig$ = () => type => type;

(() => {
  const esfuseSocket = new WebSocket(`ws://${window.location.host}/_dev/ws`);

  function injectScript(url: string) {
    return new Promise<void>((resolve, reject) => {
      const script = document.createElement(`script`);
      script.src = url;
      script.onload = () => resolve();
      script.onerror = err => reject(err);
      document.head.appendChild(script);
    });
  }

  esfuseSocket.addEventListener(`message`, async e => {
    if (!e.data)
      return;

    const data = JSON.parse(e.data);
    if (data.type !== `watch`)
      return;

    const promises: Array<Promise<void>> = [];
    for (const [p] of data.changes) {
      const moduleInfo = $esfuse$.modules.get(p);
      if (typeof moduleInfo === `undefined`)
        continue;

      const instances = moduleInfo.instances
        ? [...moduleInfo.instances]
        : [p];

      for (const instanceUrl of instances) {
        $esfuse$.log?.(`refreshed`, {url: instanceUrl});
        $esfuse$.refreshRequests.add(instanceUrl);

        promises.push(injectScript(instanceUrl));
      }
    }

    await Promise.all(promises);
    $esfuse$.refresh?.();
  });
})();
