import React, { useState } from "react";

export interface BoxProps {
  title?: string;
  collapsible?: boolean;
  collapsed?: boolean;
  children: React.ReactNode;
}

const Box: React.FC<BoxProps> = (props: BoxProps) => {
  const [collapsed, setCollapsed] = useState<boolean>(props.collapsed || false);

  const handleCollapse = () => {
    if (props.collapsible) {
      setCollapsed(!collapsed);
    }
  }

  return (
    <div className="box">
      {(props.title || props.collapsible) &&
        <div className="box-header" onClick={handleCollapse}>
          {props.collapsible && !collapsed &&
            <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor" className="box-icon">
              <path strokeLinecap="round" strokeLinejoin="round" d="m4.5 15.75 7.5-7.5 7.5 7.5" />
            </svg>
          }
          {props.collapsible && collapsed &&
            <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor" className="box-icon">
              <path strokeLinecap="round" strokeLinejoin="round" d="m19.5 8.25-7.5 7.5-7.5-7.5" />
            </svg>              
          }
          <h3 className="box-title">{props.title}</h3>
        </div>
      }
      <div className={!collapsed ? "box-content" : "box-content-hidden"}>
        {props.children}
      </div>
    </div>
  );
}

export default Box;