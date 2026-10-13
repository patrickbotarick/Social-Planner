# Decisões técnicas

Data da verificação: 13/10/2026. Escopo limitado à Fase 01.

## ADR 001 — Aplicativo local-first

Tauri 2 + React + TypeScript, sem servidor ou serviço pago obrigatório. O processo Rust controla dados e futuras integrações. Dados editoriais permanecem no perfil Windows; o frontend não recebe acesso arbitrário ao filesystem. Mantém operação local offline e atende o modelo de um operador da loja.

## ADR 002 — SQLite via rusqlite, sem plugin SQL exposto

Comandos de domínio específicos preservam a fronteira do desktop. `rusqlite` com `bundled` incorpora SQLite e elimina instalação independente. Uma conexão sincronizada é suficiente nesta fase. Não introduzir ORM, pool, engine de sincronização ou WAL sem necessidade demonstrada. Migrações usam transação única, histórico de SQL e verificação de integridade; nunca reset silencioso.

## ADR 003 — Menor esquema necessário

Somente `schema_migrations` e `app_settings` são implementadas. As nove entidades de produto têm contratos documentados. Criar tabelas editoriais antes de seus casos de uso congelaria hipóteses prematuras e violaria o roadmap.

## ADR 004 — Navegação sem dependência adicional

Links de hash dão navegação, acesso direto e histórico para oito destinos fixos. Uma página reutilizável explica recursos futuros e desabilita ação. Não há dados fictícios, contadores editoriais inventados ou conexão simulada ao banco na prévia web.

## ADR 005 — Design provisório e dependências mínimas

CSS Modules no layout e tokens globais, fontes do sistema e SVGs locais. Sem biblioteca visual, fonte remota ou pacote de ícones. Símbolo provisório próprio em SVG, convertido pela CLI Tauri em PNG/ICO. Identidade oficial será tratada na Fase 02.

## ADR 006 — Versões verificadas antes da instalação

Consultas `npm view` incluíram `version`, `engines` e `peerDependencies`; npm instalou sem incompatibilidades reportadas. Dependências diretas são fixadas e `package-lock.json` registra a resolução transitiva.

| Tecnologia | Versão escolhida | Evidência/compatibilidade |
| --- | --- | --- |
| Node / npm do ambiente | 24.11.1 / 11.6.2 | Node satisfaz Vite, Vitest, ESLint e jsdom selecionados |
| Tauri Rust / API JS | 2.12.3 / 2.12.3 | Série 2 exigida; sem usar Tauri 3 alpha |
| Tauri CLI / tauri-build | 2.12.2 / 2.7.2 | Pacotes do ecossistema possuem versões próprias |
| React / React DOM | 19.3.0 | Pares idênticos; Testing Library aceita React 19 |
| TypeScript | 5.9.3 | `strict`; versão 7.0.2 vigente não cabe no peer `<6.1.0` de typescript-eslint 8.71.2 |
| Vite / plugin-react | 8.3.4 / 6.1.2 | plugin exige Vite 8 |
| Vitest | 5.0.4 | Aceita Vite 8 e Node 24 |
| ESLint / typescript-eslint | 10.12.0 / 8.71.2 | Peers consultados; lint executado sem erros ou warnings |
| jsdom | 27.4.0 | Aceita Node >=24.0; jsdom 30.1.2 requer Node 24.15+, superior ao instalado |
| rusqlite | 0.40.2 | Última versão estável consultada; feature `bundled` |
| SQLite auxiliar dos testes Node | 3.50.4 | Só testes SQL, não é o driver operacional Rust |

Na Fase 01.5, Rust/Cargo 1.99.0 e rustup 1.29.1 foram instalados com autorização, juntamente com Build Tools 2022/MSVC e SDK. `Cargo.lock` registra 419 pacotes resolvidos, incluindo serde 1.0.229 e tempfile 3.27.0. `cargo check`, seis testes Rust, Clippy e build release passaram nesta toolchain. A versão mínima Rust não foi pesquisada/testada separadamente; a combinação instalada está comprovada. Evidências completas em FASE_01_5_VALIDACAO_WINDOWS.md.

Fontes oficiais consultadas:

- [Pré-requisitos Windows do Tauri](https://v2.tauri.app/start/prerequisites/).
- [Configuração do Tauri](https://v2.tauri.app/reference/config/).
- [Capabilities e comandos de aplicação](https://v2.tauri.app/security/capabilities/).
- [Tauri 2.12.3 no docs.rs](https://docs.rs/crate/tauri/2.12.3).
- [tauri-build 2.7.2](https://docs.rs/crate/tauri-build/2.7.2).
- [rusqlite 0.40.2](https://docs.rs/crate/rusqlite/0.40.2).
- [Requisitos do Vite](https://vite.dev/guide/).
- [Requisitos do Vitest](https://vitest.dev/guide/).
- Metadados do registro oficial npm obtidos pela CLI, com versões exatas no manifesto/lockfile.

## ADR 007 — Desenvolvimento sem porta reservada

O script desktop cria Vite na porta 0, lê a porta escolhida e envia `devUrl` à CLI. Assim não se fixa uma porta só para casar configurações. Se Cargo estiver ausente, informa o pré-requisito antes de iniciar servidores. A prévia web usa o fallback de portas do Vite e aceita `--port 0`.

## ADR 008 — Meta exige pesquisa própria

Não assumir que publicar via API equivale a agendar no Business Suite. A Fase 07 deve comprovar suporte oficial e comportamento com computador desligado; caso contrário usar fluxo assistido. Nenhuma automação de navegador ou token foi implementado. “Agendado localmente”, “enviado”, “confirmado” e “publicado” têm significados distintos.

## ADR 009 — Validação honesta em ambiente incompleto

Vitest cobre navegação, estados e contrato IPC com mocks explícitos. `node:sqlite` verifica o SQL real e persistência em arquivo temporário, sem dependência extra. Os seis testes Rust exercitam repositório, migrações, corrupção e preservação e foram executados com sucesso na Fase 01.5. A janela nativa confirmou IPC e banco reais. A instalação de toolchains de sistema foi autorizada explicitamente antes da execução.

## ADR 010 — Watcher frontend ignora arquivos Rust

A validação nativa encontrou recarga incidental do Vite ao gerar assets HTML durante o build release em `src-tauri/target`. O watcher frontend ignora `**/src-tauri/**`; o watcher próprio do Tauri continua responsável pelo Rust. A configuração evita recargas por artefatos nativos sem alterar domínio, persistência ou escopo de produto.
