import {MarkdownToc} from 'esfuse/client';

export function DocumentationNav({toc}: {toc: Array<MarkdownToc>}) {
  const links: Array<React.ReactNode> = [];

  function traverseTocNode(node: MarkdownToc) {
    links.push(
      <li key={node.id}>
        {/* The data-scrollspy-link attribute makes the scrollspy work */}
        <a data-scrollspy-link className={`relative block font-normal text-slate-600 pl-4 py-1.5 before:absolute before:-left-px before:top-2 before:bottom-2 before:w-0.5`} href={`#${node.id}`}>
          {node.name}
        </a>
      </li>,
    );

    for (const child of node.children) {
      traverseTocNode(child);
    }
  }

  for (const node of toc)
    traverseTocNode(node);

  return (
    <nav className={`hidden xl:block w-48 shrink-0`}>
      <div className={`fixed bottom-0 h-[calc(100vh-5rem)] w-48 overflow-y-auto pt-32 pb-8 no-scrollbar`}>
        <div className={`border-l border-slate-200 dark:border-slate-800`}>
          <div className={`text-xs font-[650] text-slate-400 uppercase pl-4 py-1.5 dark:text-slate-200`}>On this page</div>
          <ul className={`text-sm`}>
            {links}
          </ul>
        </div>
      </div>
    </nav>
  );
}
