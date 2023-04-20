import React, {useState} from 'react';

export type SidebarLinkGroupProps = {
  children: (handleClick: () => void, open: boolean) => React.ReactNode;
  activecondition: boolean;
};

export function SidebarLinkGroup({
  children,
  activecondition,
}: SidebarLinkGroupProps) {
  const [open, setOpen] = useState(activecondition);

  const handleClick = () => {
    setOpen(!open);
  };

  return (
    <li className={`mb-1`}>
      {children(handleClick, open)}
    </li>
  );
}
