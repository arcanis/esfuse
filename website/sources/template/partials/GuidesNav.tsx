import React from 'react';

export function GuidesNav() {
  return (
    <nav className={`hidden xl:block w-48 shrink-0`}>
      <div className={`fixed bottom-0 h-[calc(100vh-5rem)] w-48 overflow-y-auto pt-32 pb-8 no-scrollbar`}>
        <div className={`border-l border-slate-200 dark:border-slate-800`}>
          <div className={`text-xs font-[650] text-slate-400 uppercase pl-4 py-1.5 dark:text-slate-200`}>On this page</div>
          <ul className={`text-sm`}>
            <li>
              {/* The data-scrollspy-link attribute makes the scrollspy work */}
              <a data-scrollspy-link className={`relative block font-normal text-slate-600 pl-4 py-1.5 before:absolute before:-left-px before:top-2 before:bottom-2 before:w-0.5`} href={`#at-a-glance`}>
                At a glance
              </a>
            </li>
            <li>
              {/* The data-scrollspy-link attribute makes the scrollspy work */}
              <a data-scrollspy-link className={`relative block font-normal text-slate-600 pl-4 py-1.5 before:absolute before:-left-px before:top-2 before:bottom-2 before:w-0.5`} href={`#generate-api`}>
                Generate your API key
              </a>
            </li>
            <li>
              {/* The data-scrollspy-link attribute makes the scrollspy work */}
              <a data-scrollspy-link className={`relative block font-normal text-slate-600 pl-4 py-1.5 before:absolute before:-left-px before:top-2 before:bottom-2 before:w-0.5`} href={`#install`}>
                Install the client library
              </a>
            </li>
            <li>
              {/* The data-scrollspy-link attribute makes the scrollspy work */}
              <a data-scrollspy-link className={`relative block font-normal text-slate-600 pl-4 py-1.5 before:absolute before:-left-px before:top-2 before:bottom-2 before:w-0.5`} href={`#api-call`}>
                Make your first API call
              </a>
            </li>
            <li>
              {/* The data-scrollspy-link attribute makes the scrollspy work */}
              <a data-scrollspy-link className={`relative block font-normal text-slate-600 pl-4 py-1.5 before:absolute before:-left-px before:top-2 before:bottom-2 before:w-0.5`} href={`#config-status`}>
                Configuration status
              </a>
            </li>
          </ul>
        </div>
      </div>
    </nav>
  );
}
