const {ProjectHandle} = require(`@esfuse/compiler`);
const addHook = require(`pirates`).addHook;

const handler = ProjectHandle.create({
  root: __dirname,
  namespaces: {},
  onResolve: [],
  onFetch: [],
});

addHook((code, path) => {
  const locator = handler.getLocatorFromPath({
    path,
  });

  const result = handler.transformNoHooks({
    locator,
    opts: {
      swc: {
        promisifyBody: false,
        useEsfuseRuntime: false,
      },
      userData: {
      },
    },
  });

  if (result.error)
    throw new Error(JSON.stringify(result.error, null, 2));

  let transpiled = result.value.code;
  if (result.value.map)
    transpiled += `\n//# sourceMappingURL=data:application/json;base64,${Buffer.from(result.value.map).toString(`base64url`)}\n`;

  return transpiled;
}, {
  exts: [`.ts`],
});
