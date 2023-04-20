import {NavLink, useLocation}     from 'react-router-dom';
import React, {useRef, useEffect} from 'react';
import {useCloseInteractions}     from 'website/sources/template/hooks/useCloseInteractions';
import {SidebarLinkGroup}         from 'website/sources/template/utils/SidebarLinkGroup';
import {SidebarLinkSubgroup}      from 'website/sources/template/utils/SidebarLinkSubgroup';
import {Transition}               from 'website/sources/template/utils/Transition';

export type SidebarProps = {
  sidebarOpen: boolean;
  setSidebarOpen: (status: boolean) => void;
};

export function Sidebar({
  sidebarOpen,
  setSidebarOpen,
}: SidebarProps) {
  const location = useLocation();
  const {pathname} = location;

  const sidebarRef = useRef<HTMLDivElement | null>(null);

  useCloseInteractions(sidebarOpen, setSidebarOpen, {
    containerRef: sidebarRef,
  });

  const navLinkClassName = ({isActive}: {isActive: boolean}) =>
    `flex items-center space-x-3 font-medium ${isActive ? `text-blue-600` : `text-slate-800 dark:text-slate-200`}`;

  const topLevelLinkClassName = (path: string) => `relative flex items-center font-[650] text-slate-800 p-1 before:absolute before:inset-0 before:rounded before:bg-gradient-to-tr before:from-blue-400 before:to-purple-500 before:opacity-20 before:-z-10 before:pointer-events-none dark:text-slate-200 ${
    !pathname.includes(path) && `before:hidden`
  }`;

  return (
    <div>
      <Transition
        className={`md:hidden fixed inset-0 z-10 bg-slate-900 bg-opacity-20 transition-opacity`}
        aria-hidden={`true`}

        show={sidebarOpen}

        enter={`transition ease-out duration-200`}
        enterStart={`opacity-0`}
        enterEnd={`opacity-100`}

        leave={`transition ease-out duration-100`}
        leaveStart={`opacity-100`}
        leaveEnd={`opacity-0`}
      />

      <div ref={sidebarRef}>
        <Transition
          tag={`aside`}
          id={`sidebar`}
          className={`fixed left-0 top-0 bottom-0 w-64 h-screen border-r border-slate-200 md:left-auto md:shrink-0 z-10 md:!opacity-100 md:!block dark:border-slate-800 dark:bg-slate-900`}

          show={sidebarOpen}

          enter={`transition ease-out duration-200 transform`}
          enterStart={`opacity-0 -translate-x-full`}
          enterEnd={`opacity-100 translate-x-0`}

          leave={`transition ease-out duration-200`}
          leaveStart={`opacity-100`}
          leaveEnd={`opacity-0`}
        >
          {/* Gradient bg displaying on light layout only */}
          <div
            className={`absolute inset-0 -left-[9999px] bg-gradient-to-b from-slate-50 to-white pointer-events-none -z-10 dark:hidden`}
            aria-hidden={`true`}
          />

          <div className={`fixed top-0 bottom-0 w-64 px-4 sm:px-6 md:pl-0 md:pr-8 overflow-y-auto no-scrollbar`}>
            <div className={`pt-24 md:pt-28 pb-8`}>
              {/* Docs nav */}
              <nav className={`md:block`}>
                <ul className={`text-sm`}>
                  {/* 1st level */}
                  <SidebarLinkGroup activecondition={pathname.includes(`documentation`)}>
                    {(handleClick, open) => {
                      return (
                        <React.Fragment>
                          <a href={`#0`} className={topLevelLinkClassName(`documentation`)} onClick={e => {
                            e.preventDefault();
                            handleClick();
                          }}>
                            <svg className={`mr-3 shrink-0`} width={`24`} height={`24`} viewBox={`0 0 24 24`} xmlns={`http://www.w3.org/2000/svg`}>
                              <path
                                className={`fill-blue-400`}
                                d={`M19.888 7.804a.88.88 0 0 0-.314-.328l-7.11-4.346a.889.889 0 0 0-.927 0L4.426 7.476a.88.88 0 0 0-.314.328L12 12.624l7.888-4.82Z`}
                              />
                              <path
                                className={`fill-white dark:fill-slate-800`}
                                d={`M4.112 7.804a.889.889 0 0 0-.112.43v7.892c0 .31.161.597.426.758l7.11 4.346c.14.085.3.13.464.13v-8.736l-7.888-4.82Z`}
                              />
                              <path
                                className={`fill-blue-600`}
                                d={`M19.888 7.804c.073.132.112.28.112.43v7.892c0 .31-.161.597-.426.758l-7.11 4.346c-.14.085-.3.13-.464.13v-8.736l7.888-4.82Z`}
                              />
                            </svg>
                            <span>Documentation</span>
                          </a>
                          <ul className={`mb-3 ml-4 pl-6 border-l border-slate-200 dark:border-slate-800 ${!open && `hidden`}`}>
                            <li className={`mt-3`}>
                              <NavLink end to={`/documentation/fundamentals`} className={navLinkClassName}>
                                Fundamentals
                              </NavLink>
                            </li>
                            <li className={`mt-3`}>
                              <NavLink end to={`/documentation/dummy-slug`} className={navLinkClassName}>
                                Methods and Parameters
                              </NavLink>
                            </li>
                            <li className={`mt-3`}>
                              <NavLink end to={`/documentation/dummy-slug`} className={navLinkClassName}>
                                Merge Fields
                              </NavLink>
                            </li>
                            <SidebarLinkSubgroup title={`Alternative Schemas`} open={pathname.includes(`alternative-scheme`)}>
                              <li className={`mt-3`}>
                                <NavLink end to={`/documentation/alternative-scheme/dummy-slug`} className={navLinkClassName}>
                                  File system
                                </NavLink>
                              </li>
                              <li className={`mt-3`}>
                                <NavLink end to={`/documentation/alternative-scheme/dummy-slug`} className={navLinkClassName}>
                                  Describing responses
                                </NavLink>
                              </li>
                            </SidebarLinkSubgroup>
                            <SidebarLinkSubgroup title={`E-Commerce`} open={pathname.includes(`ecommerce`)}>
                              <li className={`mt-3`}>
                                <NavLink end to={`/documentation/ecommerce/dummy-slug`} className={navLinkClassName}>
                                  Path parameters
                                </NavLink>
                              </li>
                              <li className={`mt-3`}>
                                <NavLink end to={`/documentation/ecommerce/dummy-slug`} className={navLinkClassName}>
                                  Query string parameters
                                </NavLink>
                              </li>
                            </SidebarLinkSubgroup>
                            <li className={`mt-3`}>
                              <NavLink end to={`/documentation/dummy-slug`} className={navLinkClassName}>
                                Account Exports
                              </NavLink>
                            </li>
                            <li className={`mt-3`}>
                              <NavLink end to={`/documentation/dummy-slug`} className={navLinkClassName}>
                                Integrations
                              </NavLink>
                            </li>
                            <li className={`mt-3`}>
                              <NavLink end to={`/documentation/dummy-slug`} className={navLinkClassName}>
                                Add a contact
                              </NavLink>
                            </li>
                          </ul>
                        </React.Fragment>
                      );
                    }}
                  </SidebarLinkGroup>
                  {/* 1st level */}
                  <SidebarLinkGroup activecondition={pathname.includes(`guides`)}>
                    {(handleClick, open) => {
                      return (
                        <React.Fragment>
                          <a href={`#0`} className={topLevelLinkClassName(`guides`)} onClick={e => {
                            e.preventDefault();
                            handleClick();
                          }}>
                            <svg className={`mr-3 shrink-0`} width={`24`} height={`24`} viewBox={`0 0 24 24`} xmlns={`http://www.w3.org/2000/svg`}>
                              <path className={`fill-purple-400`} d={`M19.888 7.804a.88.88 0 0 0-.314-.328l-7.11-4.346a.889.889 0 0 0-.927 0L4.426 7.476a.88.88 0 0 0-.314.328L12 12.624l7.888-4.82Z`}/>
                              <path className={`fill-white dark:fill-slate-800`} d={`M4.112 7.804a.889.889 0 0 0-.112.43v7.892c0 .31.161.597.426.758l7.11 4.346c.14.085.3.13.464.13v-8.736l-7.888-4.82Z`}/>
                              <path className={`fill-purple-600`} d={`M19.888 7.804c.073.132.112.28.112.43v7.892c0 .31-.161.597-.426.758l-7.11 4.346c-.14.085-.3.13-.464.13v-8.736l7.888-4.82Z`}/>
                            </svg>
                            <span>
                              Guides / Tutorials
                            </span>
                          </a>
                          <ul className={`mb-3 ml-4 pl-6 border-l border-slate-200 dark:border-slate-800 ${!open && `hidden`}`}>
                            <li className={`mt-3`}>
                              <NavLink end to={`/guides/marketing-api`} className={navLinkClassName}>
                                Marketing API Quick Start
                              </NavLink>
                            </li>
                            <li className={`mt-3`}>
                              <NavLink end to={`/guides/dummy-slug`} className={navLinkClassName}>
                                Create an account
                              </NavLink>
                            </li>
                            <li className={`mt-3`}>
                              <NavLink end to={`/guides/dummy-slug`} className={navLinkClassName}>
                                Generate your API key
                              </NavLink>
                            </li>
                            <li className={`mt-3`}>
                              <NavLink end to={`/guides/dummy-slug`} className={navLinkClassName}>
                                Make your first API call
                              </NavLink>
                            </li>
                            <li className={`mt-3`}>
                              <NavLink end to={`/guides/dummy-slug`} className={navLinkClassName}>
                                Next steps
                              </NavLink>
                            </li>
                          </ul>
                        </React.Fragment>
                      );
                    }}
                  </SidebarLinkGroup>
                  {/* 1st level */}
                  <li className={`mb-1`}>
                    <NavLink end to={`/help`} className={topLevelLinkClassName(`help`)}>
                      <svg className={`mr-3 shrink-0`} width={`24`} height={`24`} viewBox={`0 0 24 24`} xmlns={`http://www.w3.org/2000/svg`}>
                        <path className={`fill-sky-400`} d={`M19.888 7.804a.88.88 0 0 0-.314-.328l-7.11-4.346a.889.889 0 0 0-.927 0L4.426 7.476a.88.88 0 0 0-.314.328L12 12.624l7.888-4.82Z`}/>
                        <path className={`fill-white dark:fill-slate-800`} d={`M4.112 7.804a.889.889 0 0 0-.112.43v7.892c0 .31.161.597.426.758l7.11 4.346c.14.085.3.13.464.13v-8.736l-7.888-4.82Z`}/>
                        <path className={`fill-sky-600`} d={`M19.888 7.804c.073.132.112.28.112.43v7.892c0 .31-.161.597-.426.758l-7.11 4.346c-.14.085-.3.13-.464.13v-8.736l7.888-4.82Z`}/>
                      </svg>
                      <span>
                        Help / Support
                      </span>
                    </NavLink>
                  </li>
                </ul>
              </nav>
            </div>
          </div>
        </Transition>
      </div>
    </div>
  );
}
