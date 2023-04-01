const {transformNoHooks} = require(`@esfuse/compiler`);
const addHook = require(`pirates`).addHook;
const path = require(`path`);

const project = {
  root: __dirname,
  namespaces: {},
};

addHook((code, file) => {
  return transformNoHooks({project, file}).value.code;
}, {
  exts: [`.ts`],
});
