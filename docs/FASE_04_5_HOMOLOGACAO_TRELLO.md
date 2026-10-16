# Fase 04.5 — Homologação real do Trello

Atualização: 16/10/2026, correção 04.5B. **Concluída: publicação operacional recuperada, imagem exibida, segunda sincronização sem duplicação e persistência após reabertura confirmada.** A importação real de imagem e a recuperação deste cartão estão homologadas. Esta etapa não inicia a Fase 05; vídeo e múltiplas mídias reais permanecem pendências da homologação ampliada da Fase 04.

## Diagnóstico confirmado

A conexão real e a consulta do quadro de outubro funcionaram (5 listas, 82 cartões). A importação criou a publicação, preservando a descrição, mas o anexo falhou: o endpoint autenticado respondeu HTTP 302 para o host exato `files.trello.com`, bloqueado pela política anterior que recusava todos os redirecionamentos. O destino respondeu HTTP 200 com PNG válido. Não foi necessário ampliar permissões da conta nem encaminhar a autenticação ao destino.

Identidade do alvo autorizado, independente do título não exclusivo “2”:

| Entidade | Identificador |
| --- | --- |
| Quadro (shortLink) | `R23cR81w` |
| Cartão remoto | `6abd09b407e614a3bbf10559` (`ScnVGpen`) |
| Anexo remoto | `6abd09b407e614a3bbf1057e` |
| Publicação existente | `fafc431b-1857-4c3f-819f-7a1baaaf940d` |

O estado observado antes da recuperação era `trello/pending_review`, versão 1, sem mídia associada. A outra publicação existente, “1”, está fora do escopo. Nenhuma publicação foi excluída ou recriada nesta correção.

## Correção e segurança

- `trello_redirect.rs` aplica até três saltos 301/302/303/307/308 exclusivamente para `https://files.trello.com`, porta 443. Cada destino é revalidado, inclusive saltos posteriores e referências relativas.
- Apenas a rota tipada de download de anexo pode seguir redirecionamentos. As demais consultas continuam sem redirecionamento automático.
- Outros hosts/subdomínios, esquemas inseguros, portas alternativas, credenciais embutidas, fragmentos e parâmetros explícitos de credenciais são recusados.
- Cliente separado, sem Authorization, Cookie, Referer ou proxy herdado. O token permanece na requisição inicial a `api.trello.com`. TLS e certificados continuam verificados.
- DNS com espera limitada; todos os IPs precisam ser públicos, inclusive em respostas mistas. Os endereços validados são fixados na conexão. Localhost, privados, loopback, link-local e reservados são bloqueados.
- Cada salto desconta o tempo já consumido. Streaming conserva timeout, limites de bytes, conferência de Content-Length, validação efetiva pelo MediaStore e SHA-256. Content-Type incompatível é recusado, sem confiar somente no MIME.
- URLs assinadas permanecem apenas em memória. Erros são redigidos sem URL completa, token ou credencial.

Sem mudanças de dependências, migrações, permissões IPC ou arquitetura. A sincronização continua preservando a composição editorial existente: um anexo recuperado fica disponível na origem e é adicionado explicitamente pelo editor à mesma publicação.

## Download real executado

O teste opt-in `verify_authorized_attachment_download` usou a conexão existente no Windows Credential Manager e os IDs autorizados. Executou o downloader de produção e o MediaStore em diretório temporário, sem gravar no banco operacional ou importar outro cartão.

Resultado: **1 PNG válido, 1.347.966 bytes, 1086 × 1448 pixels**. SHA-256: `3db40be7b40602d5b975d93b704148782b2ca6741c810e342d891f73cbf3f465`.

Depois dessa prova isolada, o mesmo anexo foi recuperado pelo fluxo nativo no banco operacional, conforme evidências abaixo.

## Validações executadas em 16/10/2026

| Comando/verificação | Resultado |
| --- | --- |
| `npm.cmd run check` | Typecheck, lint, 41 testes frontend, 4 testes SQL e build frontend aprovados |
| `cargo test --manifest-path src-tauri/Cargo.toml` | 55 aprovados, 0 falhas, 3 testes opt-in ignorados na execução padrão |
| `cargo check --manifest-path src-tauri/Cargo.toml` | Aprovado |
| `cargo fmt --manifest-path src-tauri/Cargo.toml --check` | Aprovado |
| `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings` | Aprovado |
| `cargo test --manifest-path src-tauri/Cargo.toml verify_authorized_attachment_download -- --ignored --nocapture` | 1 aprovado contra o anexo real, habilitado explicitamente com IDs e flag de diagnóstico, sem segredos em argumentos |
| `npm.cmd run build:desktop` | Aprovado, executável release gerado; comando existente usa `--no-bundle` |
| Janela Tauri release no contexto operacional | Recuperação, composição, imagem, segunda sincronização e persistência após reabertura aprovadas |

Total: **100 testes automatizados locais + 1 teste real = 101 aprovações**, sem falhas finais. O teste real está entre os três ignorados da suíte padrão e foi executado separadamente; os outros dois opt-in não foram repetidos nesta etapa.

Regressões novas: 302 legítimo; rejeição por host/esquema/credenciais/porta em cada salto; limite de saltos e Location ausente; ausência de headers sensíveis; DNS privado/reservado/misto; HTML e prazo esgotado; restrição à rota de download; recuperação idempotente com inclusão explícita na composição e preservação dos campos editoriais. Testes existentes de conteúdo inválido, leitura incompleta, tamanho, cancelamento e preservação de mídias continuam aprovados.

## Recuperação operacional e segunda sincronização

A abertura pela automação mostrou inicialmente um banco vazio, diferente do contexto operacional. O usuário abriu o mesmo executável release pelo contexto habitual e a janela exibiu as duas publicações originais. A causa da diferença de contexto permanece não confirmada; nenhum banco foi substituído, restaurado ou limpo. A identidade foi conferida pela rota da publicação, pelo link remoto `ScnVGpen` e por cópia somente leitura do SQLite efetivamente aberto pelo processo. Snapshots de diagnóstico permaneceram em `.tmp`, fora do Git.

Na janela nativa, a consulta retornou 5 listas e 82 cartões. Todos foram desmarcados e somente o cartão vinculado ao UUID original foi selecionado. As duas operações processaram exatamente um cartão:

| Operação (hora local) | Novas publicações | Anexos baixados | Falhas de cartão/anexo | Resultado |
| --- | --- | --- | --- | --- |
| Recuperação, 09:24 | 0 | 1 | 0 / 0 | Concluída |
| Segunda sincronização, 09:26 | 0 | 0 | 0 / 0 | Concluída |

O PNG recuperado ficou disponível na origem Trello. Foi adicionado explicitamente à composição e salvo na publicação existente. A janela de detalhes confirmou **1 arquivo**, imagem renderizada, origem Trello e estado pendente de revisão. Não se alterou o texto original.

Verificações no SQLite e no disco:

- Publicação original `fafc431b-1857-4c3f-819f-7a1baaaf940d` preservada. Título, legenda (incluindo quebras e Markdown), formato, origem, estado e criação idênticos ao snapshot anterior.
- Somente `version` (1 → 2) e `updated_at` mudaram na linha editorial ao salvar a associação; a segunda sincronização não os alterou.
- Um MediaAsset: `99ac7bc9-1206-426d-b0af-dddf33f77f76`, PNG de 1.347.966 bytes, 1086 × 1448. Um vínculo PublicationMedia, ordem 0, com a publicação original.
- Um único arquivo em `%APPDATA%\br.com.gmad.socialplanner\media\`, nome `<SHA-256>.png`, com o checksum acima confirmado por leitura do arquivo físico.
- `publications`, `media_assets` e `publication_media` integralmente idênticas antes/depois da segunda sincronização. A outra publicação permaneceu integralmente idêntica à cópia anterior à recuperação. Total da biblioteca: duas publicações.
- `PRAGMA quick_check`: `ok`; `PRAGMA foreign_key_check`: nenhum erro.
- Nenhum cartão adicional importado e nenhuma escrita no Trello remoto.

O aplicativo foi fechado normalmente e reaberto pelo usuário no mesmo contexto operacional. O novo processo do executável release iniciou às 14:04 (hora local). Dashboard e biblioteca exibiram duas publicações; os detalhes do UUID original exibiram a imagem renderizada, 1 arquivo, legenda original, origem Trello e estado pendente de revisão. Uma nova cópia somente leitura do SQLite aberto por esse processo foi comparada ao snapshot posterior à segunda sincronização: `publications`, `media_assets`, `publication_media`, `imported_cards` e `trello_attachments` integralmente idênticas. Integridade `quick_check=ok` e nenhuma violação de chave estrangeira. **Persistência aprovada.**

Critérios da Fase 04.5B atendidos. Commit desta entrega: `fix: support secure Trello attachment redirects` (consultar histórico Git). Somente código, testes e documentação são versionados; snapshots, dados, imagem e builds permanecem fora do Git. Sem push ou tag.

## Pendências gerais da Fase 04.5

Vídeo e múltiplas mídias em cartões reais dependem de confirmação antes de importar cartões adicionais. Marcadores Markdown como `**` continuam exibidos literalmente; a legenda original é preservada e a apresentação será tratada futuramente, fora desta correção. A Fase 05 não foi iniciada.
