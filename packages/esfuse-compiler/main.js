const {bundle} = require(`./index.js`);

console.log(require(`util`).inspect(bundle({
  root: `/private/var/folders/rz/nkpqtzyd2bd8jvb3bdknp48r0000gq/T/tmp.1LIYkYg74M`,
  entry: `/private/var/folders/rz/nkpqtzyd2bd8jvb3bdknp48r0000gq/T/tmp.1LIYkYg74M/app.js`,
  ns: {
    [`ylc`]: `/private/var/folders/rz/nkpqtzyd2bd8jvb3bdknp48r0000gq/T/tmp.1LIYkYg74M/.yarn/cache`,
    [`ygc`]: `/Users/mael.nison/.yarn/berry/cache`
  },
}), {
  depth: Infinity
}));
