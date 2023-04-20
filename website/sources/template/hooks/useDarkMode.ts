import {useEffect, useState} from 'react';

export function useDarkMode() {
  // Handle light modes
  const [darkMode, setDarkMode] = useState(() => {
    const dark = localStorage.getItem(`dark-mode`);
    if (dark === null) {
      return false;
    } else {
      return dark === `true`;
    }
  });

  useEffect(() => {
    localStorage.setItem(`dark-mode`, `${darkMode}`);
    if (darkMode) {
      document.documentElement.classList.add(`dark`);
    } else {
      document.documentElement.classList.remove(`dark`);
    }
  }, [darkMode]);

  return [darkMode, setDarkMode] as const;
}
