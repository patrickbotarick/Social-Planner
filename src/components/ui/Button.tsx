import type { ButtonHTMLAttributes, ReactNode } from 'react';
export type ButtonProps = ButtonHTMLAttributes<HTMLButtonElement> & { variant?: 'primary' | 'secondary' | 'ghost' | 'danger'; size?: 'normal' | 'small'; icon?: ReactNode };
export function Button({ variant = 'primary', size = 'normal', icon, className = '', children, type = 'button', ...props }: ButtonProps) {
  return <button type={type} className={`ui-button ui-button--${variant} ui-button--${size} ${className}`} {...props}>{icon}{children}</button>;
}
export function IconButton({ label, children, ...props }: Omit<ButtonProps, 'icon'> & { label: string }) {
  return <Button variant="ghost" {...props} className={`ui-icon-button ${props.className ?? ''}`} aria-label={label}>{children}</Button>;
}
