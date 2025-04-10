import * as yup from "yup";
import { yupResolver } from "@hookform/resolvers/yup";

import { Field, FieldType } from "../";
import * as Schema from "./schemas";

import type { AnySchema, AnyObjectSchema } from "yup";
import { FieldValues, Resolver } from "react-hook-form";

const getFieldSchema = (fieldType: FieldType, required: boolean): AnySchema => {
  let schema: yup.StringSchema | yup.NumberSchema | yup.BooleanSchema = Schema.Text;
  switch (fieldType) {
    case FieldType.Int: schema = Schema.Int; break;
    case FieldType.Bool: schema = Schema.Bool; break;
    case FieldType.Bytes: schema = Schema.Bytes; break;
    case FieldType.Address: schema = Schema.Address; break;
    case FieldType.UtxoRef: schema = Schema.UtxoRef; break;
    case FieldType.Custom: schema = Schema.Custom; break;
  }
  return !required ? schema : schema.required('The field is required');
};

export function buildFormResolver<T extends FieldValues = any>(fields: Field[]): Resolver<T> {
  const schema = {} as {[key: string]: AnySchema};
  for (const field of fields) {
    schema[field.name] = getFieldSchema(field.type, field.required ?? false);
  }
  return yupResolver(yup.object().shape(schema) as AnyObjectSchema) as Resolver<T>;
}