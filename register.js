const {ProjectHandle} = require(`@esfuse/compiler`);
const addHook = require(`pirates`).addHook;
const path = require(`path`);

const handler = ProjectHandle.create({
  root: __dirname,
  namespaces: {},
});

addHook((code, file) => {
  return handler.transformNoHooks({file}).value.code;
}, {
  exts: [`.ts`],
});
