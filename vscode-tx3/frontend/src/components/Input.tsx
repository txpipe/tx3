import React from "react";

export interface InputProps {
  label: string;
  placeholder?: string;
  name?: string;
  type?: string;
  value?: string;
  onChange?: (value: string) => void;
  disabled?: boolean;
}

const Input: React.FC<InputProps> = (props: InputProps) => (
  <>
    <p className="label">{props.label}</p>
    <input
      className="form-input input"
      value={props.value}
      name={props.name}
      placeholder={props.placeholder}
      type={props.type === "Int" ? "number" : "text"}
      onChange={(e) => props.onChange && props.onChange(e.target.value)}
      disabled={props.disabled}
    />
  </>
);

export default Input;