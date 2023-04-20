import React, {useEffect} from 'react';

export type UseCloseInteractionsOpts = {
  containerRef?: React.RefObject<HTMLElement>;
  shortcut?: string;
};

export function useCloseInteractions(status: boolean, setStatus: (value: boolean) => void, opts: UseCloseInteractionsOpts) {
  // close on click outside
  useEffect(() => {
    const clickHandler = ({target}: MouseEvent) => {
      if (!status || opts.containerRef?.current!.contains(target as any))
        return;

      setStatus(false);
    };

    document.addEventListener(`click`, clickHandler);
    return () => document.removeEventListener(`click`, clickHandler);
  }, []);

  // close if the esc key is pressed
  useEffect(() => {
    const keyHandler = (event: KeyboardEvent) => {
      if (status && event.key === `Escape`)
        setStatus(false);

      if (!status && event.key === opts.shortcut) {
        event.preventDefault();
        setStatus(true);
      }
    };

    document.addEventListener(`keydown`, keyHandler);
    return () => document.removeEventListener(`keydown`, keyHandler);
  }, [
    opts.shortcut,
    status,
  ]);
}
