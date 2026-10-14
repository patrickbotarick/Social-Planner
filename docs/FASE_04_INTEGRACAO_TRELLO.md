# Fase 04 — integração Trello

Registro de implementação e validação em 14/10/2026. Base: commit 08a1700 (Fase 03), árvore limpa no início. Escopo exclusivamente da Fase 04.

**Estado: implementação concluída e validada com testes offline e desktop local; aceite da integração com serviço/quadro reais pendente de configuração e autorização do operador.** Não houve autenticação real nem leitura/importação de cartões reais. A Fase 05 não foi iniciada.

## Entrega

- Configurações → Trello: conectar, testar, renovar e desconectar. Chave/token do fluxo oficial pessoal 1/authorize, scope=read, expiração solicitada de 30 dias. Autorização feita pelo usuário no navegador; segredo armazenado pelo Rust no Windows Credential Manager. Sem senha Trello, localStorage ou token no SQLite.
- Importações: URL oficial validada, prévia sem gravação, pesquisa/listas/seleção, confirmação, limites de download, progresso, cancelamento, histórico e consulta para sincronizar.
- Backend Rust: HTTP GET restrito à API oficial, timeout, paginação, tratamento de autenticação/acesso, retry limitado e cancelamento. Referências externas não são baixadas.
- Textos/snapshots completos separados da publicação editorial. Importações geram pending_review, sem aprovação, calendário ou publicação externa. Cadastro manual preservado.
- JPEG/PNG/WebP/MP4 reutilizam MediaStore, streaming, inspeção de conteúdo, SHA-256 e armazenamento local. Anexos incompatíveis/externos/falhos permanecem registrados como referências.
- Sincronização por ID remoto, sem duplicar publicação inicial nem sobrescrever título, legenda, formato, estado, versão ou composição local. Origem alterada/removida e anexos removidos ficam sinalizados. Arquivos e publicações são preservados.
- Migração aditiva 003: trello_boards, imported_cards, trello_attachments, import_jobs. Migrações 001/002 intactas, verificadas pelo Git e testes de atualização.

Arquitetura, APIs, autenticação, configuração manual e fontes oficiais estão detalhadas em [INTEGRACAO_TRELLO.md](INTEGRACAO_TRELLO.md), [ARQUITETURA.md](ARQUITETURA.md) e [MODELO_DADOS.md](MODELO_DADOS.md).

## Validações automatizadas

| Comando | Resultado final |
| --- | --- |
| npm.cmd run typecheck (via check) | Aprovado, TypeScript estrito |
| npm.cmd run lint (via check) | Aprovado, zero avisos ESLint |
| npm.cmd test (via check) | 41 testes aprovados, cinco arquivos |
| npm.cmd run test:sql (via check) | 4 testes aprovados |
| npm.cmd run build (via check e build desktop) | Aprovado, 61 módulos; JS 301,50 kB / gzip 90,61 kB |
| cargo fmt --manifest-path src-tauri/Cargo.toml --check | Aprovado |
| cargo check --manifest-path src-tauri/Cargo.toml | Aprovado |
| cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings | Aprovado |
| cargo test --manifest-path src-tauri/Cargo.toml | 47 aprovados, zero falhas; dois testes opt-in ignorados por padrão |
| cargo test --manifest-path src-tauri/Cargo.toml windows_vault_roundtrip_isolated -- --ignored | 1 aprovado, cofre Windows real em entrada exclusiva de QA |
| cargo test --manifest-path src-tauri/Cargo.toml prepare_native_qa_fixture -- --ignored | 1 aprovado, GMAD_TRELLO_QA_ROOT explícito em .tmp/phase04-native-20261014 |

**92 testes na suíte padrão + dois opt-in = 94 testes distintos aprovados.** Os testes de API usam respostas simuladas; isso não comprova acesso ao Trello real. O teste de cofre usa uma string de teste, sem credencial Trello, e remove somente sua própria entrada UUID. A preparação nativa produz três registros no perfil isolado, nunca no operacional.

Cobertura inclui URL/SSRF, acesso/autenticação, listas/cartões/paginação, cursor repetido, 429/5xx/retry, download autenticado com cabeçalho sensível, redirecionamento/tamanho, streaming interrompido e inválido, imagem/vídeo/carrossel, texto sem mídia, referências externas, falhas parciais/recuperação, cancelamento, idempotência, alteração/remoção na origem, preservação editorial e de mídia manual, exclusão lógica prévia, reinício/SQLite, migração 002→003 e ausência de token/URL autenticada nos dados persistidos. Testes adicionais garantem rollback quando o arquivo muda entre promoção e commit e impedem duplicação de uma publicação cuja mídia foi removida do disco. A duplicação preserva essa proteção da Fase 03, inclusive para conteúdos Trello.

Ocorrências de ambiente: os primeiros testes Rust sob sandbox tiveram erros MEDIA_IO inclusive em casos anteriores da Fase 03; repetição no ambiente Windows autorizado passou. Vite também precisou execução fora da restrição de loopback. Cargo informou fallback de hardlink para cópia no cache incremental; não foi erro de código nem reprovação do Clippy. Node informou aviso de API experimental node:sqlite. Nenhuma falha de teste permanece na execução final.

## Execução nativa isolada

Executado npm.cmd run dev -- --window-size 1366x768 com GMAD_QA_DATA_DIR apontando para .tmp/phase04-native-20261014. Janela Tauri real, não navegador. Fixtures são geradas por teste Rust com transporte simulado e banco/arquivos reais. Não há mocks no fluxo operacional da aplicação.

| Verificação | Evidência/resultado |
| --- | --- |
| Navegação e IPC local | Dashboard, Importações, Configurações, Biblioteca e detalhes operacionais |
| Configurações Trello | Cofre consultado; estado Desconectado, sem credenciais inseridas |
| Consulta sem autorização | URL https://trello.com/b/R23cR81w/outubro-2026-matriz-grupo-gmad reconhecida; mensagem orienta conectar/renovar. Bloqueio local antes da API, sem alegar nome/contagem desse quadro |
| Biblioteca e Dashboard | Três publicações de QA originadas pelo importador, todas pendentes; duas mídias físicas; nenhuma aprovação |
| Imagem | PNG gerado em teste exibido pelo protocolo asset local |
| Edição | Legenda alterada na janela para “Legenda local preservada 😊 / #GMAD #RioPreto”; versão incrementada para 2, estado pending_review mantido |
| Snapshot | Descrição original preservada separadamente, confirmada em consulta SQLite somente leitura |
| Vídeo | Amostra MP4 licenciada existente, 320×240 / cerca de 0,1 s, abriu no player nativo; ação reproduzir mudou para pausar e retornou ao fim, sem erro |
| Reinício | Fechamento normal e reabertura com o mesmo perfil; contagens, imagem e legenda editada preservadas |
| Integridade | PRAGMA quick_check=ok, foreign_key_check sem violações, SHA-256 dos dois arquivos confere com o catálogo |

A duração muito curta da amostra aparece como 0:00 no player. Não foi validada reprodução de todos os codecs, vídeos grandes ou áudio. Carrossel e importações remotas são cobertos por testes automatizados; a seleção/download reais na janela dependem da conta autorizada.

Na primeira saída debug houve aviso de fallback IPC custom protocol → postMessage durante Window.destroy; processo encerrou com código 0, sem perda no reinício. Na reabertura/saída posterior não houve falha funcional observada.

Evidências do perfil isolado:

- [Configuração desconectada](evidencias/fase04/trello-desconectado.jpg)
- [Consulta sem autorização](evidencias/fase04/consulta-sem-autorizacao.jpg)
- [Biblioteca QA](evidencias/fase04/biblioteca-qa.jpg)
- [Legenda local editada](evidencias/fase04/imagem-legenda-local.jpg)
- [Vídeo local](evidencias/fase04/video-local-qa.jpg)
- [Dashboard após reabrir](evidencias/fase04/dashboard-apos-reabrir.jpg)
- [Imagem e legenda após reabrir](evidencias/fase04/imagem-apos-reabrir.jpg)

## Build final e banco operacional

npm.cmd run build:desktop (tauri build --no-bundle) terminou com código 0 e gerou src-tauri/target/release/gmad-social-planner.exe. O executável final abriu nativamente pelo protocolo tauri.localhost, sem Vite, com navegação e contagens reais.

Antes da abertura, o banco operacional em %APPDATA%/br.com.gmad.socialplanner/planner.sqlite3 estava na versão 2, com zero publicações e zero assets. A abertura aplicou somente a migração 003. Comparação somente leitura antes/depois confirmou preferências e registros das migrações 001/002 idênticos, mesmas contagens, zero cartões importados, quick_check=ok e foreign_key_check sem violações. Nenhum dado de QA entrou no operacional. A migração com publicações existentes também foi coberta por teste Rust dedicado, preservando texto, estado e versão.

O executável é build local sem instalador; distribuição/assinatura pertencem à Fase 09. Não foram versionados builds ou bancos.

## Limitações e aceite externo

1. Configuração manual: criar/usar chave no portal oficial Trello Apps Admin, autorizar somente leitura no navegador, colar token em Configurações → Trello e testar. Nunca enviar o token pelo chat. Seguir o [guia operacional](INTEGRACAO_TRELLO.md#configurar-uma-conta).
2. Após autorização, consultar o quadro real, selecionar poucos cartões representativos (imagem/vídeo/múltiplas mídias/sem mídia), importar e conferir. Repetir, editar uma legenda e sincronizar novamente para completar o aceite externo. Até isso ocorrer, conexão/consulta/download reais permanecem não validados.
3. Redirecionamentos HTTP são rejeitados. Se o serviço exigir redirecionamento em anexo real, haverá falha individual; investigar destinos oficiais antes de ampliar essa política. Links externos ficam apenas como referências e têm query/fragmento removidos por proteção de credenciais.
4. Consulta limitada a dez páginas e respostas JSON de 16 MiB; dez páginas cheias são recusadas por não provar completude. A API não oferece snapshot transacional entre páginas.
5. Cancelamento em leitura de rede já bloqueada pode aguardar timeout. Verificação final de checksum segura a transação e pode atrasar consultas locais em arquivos grandes. Não houve teste de carga/disco fisicamente cheio; erros de escrita e limites foram exercitados em testes controlados.
6. Arquivos completos sem vínculo podem permanecer após falha/cancelamento; não existe coletor destrutivo automático. Falhas de cartão têm contagens/mensagem final, sem log detalhado por tentativa. O indicador de origem alterada não é aceite editorial; resolução de conflitos/revisão pertence à fase seguinte.

## Critérios de aceite

Configuração, URL, seleção, importador, persistência, biblioteca, idempotência, preservação local, falhas parciais e testes estão implementados/verificados no escopo descrito. Consulta de quadro autorizado, respostas do serviço e downloads reais ainda exigem execução externa autorizada. Portanto, **não declarar aceite completo da Fase 04**. A base técnica para a Fase 05 está preparada; a fase seguinte não foi iniciada e depende de solicitação do usuário.

## Arquivos e Git

Principais alterações: módulos Rust trello_*, migração 003, manifesto/capability IPC, tipos/serviço Trello, Imports, TrelloSettings/TrelloSource e integração mínima com editor/biblioteca. Migrações antigas e dependências npm preservadas. Documentação atualizada: README, PROJETO_MESTRE, ARQUITETURA, MODELO_DADOS, ROADMAP e os dois documentos da Fase 04. Evidências visuais de QA em docs/evidencias/fase04; bancos, mídias de QA, tokens, .tmp e builds ficam fora do Git.

Commit desta entrega: feat: integrate Trello board importer (hash informado no relatório final e consultável no Git). Sem push, tag ou início da Fase 05.
