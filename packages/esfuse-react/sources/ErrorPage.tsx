import Editor, {Monaco}              from '@monaco-editor/react';
import type * as monaco              from 'monaco-editor';
import {useRouteError}               from 'react-router-dom';
import {useEffect, useRef, useState} from 'react';

import {styles}                      from '@esfuse/react/sources/ErrorPage.module.css';

import {errorUtils}                  from 'esfuse/client';

function StackSeparator() {
  return <div className={styles.stackSeparator}/>;
}

function Highlight({highlight, onClick, isActive}: {highlight: errorUtils.CategorizedHighlight, onClick: () => void, isActive: boolean}) {
  return (
    <div className={styles.stackFrame} onClick={() => onClick()} data-isactive={isActive}>
      {highlight.packageFile !== null && (
        <div className={styles.stackFrameSource}>
          {highlight.packageName !== null && (
            <div className={styles.packageName}>
              {highlight.packageName}
            </div>
          )}
          {highlight.span ? (
            <>{highlight.packageFile}:{highlight.span.start.row}</>
          ) : (
            <>{highlight.packageFile}</>
          )}
        </div>
      )}
      <div>
        {highlight.subject}
      </div>
    </div>
  );
}

function CategorizedHighlights({categorizedHighlights, onChange, activeFrame}: {categorizedHighlights: errorUtils.CategorizedHighlights, onChange: (frame: errorUtils.CategorizedHighlight) => void, activeFrame: errorUtils.CategorizedHighlight | null}) {
  const rows: Array<React.ReactNode> = [];
  let firstFrame: errorUtils.CategorizedHighlight | null = null;

  for (const {type, highlights} of categorizedHighlights) {
    if (type === `extern`) {
      rows.push(<StackSeparator key={rows.length}/>);
    } else {
      for (const entry of highlights) {
        rows.push(<Highlight key={rows.length} highlight={entry} onClick={() => onChange(entry)} isActive={activeFrame === entry}/>);
        firstFrame ??= entry;
      }
    }
  }

  useEffect(() => {
    firstFrame && onChange(firstFrame);
  }, []);

  return <>
    {rows}
  </>;
}

function HighlightContainer({highlights, sources}: {highlights: Array<errorUtils.Highlight>, sources: Map<string, string>}) {
  const [context, setContext] = useState<{
    monaco: typeof monaco;
    editor: monaco.editor.IStandaloneCodeEditor;
  } | null>(null);

  const decorationRef = useRef<Array<string>>([]);

  const categorizedHighlights = errorUtils.categorizeStack(highlights);
  const highlightRef = useRef(categorizedHighlights[0]?.highlights[0] ?? null);

  const highlightLine = (highlight: errorUtils.CategorizedHighlight) => {
    if (!context)
      return;

    if (typeof highlight.source !== `string`)
      return;

    const editorContent = sources.get(highlight.source);
    if (typeof editorContent === `undefined`)
      return;

    const model = context.monaco.editor.createModel(editorContent, `typescript`);
    const highlightInfo: Array<monaco.editor.IModelDeltaDecoration> = [];

    const lineNumber = highlight.span?.start.row ?? null;
    if (lineNumber !== null) {
      highlightInfo.push({
        range: new context.monaco.Range(lineNumber, 1, lineNumber, 1),
        options: {isWholeLine: true, className: styles.highlightedLine},
      });
    }

    const oldModel = context.editor.getModel();
    context.editor.setModel(model);
    oldModel?.dispose();

    context.editor.deltaDecorations(decorationRef.current, highlightInfo);
    context.editor.revealLineNearTop(lineNumber ?? 1);

    highlightRef.current = highlight;
  };

  const registerEditor = (editor: monaco.editor.IStandaloneCodeEditor, monaco: Monaco) => {
    monaco.languages.typescript.typescriptDefaults.setDiagnosticsOptions({
      noSemanticValidation: true,
      noSyntaxValidation: true,
    });

    setContext({
      monaco,
      editor,
    });

    if (highlightRef.current) {
      highlightLine(highlightRef.current);
    }
  };

  return (
    <div className={styles.stackTrace}>
      <div className={styles.stackTraceLeft}>
        <div className={styles.stackFrames}>
          <CategorizedHighlights categorizedHighlights={categorizedHighlights} onChange={highlightLine} activeFrame={highlightRef.current}/>
        </div>
      </div>
      <div className={styles.stackTraceRight}>
        <Editor onMount={registerEditor} theme={`vs-dark`}/>
      </div>
    </div>
  );
}

function DiagnosticPage({diagnostic, sources}: {diagnostic: errorUtils.Diagnostic, sources: Map<string, string>}) {
  return (
    <div className={styles.container}>
      <div className={styles.exceptionContainer}>
        {diagnostic.message}
      </div>
      <HighlightContainer highlights={diagnostic.highlights} sources={sources}/>
    </div>
  );
}

export function ErrorPage() {
  const routeErr = useRouteError();

  const dummyErr: errorUtils.CompilationError = routeErr instanceof Error
    ? errorUtils.normalize(routeErr)
    : routeErr as any;

  const [
    normalizedError,
    setNormalizedError,
  ] = useState<React.ReactNode>(<DiagnosticPage diagnostic={dummyErr.diagnostics[0]} sources={new Map()}/>);

  useEffect(() => {
    if (routeErr instanceof Error) {
      errorUtils.normalizeWithSourceMaps(routeErr).then(normalizedError => {
        setNormalizedError(<DiagnosticPage diagnostic={normalizedError.error.diagnostics[0]} sources={normalizedError.sources}/>);
      });
    }
  }, []);

  return normalizedError;
}
