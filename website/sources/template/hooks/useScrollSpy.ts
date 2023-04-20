import {useEffect} from 'react';

const TARGET_MARGIN = 100;

export function useScrollSpy() {
  const links = document.querySelectorAll(`[data-scrollspy-link]`);
  const targets = document.querySelectorAll(`[data-scrollspy-target]`);

  const setActive = (i: number) => {
    removeAllActive();
    addActive(i);
  };

  const addActive = (i: number) => {
    links[i].classList.add(`scrollspy-active`);
  };

  const removeActive = (i: number) => {
    links[i].classList.remove(`scrollspy-active`);
  };

  const removeAllActive = () => {
    for (let t = 0; t < targets.length; ++t) {
      removeActive(t);
    }
  };

  useEffect(() => {
    const handler = () => {
      let current = null;
      for (let t = targets.length - 1; t >= 0; --t) {
        const target = targets[t] as HTMLElement;
        if (window.scrollY >= target.offsetTop - TARGET_MARGIN) {
          current = t;
          break;
        }
      }

      if (current !== null && current !== currentActive) {
        setActive(current);
      }
    };

    let currentActive = links.length > 0 ? 0 : null;
    if (currentActive !== null)
      addActive(currentActive);

    window.addEventListener(`scroll`, handler);
    return () => {
      window.removeEventListener(`scroll`, handler);
    };
  }, []);
}
