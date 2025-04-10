import React from "react";

export interface ButtonProps {
  label: string;
  loading?: boolean;
  loadingLabel?: string;
  onClick?: () => void;
}

const Button: React.FC<ButtonProps> = (props: ButtonProps) => (
  <button
    type="submit"
    disabled={props.loading}
    className={props.loading ? "button-loading" : "button"}
  >
    {props.loading && props.loadingLabel ? props.loadingLabel : props.label}
  </button>
);

export default Button;