// @ts-check

const {workerData, parentPort} = require(`worker_threads`);

process.argv = [process.argv[0], __filename, ...workerData.params.argv ?? []];

Promise.resolve().then(() => {
  return require(workerData.p);
}).then(value => {
  parentPort?.postMessage({type: `async/resolve`, value});
}, value => {
  if (value instanceof Error) {
    parentPort?.postMessage({type: `async/reject/native`, name: value.name, message: value.message, stack: value.stack});
  } else {
    parentPort?.postMessage({type: `async/reject/unknown`, value});
  }
});
