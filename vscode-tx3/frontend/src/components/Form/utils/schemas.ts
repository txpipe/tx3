import * as yup from "yup";

export const Text = yup.string()
  .typeError('The field value must be a valid string');

export const Int = yup.number()
  .integer('The field value must be a valid integer')
  .typeError('The field value must be a valid integer');

export const Bool = yup.bool()
  .typeError('The field value must be a valid boolean');

export const Bytes = yup.string()
  .typeError('The field value must be a valid string');

export const Address = yup.string()
  .matches(/^[a-zA-Z0-9_]*$/, {
    message: 'The field value must be a valid address',
    excludeEmptyString: true
  })
  .typeError('The field value must be a valid address');

export const UtxoRef = yup.string()
  .typeError('The field value must be a valid string');

export const Custom = yup.string()
  .typeError('The field value must be a valid string');