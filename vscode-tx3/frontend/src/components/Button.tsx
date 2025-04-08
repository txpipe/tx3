import React from "react";

export interface ButtonProps {
  label: string;
  loading?: boolean;
  loadingLabel?: string;
  onClick?: () => void;
}

const Button: React.FC<ButtonProps> = (props: ButtonProps) => (
  <input
    type="submit"
    disabled={props.loading}
    value={props.loading && props.loadingLabel ? props.loadingLabel : props.label}
    className={props.loading ? "button-loading" : "button"}
  />
);

export default Button;