// @ts-check

const {transform} = require(`@esfuse/compiler`);
const addHook = require(`pirates`).addHook;
const {MessageChannel, parentPort, receiveMessageOnPort} = require(`worker_threads`);

const project = {
  root: __dirname,
  namespaces: {},
};

/** @typedef {import('./packages/esfuse/sources/Project').Project} Project */
/** @typedef {keyof {[K in keyof Project as Project[K] extends (...args: Array<any>) => any ? K : never]: K;}} ProjectMethods */

class WorkerClient {
  // These two require() calls are in deferred so that they are not imported in
  // older Node.js versions (which don't support workers).
  // TODO: Hoist them in Babel 8.

  signal = new Int32Array(new SharedArrayBuffer(4));

  /**
   * @template {ProjectMethods} T
   * @param {T} fn 
   * @param {Parameters<Project[T]>} args 
   * @returns {Awaited<ReturnType<Project[T]>>}
   */
  send(fn, args) {
    this.signal[0] = 0;
    const subChannel = new MessageChannel();

    if (!parentPort)
      throw new Error(`Assertion failed: Cannot call send from a non-worker`);

    parentPort.postMessage(
      {type: `async`, signal: this.signal, port: subChannel.port1, fn, args},
      [subChannel.port1],
    );

    Atomics.wait(this.signal, 0, 0);
    const recv = receiveMessageOnPort(subChannel.port2);
    if (!recv)
      throw new Error(`Assertion failed: Expected a message to be received`);

    if (recv.message.error) {
      throw Object.assign(recv.message.error, recv.message.errorData);
    } else {
      return recv.message.result;
    }
  }
}

const client = new WorkerClient();

addHook((code, filename) => {
  const res = client.send(`transform`, [filename]);
  if (res.error)
    throw new Error(`Failed to compile the original module`);
  return Buffer.from(res.body).toString();
}, {
  exts: [`.ts`, `.tsx`, `.js`, `.jsx`],
});
