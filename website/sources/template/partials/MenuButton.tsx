import React from 'react';

export type MenuButtonProps = {
  sidebarOpen: boolean;
  setSidebarOpen: (open: boolean) => void;
};

export function MenuButton({
  sidebarOpen,
  setSidebarOpen,
}: MenuButtonProps) {
  const handleClick = (e: React.MouseEvent) => {
    e.stopPropagation();
    setSidebarOpen(!sidebarOpen);
  };

  return (
    <button className={`hamburger`} aria-controls={`sidebar`} aria-expanded={sidebarOpen} onClick={handleClick}>
      <span className={`sr-only`}>Menu</span>
      <svg className={`w-6 h-6 fill-slate-600 dark:fill-slate-400`} viewBox={`0 0 24 24`} xmlns={`http://www.w3.org/2000/svg`}>
        <rect x={`4`} y={`5`} width={`16`} height={`2`} />
        <rect x={`4`} y={`11`} width={`16`} height={`2`} />
        <rect x={`4`} y={`17`} width={`16`} height={`2`} />
      </svg>
    </button>
  );
}
