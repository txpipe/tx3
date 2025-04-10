import React from "react";

export interface TitleProps {
  children: React.ReactNode;
}

const Title: React.FC<TitleProps> = (props: TitleProps) => (
  <h3 className="title">{props.children}</h3>
);

export default Title;