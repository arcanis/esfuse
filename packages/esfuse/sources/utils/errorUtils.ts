import {SourceMapConsumer}                    from 'source-map';
import {StackFrame, parse as parseStacktrace} from 'stacktrace-parser';

import {CompilationError, Highlight}          from '@esfuse/compiler';

export type {Diagnostic, Highlight, Span, CompilationError} from '@esfuse/compiler';

export function normalize(error: Error): CompilationError {
  if (error instanceof $esfuse$.CompilationError)
    return error.error;

  return {
    diagnostics: [{
      message: error.message,
      highlights: parseStacktrace(error.stack!).map(frame => ({
        source: frame.file?.replace(/^.*?(\/_dev\/)/, `$1`),
        subject: error.name,
        span: {
          start: {
            row: frame.lineNumber!,
            col: frame.column!,
          },
          end: {
            row: frame.lineNumber!,
            col: frame.column!,
          },
        },
      })),
    }],
  };
}

export async function normalizeWithSourceMaps(error: Error): Promise<{error: CompilationError, sources: Map<string, string>}> {
  const {
    resolvedSources,
    resolvedStack,
  } = await resolveStack(parseStacktrace(error.stack!));

  return {
    sources: resolvedSources,
    error: {
      diagnostics: [{
        message: error.message,
        highlights: resolvedStack.map(frame => ({
          source: frame.file?.replace(/^.*?(\/_dev\/)/, `$1`),
          subject: frame.methodName,
          span: {
            start: {
              row: frame.lineNumber!,
              col: frame.column!,
            },
            end: {
              row: frame.lineNumber!,
              col: frame.column!,
            },
          },
        })),
      }],
    },
  };
}

async function resolveStack(stack: Array<StackFrame>) {
  // @ts-expect-error
  const {default: sourceMapWasmUrl} = await import(`source-map/lib/mappings.wasm?transform=url`);

  // @ts-expect-error
  await SourceMapConsumer.initialize({
    [`lib/mappings.wasm`]: sourceMapWasmUrl,
  });

  const httpEntries = stack.filter(entry => {
    return entry.file?.match(/^https?:\/\//);
  });

  const sourceFileSet = new Set(httpEntries.map(entry => {
    return entry.file!;
  }));

  const resolvedSourceMaps = new Map(await Promise.all([...sourceFileSet].map(async sourceFile => {
    let content: string | null = null;
    try {
      content = await (await fetch(sourceFile)).text();
    } catch (err) {
      console.log(`Failed to get the content from ${sourceFile}:`, err);
    }

    let sourceMap: SourceMapConsumer | null = null;
    if (content !== null) {
      const sourceMapTag = content.match(/^\/\/# sourceMappingURL=([^\n]+)/m);
      if (sourceMapTag) {
        const sourceMapUrl = sourceMapTag[1];

        try {
          const res = await fetch(sourceMapUrl);
          const content = await res.json();

          sourceMap = await new SourceMapConsumer(content);
        } catch (err) {
          console.log(`Failed to get the source map from ${sourceMapUrl}:`, err);
        }
      } else {
        console.log(`No source map tag found in ${sourceFile}`);
      }
    }

    console.log(`Resolved source maps for ${sourceFile}`, {content, sourceMap});

    return [sourceFile, {
      content,
      sourceMap,
    }] as const;
  })));

  const resolvedSources = new Map<string, string>();

  const resolvedStack = stack.map(entry => {
    if (typeof entry.file !== `string`)
      return {...entry, source: null};

    const resolvedSourceMap = resolvedSourceMaps.get(entry.file);
    if (typeof resolvedSourceMap === `undefined` || resolvedSourceMap.content === null)
      return {...entry, source: null};

    if (!resolvedSourceMap.sourceMap || entry.lineNumber === null || entry.column === null)
      return {...entry, source: resolvedSourceMap.content};

    const resolvedPosition = resolvedSourceMap.sourceMap.originalPositionFor({
      line: entry.lineNumber,
      column: entry.column,
    });

    // SWC adds `<...>` around the file names because we use FileName::Custom; we
    // need to remove them when they target a devserver url
    const cleanedFileName = resolvedPosition.source?.replace(/^<(\/_dev\/.*)>$/, `$1`) ?? null;

    if (cleanedFileName && resolvedPosition.source !== null) {
      if (!resolvedSources.has(cleanedFileName)) {
        const res = resolvedSourceMap.sourceMap.sourceContentFor(resolvedPosition.source);
        if (res) {
          resolvedSources.set(cleanedFileName, res);
        }
      }
    }

    return {
      file: cleanedFileName,
      lineNumber: resolvedPosition.line,
      column: resolvedPosition.column,
      methodName: resolvedPosition.name ?? entry.methodName,
      arguments: entry.arguments,
    };
  });

  for (const {sourceMap} of resolvedSourceMaps.values())
    sourceMap?.destroy();

  return {
    resolvedStack,
    resolvedSources,
  };
}

const HIDDEN_PACKAGES = new Set([
  `@remix-run/router`,
  `runtime`,
]);

export type CategorizedHighlights = ReturnType<typeof categorizeStack>;
export type CategorizedHighlight = CategorizedHighlights[number][`highlights`][number];

export function categorizeStack(highlights: Array<Highlight>) {
  const extendedHighlights = highlights.map(entry => {
    let packageName: string | null = null;

    let packageFile = entry.source?.replace(/^\/_dev\/file\//, ``) ?? null;
    if (packageFile === `<anonymous>`)
      packageFile = null;

    const nmMatch = packageFile?.match(/^.*?\/node_modules\/((?:@[^/]*\/)?[^/]+)\/(.*)+$/);
    if (nmMatch) {
      packageName = nmMatch[1];
      packageFile = nmMatch[2];
    }

    if (packageName === null && packageFile === `runtime`) {
      packageName = `runtime`;
      packageFile = ``;
    }

    return {...entry, packageName, packageFile};
  });

  const cleanedHighlights: Array<{
    type: `user` | `extern`;
    highlights: Array<(typeof extendedHighlights)[number]>;
  }> = [];

  let wasExtern: boolean | undefined;

  for (let t = extendedHighlights.length - 1; t >= 0; --t) {
    const entry = extendedHighlights[t];

    let isExtern = false;
    if (typeof entry.packageName === `string`) {
      if (HIDDEN_PACKAGES.has(entry.packageName)) {
        isExtern = true;
        continue;
      }
    }

    if (wasExtern ?? true) {
      if (entry.packageFile === null) {
        isExtern = true;
      }
    }

    // Those entries are created by Esfuse
    if (entry.subject === `Object.factory`)
      isExtern = true;

    if (isExtern !== wasExtern) {
      cleanedHighlights.unshift({
        type: isExtern ? `extern` : `user`,
        highlights: [],
      });
    }

    cleanedHighlights[0].highlights.unshift(entry);
  }

  return cleanedHighlights;
}
