import {CSSTransition as ReactCSSTransition}  from 'react-transition-group';
import React, {useRef, useEffect, useContext} from 'react';

export type TransitionProps = React.HTMLAttributes<HTMLDivElement> & {
  show: boolean;
  appear?: boolean;
  unmountOnExit?: boolean;

  enter: string;
  enterStart: string;
  enterEnd: string;

  leave: string;
  leaveStart: string;
  leaveEnd: string;

  tag?: string;
  children?: React.ReactNode;
};

const TransitionContext = React.createContext<{
  parent: {
    show: boolean;
    isInitialRender: boolean;
    appear: boolean;
  };
}>({
  parent: {
    show: false,
    isInitialRender: false,
    appear: false,
  },
});

function useIsInitialRender() {
  const isInitialRender = useRef(true);
  useEffect(() => {
    isInitialRender.current = false;
  }, []);
  return isInitialRender.current;
}

function CSSTransition({
  show,
  appear = false,
  unmountOnExit = false,
  enter = ``,
  enterStart = ``,
  enterEnd = ``,
  leave = ``,
  leaveStart = ``,
  leaveEnd = ``,
  tag = `div`,
  children,
  ...rest
}: TransitionProps) {
  const enterClasses = enter.split(` `).filter(s => s.length);
  const enterStartClasses = enterStart.split(` `).filter(s => s.length);
  const enterEndClasses = enterEnd.split(` `).filter(s => s.length);

  const leaveClasses = leave.split(` `).filter(s => s.length);
  const leaveStartClasses = leaveStart.split(` `).filter(s => s.length);
  const leaveEndClasses = leaveEnd.split(` `).filter(s => s.length);

  function addClasses(node: Element, classes: Array<string>) {
    classes.length && node.classList.add(...classes);
  }

  function removeClasses(node: Element, classes: Array<string>) {
    classes.length && node.classList.remove(...classes);
  }

  const nodeRef = React.useRef<HTMLElement | null>(null);
  const Component: any = tag;

  return (
    <ReactCSSTransition
      appear={appear}
      nodeRef={nodeRef}
      unmountOnExit={unmountOnExit}
      in={show}
      addEndListener={done => {
        nodeRef.current?.addEventListener(`transitionend`, done, false);
      }}
      onEnter={() => {
        if (!unmountOnExit) nodeRef.current!.style.display = ``;
        addClasses(nodeRef.current!, [...enterClasses, ...enterStartClasses]);
      }}
      onEntering={() => {
        removeClasses(nodeRef.current!, enterStartClasses);
        addClasses(nodeRef.current!, enterEndClasses);
      }}
      onEntered={() => {
        removeClasses(nodeRef.current!, [...enterEndClasses, ...enterClasses]);
      }}
      onExit={() => {
        addClasses(nodeRef.current!, [...leaveClasses, ...leaveStartClasses]);
      }}
      onExiting={() => {
        removeClasses(nodeRef.current!, leaveStartClasses);
        addClasses(nodeRef.current!, leaveEndClasses);
      }}
      onExited={() => {
        removeClasses(nodeRef.current!, [...leaveEndClasses, ...leaveClasses]);
        if (!unmountOnExit) {
          nodeRef.current!.style.display = `none`;
        }
      }}
    >
      <Component ref={nodeRef} {...rest} style={{display: !unmountOnExit ? `none` : null}}>{children}</Component>
    </ReactCSSTransition>
  );
}

export function Transition({
  show,
  appear = false,
  ...rest
}: TransitionProps) {
  const {parent} = useContext(TransitionContext);
  const isInitialRender = useIsInitialRender();
  const isChild = show === undefined;

  if (isChild) {
    return (
      <CSSTransition
        appear={parent.appear || !parent.isInitialRender}
        show={parent.show}
        {...rest}
      />
    );
  }

  return (
    <TransitionContext.Provider
      value={{
        parent: {
          show,
          isInitialRender,
          appear,
        },
      }}
    >
      <CSSTransition appear={appear} show={show} {...rest} />
    </TransitionContext.Provider>
  );
}
