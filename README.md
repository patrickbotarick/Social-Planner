# GMAD Social Planner

Aplicativo Windows local-first para organizar o conteúdo editorial de Instagram da GMAD Rio Preto, recebido mensalmente do Trello da matriz.

**Estado:** Fase 01 implementada, com aceite desktop pendente por ausência de Rust e C++ Build Tools neste ambiente. Interface, documentação, SQLite mínimo e testes preparados. Biblioteca, Trello, revisão, calendário e Meta ainda não estão implementados. Nenhuma publicação real é executada. Resultado de validação em [docs/FASE_01_FUNDACAO.md](docs/FASE_01_FUNDACAO.md).

## Pré-requisitos

- Windows 10/11 x64 com WebView2. Neste computador, WebView2 155.0.4283.45 foi detectado.
- Node.js 24 LTS e npm. Ambiente verificado: Node 24.11.1 e npm 11.6.2.
- Git, disponível neste computador.
- Para a janela desktop: **Microsoft C++ Build Tools**, workload **Desktop development with C++**, compilador MSVC x64/x86 e Windows SDK; **Rust estável com target MSVC**.

Rust/Cargo e Build Tools não foram encontrados. Para preparar uma vez:

1. Baixe o [instalador oficial do Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/). Selecione **Desktop development with C++**, incluindo MSVC x64/x86 e Windows 10 ou 11 SDK, e instale.
2. Baixe e execute [rustup-init.exe oficial](https://rustup.rs/). Use a instalação padrão com host `x86_64-pc-windows-msvc`.
3. Reabra o terminal (e o aplicativo de desenvolvimento se o PATH estiver antigo). Confirme `cargo --version` e `rustc --version`.
4. Caso falte WebView2 em outro PC, instale o [Evergreen Runtime oficial](https://developer.microsoft.com/microsoft-edge/webview2/).

Detalhes: [pré-requisitos oficiais do Tauri](https://v2.tauri.app/start/prerequisites/). Não é necessário instalar SQLite separadamente; o Rust o incorpora. Internet é usada para instalar dependências; as telas locais não dependem de internet depois da instalação.

## Instalação e inicialização

Na pasta do projeto, primeira instalação:

```powershell
npm.cmd ci
```

Iniciar a janela desktop, depois de instalar os pré-requisitos:

```powershell
npm.cmd run dev
```

O script escolhe uma porta livre de loopback e inicia Vite + Tauri. O primeiro build Rust baixa e compila as crates, podendo demorar. Após a primeira resolução Cargo, versionar `src-tauri/Cargo.lock` e executar os testes Rust. Não editar ou remover banco para corrigir erro de inicialização.

Para conferir apenas a interface agora, sem Rust:

```powershell
npm.cmd run dev:web
```

Abra a URL exibida no terminal. A prévia informa que o SQLite só existe na janela desktop e não simula dados. `npm.cmd` evita bloqueio de `npm.ps1` por políticas do PowerShell, sem alterar a política do sistema.

## Scripts

| Comando (`npm.cmd run …`) | Função |
| --- | --- |
| dev | Vite em porta livre + janela Tauri; exige Cargo |
| dev:web | Prévia da interface no navegador |
| typecheck | TypeScript estrito, sem emissão |
| lint | ESLint, zero warnings |
| test / test:watch | Vitest em execução única / observação |
| test:sql | SQL real, integridade e persistência via SQLite nativo do Node |
| build | Typecheck e build frontend em `dist/` |
| check | Typecheck, lint, Vitest, testes SQL e build frontend |
| test:rust | Testes reais do repositório Rust e migrações |
| check:rust | rustfmt e Clippy, sem warnings |
| build:desktop | Compila executável desktop sem instalador |
| preview | Prévia do build frontend |
| tauri -- info | Diagnóstico dos pré-requisitos Tauri |

Validação local completa quando Rust estiver disponível:

```powershell
npm.cmd run check
npm.cmd run test:rust
npm.cmd run check:rust
npm.cmd run build:desktop
npm.cmd run dev
```

Na janela: conferir “Banco conectado”, caminho do SQLite e fuso real; navegar por todas as áreas; fechar/reabrir; conferir preservação do banco e ausência de erro crítico. O build sem bundle não cria instalador; distribuição pertence à Fase 09.

## Dados e segurança

Banco resolvido via Tauri `app_data_dir()`, normalmente `%APPDATA%\br.com.gmad.socialplanner\planner.sqlite3`. A UI mostra o caminho real. Não fica dentro da instalação. Esquema mínimo: histórico de migrações e preferência de fuso. Nenhum token, mídia editorial ou publicação de exemplo é gravado.

Frontend chama apenas `foundation_status`. SQLite pertence ao Rust; não há SQL arbitrário, login interno, backend remoto ou serviços pagos. Segredos de integrações futuras devem usar o cofre do Windows. Backup ainda é planejamento documentado, não função disponível.

## Organização

```text
docs/                    documentação oficial e relatório da fase
public/                  símbolo SVG provisório local
scripts/                 inicialização desktop com porta livre
src/
  app/                   composição e navegação
  components/            elementos comuns e estados
  hooks/                 consulta de diagnóstico
  layouts/               moldura responsiva, CSS Modules
  pages/                 Dashboard, Configurações, áreas futuras
  services/              ponte IPC tipada
  styles/                tokens e estilos comuns
  types/                 contratos da fundação
src-tauri/
  capabilities/          permissões mínimas
  icons/                 PNG/ICO provisórios
  migrations/            SQL versionado
  src/                   comandos, erros e repositório Rust
tests/                   navegação, IPC e migração SQL
```

Pastas `features` e `utils` surgirão quando tiverem conteúdo real; não há arquivos vazios para simular arquitetura.

## Documentação oficial

- [Projeto mestre](docs/PROJETO_MESTRE.md) — fonte de escopo e contratos.
- [Arquitetura](docs/ARQUITETURA.md).
- [Modelo de dados](docs/MODELO_DADOS.md).
- [Roadmap de nove fases](docs/ROADMAP.md).
- [Decisões técnicas e versões](docs/DECISOES_TECNICAS.md).
- [Resultado da Fase 01](docs/FASE_01_FUNDACAO.md).
- [Preparação e validação Windows — Fase 01.5](docs/FASE_01_5_VALIDACAO_WINDOWS.md).
- [Instruções persistentes para agentes](AGENTS.md).

Não avançar automaticamente para a Fase 02. Não executar publicação real sem autorização explícita. Git ignora dependências, builds, bancos operacionais, mídias e segredos; não há push automático.
