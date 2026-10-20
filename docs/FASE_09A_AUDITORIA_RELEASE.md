# Fase 09A — auditoria, backup e candidata Windows

Execução iniciada em 19/10/2026, retomada em 20/10/2026. Base: `main`, commit `9a2ed2efedd2bf5798a217a7f71286b90574b4fe`, árvore inicialmente limpa e sincronizada. Escopo autorizado inclui commit e push normal, sem tag, release definitivo ou Fase 09B.

**Estado: implementação e QA local aprovados; encerramento Git e artefato final registrados nesta entrega.** Backup/restauração nativos e executável release em perfil isolado validados. Instalação adiada expressamente pelo usuário; testes de instalação, Windows limpo, reinício do sistema e Meta real permanecem pendências de homologação. Não considerar a versão estável liberada.

## Auditoria funcional e correções

| Área | Evidência e resultado |
| --- | --- |
| Cadastro, edição, biblioteca, mídia e duplicação | Regressão frontend/Rust aprovada; composição, integridade e persistência cobertas pelos testes existentes. Não foram recriados registros operacionais. |
| Trello e retomada | Regressão de identidade remota, edições locais, anexos, redirecionamento controlado e idempotência aprovada. Homologação remota real permanece a da Fase 04.5; nenhuma nova importação ou escrita remota nesta auditoria. |
| Revisão e histórico | Regressão de versões, aprovação/ignorar, lotes e eventos imutáveis aprovada. |
| Calendário | Regressão de fuso/DST, futuro, conflitos, reagendamento, cancelamento e invalidação transacional aprovada. |
| Preparação e transferência | Regressão de elegibilidade, exportação, checksum, pacote desatualizado, declaração, divergência e reconciliação aprovada. Aberturas de pasta/Business Suite e vídeo já homologadas na Fase 08; não equivalem a agendamento real. |
| Dashboard/Configurações | Contagens reais do perfil QA conferidas na janela. Controle inativo de conexão Meta substituído por orientação e acesso ao caminho assistido existente. |
| Backup/restauração | Implementação nova com testes reais de disco/SQLite e testes de interface. Resultado nativo detalhado abaixo. |

Correções identificadas durante QA: snapshot inicialmente exigia fechamento do objeto Backup e conversão do destino para journal DELETE; sincronização Windows exigia abrir o arquivo com permissão de escrita. A promoção da pasta de backup por rename falhou nativamente com `PermissionDenied`, código Windows 5, após verificação completa. O backup usa agora nome estável desde o início e manifesto completo ao final, seguido de inspeção integral. Nenhum sucesso é mostrado antes da verificação. A restauração também encontrou erro 5 na renomeação do perfil; passou após excluir `**/.tmp/**` do observador Vite e reiniciar o ambiente. A comparação aponta interferência do observador nas pastas QA, mas o handle responsável não foi identificado; não atribuímos a falha a antivírus. A troca na restauração continua protegida por diário e tentativas limitadas, sem suprimir erros ou enfraquecer validação. Diagnóstico temporário removido.

## Arquitetura e segurança

`backup.rs` reutiliza rusqlite com feature `backup`, hashes, validação de mídia e checagem de espaço existentes. Nenhuma crate nova. `backup_create`, `backup_inspect`, `backup_restore` são os únicos novos comandos, com permissões específicas. Seletor nativo escolhe caminhos; a restauração recebe ticket opaco mantido no Rust, confirmação explícita e reinspeciona o pacote. Não existe comando frontend para SQL, shell, caminho genérico ou acesso ao cofre.

SQLite Online Backup API produz snapshot consistente de banco aberto, com timeout de 120 segundos. Pacote autossuficiente contém banco, mídia física por checksum e manifesto; preferências já estão no SQLite. Hashes, bytes, whitelist de caminhos, formato/conteúdo de mídia, esquema SQL exato, ledger de migrações, `integrity_check` e foreign keys são verificados. Pastas locais canônicas apenas, sem traversal, credenciais em caminhos, links ou UNC. Esquema continua **6**, migrações 001–006 preservadas.

Restauração cria staging e recuperação verificada antes de fechar o repositório. Diário contém UUID, nunca caminhos arbitrários. Diretório anterior permanece preservado. Na interrupção anterior ao commit, startup tenta rollback conservador, sem criar banco vazio para encobrir erro. Bloqueio exclusivo por perfil evita duas instâncias desta versão; versões antigas precisam ser fechadas. Pacotes assistidos restaurados são desatualizados, declarações exigem revisão e a preferência de exportação é removida. Histórico/editorial e cofre não são apagados. Limites operacionais no [guia de backup](GUIA_BACKUP_RESTAURACAO.md).

CSP de produção mantém scripts locais, sem inline/eval, rede frontend limitada ao IPC, objetos/frames/formulários bloqueados; asset scope continua mínimo. Credenciais Trello permanecem no Credential Manager, sem inclusão no pacote. Downloader continua HTTPS/TLS, hosts e saltos limitados, sem encaminhar autorização para outro host. URLs de abertura permanecem fixas ou validadas pelos contratos existentes. SQL parametrizado, migrações transacionais, triggers e DTOs existentes preservados. Não foram adicionados logs de tokens, conteúdo editorial ou URLs assinadas.

Dependências: `npm audit --json` e `npm audit --omit=dev --json` retornaram zero vulnerabilidades nesta execução. Consulta OSV dos 485 pares públicos nome/versão do Cargo.lock encontrou glib 0.18.5 ([RUSTSEC-2024-0429](https://rustsec.org/advisories/RUSTSEC-2024-0429.html)) e proc-macro-error 1.0.4, não mantido ([RUSTSEC-2024-0370](https://rustsec.org/advisories/RUSTSEC-2024-0370.html)). Ambos estão fora da árvore compilada `x86_64-pc-windows-msvc`, conforme `cargo tree --target ... -i ...`. Não foram forçadas atualizações GTK/Linux incompatíveis com Tauri para esta candidata Windows. `cargo-audit` não estava instalado; não alegamos sua execução nem ausência universal de vulnerabilidades. Nenhum segredo foi enviado: consulta contém somente metadados públicos de dependências.

Referências de implementação: [SQLite Online Backup API](https://www.sqlite.org/backup.html), [distribuição Windows Tauri](https://v2.tauri.app/distribute/windows-installer/) e [configuração Tauri](https://v2.tauri.app/reference/config/).

## Testes finais — 20/10/2026

| Comando | Resultado |
| --- | --- |
| `npm.cmd run check` | Aprovado: TypeScript estrito, ESLint zero warnings, 96 Vitest, 7 SQL, build Vite. |
| `cargo test --manifest-path src-tauri/Cargo.toml --quiet` | 94 aprovados; 5 opt-in ignorados, sem execução remota automática. |
| `cargo check --manifest-path src-tauri/Cargo.toml --quiet` | Aprovado. |
| `npm.cmd run check:rust` | Rustfmt e Clippy em todos os targets locais, `-D warnings`, aprovados. |
| `npm.cmd run build:installer` | Build frontend, executável release MSVC e NSIS final aprovados. Hash final nas notas da candidata; artefatos fora do Git. |

**197 testes aprovados**, cinco opt-in ignorados. Os 14 testes Rust de backup/recuperação estão incluídos nos 94, não somados duas vezes. Cobrem snapshot com WAL e transação concorrente não confirmada, preferências/mídias, repetição sem sobrescrita, manifesto alterado, arquivos ausentes, traversal, futuro incompatível, banco inválido, esquema adulterado mesmo com hash recalculado, espaço/escrita, restauração, recuperação anterior e posterior à promoção, preservação de ambos os conjuntos, diário forjado, exclusividade Windows e reabertura/atualização. Sete testes frontend cobrem seletor, relatório, inspeção, confirmação, cancelamento, erros e guarda de operação.

Falhas intermediárias foram corrigidas e testes repetidos. Uma chamada de testes foi interrompida pelo limite de uso da revisão automática de aprovação; ela não iniciou nem avaliou a ação como insegura. A repetição em 20/10 passou. Não se contabiliza execução bloqueada como aprovação.

## QA nativo isolado

Perfil `.tmp/phase09a-qa-20261019`, cópia fechada do QA da Fase 08: cinco publicações, sete mídias/vínculos e três planejamentos ativos. Banco operacional não foi restaurado ou modificado. Dados privados, QA, backups, resultados brutos e builds ficam ignorados pelo Git.

Em janela Tauri com área 1366×768, configurações/versão e diálogo de destino conferidos. Backup nativo concluído em 20/10: `GMAD-Backup-20261020-115109-661102f5-05ba-47fd-b28e-1f8cae8e6a66`, cinco publicações, sete mídias, esquema 6; manifesto SHA-256 `fd74d421e54c5a50f2de55bb61637d1c671eb366927a458582c4aa2b6d89ec65`. Conferência independente dos oito arquivos por bytes e SHA-256 aprovada. Inspeção nativa mostrou o resumo e manteve restauração desabilitada até checkbox. Confirmação de substituição no QA solicitada antes da ação.

Restauração nativa explicitamente autorizada e concluída no mesmo perfil QA. Recuperação verificada `GMAD-Backup-20261020-123125-dd11610d-9056-4850-a49a-22914f6afdb8`, sob diretório irmão recovery; diretório anterior before preservado, ambos identificados na interface. Tentativas anteriores falharam sem ativar staging e mantiveram o banco atual. Reabertura do Tauri e comparação independente com baseline confirmaram as cinco linhas editoriais integralmente iguais, sete MediaAsset, sete PublicationMedia e sete hashes físicos iguais, `integrity_check=ok` e zero violações FK. Três planos ativos e um cancelado preservados. Quatro preparações ficaram outdated; uma declaração requires_review e outra superseded; preferência de raiz NULL. Dashboard refletiu os estados restaurados.

Release QA gerado com o mesmo código e configuração temporária de produto/identificador separados (`br.com.gmad.socialplanner.qa09a`). Perfil novo em AppData recebeu somente banco fechado e sete mídias do QA, sem sobrescrita. Executável aberto pelo caminho, sem terminal, com frontend embarcado `tauri.localhost`, IPC/SQLite e cinco publicações. Biblioteca, prévias e detalhes com imagem/legenda/origem conferidos; fechamento normal e reabertura mantiveram as contagens. Layout em 1366×768 e janela maximizada no monitor 1920×1080 conferidos. O build operacional final mantém a identidade original; não foi aberto para evitar tocar o perfil operacional. Configuração QA e dados não entram no Git.

Instalação não executada: o usuário respondeu **Não instalar agora**. Isso também impede alegar homologação empírica de atalhos, update ou desinstalação. O script NSIS foi inspecionado estaticamente; validação release acima é execução portátil, não instalação.

## Distribuição e portão de aceite

Versão candidata `1.0.0-rc.1`, NSIS x64 por usuário, identificador operacional `br.com.gmad.socialplanner` preservado. Bundle não incorpora Rust/Node/Codex como requisito de execução. WebView2 Evergreen continua necessário. Configuração impede downgrade e script NSIS gerado mantém dados em atualização; na desinstalação, exclusão de dados é opção separada inicialmente desmarcada, ignorada em update. A candidata não tem certificado de assinatura; SmartScreen/políticas corporativas podem exigir avaliação humana, sem contorno automático.

Ainda não homologados: Windows limpo sem ferramentas de desenvolvimento, reinício real do Windows, instalação/atualização/desinstalação nativas e agendamento real na Meta. Somente este último pode ser realizado com conteúdo/conta/data previamente autorizados, seguindo os nove passos do [guia Meta](GUIA_AGENDAMENTO_META.md). Nenhum upload, publicação, confirmação fictícia ou alteração remota foi realizado na Fase 09A.

Backup/restauração, regressão, release QA e documentação atendem o escopo técnico executável nesta sessão. Artefato/hash em [candidata Windows](RELEASE_CANDIDATE_V1.md). Encerramento autorizado: commit `chore: prepare GMAD Social Planner v1 release candidate` e push normal para origin/main, após auditoria dos arquivos; consulte `git log -1` e o relatório final para o identificador efetivo. Não há tag ou release publicado. A Fase 09 permanece com pendências de homologação e não recebe aceite integral de distribuição/versão estável. A 09A entrega uma candidata para essa homologação; 09B não foi iniciada.
