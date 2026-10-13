import { useId } from 'react';
import type { InputHTMLAttributes, TextareaHTMLAttributes, SelectHTMLAttributes, ReactNode } from 'react';
import { Icon } from '../Icon';
type FieldProps = { label: string; hint?: string; error?: string; hideLabel?: boolean };
function Field({ label, hint, error, hideLabel, id, children }: FieldProps & { id: string; children: ReactNode }) {
  return <div className="field"><label className={hideLabel ? 'visually-hidden' : 'field-label'} htmlFor={id}>{label}</label>{children}{(error || hint) && <p id={`${id}-description`} className={error ? 'field-error' : 'field-hint'}>{error ?? hint}</p>}</div>;
}
function fieldOptions(hint?: string, error?: string, hideLabel?: boolean) { return { ...(hint ? { hint } : {}), ...(error ? { error } : {}), ...(hideLabel ? { hideLabel } : {}) }; }
function descriptionProps(id: string, hint?: string, error?: string) { return { 'aria-describedby': hint || error ? `${id}-description` : undefined, 'aria-invalid': error ? true : undefined } as const; }
export type InputProps = InputHTMLAttributes<HTMLInputElement> & FieldProps;
export function Input({ label, hint, error, hideLabel, id: providedId, className = '', ...props }: InputProps) {
  const generatedId = useId(); const id = providedId ?? generatedId;
  return <Field label={label} id={id} {...fieldOptions(hint, error, hideLabel)}><input id={id} className={`ui-input ${className}`} {...descriptionProps(id, hint, error)} {...props} /></Field>;
}
export function Textarea({ label, hint, error, hideLabel, id: providedId, ...props }: TextareaHTMLAttributes<HTMLTextAreaElement> & FieldProps) {
  const generatedId = useId(); const id = providedId ?? generatedId;
  return <Field label={label} id={id} {...fieldOptions(hint, error, hideLabel)}><textarea id={id} className="ui-input" {...descriptionProps(id, hint, error)} {...props} /></Field>;
}
export function Select({ label, hint, error, hideLabel, id: providedId, children, ...props }: SelectHTMLAttributes<HTMLSelectElement> & FieldProps) {
  const generatedId = useId(); const id = providedId ?? generatedId;
  return <Field label={label} id={id} {...fieldOptions(hint, error, hideLabel)}><select id={id} className="ui-input" {...descriptionProps(id, hint, error)} {...props}>{children}</select></Field>;
}
export function SearchField({ label, hint, error, id: providedId, ...props }: Omit<InputProps, 'hideLabel'>) {
  const generatedId = useId(); const id = providedId ?? generatedId;
  return <div className="search-field"><Field label={label} id={id} {...fieldOptions(hint, error, true)}><div className="search-input"><Icon name="search" /><input id={id} type="search" className="ui-input" {...descriptionProps(id, hint, error)} {...props} /></div></Field></div>;
}
