# Fase 02 — Identidade visual e experiência desktop

Data: 13/10/2026. Base: fundação `6c13397` e validação Windows `e1806d0`. Estado: implementação e validação técnica concluídas; evidências disponíveis para apreciação visual do usuário. Nenhuma funcionalidade da Fase 03 foi iniciada.

## Entrega e decisões

O aplicativo foi transformado em um espaço editorial com sidebar estável, topbar compacta, hierarquia de títulos e superfícies claras. Verde profundo identifica o software, verde de ação indica interação e laranja aparece no detalhe da marca. Não há gradientes, sombras fortes, métricas fictícias ou dependência de fontes remotas. A identidade é uma proposta para o software, não declaração dos códigos oficiais da GMAD.

Inter Variable 4.1 foi incorporada localmente com licença SIL OFL integral. SHA-256 do WOFF2: `693B77D4F32EE9B8BFC995589B5FAD5E99ADF2832738661F5402F9978429A8E3`. Não houve instalação de fontes no sistema nem adição de dependências npm/Rust. Tokens, tipografia, medidas, estados, acessibilidade e contratos estão em [DESIGN_SYSTEM.md](DESIGN_SYSTEM.md).

O shell e a navegação por hash existentes foram reaproveitados. Cada uma das oito áreas agora tem uma composição própria. O símbolo existente recebeu a paleta do software e os ícones PNG/ICO Windows foram regenerados pela CLI Tauri. O script de desenvolvimento aceita `--window-size` transitório para reproduzir o QA, mantendo porta livre e configuração persistente intacta.

| Área | Entrega da Fase 02 | Limite explícito |
| --- | --- | --- |
| Dashboard | Resumo editorial compacto, próximas publicações vazias e três atalhos de navegação | Contagens indisponíveis representadas por travessão; não inventar zero ou consultar entidades inexistentes |
| Biblioteca | Pesquisa, filtros, alternância de visualização e grid vazio | Controles desabilitados; nenhum CRUD ou registro de exemplo |
| Importações | Campo de URL, orientação do fluxo e resultados vazios | Nenhuma conexão Trello ou importação |
| Revisão | Mídia/origem à esquerda e legenda/ações à direita | Aprovar, ignorar, salvar e navegar entre posts indisponíveis |
| Calendário | Grade mensal de 42 dias, dia atual pelo fuso existente, anterior/próximo/Hoje funcionais | Navegação exclusivamente visual; sem agendamento, drag and drop ou alterações persistentes |
| Nova publicação | Estrutura de mídia, título, formato e legenda | Upload e salvamento indisponíveis |
| Histórico | Filtros e tabela sem eventos | Sem eventos simulados ou integração |
| Configurações | Abas Preferências/Armazenamento/Integrações, fuso real em consulta, diagnóstico e cópia do caminho | Trello/Meta indisponíveis; sem alteração de preferências nesta fase |

Componentes: AppShell, Sidebar, Topbar, PageHeader, Button, IconButton, Input, Textarea, Select, Badge, Card, EmptyState, Modal, ConfirmDialog, ToastProvider/useToast, Tabs, SearchField, Toolbar, MediaThumbnail, Skeleton e Tooltip. ConfirmDialog permanece preparado para futuras decisões, exercitado apenas em testes isolados; a ajuda é o único modal de produto nesta fase.

## Preservação da fundação

Contratos de `foundation_status`, serviço IPC, hook de inicialização, tipos da fundação, código Rust, migração `001_foundation.sql`, identificador da aplicação, capability e CSP não foram alterados. Package/Cargo lockfiles e dependências permaneceram iguais. SQLite continua exclusivo do Rust.

Banco operacional observado: `%APPDATA%\br.com.gmad.socialplanner\planner.sqlite3`. Preferência real `America/Sao_Paulo`, versão do esquema 1. A interface nativa exibiu banco conectado e integridade concluída; “Verificar novamente” passou pelo estado de carregamento e retornou ao resultado real. “Copiar caminho” exibiu toast de sucesso.

Antes da implementação, foi registrado um snapshot somente leitura das preferências e do histórico de migrações em `.tmp/`, fora do Git. Depois de várias reaberturas de desenvolvimento e da abertura do executável release, leitura `DatabaseSync(..., { readOnly: true })` confirmou `PRAGMA quick_check = ok` e igualdade dos valores, timestamps e registros de migração. Datas de criação/atualização da preferência e aplicação da migração permaneceram `2026-10-13T18:51:24.317Z`. Nenhum dado operacional foi apagado, recriado ou incluído no repositório.

## Testes executados

Todos os comandos abaixo terminaram com código 0. O PATH de processo foi acrescido de `C:\Users\patrick.silva\.cargo\bin` para receber Rust já instalado; nenhuma nova alteração global de ambiente foi necessária nesta fase.

| Comando | Resultado final |
| --- | --- |
| `npm.cmd run check` | Typecheck, lint com zero warnings, frontend, SQL e build aprovados |
| `npm.cmd run typecheck` | TypeScript estrito aprovado, também executado dentro de check/build |
| `npm.cmd run lint` | ESLint aprovado |
| `npm.cmd test` | 21 testes, 3 arquivos, aprovados |
| `npm.cmd run test:sql` | 2 testes SQLite aprovados |
| `cargo check --locked --manifest-path src-tauri/Cargo.toml` | Aprovado |
| `cargo test --locked --manifest-path src-tauri/Cargo.toml` | 6 testes Rust aprovados; testes main/doc sem casos adicionais |
| `npm.cmd run check:rust` | rustfmt e Clippy em todos os targets aprovados, zero warnings |
| `npm.cmd run build` | Vite 8.3.4, 47 módulos; ativos frontend compilados |
| `npm.cmd run build:desktop` | Release aprovado em 24,15 s; executável em `src-tauri/target/release/gmad-social-planner.exe`; sem instalador |
| `npm.cmd run dev -- --window-size …` | Janela nativa aberta em cada dimensão de QA; `DATABASE_READY` a cada reabertura |
| `node .tmp/verify-phase02-data.mjs` | Integridade e igualdade do snapshot real aprovadas, em modo somente leitura |
| `git diff --check` | Sem erros de whitespace |

Total: **29 testes aprovados, nenhum reprovado no resultado final**. O aviso experimental de `node:sqlite` no Node 24 é do auxiliar SQL; não é falha nem dependência SQLite da aplicação Tauri.

Os novos testes cobrem label/erro acessível, ação desabilitada, abas por setas/Home/End, foco inicial/contenção/restauração no diálogo, cancelamento sem confirmação, tooltip por teclado, toast dispensável, fronteira UTC do fuso, fevereiro bissexto e calendário visual. Os testes existentes de navegação, IPC, falhas, persistência e proteção das migrações foram preservados e ajustados às novas composições.

Ambiente usado: Windows x64, Node 24.11.1/npm 11.6.2, Rust/Cargo 1.99.0 MSVC x64, rustup 1.29.1, Build Tools 2022 17.14.41, MSVC 19.44.35229, SDK 10.0.26100.0 e WebView2 155.0.4283.45, já preparados na Fase 01.5.

## Validação visual nativa

A janela inspecionada foi `gmad-social-planner.exe` hospedada pelo Tauri/WebView2. A prévia em navegador foi usada apenas como verificação complementar de geometria e teclado DOM.

| Referência | Inspeção Windows e evidência |
| --- | --- |
| 1366×768 | Todas as oito áreas; abas de Configurações; banco/IPC; calendário e ajuda. Capturas externas 1368×800, incluindo moldura |
| 1600×900 | Dashboard e Revisão; colunas, mídia, legenda e ações legíveis. Capturas externas 1602×932 |
| Monitor 1920×1080 | Calendário em janela maximizada: grade completa, dia atual e navegação; captura 1920×1080 inclui barra de título |
| 1024×768 | Dashboard e Biblioteca: controles e conteúdo sem transbordamento horizontal. Capturas externas 1026×800 |
| Release padrão | Executável release aberto em `http://tauri.localhost/`, Dashboard e armazenamento conectado observados, sem servidor Vite |

O parâmetro `--window-size` configura a área de conteúdo, não o tamanho externo. Para Full HD a janela foi maximizada para respeitar a área do monitor; não se afirma que a área útil nativa tenha exatamente 1920×1080 após descontar a barra de título. A matriz complementar testou viewports exatos 1366×768, 1600×900, 1920×1080 e 1024×768 nas oito rotas: **32 verificações sem scroll horizontal**, com Inter carregada localmente. Valores de `documentElement.scrollWidth/clientWidth` em [matriz-layout.json](evidencias/fase02/matriz-layout.json). Os ajustes posteriores de densidade do Dashboard, largura do campo de pesquisa e foco foram conferidos por testes e na janela nativa. Console da prévia sem erros/warnings na inspeção.

A navegação nativa funcionou nas oito áreas; o calendário avançou a novembro e voltou a outubro pelo botão Hoje, destacando 13/10 no fuso existente. Ajuda abriu diálogo identificado, Tab manteve foco no controle de fechar e Escape restaurou o foco ao botão de ajuda; contorno visível e tooltip registrados. Contrastes pontuais estão no design system.

Capturas integrais da janela, sem edição: [Dashboard 1366](evidencias/fase02/dashboard-1366x768.jpg), [Dashboard 1600](evidencias/fase02/dashboard-1600x900.jpg), [Dashboard 1024](evidencias/fase02/dashboard-1024x768.jpg), [Biblioteca 1366](evidencias/fase02/biblioteca-1366x768.jpg), [Biblioteca 1024](evidencias/fase02/biblioteca-1024x768.jpg), [Importações](evidencias/fase02/importacoes-1366x768.jpg), [Revisão 1366](evidencias/fase02/revisao-1366x768.jpg), [Revisão 1600](evidencias/fase02/revisao-1600x900.jpg), [Calendário 1366](evidencias/fase02/calendario-1366x768.jpg), [Calendário Full HD](evidencias/fase02/calendario-1920x1080.jpg), [Nova publicação](evidencias/fase02/nova-publicacao-1366x768.jpg), [Histórico](evidencias/fase02/historico-1366x768.jpg), [Preferências](evidencias/fase02/configuracoes-preferencias-1366x768.jpg), [Armazenamento](evidencias/fase02/configuracoes-armazenamento-1366x768.jpg), [Integrações](evidencias/fase02/configuracoes-integracoes-1366x768.jpg), [Cópia com feedback](evidencias/fase02/copiar-caminho-feedback-1366x768.jpg), [Ajuda com foco](evidencias/fase02/ajuda-foco-1366x768.jpg) e [Foco restaurado](evidencias/fase02/ajuda-foco-restaurado-1366x768.jpg). Algumas mostram o cursor ou a sobreposição do pet do desktop, que não pertencem à interface do aplicativo.

## Problemas encontrados e corrigidos

- O campo interno da pesquisa herdava uma largura menor que seu container e cortava o placeholder; passou a ocupar a largura reservada e foi conferido em 1366 e 1024.
- O diálogo nativo com um único controle permitia Tab avançar para fora em um ambiente de navegador; contenção explícita de Tab/Shift+Tab adicionada, com teste automatizado, navegador e confirmação no WebView2.
- O Dashboard consumia altura com diagnóstico duplicado; o estado conectado fica resumido na Topbar e os detalhes reais continuam em Configurações. Erro/carregamento/prévia web continuam visíveis no Dashboard.
- A capitalização do calendário foi restrita à primeira letra, preservando “Outubro de 2026”.
- Tipagem estrita de propriedades opcionais do tooltip e expectativas de testes da interface antiga foram ajustadas; resultado final sem erros.

Não foram encontrados problemas de compatibilidade rusqlite/Tauri nesta fase; código nativo e arquitetura permanecem intactos.

## Arquivos relevantes

- `src/styles/{tokens,global,pages}.css`: identidade, primitivas e composições responsivas.
- `src/components/ui/*`, `PageHeader.tsx`, `StorageStatus.tsx`, `Icon.tsx`: componentes e diagnóstico preservado.
- `src/layouts/{AppLayout,Sidebar,Topbar}.tsx` e `AppShell.module.css`: shell e navegação; antigo AppLayout.module.css removido.
- `src/pages/*`, `src/app/App.tsx`, `src/main.tsx`, `src/utils/calendar.ts`: oito páginas e composição; PlannedPage genérica substituída.
- `tests/{app,design-system}.test.tsx`, `tests/setup.ts`: regressão e acessibilidade.
- `public/brand.svg`, `public/fonts/*`, `src-tauri/icons/{icon.png,icon.ico}`: ativos locais.
- `scripts/desktop-dev.mjs`: override transitório de dimensões para QA.
- `docs/DESIGN_SYSTEM.md`, este relatório, `PROJETO_MESTRE.md`, `ROADMAP.md`, `ARQUITETURA.md`, evidências, `README.md`, `AGENTS.md` e `.gitattributes`: documentação e regras persistentes.

## Aceite, limites e continuidade

| Critério solicitado | Resultado |
| --- | --- |
| Design system documentado e consistente | Atendido |
| Oito áreas coerentes e navegação desktop | Atendido |
| Componentes reutilizáveis, foco e contraste | Atendido nas verificações descritas |
| Ações futuras explícitas, sem métricas inventadas | Atendido |
| SQLite, IPC e preferências preservados | Atendido por janela nativa e comparação somente leitura |
| Validação visual Windows e resoluções | Atendido, com distinção de área útil/monitor descrita acima |
| Testes e builds | Atendido: 29 testes, typecheck, lint, cargo check e builds |
| Documentação atualizada | Atendido |
| Commit da fase | Alterações entregues no commit `feat: establish GMAD Social Planner design system`; hash informado no relatório final |

Não há bloqueio técnico remanescente da Fase 02. A apreciação estética final pertence ao usuário; não foi presumida aprovação humana. As verificações não equivalem a certificação WCAG completa, cobertura de todos os fatores de escala/DPI ou teste em Windows limpo. Instalador, atualização e distribuição continuam na Fase 09. SQLite tem somente preferências e migrações; criação/importação/revisão/planejamento reais permanecem no roadmap.

Git estava limpo antes da fase. Somente código, ativos locais licenciados, testes, documentação e evidências da interface entram no commit. Não houve push ou tag. A Fase 03 está **tecnicamente pronta para ser solicitada**, mas não foi iniciada automaticamente.
