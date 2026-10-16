import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { CaptionText } from '../src/features/editorial/CaptionText';

describe('Apresentação limitada da legenda Trello', () => {
  it('exibe negrito e conserva quebras, emojis e hashtags sem mudar a entrada', () => {
    const text = '**Campanha 😊**\n\n\\#GMAD #Loja';
    const { container } = render(<CaptionText text={text} />);
    expect(container.querySelector('strong')).toHaveTextContent('Campanha 😊');
    expect(container.textContent).toBe('Campanha 😊\n\n#GMAD #Loja');
    expect(text).toBe('**Campanha 😊**\n\n\\#GMAD #Loja');
  });
  it('mantém HTML, scripts e links maliciosos como texto inerte', () => {
    const text = '<img src=x onerror=alert(1)><script>alert(2)</script> **Seguro** [link](javascript:alert(3))';
    const { container } = render(<CaptionText text={text} />);
    expect(container.querySelector('img,script,a,iframe')).toBeNull();
    expect(container.textContent).toBe(text.replace('**Seguro**', 'Seguro'));
    expect(screen.getByText('Seguro').tagName).toBe('STRONG');
  });
  it('preserva marcadores incompletos e respeita asteriscos escapados', () => {
    const { container } = render(<CaptionText text={'\\*\\*literal\\*\\* **sem fechamento\n😊'} />);
    expect(container.textContent).toBe('**literal** **sem fechamento\n😊');
    expect(container.querySelector('strong')).toBeNull();
  });
  it('suporta vários trechos sem descartar texto entre eles ou pares vazios', () => {
    const { container } = render(<CaptionText text={'Antes **um** e **dois**. **** #fim'} />);
    expect(container.textContent).toBe('Antes um e dois. **** #fim');
    expect(container.querySelectorAll('strong')).toHaveLength(2);
  });
});
