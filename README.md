# GMAD Social Planner

Aplicativo Windows local-first para organizar o conteúdo editorial de Instagram da GMAD Rio Preto, recebido mensalmente do Trello da matriz.

**Estado:** Fases 01–05 validadas, incluindo [homologação Trello real](docs/FASE_04_5_HOMOLOGACAO_TRELLO.md) e [revisão editorial](docs/FASE_05_REVISAO_EDITORIAL.md). A [Fase 06](docs/FASE_06_CALENDARIO_EDITORIAL.md) implementa calendário persistente, reagendamento, conflitos e histórico local: 152 testes aprovados, builds e persistência no Tauri Windows. Cancelamento nativo e persistência homologados no QA, preservando publicações e mídias; aceite técnico concluído. A Fase 07 concluiu a pesquisa documental de viabilidade Meta, recomendando preparação local e agendamento manual no Business Suite; integração não implementada e conta real não homologada. Fases 08–09 não iniciadas. Nenhuma publicação externa é executada.

## Pré-requisitos

- Windows 10/11 x64 com WebView2. Neste computador, WebView2 155.0.4283.45 foi detectado.
- Node.js 24 LTS e npm. Ambiente verificado: Node 24.11.1 e npm 11.6.2.
- Git, disponível neste computador.
- Para a janela desktop: **Microsoft C++ Build Tools**, workload **Desktop development with C++**, compilador MSVC x64/x86 e Windows SDK; **Rust estável com target MSVC**.

Neste computador já estão instalados Rust/Cargo 1.99.0 (MSVC x64), rustup 1.29.1, Build Tools 2022 17.14.41 e Windows SDK 10.0.26100.0. O PATH de usuário inclui `.cargo\bin`. Para preparar outro computador:

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

O script escolhe uma porta livre de loopback e inicia Vite + Tauri. O primeiro build Rust baixa e compila as crates, podendo demorar. `src-tauri/Cargo.lock` já registra a resolução validada; não removê-lo para atualizar dependências implicitamente. Não editar ou remover banco para corrigir erro de inicialização. Se este terminal estava aberto antes de instalar Rust, abra um terminal novo para receber o PATH atualizado.

Para conferir apenas a interface no navegador:

```powershell
npm.cmd run dev:web
```

Para inspecionar uma dimensão específica da janela nativa: `npm.cmd run dev -- --window-size 1366x768`. O override é temporário; tamanhos aceitos de 640×520 a 2560×1440. A moldura Windows aumenta as dimensões externas; em Full HD use a janela maximizada para ocupar o monitor.

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

Na janela: conferir “Armazenamento conectado” na Topbar e o diagnóstico completo em Configurações → Armazenamento, incluindo caminho do SQLite e integridade; em Preferências, conferir o fuso real. Navegar por todas as áreas; fechar/reabrir; conferir preservação do banco e ausência de erro crítico. O build sem bundle não cria instalador; distribuição pertence à Fase 09.

## Dados e segurança

Banco resolvido via Tauri `app_data_dir()`, normalmente `%APPDATA%\br.com.gmad.socialplanner\planner.sqlite3`. A UI mostra o caminho real. Migração incremental 002 acrescenta publicações, assets e vínculos ordenados, preservando histórico e preferências. Mídias importadas pelo operador ficam em `media/<sha256>.<extensão>` nessa raiz; staging usa `temp/imports/<sessão>`. Originais não são alterados. Nenhum conteúdo de demonstração ou token é inserido na base operacional.

Frontend chama comandos IPC específicos de diagnóstico, biblioteca e Trello. SQLite, seleção nativa, validação e cópia de arquivos pertencem ao Rust. Prévia de mídia usa protocolo asset limitado aos diretórios controlados. Não há SQL ou filesystem arbitrário, login interno, backend remoto ou serviços pagos. Exclusão é lógica e preserva arquivos compartilhados; não existe limpeza física automática. As credenciais Trello usam o cofre do Windows, com acesso exclusivo pelo Rust. Backup ainda é planejamento documentado.

Para cadastrar: Criar publicação → título, formato e legenda → Selecionar mídias → Salvar publicação. Feed aceita uma imagem; carrossel aceita 2–20 mídias ordenadas; Reels aceita um MP4; Stories aceita uma imagem ou MP4. Imagens JPEG/PNG/WebP até 64 MiB e vídeos MP4 até 512 MiB, sujeitos à validação do conteúdo. Reprodução depende do codec disponível no WebView2, sem transcodificação. Legendas preservam emojis e quebras de linha; alterações não salvas têm guarda de saída.

## Importar do Trello

Configure a conexão em Configurações → Trello seguindo o [guia de integração](docs/INTEGRACAO_TRELLO.md). Depois abra Importações, consulte a URL do quadro, confira a prévia/seleção e confirme. Os posts chegam como pendentes de revisão. Reimportar atualiza a origem separadamente, preservando o conteúdo local. Downloads que falham podem ser repetidos; anexos recuperados/novos ficam disponíveis em Origem Trello para inclusão explícita na composição. Não envie tokens pelo chat ou grave-os em arquivos.

## Organização

```text
docs/                    documentação oficial e relatório da fase
public/                  símbolo SVG e Inter local com licença
scripts/                 inicialização desktop com porta livre
src/
  app/                   composição e navegação
  components/            elementos comuns e estados
  features/editorial/    editor e prévias de mídia
  hooks/                 consultas de diagnóstico e biblioteca
  layouts/               moldura responsiva, CSS Modules
  pages/                 biblioteca, importações, detalhes, cadastro e layouts futuros
  services/              ponte IPC tipada
  styles/                tokens e estilos comuns
  types/                 contratos da fundação, biblioteca e Trello
  utils/                 cálculo visual do calendário e fuso
src-tauri/
  capabilities/          permissões mínimas
  icons/                 PNG/ICO da identidade do software
  migrations/            SQL versionado
  src/                   comandos, domínio, repositórios e armazenamento Rust
tests/                   componentes, migrações SQL e fixture MP4 licenciada
```

Diretrizes visuais e contratos dos componentes em [Design system](docs/DESIGN_SYSTEM.md). Testes Rust usam banco/arquivos temporários; `GMAD_QA_DATA_DIR` permite uma raiz isolada somente em debug, sem mudar a localização de produção.

## Documentação oficial

- [Viabilidade Meta — Fase 07](docs/FASE_07_VIABILIDADE_META.md).
- [Contratos futuros da integração Meta](docs/INTEGRACAO_META.md).
- [Decisão recomendada de agendamento](docs/DECISAO_AGENDAMENTO_META.md).
- [Projeto mestre](docs/PROJETO_MESTRE.md) — fonte de escopo e contratos.
- [Arquitetura](docs/ARQUITETURA.md).
- [Modelo de dados](docs/MODELO_DADOS.md).
- [Roadmap de nove fases](docs/ROADMAP.md).
- [Decisões técnicas e versões](docs/DECISOES_TECNICAS.md).
- [Resultado da Fase 01](docs/FASE_01_FUNDACAO.md).
- [Preparação e validação Windows — Fase 01.5](docs/FASE_01_5_VALIDACAO_WINDOWS.md).
- [Design system](docs/DESIGN_SYSTEM.md).
- [Identidade e interface — Fase 02](docs/FASE_02_IDENTIDADE_INTERFACE.md).
- [Biblioteca editorial — Fase 03](docs/FASE_03_BIBLIOTECA_EDITORIAL.md).
- [Integração Trello — configuração e limites](docs/INTEGRACAO_TRELLO.md).
- [Entrega e validação da Fase 04](docs/FASE_04_INTEGRACAO_TRELLO.md).
- [Instruções persistentes para agentes](AGENTS.md).

Não avançar automaticamente para a Fase 08. Não executar publicação externa sem autorização explícita. Git ignora dependências, builds, bancos operacionais, mídias e segredos; push exige autorização (fornecida especificamente para as entregas das Fases 06 e 07 após validação e auditoria).

## Revisão editorial

Abra Revisão, filtre a fila e entre em uma publicação. Confira mídia/legenda, edite se necessário e escolha Aprovar ou Ignorar; motivo e observação são opcionais. Aprovadas/ignoradas podem retornar à revisão. Editar uma decisão já tomada exige nova revisão ao salvar. Na listagem, selecione itens e confirme a ação em lote; erros são informados por item. Consulte Histórico para transições e versões. Setas navegam fora dos campos; Escape fecha diálogos. O aplicativo oferece salvar, descartar ou permanecer ao sair com alterações. Aprovação é exclusivamente editorial, sem agendar/publicar.

## Calendário editorial local

Abra Calendário, escolha um dia e use Agendar em uma publicação aprovada disponível. Informe data, hora e fuso IANA e confirme. Também é possível arrastar da fila ou mover um item entre dias; o arraste abre o formulário antes de gravar. Use os detalhes do dia ou a visualização Lista para alterar data/horário ou remover do planejamento, preservando publicação e mídias. Filtros ficam recolhidos para aproveitar o espaço; Horários e proximidade configura sugestões e alertas. O Dashboard mostra os próximos planos; Histórico → Planejamento local registra as alterações.

Editar uma publicação aprovada invalida seu plano na mesma transação; revise, aprove e planeje novamente explicitamente. Conflitos são avisos locais. Computador desligado não executa qualquer publicação. Detalhes, testes e limites em [Fase 06](docs/FASE_06_CALENDARIO_EDITORIAL.md).
