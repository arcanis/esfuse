import React from 'react';

export function DocumentationNav() {
  return (
    <nav className={`hidden xl:block w-48 shrink-0`}>
      <div className={`fixed bottom-0 h-[calc(100vh-5rem)] w-48 overflow-y-auto pt-32 pb-8 no-scrollbar`}>
        <div className={`border-l border-slate-200 dark:border-slate-800`}>
          <div className={`text-xs font-[650] text-slate-400 uppercase pl-4 py-1.5 dark:text-slate-200`}>On this page</div>
          <ul className={`text-sm`}>
            <li>
              {/* The data-scrollspy-link attribute makes the scrollspy work */}
              <a data-scrollspy-link className={`relative block font-normal text-slate-600 pl-4 py-1.5 before:absolute before:-left-px before:top-2 before:bottom-2 before:w-0.5`} href={`#basics`}>
                The basics
              </a>
            </li>
            <li>
              {/* The data-scrollspy-link attribute makes the scrollspy work */}
              <a data-scrollspy-link className={`relative block font-normal text-slate-600 pl-4 py-1.5 before:absolute before:-left-px before:top-2 before:bottom-2 before:w-0.5`} href={`#api-structure`}>
                API structure
              </a>
            </li>
            <li>
              {/* The data-scrollspy-link attribute makes the scrollspy work */}
              <a data-scrollspy-link className={`relative block font-normal text-slate-600 pl-4 py-1.5 before:absolute before:-left-px before:top-2 before:bottom-2 before:w-0.5`} href={`#connect-api`}>
                Connecting to the API
              </a>
            </li>
            <li>
              {/* The data-scrollspy-link attribute makes the scrollspy work */}
              <a data-scrollspy-link className={`relative block font-normal text-slate-600 pl-4 py-1.5 before:absolute before:-left-px before:top-2 before:bottom-2 before:w-0.5`} href={`#api-limits`}>
                API limits
              </a>
            </li>
          </ul>
        </div>
      </div>
    </nav>
  );
}
