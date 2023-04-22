import {Link}            from 'react-router-dom';
import React, {useState} from 'react';
import {useDarkMode}     from 'website/sources/template/hooks/useDarkMode';
import Logo              from 'website/sources/template/images/Logo.svg?transform=url';
import {ModalSearch}     from 'website/sources/template/partials/ModalSearch';

export function Header() {
  const [searchModalOpen, setSearchModalOpen] = useState(false);
  const [darkMode, setDarkMode] = useDarkMode();

  const handleSearchStart = (e: React.MouseEvent) => {
    e.stopPropagation();
    setSearchModalOpen(true);
  };

  return (
    <header className={`fixed w-full z-30`}>
      <div
        className={`absolute inset-0 bg-white bg-opacity-70 border-b border-slate-200 backdrop-blur -z-10 dark:bg-slate-900 dark:border-slate-800`}
        aria-hidden={`true`}
      />
      <div className={`max-w-7xl mx-auto px-4 sm:px-6`}>
        <div className={`flex items-center justify-between h-16 md:h-20`}>
          {/* Site branding */}
          <div className={`grow`}>
            <div className={`flex items-center`}>
              {/* Logo */}
              <Link to={`/`} aria-label={`Cruip`}>
                <img src={Logo} width={`32`} height={`32`} alt={`Docs`} />
              </Link>
              {/* Search */}
              <div className={`grow ml-4 md:ml-8`}>
                <button className={`w-full sm:w-[380px] text-[15px] bg-white text-slate-400 inline-flex items-center justify-between leading-5 pl-3 pr-2 py-[7px] rounded border border-slate-200 hover:border-slate-300 shadow-sm whitespace-nowrap dark:text-slate-500 dark:bg-slate-800 dark:border-slate-700 dark:hover:border-slate-600`} onClick={handleSearchStart} aria-controls={`search-modal`}>
                  <div className={`flex items-center justify-center`}>
                    <svg className={`w-4 h-4 fill-slate-500 mr-3 shrink-0 dark:fill-slate-400`} width={`16`} height={`16`} viewBox={`0 0 16 16`} xmlns={`http://www.w3.org/2000/svg`}>
                      <path d={`m14.707 13.293-1.414 1.414-2.4-2.4 1.414-1.414 2.4 2.4ZM6.8 12.6A5.8 5.8 0 1 1 6.8 1a5.8 5.8 0 0 1 0 11.6Zm0-2a3.8 3.8 0 1 0 0-7.6 3.8 3.8 0 0 0 0 7.6Z`} />
                    </svg>
                    <span>
                      Search<span className={`hidden sm:inline`}> for anything</span>…
                    </span>
                  </div>
                  <div className={`flex items-center justify-center h-5 w-5 font-medium text-slate-500 rounded border border-slate-200 shadow-sm ml-3 dark:bg-slate-700 dark:text-slate-400 dark:border-slate-600`}>
                    /
                  </div>
                </button>
                <div>
                  <ModalSearch id={`search-modal`} searchId={`search`} modalOpen={searchModalOpen} setModalOpen={setSearchModalOpen} />
                </div>
              </div>
            </div>
          </div>

          {/* Desktop navigation */}
          <nav className={`flex`}>
            {/* Right side elements links */}
            <ul className={`flex grow justify-end flex-wrap items-center`}>
              <li className={`ml-4`}>
                <a className={`px-3 py-2 text-sm font-[500] inline-flex items-center justify-center rounded-full leading-5 whitespace-nowrap transition duration-150 ease-in-out inline-flex items-center text-slate-100 bg-blue-600 hover:bg-blue-700 shadow-sm`} href={`#0`}>
                  Support
                </a>
              </li>
              {/* Lights switch */}
              <li>
                <div className={`flex flex-col justify-center ml-3`}>
                  <input
                    type={`checkbox`}
                    name={`light-switch`}
                    id={`light-switch`}
                    className={`light-switch sr-only`}
                    checked={darkMode}
                    onChange={() => setDarkMode(!darkMode)}
                  />
                  <label className={`relative cursor-pointer p-2`} htmlFor={`light-switch`}>
                    <svg className={`dark:hidden`} width={`16`} height={`16`} xmlns={`http://www.w3.org/2000/svg`}>
                      <path className={`fill-blue-400`} d={`M7 0h2v2H7zM12.88 1.637l1.414 1.415-1.415 1.413-1.413-1.414zM14 7h2v2h-2zM12.95 14.433l-1.414-1.413 1.413-1.415 1.415 1.414zM7 14h2v2H7zM2.98 14.364l-1.413-1.415 1.414-1.414 1.414 1.415zM0 7h2v2H0zM3.05 1.706 4.463 3.12 3.05 4.535 1.636 3.12z`}/>
                      <path className={`fill-blue-500`} d={`M8 4C5.8 4 4 5.8 4 8s1.8 4 4 4 4-1.8 4-4-1.8-4-4-4Z`} />
                    </svg>
                    <svg className={`hidden dark:block`} width={`16`} height={`16`} xmlns={`http://www.w3.org/2000/svg`}>
                      <path className={`fill-blue-500`} d={`M6.2 1C3.2 1.8 1 4.6 1 7.9 1 11.8 4.2 15 8.1 15c3.3 0 6-2.2 6.9-5.2C9.7 11.2 4.8 6.3 6.2 1Z`}/>
                      <path className={`fill-blue-500`} d={`M12.5 5a.625.625 0 0 1-.625-.625 1.252 1.252 0 0 0-1.25-1.25.625.625 0 1 1 0-1.25 1.252 1.252 0 0 0 1.25-1.25.625.625 0 1 1 1.25 0c.001.69.56 1.249 1.25 1.25a.625.625 0 1 1 0 1.25c-.69.001-1.249.56-1.25 1.25A.625.625 0 0 1 12.5 5Z`}/>
                    </svg>
                    <span className={`sr-only`}>
                      Switch to light / dark version
                    </span>
                  </label>
                </div>
              </li>
            </ul>
          </nav>
        </div>
      </div>
    </header>
  );
}
