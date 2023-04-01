import {TransformErr, TransformOk, resolveToPath}                      from '@esfuse/compiler';
import {Server as HttpServer, createServer} from 'http';
import fs from 'fs';
import {parseDocument} from 'htmlparser2';
import {Element} from 'domhandler';
import {selectOne, selectAll} from 'css-select';
import {render as stringifyDocument} from 'dom-serializer';
import {WebSocketServer}                    from 'ws';

import {Project}                            from './Project';
import * as esfuseUtils                     from './utils/esfuseUtils';
import * as miscUtils                       from './utils/miscUtils';
import * as nodeUtils                       from './utils/nodeUtils';
import { ServerConfig } from './Config';
import { Router } from './Router';
import { AddressInfo } from 'net';

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
    [`GET:/_dev/file`]: this.fileHandler,
    [`GET:/_dev/internal/runtime`]: this.runtimeHandler,
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
      handler = req => this.endpoints[endpoint].call(this, req);

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
      this.broadcast(JSON.stringify({
        type: `watch`,
        changes: [...e.changes].map(([p, action]) => {
          return [esfuseUtils.getPublicPath(p, {root: this.project.root}), action];
        }),
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
    const scriptUrl = script && this.project.pathToUrl(script);

    let tailwind: string | null = null;
    try {
      tailwind = script && nodeUtils.findClosestFile(script, `tailwind.config.ts`);
    } catch {}

    const templateText = await fs.promises.readFile(template, `utf8`);
    const templateDom = parseDocument(templateText);

    if (script && scriptUrl) {
      const head = selectOne(`head`, templateDom);
      if (head) {
        head.childNodes.push(new Element(`script`, {
          defer: `true`,
          src: `/_dev/internal/runtime`,
        }));

        head.childNodes.push(new Element(`script`, {
          defer: `true`,
          src: scriptUrl,
        }));

        head.childNodes.push(new Element(`script`, {
          defer: `true`,
          src: `data:application/javascript,$esfuse$.require(${JSON.stringify(scriptUrl)})`,
        }));
      }
    }

    return this.renderTransformResult({
      value: {
        mimeType: `text/html`,
        code: stringifyDocument(templateDom),
        imports: [],
      },
      error: null,
    });
  }

  async fileHandler(req: Request): Promise<Response> {
    return await miscUtils.route(req.url.searchParams.get(`type`) ?? `bundle`, {
      bundle: async () => {
        const res = await this.project.devBundle(req.url.pathname);

        return {
          code: 200,
          headers: {[`Content-Type`]: res.mimeType},
          body: Buffer.from(res.code),
        };
      },
    });
  }

  async runtimeHandler(): Promise<Response> {
    const res = await this.project.transform(require.resolve(`./runtime`));

    return this.renderTransformResult(res);
  }

  renderTransformResult(res: TransformOk | TransformErr) {
    if (res.value) {
      return {
        code: 200,
        headers: {[`Content-Type`]: res.value.mimeType},
        body: Buffer.from(res.value.code),
      };    
    } else {
      return {
        code: 500,
        headers: {[`Content-Type`]: `application/json`},
        body: Buffer.from(JSON.stringify(res.error, null, 2)),
      };
    }
  }
}
