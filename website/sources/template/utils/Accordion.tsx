import React, {useState} from 'react';

export type AccordionProps = {
  children: React.ReactNode;
  title: string;
  open?: boolean;
};

export function Accordion({
  children,
  title,
  open,
}: AccordionProps) {
  const [accordionOpen, setAccordionOpen] = useState(open);

  const handleButtonClick = (e: React.MouseEvent) => {
    e.preventDefault();
    setAccordionOpen(!accordionOpen);
  };

  return (
    <li>
      <button className={`flex items-center w-full text-slate-800 font-medium text-left dark:text-slate-200`} onClick={handleButtonClick} aria-expanded={accordionOpen}>
        <div className={`shrink-0 mr-3`}>
          <svg className={`fill-slate-400 dark:fill-slate-500 ${accordionOpen && `rotate-90`}`} xmlns={`http://www.w3.org/2000/svg`} width={`8`} height={`12`}>
            <path d={`m4.586 6-4-4L2 .586 7.414 6 2 11.414.586 10z`} />
          </svg>
        </div>
        <span>
          {title}
        </span>
      </button>
      <div className={`${!accordionOpen && `hidden`}`}>
        <div className={`pl-5 mt-2`}>
          {children}
        </div>
      </div>
    </li>
  );
}
