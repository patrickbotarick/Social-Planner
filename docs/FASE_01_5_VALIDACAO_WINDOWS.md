# Fase 01.5 — Preparação e validação Windows

Data: 13/10/2026 (America/Sao_Paulo). Base: `6c13397` — `feat: initialize GMAD Social Planner foundation`.

**Estado atual: diagnóstico e preparação executados; instalação aguarda autorização do usuário. A validação nativa e o aceite completo da Fase 01 permanecem bloqueados.** Nenhuma funcionalidade da Fase 02 foi implementada.

## Diagnóstico inicial

AGENTS.md, README.md e PROJETO_MESTRE.md foram lidos antes das alterações. `git status --short` estava vazio; o repositório estava no commit de fundação.

| Componente | Resultado observado |
| --- | --- |
| Windows | Microsoft Windows 11 Pro, 64 bits, versão 10.0.26300 confirmada por CIM; registro: build 26300.9550, DisplayVersion 26H2. O campo legado ProductName informa Windows 10 Pro e não foi usado para inferir a edição atual |
| Node.js | 24.11.1, `C:\Program Files\nodejs\node.exe` |
| npm | 11.6.2, `C:\Program Files\nodejs\npm.cmd` |
| rustc | Ausente no PATH; diretório padrão de toolchains inexistente |
| Cargo | Ausente no PATH; `%USERPROFILE%\.cargo\bin` inexistente |
| rustup | Ausente no PATH; `%USERPROFILE%\.rustup\toolchains` inexistente |
| Microsoft C++ Build Tools | Não encontrado nos diretórios padrão nem entre programas registrados; vswhere ausente |
| MSVC x64/x86 | Não disponível; `cl` não encontrado e nenhuma instalação Visual Studio/Build Tools localizada |
| Windows SDK | Não encontrado em Windows Kits/10/Include nem em Installed Roots do registro |
| WebView2 | Versão 155.0.4283.45 já presente; também existe diretório 154.0.4258.62. Não requer reinstalação |

O PATH de usuário e máquina foi lido sem alteração: Node está disponível; `.cargo\bin` não está configurado. Não foram encontradas variáveis CARGO_HOME, RUSTUP_HOME, VSCMD, VCINSTALLDIR ou WindowsSdkDir que apontassem para uma instalação alternativa. Ausência de `cl` em terminal comum, isoladamente, não prova ausência de MSVC; a conclusão usa também diretórios e registro.

Somente Rust/rustup/Cargo, Build Tools/MSVC e Windows SDK precisam de instalação. Não é necessário reinstalar Node, npm ou WebView2, instalar Visual Studio IDE completo, SQLite independente ou habilitar componentes de criação de MSI para este build sem bundle.

## Preparação da instalação

Instaladores baixados de fontes oficiais para `.tmp/windows-setup/`, diretório ignorado pelo Git. **Baixar não significa instalar. Nenhum instalador foi executado nesta etapa.**

| Arquivo | Origem | Verificação local |
| --- | --- | --- |
| rustup-init.exe | https://win.rustup.rs/x86_64, disponibilizado em rustup.rs | 12.721.664 bytes; SHA-256 `6F4BEF66261261FCB43131BE8720BAB817D403A09EDEC7455C371974B90BDB7E` |
| vs_BuildTools.exe | https://aka.ms/vs/17/release/vs_BuildTools.exe | 4.478.032 bytes; versão do bootstrapper 17.14.37710.0; Authenticode **Valid**, Microsoft Corporation; SHA-256 `985969F472CAAD75D993A5CB4C35A6A4271460CC12B343E2433B994D173AA990` |

Os hashes acima identificam os downloads desta execução; não são uma comparação com checksum publicado separadamente. A versão do bootstrapper não representa uma toolchain instalada.

Escopo proposto: Build Tools 2022, workload `Microsoft.VisualStudio.Workload.VCTools`, compilador `Microsoft.VisualStudio.Component.VC.Tools.x86.x64` e SDK `Microsoft.VisualStudio.Component.Windows11SDK.26100`, com suas dependências obrigatórias. Rust estável via rustup, host `x86_64-pc-windows-msvc`, perfil mínimo mais rustfmt e Clippy. Rust no perfil do usuário; `.cargo\bin` no PATH de usuário. Não reiniciar automaticamente.

A instalação administrativa e a alteração persistente de PATH dependem de autorização explícita solicitada nesta fase, conforme a restrição do usuário. Nenhuma ausência de resposta equivale a aprovação.

## Instalação manual, caso escolhida

1. Na pasta do projeto, abra `.tmp\windows-setup\vs_BuildTools.exe`. Autorize o UAC conscientemente. Selecione **Desktop development with C++ / Desenvolvimento para desktop com C++**.
2. Confirme os componentes **MSVC v143 — VS 2022 C++ x64/x86 build tools** e **Windows 11 SDK 10.0.26100**. Instale apenas Build Tools e dependências desse escopo. Não é necessário instalar a IDE Visual Studio ou cargas de trabalho web/mobile.
3. Aguarde o término. Se o instalador exigir reinicialização, salve seu trabalho e decida quando reiniciar; esta tarefa não reiniciará o computador automaticamente.
4. Execute `.tmp\windows-setup\rustup-init.exe`. Use host `x86_64-pc-windows-msvc`, canal stable, perfil default (inclui rustfmt/Clippy), com `.cargo\bin` no PATH de usuário. Alternativamente, o perfil mínimo com rustfmt/Clippy atende às validações sem baixar documentação offline.
5. Abra um terminal novo. Se o Codex não enxergar o PATH atualizado, reabra o aplicativo. Verifique `rustc --version`, `cargo --version`, `rustup show active-toolchain` e `rustup target list --installed`; devem indicar host/target MSVC x64.
6. Informe que a instalação terminou para retomar compilação e inspeção nativa. Não apagar bancos ou dependências existentes.

## Validações executadas antes da instalação

| Comando / validação | Resultado nesta fase |
| --- | --- |
| `node --version` / `npm.cmd --version` | Aprovado: 24.11.1 / 11.6.2 |
| Localização de `rustc --version`, `cargo --version`, `rustup show` | Bloqueada: executáveis ausentes; nenhuma versão Rust instalada foi presumida |
| `cargo check --manifest-path src-tauri/Cargo.toml` | Pré-checagem bloqueada: Cargo não encontrado; não compilou |
| `npm.cmd run typecheck` | Aprovado pela cadeia `npm.cmd run check` |
| `npm.cmd run lint` | Aprovado, zero warnings |
| `npm.cmd test` | Aprovado: 11 testes, 2 arquivos, incluindo navegação e contrato IPC com mocks |
| `npm.cmd run test:sql` | Aprovado: 2 testes; SQL, persistência em arquivo temporário, integridade, constraints e rollback |
| `npm.cmd run build` | Aprovado: 30 módulos, build frontend gerado |
| `npm.cmd run check` | Aprovado, código de saída 0 |
| `npm.cmd run test:rust` | Bloqueado: Cargo não reconhecido; 6 testes Rust ainda não executados |
| `npm.cmd run check:rust` | Bloqueado: Cargo ausente; rustfmt e Clippy pendentes |
| `npm.cmd run build:desktop` | Bloqueado em `cargo metadata`, `program not found` |
| `npm.cmd run dev` | Saiu com código 1 e mensagem de Cargo ausente; não abriu janela Tauri |

`node:sqlite` continua emitindo aviso de API experimental do Node 24.11.1, sem falha nos testes. O SQL auxiliar não comprova integração com rusqlite/Tauri.

## Execução desktop e persistência

O script existente foi inspecionado: verifica Cargo, inicia Vite em loopback com porta dinâmica e entrega a URL à CLI Tauri. A execução parou corretamente na verificação de Cargo. Não houve janela nativa, IPC real, criação de banco operacional, verificação de integridade pelo Rust ou ciclo fechar/reabrir.

O caminho configurado permanece `app.path().app_data_dir()/planner.sqlite3`, normalmente `%APPDATA%\br.com.gmad.socialplanner\planner.sqlite3`. O caminho efetivo e o arquivo operacional só poderão ser confirmados com o processo nativo em execução.

Sem compilação Rust, não é possível concluir que as versões fixadas de Tauri e rusqlite estejam livres de incompatibilidades. Não houve alteração de arquitetura nem correção especulativa de código. `Cargo.lock` ainda não foi gerado.

## Critérios para o aceite completo da Fase 01

- [x] Documentação, frontend, navegação automatizada e testes SQL validados.
- [x] Dependências realmente ausentes identificadas e instaladores oficiais preparados.
- [ ] Autorização e instalação dos pré-requisitos ausentes.
- [ ] Versões Rust/Cargo e host/target MSVC confirmados após instalação.
- [ ] `cargo check`, testes Rust, rustfmt/Clippy e build desktop aprovados.
- [ ] `Cargo.lock` gerado e versionado.
- [ ] Janela Tauri aberta, frontend e navegação conferidos nela.
- [ ] IPC retorna dados reais, schema 1 e integridade ok.
- [ ] Banco operacional no diretório correto do perfil Windows.
- [ ] Aplicativo fechado e reaberto com dados preservados e sem erro crítico.

**Fase 01 sem aceite completo; Fase 02 não liberada por esta validação e não iniciada.**

## Fontes oficiais

- [Pré-requisitos do Tauri no Windows](https://v2.tauri.app/start/prerequisites/).
- [Microsoft C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/).
- [Workload e IDs de componentes Build Tools 2022](https://learn.microsoft.com/en-us/visualstudio/install/workload-component-id-vs-build-tools?view=vs-2022).
- [Rustup oficial](https://rustup.rs/) e [toolchains Windows](https://rust-lang.github.io/rustup/installation/windows.html).

## Git

Estado inicial limpo no commit `6c13397`. Instaladores e saídas de build ficam ignorados. Alterações versionadas desta etapa limitam-se à documentação do diagnóstico e do aceite; nenhum push ou tag autorizado. O identificador do commit de entrega será informado no relatório final e pode ser consultado com `git log -1 --oneline`.
