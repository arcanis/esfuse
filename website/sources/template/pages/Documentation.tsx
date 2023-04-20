import {useLoaderData}    from 'react-router-dom';
import React, {useState}  from 'react';
import Illustration       from 'website/sources/template/images/hero-illustration.svg';
import {DocumentationNav} from 'website/sources/template/partials/DocumentationNav';
import {Header}           from 'website/sources/template/partials/Header';
import {MenuButton}       from 'website/sources/template/partials/MenuButton';
import {PageFooter}       from 'website/sources/template/partials/PageFooter';
import {PageNavigation}   from 'website/sources/template/partials/PageNavigation';
import {Sidebar}          from 'website/sources/template/partials/Sidebar';

const components: Record<string, React.FunctionComponent<{children?: React.ReactNode}>> = {
  [`h1`]: props => {
    return <h1 className={`text-4xl font-[650] text-slate-800 dark:text-slate-200`} {...props}/>;
  },
  [`h2`]: props => {
    return <h2 className={`text-2xl font-[650] text-slate-800 dark:text-slate-200`} {...props}/>;
  },
  [`h3`]: props => {
    return <h3 className={`text-xl font-[650] text-slate-800 dark:text-slate-200`} {...props}/>;
  },
  [`a`]: props => {
    return <a className={`text-blue-600 font-medium hover:underline`} {...props}/>;
  },
  [`ul`]: props => {
    return <ul className={`list-disc list-inside space-y-2`} {...props}/>;
  },
  [`li`]: props => {
    return <li {...props}/>;
  },
  [`callout`]: props => {
    return (
      <div className={`text-sm p-4 bg-slate-50 border border-slate-200 rounded dark:bg-slate-800 dark:border-slate-700`}>
        <div className={`flex items-center leading-6`}>
          <svg className={`fill-purple-500 shrink-0 mr-4`} width={`16`} height={`16`} viewBox={`0 0 16 16`} xmlns={`http://www.w3.org/2000/svg`}>
            <path d={`M8 0C3.6 0 0 3.6 0 8s3.6 8 8 8 8-3.6 8-8-3.6-8-8-8Zm0 12c-.6 0-1-.4-1-1s.4-1 1-1 1 .4 1 1-.4 1-1 1Zm1-3H7V4h2v5Z`} />
          </svg>
          <div className={`space-y-4`}>
            {props.children}
          </div>
        </div>
      </div>
    );
  },
};

export function Documentation() {
  const {MDXContent} = useLoaderData() as any;
  const [sidebarOpen, setSidebarOpen] = useState(false);

  const prevArticle = null;
  const nextArticle = {
    title: `Methods and Parameters`,
    link: `#0`,
  };

  return (
    <div className={`flex flex-col min-h-screen overflow-hidden`}>
      {/*  Site header */}
      <Header />

      {/*  Page content */}
      <main className={`grow`}>
        <section className={`relative`}>
          <div className={`absolute top-0 left-1/2 -translate-x-1/2 pointer-events-none -z-10`}>
            <img className={`max-w-none`} src={Illustration} width={`1972`} height={`392`} aria-hidden={`true`} />
          </div>

          <div className={`max-w-7xl mx-auto px-4 sm:px-6`}>
            {/* Main content */}
            <div>
              {/* Sidebar */}
              <Sidebar sidebarOpen={sidebarOpen} setSidebarOpen={setSidebarOpen} />

              {/* Page container */}
              <div className={`md:grow md:pl-64 lg:pr-6 xl:pr-0`}>
                <div className={`pt-24 md:pt-28 pb-8 md:pl-6 lg:pl-12`}>
                  <article className={`flex xl:space-x-12`}>

                    {/* Main area */}
                    <div className={`min-w-0`}>

                      {/* Mobile hamburger + breadcrumbs */}
                      <div className={`md:hidden flex items-center mb-8`}>
                        {/* Hamburger button */}
                        <MenuButton sidebarOpen={sidebarOpen} setSidebarOpen={setSidebarOpen} />
                        {/* Breadcrumbs */}
                        <div className={`flex items-center text-sm whitespace-nowrap min-w-0 ml-3`}>
                          <span className={`text-slate-600 dark:text-slate-400`}>Documentation</span>
                          <svg className={`fill-slate-400 shrink-0 mx-2 dark:fill-slate-500`} width={`8`} height={`10`} xmlns={`http://www.w3.org/2000/svg`}>
                            <path d={`M1 2 2.414.586 6.828 5 2.414 9.414 1 8l3-3z`} />
                          </svg>
                          <span className={`text-slate-800 font-medium truncate dark:text-slate-200`}>Fundamentals</span>
                        </div>
                      </div>

                      {/* Article content */}
                      <div className={`space-y-6 text-slate-600 dark:text-slate-400`}>
                        <MDXContent components={components}/>
                      </div>

                      {/* Page navigation */}
                      <PageNavigation prevArticle={prevArticle} nextArticle={nextArticle} />

                      {/* Content footer */}
                      <PageFooter />

                    </div>

                    {/* Secondary navigation */}
                    <DocumentationNav />

                  </article>

                </div>
              </div>

            </div>

          </div>

        </section>

      </main>

    </div>
  );
}
