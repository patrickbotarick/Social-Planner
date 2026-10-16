import type { ReactNode } from 'react';

// Apresentação limitada: negrito e escapes comuns. HTML e links continuam texto.
// O valor persistido e as operações de cópia nunca passam por esta função.
export function CaptionText({ text }: { text: string }) {
  const output: ReactNode[] = [];
  let plain = '';
  let bold: string | null = null;
  for (const token of text.split(/(\\[\\*#_]|\*\*)/u)) {
    if (token === '**') {
      if (bold === null) {
        output.push(plain);
        plain = '';
        bold = '';
      } else {
        output.push(bold ? <strong key={output.length}>{bold}</strong> : '****');
        bold = null;
      }
    } else {
      const value = /^\\[\\*#_]$/u.test(token) ? token.slice(1) : token;
      if (bold === null) plain += value;
      else bold += value;
    }
  }
  output.push(plain, bold === null ? '' : '**' + bold);
  return <>{output}</>;
}
