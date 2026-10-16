# Design system — GMAD Social Planner

Referência persistente da Fase 02, implementada em 13/10/2026. Escopo e fronteiras em [Projeto mestre](PROJETO_MESTRE.md); evidências em [Fase 02](FASE_02_IDENTIDADE_INTERFACE.md).

## Conceito

Editorial workspace: uma mesa de trabalho editorial local, com navegação estável, hierarquia compacta e superfície clara para leitura prolongada. Verde identifica marca, ação e seleção; laranja fica restrito ao símbolo e ao foco em tonalidade de maior contraste. A paleta é a identidade proposta para o software, sem afirmar códigos oficiais da marca GMAD.

Não usar gradientes, sombras fortes, métricas decorativas, animações ornamentais ou cards coloridos sem significado. Ícones complementam rótulos; não substituem nomes acessíveis. O símbolo geométrico existente foi reaproveitado e suas cores aplicadas também aos ícones Windows.

## Paleta semântica

Fonte: `src/styles/tokens.css`. Consumir variáveis; evitar cores literais nas páginas.

| Token | Valor | Uso |
| --- | --- | --- |
| `--color-background` | `#F5F6F3` | Área de trabalho |
| `--color-surface` | `#FFFFFF` | Painéis e navegação |
| `--color-surface-muted` | `#F0F3EF` | Mídias vazias e hover discreto |
| `--color-text-primary` | `#262D2A` | Texto principal |
| `--color-text-secondary` | `#68736D` | Descrições e metadados |
| `--color-border` | `#DFE5DF` | Separadores decorativos |
| `--color-border-strong` | `#B9C5BD` | Estruturas que precisam de mais definição |
| `--color-border-control` | `#85938B` | Contorno de controles disponíveis |
| `--color-brand` | `#163D32` | Marca e títulos de navegação |
| `--color-primary` | `#267A55` | Ação principal |
| `--color-primary-hover` | `#1F6546` | Hover de ação |
| `--color-primary-active` | `#184F37` | Ação pressionada |
| `--color-accent` | `#F29A38` | Detalhe do símbolo; não usar como texto em branco |
| `--color-focus` | `#9E5315` | Contorno de foco |
| `--color-success` / `--color-success-surface` | `#216647` / `#EAF3EC` | Resultado verificado |
| `--color-warning` / `--color-warning-surface` | `#87501A` / `#FFF3E1` | Atenção |
| `--color-danger` / `--color-danger-surface` | `#A33332` / `#FBEEED` | Erro ou ação destrutiva |
| `--color-disabled` / `--color-disabled-surface` | `#6B756F` / `#EEF1ED` | Indisponibilidade |

Tokens auxiliares de superfície, ícone e navegação estão no mesmo arquivo. Status sempre têm texto; cor não é a única informação. Um badge verde só representa resultado real, como a integridade do SQLite retornada pelo Rust.

## Tipografia e medidas

Inter Variable 4.1 normal, pesos 100–900, incorporada em `public/fonts/InterVariable.woff2`; licença SIL OFL 1.1 integral em `public/fonts/OFL-Inter.txt`. Sem CDN ou instalação de fonte no Windows. Fallback: Segoe UI/sans-serif. Fontes oficiais: [Inter](https://rsms.me/inter/), [release 4.1](https://github.com/rsms/inter/releases/tag/v4.1) e [licença](https://github.com/rsms/inter/blob/v4.1/LICENSE.txt).

| Papel | Tamanho | Peso |
| --- | --- | --- |
| Título da página | 24 px / `--font-title` | 600 |
| Seção | 16 px / `--font-section` | 600 |
| Subtítulo, corpo e controles | 14 px / `--font-lg` | 400–600 |
| Navegação e texto compacto | 13 px / `--font-md` | 400–600 |
| Descrição curta e rótulo auxiliar | 12 px / `--font-sm` | 400–500 |
| Metadados | 11 px / `--font-xs` | 400–500 |

Base 16 px, entrelinha `--line-body: 1.55`. Títulos compactos dentro de cards usam 14 px. Pesos normal/medium/semibold/bold: 400/500/600/700. Valores editoriais usam números tabulares; valores desconhecidos usam travessão com nome acessível, nunca zero simulado.

Espaçamentos: 4, 8, 12, 16, 20, 24, 32 e 40 px (`--space-1` a `--space-10`, conforme arquivo). Dimensões estruturais podem usar medidas específicas. Bordas de 1 px; raios 4/6/8 px; controle padrão 36 px. Foco de 2 px com afastamento padrão de 3 px. Cards usam borda discreta, sem sombra.

## Componentes e contratos

| Componente | Responsabilidade e uso |
| --- | --- |
| AppShell, Sidebar, Topbar | Estrutura da janela, oito rotas, contexto e diagnóstico real resumido. AppShell reaproveita `layouts/AppLayout.tsx`. |
| PageHeader, AvailabilityNote | Título, descrição, ação contextual e indicação da fase futura. |
| Button, IconButton | Variantes primary/secondary/ghost/danger; tamanho normal/small; `type="button"` por padrão; nome obrigatório no botão de ícone. |
| Input, Textarea, Select | Label associado por ID, descrição/erro via `aria-describedby`, erro via `aria-invalid`; atributos HTML nativos. |
| SearchField, Toolbar | Pesquisa com nome acessível e agrupamento flexível de controles. |
| Badge, Card, EmptyState | Status semântico, superfície reutilizável e orientação sem dados fictícios. |
| MediaThumbnail | Mídia real quando fornecida; placeholder explícito quando ausente. Não gera registros editoriais. |
| Skeleton | Estrutura visual de carregamento, sem pulsação; acompanhado de status textual. |
| Tabs | Seleção, tab/panel associados, roving tabindex, setas/Home/End. Usado em Configurações. |
| Modal | Dialog nativo, título acessível, fundo inerte, foco inicial, Tab contido, Escape e restauração do foco. Usado na ajuda. |
| ConfirmDialog | Composição simples de Modal para decisões relevantes, como exclusão lógica na biblioteca. Não confirmar navegação trivial. |
| UnsavedDialog | Salvar e continuar, descartar e sair ou permanecer; usado pela guarda global e pela revisão editorial. Falhas mantêm o formulário aberto. |
| ToastProvider / useToast | Feedback acessível de cópia do caminho; status para sucesso, alert para erro; sucesso fecha após 6 s, erro permanece até dispensado. |
| Tooltip | Informação complementar ao hover/foco; associação `aria-describedby`; Escape dispensa. Não contém instrução essencial exclusiva. |

Primitivos em `src/components/ui/`; estilos comuns em `src/styles/global.css`; composições de páginas em `src/styles/pages.css`; shell em CSS Module. Componentes de apresentação não consultam SQLite ou integrações. Nenhuma biblioteca completa de UI foi adicionada.

## Estados e acessibilidade

Controles futuros têm `disabled` nativo e orientação visível da fase. Atalhos de importação, criação e revisão navegam para os fluxos implementados; calendário persistente continua futuro. Loading, erro, prévia web e conexão real continuam distintos. Mensagens de erro são claras e não expõem falhas internas desconhecidas. Revisão usa erros impeditivos e avisos separados, sem depender apenas da cor; lotes exigem confirmação e mostram resultados por item.

Preservar skip link, landmarks, hierarquia de títulos, links reais, `aria-current` e mudança de foco/título ao navegar. O calendário possui 42 dias com datas completas acessíveis e `aria-current="date"` no dia atual; nenhuma célula vazia simula ação de agendamento. Campos não dependem apenas de placeholder.

Contrastes calculados pela luminância relativa sRGB: texto principal/branco 14,08:1; secundário/branco 4,93:1; ação/branco 5,25:1; foco/fundo 5,23:1; sucesso 6,06:1; atenção 6,00:1; erro 6,03:1; contorno de controle/branco 3,21:1. Referências: [WCAG — contraste de texto](https://www.w3.org/WAI/WCAG22/Understanding/contrast-minimum.html) e [contraste não textual](https://www.w3.org/WAI/WCAG22/Understanding/non-text-contrast.html). São verificações pontuais, não certificação completa WCAG.

Hover/focus/active usam alteração discreta de cor (120 ms). Respeitar `prefers-reduced-motion`. Não remover foco visível. Testar teclado em componentes interativos e janela WebView2, além de testes DOM.

## Navegação e responsividade

Sidebar com marca, unidade e grupos Planejamento (Dashboard/Biblioteca/Calendário), Conteúdo (Importações/Revisão/Nova publicação) e Acompanhamento (Histórico); Configurações permanece ao final. Seleção usa fundo suave, borda verde e peso de texto. Topbar pequena informa unidade, página e estado real do armazenamento.

Sidebar padrão 216 px; abaixo de 1100 px, 194 px. Conteúdo com `minmax(0, 1fr)`, largura máxima 1800 px e padding 20–36 px conforme viewport. Painéis e toolbars podem quebrar linhas; em janelas menores composições passam a uma coluna. Até 720 px a navegação vira faixa horizontal interna; o documento não deve transbordar horizontalmente. Scroll vertical permanece no documento; sidebar só rola quando necessário.

Referências desktop: 1366×768, 1600×900 e monitor 1920×1080; adicional 1024×768. Dimensões de conteúdo e dimensões externas da janela são diferentes devido à moldura Windows. Registros exatos e limitações em [Fase 02](FASE_02_IDENTIDADE_INTERFACE.md).
