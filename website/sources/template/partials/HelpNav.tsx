import React from 'react';

export function HelpNav() {
  return (
    <nav className={`hidden xl:block w-48 shrink-0`}>
      <div className={`fixed bottom-0 h-[calc(100vh-5rem)] w-48 overflow-y-auto pt-32 pb-8 no-scrollbar`}>
        <div className={`border-l border-slate-200 dark:border-slate-800`}>
          <div className={`text-xs font-[650] text-slate-400 uppercase pl-4 py-1.5 dark:text-slate-200`}>On this page</div>
          <ul className={`text-sm`}>
            <li>
              {/* The data-scrollspy-link attribute makes the scrollspy work */}
              <a data-scrollspy-link className={`relative block pl-4 py-1.5 before:absolute before:-left-px before:top-2 before:bottom-2 before:w-0.5`} href={`#overview`}>
                Overview
              </a>
            </li>
            <li>
              {/* The data-scrollspy-link attribute makes the scrollspy work */}
              <a data-scrollspy-link className={`relative block pl-4 py-1.5 before:absolute before:-left-px before:top-2 before:bottom-2 before:w-0.5`} href={`#support`}>
                Support
              </a>
            </li>
            <li>
              {/* The data-scrollspy-link attribute makes the scrollspy work */}
              <a data-scrollspy-link className={`relative block pl-4 py-1.5 before:absolute before:-left-px before:top-2 before:bottom-2 before:w-0.5`} href={`#security`}>
                Security & Privacy
              </a>
            </li>
            <li>
              {/* The data-scrollspy-link attribute makes the scrollspy work */}
              <a data-scrollspy-link className={`relative block pl-4 py-1.5 before:absolute before:-left-px before:top-2 before:bottom-2 before:w-0.5`} href={`#troubleshooting`}>
                Troubleshooting
              </a>
            </li>
          </ul>
        </div>
      </div>
    </nav>
  );
}
