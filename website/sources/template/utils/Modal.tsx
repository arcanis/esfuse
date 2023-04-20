import React, {useRef}        from 'react';
import {useCloseInteractions} from 'website/sources/template/hooks/useCloseInteractions';
import {Transition}           from 'website/sources/template/utils/Transition';

export type ModalProps = {
  children: React.ReactNode;
  id: string;
  ariaLabel: string;
  show: boolean;
  handleClose: () => void;
};

export function Modal({
  children,
  id,
  ariaLabel,
  show,
  handleClose,
}: ModalProps) {
  const modalContentRef = useRef<HTMLDivElement | null>(null);

  useCloseInteractions(show, status => {
    !status && handleClose();
  }, {
    containerRef: modalContentRef,
  });

  return (
    <>
      {/* Modal backdrop */}
      <Transition
        className={`fixed inset-0 bg-slate-900 bg-opacity-20 z-50 transition-opacity`}
        aria-hidden={`true`}

        show={show}

        enter={`transition ease-out duration-200`}
        enterStart={`opacity-0`}
        enterEnd={`opacity-100`}

        leave={`transition ease-out duration-100`}
        leaveStart={`opacity-100`}
        leaveEnd={`opacity-0`}
      />

      {/* Modal dialog */}
      <Transition
        id={id}
        className={`fixed inset-0 z-50 overflow-hidden flex items-center justify-center px-4 sm:px-6`}
        role={`dialog`}
        aria-modal={`true`}
        aria-labelledby={ariaLabel}

        show={show}

        enter={`transition ease-out duration-200`}
        enterStart={`opacity-0 scale-95`}
        enterEnd={`opacity-100 scale-100`}

        leave={`transition ease-out duration-200`}
        leaveStart={`opacity-100 scale-100`}
        leaveEnd={`opacity-0 scale-95`}
      >
        <div className={`bg-white overflow-auto max-w-4xl w-full max-h-full dark:bg-slate-900`} ref={modalContentRef}>
          {children}
        </div>
      </Transition>
    </>
  );
}
