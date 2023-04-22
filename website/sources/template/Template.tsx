import 'website/sources/template/css/additional-styles/custom-fonts.css';
import 'website/sources/template/css/additional-styles/range-slider.css';
import 'website/sources/template/css/additional-styles/theme.css';
import 'website/sources/template/css/additional-styles/toggle-switch.css';
import 'website/sources/template/css/additional-styles/utility-patterns.css';
import 'website/sources/template/css/style.css';
import React, {useEffect} from 'react';
import {useScrollSpy}     from 'website/sources/template/hooks/useScrollSpy';

export type AppProps = {
  children: React.ReactNode;
};

export function Template({children}: AppProps) {
  useEffect(() => {
    const isDarkModeEnabled = localStorage.getItem(`dark-mode`) === `true`;
    document.documentElement.classList.toggle(`dark`, isDarkModeEnabled);
    document.body.className = `font-aspekta antialiased text-slate-800 font-[350] bg-white dark:bg-slate-900 dark:text-slate-200`;
  }, []);

  useEffect(() => {
    document.documentElement.style.scrollBehavior = `auto`;
    window.scroll({top: 0});
    document.documentElement.style.scrollBehavior = ``;
  }, [location.pathname]); // triggered on route change

  useScrollSpy();

  return (
    <>{children}</>
  );
}
