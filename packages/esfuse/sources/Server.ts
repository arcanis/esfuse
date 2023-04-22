import {selectOne}                          from 'css-select';
import {render as stringifyDocument}        from 'dom-serializer';
import {Element}                            from 'domhandler';
import fs                                   from 'fs';
import {parseDocument}                      from 'htmlparser2';
import {Server as HttpServer, createServer} from 'http';
import {AddressInfo}                        from 'net';
import path                                 from 'path';
import {WebSocketServer}                    from 'ws';

import {ServerConfig}                       from 'esfuse/sources/Config';
import {Project}                            from 'esfuse/sources/Project';
import {Router}                             from 'esfuse/sources/Router';
import * as nodeUtils                       from 'esfuse/sources/utils/nodeUtils';

export type Request = {
  method: string;
  url: URL;
  body: Buffer | null;
};

export type Response = {
  code?: number;
  headers?: Record<string, string>;
  body?: Buffer;
};

export class Server {
  router: Router;

  constructor(public project: Project, public server: ServerConfig) {
    this.router = new Router(project, server);
    this.router.init();
  }

  endpoints = {
    [`GET:/_dev/bundle`]: this.bundleHandler,
    [`GET:/_dev/file`]: this.fileHandler,
    [`GET:/_dev/internal/runtime`]: this.runtimeHandler,
    [`GET:/_dev/internal/tailwind`]: this.tailwindHandler,
  };

  http?: HttpServer;
  ws = new WebSocketServer({
    noServer: true,
  });

  broadcast(data: string) {
    for (const client of this.ws.clients) {
      client.send(data);
    }
  }

  async handle(req: Request) {
    let handler: ((req: Request) => Promise<Response>) | undefined;

    const endpoints = Object.keys(this.endpoints);
    for (let t = 0; t < endpoints.length; ++t) {
      const endpoint = endpoints[t] as keyof Server[`endpoints`];

      const pathname = `${req.method}:${req.url.pathname}`;
      if (pathname !== endpoint && !pathname.startsWith(`${endpoint}/`))
        continue;

      const subPath = pathname.slice(endpoints[t].length) || `/`;
      handler = req => this.endpoints[endpoint].call(this, req, subPath);

      break;
    }

    if (!handler) {
      if (req.method === `GET` && !req.url.pathname.match(/\/_dev(\/|$)/)) {
        handler = req => this.catchAll(req);
      } else {
        handler = this.error(404, `Route not found (${req.method}:${req.url.pathname})`);
      }
    }

    let handlerRes: Response;
    try {
      handlerRes = await handler(req);
    } catch (err: any) {
      handlerRes = {code: 500, body: Buffer.from(JSON.stringify({stack: err.stack.split(/\n/)}, null, 2)), headers: {[`Content-Type`]: `application/json`}};
    }

    return handlerRes;
  }

  listen() {
    const http = this.http = createServer(async (req, res) => {
      if (typeof req.method === `undefined`)
        throw new Error(`Assertion failed: Missing method`);

      const body = req.method === `POST`
        ? await nodeUtils.consumeStream(req)
        : null;

      const parsed = new URL(req.url!, `http://${req.headers.host}`);
      const handlerRes = await this.handle({method: req.method, url: parsed, body});

      res.writeHead(handlerRes.code ?? 200, handlerRes.headers);
      res.end(handlerRes.body);
    });

    const unwatch = this.project.watch(e => {
      const changes = [...e.changes].map(([subject, action]) => {
        const locator = typeof subject === `string`
          ? this.project.locatorFromPath(path.join(this.project.root, subject))
          : subject;

        return [locator?.url, action];
      }).filter(([p]) => {
        return !!p;
      });

      if (changes.length === 0)
        return;

      this.broadcast(JSON.stringify({
        type: `watch`,
        changes,
      }));
    });

    this.http.on(`upgrade`, (request, socket, head) => {
      if (request.url === `/_dev/ws`) {
        this.ws.handleUpgrade(request, socket, head, ws => {
          this.ws.emit(`connection`, ws, request);
        });
      } else {
        socket.destroy();
      }
    });

    this.http.on(`close`, () => {
      unwatch();
    });

    return new Promise<AddressInfo>(resolve => {
      http.listen(8080, () => {
        resolve(http.address() as AddressInfo);
      });
    });
  }

  error(code: number, message: string) {
    return async () => ({
      code,
      body: Buffer.from(
        JSON.stringify({
          stack: [message],
        }, null, 2),
      ),
    });
  }

  async catchAll(req: Request) {
    const {template, script} = this.router.lookup(req.url.pathname);
    const scriptLocator = script && this.project.locatorFromPath(script);

    const templateText = await fs.promises.readFile(template, `utf8`);
    const templateDom = parseDocument(templateText);

    const head = selectOne(`head`, templateDom);
    if (head) {
      head.childNodes.push(new Element(`script`, {
        defer: `true`,
        src: `/_dev/internal/runtime/base`,
      }));

      head.childNodes.push(new Element(`script`, {
        defer: `true`,
        src: `/_dev/internal/runtime/hmr`,
      }));

      if (this.server.pageFolder !== null) {
        const tailwindPath = await this.project.tailwind.find(path.join(this.project.root, this.server.pageFolder));
        if (tailwindPath) {
          head.childNodes.push(new Element(`script`, {
            defer: `true`,
            src: path.posix.join(`/_dev/internal/tailwind`, tailwindPath),
          }));
        }
      }

      if (scriptLocator) {
        head.childNodes.push(new Element(`script`, {
          defer: `true`,
          src: scriptLocator.url.replace(/^\/_dev\/file\//, `/_dev/bundle/`),
        }));
      }
    }

    return this.renderTransformResult({
      value: {
        mimeType: `text/html`,
        code: stringifyDocument(templateDom),
      },
      error: null,
    });
  }

  async bundleHandler(req: Request): Promise<Response> {
    const locator = this.project.locatorFromUrl(req.url.pathname.replace(/\.map$/, ``).replace(/^\/_dev\/bundle\//, `/_dev/file/`) + req.url.search)!;
    const res = await this.project.bundle(locator, {requireOnLoad: true, userData: this.getUserData()});

    if (res.value && req.url.pathname.endsWith(`.map`))
      Object.assign(res.value, {mimeType: `application/json`, code: res.value.map});

    return this.renderTransformResult(res);
  }

  async fileHandler(req: Request): Promise<Response> {
    const locator = this.project.locatorFromUrl(req.url.pathname.replace(/\.map$/, ``) + req.url.search)!;
    const res = await this.project.bundle(locator, {requireOnLoad: true, userData: this.getUserData(), onlyEntryPoint: true});

    if (res.value && req.url.pathname.endsWith(`.map`))
      Object.assign(res.value, {mimeType: `application/json`, code: res.value.map});

    return this.renderTransformResult(res);
  }

  async tailwindHandler(req: Request, subPath: string): Promise<Response> {
    const url = `${path.posix.join(`/_dev/internal/tailwind`, subPath)}?transform=js`;

    const locator = this.project.locatorFromUrl(url)!;
    const res = await this.project.bundle(locator, {requireOnLoad: true});

    return this.renderTransformResult(res);
  }

  async runtimeHandler(req: Request, subPath: string): Promise<Response> {
    const locator = this.project.locatorFromPath(require.resolve(`./runtimes${subPath}.ts`))!;

    const res = subPath === `/base`
      ? await this.project.transform(locator)
      : await this.project.bundle(locator, {requireOnLoad: true});

    return this.renderTransformResult(res);
  }

  renderTransformResult(res: {value: {mimeType: string, code: string}, error: null} | {value: null, error: any}) {
    if (res.value) {
      const body = res.value.mimeType.startsWith(`text/`) || res.value.mimeType === `application/json`
        ? Buffer.from(res.value.code)
        : Buffer.from(res.value.code, `base64`);

      return {
        code: 200,
        headers: {[`Content-Type`]: res.value.mimeType},
        body,
      };
    } else {
      return {
        code: 500,
        headers: {[`Content-Type`]: `application/json`},
        body: Buffer.from(JSON.stringify(res.error, null, 2)),
      };
    }
  }

  private getUserData() {
    return {
      pageFolder: this.server.pageFolder,
    };
  }
}
