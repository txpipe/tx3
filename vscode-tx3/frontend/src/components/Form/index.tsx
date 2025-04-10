import React from "react";
import { useForm } from "react-hook-form";
import { buildFormResolver } from "./utils";

import Button from "../Button";

import type { UseFormRegister, UseFormGetFieldState, FormState, FieldValues } from "react-hook-form";

export enum FormMode {
  Submit = "Submit",
  Blur = "Blur"
}

export enum FieldType {
  Int = "Int",
  Bool = "Bool",
  Bytes = "Bytes",
  Address = "Address",
  UtxoRef = "UtxoRef",
  Custom = "Custom",
  Select = "Select",
  Text = "Text",
}

export interface Field {
  name: string;
  type: FieldType;
  label?: string;
  placeholder?: string;
  defaultValue?: string|number|boolean;
  required?: boolean;
  disabled?: boolean;
  hidden?: boolean;
  options?: {
    value: string,
    label: string
  }[];
}

export interface FormProps {
  loading?: boolean;
  mode?: FormMode;
  fields: Field[];
  onSubmit: (data: Record<string, string|number|boolean>) => void;
}

interface InputProps<T extends FieldValues = any> {
  field: Field;
  loading?: boolean;
  formState: FormState<T>;
  register: UseFormRegister<T>;
  getFieldState: UseFormGetFieldState<T>;
}

const Input: React.FC<InputProps> = (props: InputProps) => (
  <div className="input-container">
    <p className="input-label">{props.field.label || props.field.name}</p>
    {props.field.type !== FieldType.Select &&
      <input
        type="text"
        className="form-input input"
        placeholder={props.field.placeholder}
        disabled={props.field.disabled || props.loading}
        {...props.register(props.field.name, { required: true })}
      />
    }
    {props.field.type === FieldType.Select &&
      <select
        className="form-input input"
        disabled={props.field.disabled || props.loading}
        {...props.register(props.field.name, { required: true })}
      >
        {props.field.options?.map((option, index) =>
          <option key={index} value={option.value}>{option.label}</option>
        )}
      </select>
    }
    {props.getFieldState(props.field.name, props.formState).error !== undefined &&
      <p className="input-error">
        {props.getFieldState(props.field.name, props.formState).error?.message}
      </p>
    }
  </div>
);

const Form: React.FC<FormProps> = (props: FormProps) => {
  const { formState, register, handleSubmit, getFieldState, getValues } = useForm({
    defaultValues: props.fields.reduce((acc, field) => {
      if (field.defaultValue !== undefined) {
        acc[field.name] = field.defaultValue;
      }
      return acc;
    }, {} as Record<string, string|number|boolean>),
    resolver: buildFormResolver(props.fields),
    mode: props.mode === FormMode.Blur ? 'onBlur' : 'onSubmit',
  });

  const handleChange = () => {
    if (props.mode === FormMode.Blur) {
      const values = getValues();
      props.onSubmit(values);
    }
  }

  return (
    <form onSubmit={handleSubmit(props.onSubmit)} onChange={handleChange}>
      {props.fields.map((field, index) => !field.hidden &&
        <Input
          key={index}
          field={field}
          loading={props.loading}
          formState={formState}
          register={register}
          getFieldState={getFieldState}
        />
      )}

      {(!props.mode || props.mode === FormMode.Submit) &&
        <div className="text-right">
          <Button
            loading={props.loading}
            loadingLabel="Resolving..."
            label="Resolve Tx"
          />
        </div>
      }
    </form>
  );
}

export default Form;