# Fase 04.5 — Homologação real do Trello

Estado mais recente — 04.5C: **concluída em 16/10/2026; Fase 04 homologada no Windows para o escopo implementado.** Importação autorizada de dois vídeos e quatro imagens, prévias, reprodução, sincronização repetida, melhoria de legenda e persistência após reabertura aprovadas. Dados anteriores preservados. A Fase 05 não foi iniciada. A seção 04.5B abaixo preserva o registro histórico.

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

## Pendências após a 04.5B (histórico)

Vídeo e múltiplas mídias em cartões reais dependem de confirmação antes de importar cartões adicionais. Marcadores Markdown como `**` continuam exibidos literalmente; a legenda original é preservada e a apresentação será tratada futuramente, fora desta correção. A Fase 05 não foi iniciada.

## Fase 04.5C — homologação ampliada concluída

Solicitada em 16/10/2026. Base Git limpa no commit `47d9851`. A consulta autenticada somente leitura confirmou 5 listas e 82 cartões. O diagnóstico opt-in `inspect_authorized_media_candidates` lê metadados pelo transporte de produção, não baixa anexos nem grava o banco operacional e não registra URLs assinadas. A inspeção passou (1 teste remoto). Snapshots anteriores e o snapshot de segurança desta etapa confirmaram as duas publicações e a mídia existentes integralmente preservadas.

### Candidatos apresentados para confirmação

| Caso | Cartão / ID remoto | Lista | Anexos suportados / tamanho informado pela API | Comportamento esperado |
| --- | --- | --- | --- | --- |
| Vídeo isolado | `23` / `6abccbdc77fe6c6b13ece0ac` / `HeKKJ6t0` | Sem 4 | `corte_gmad.mp4`, 46.422.938 bytes | Uma publicação com vídeo; conferir streaming, integridade e reprodução WebView2 |
| Múltiplas imagens | `2` / `6abcc683705f9e026bc1619e` / `Sh6EV1dH` | Sem 1 | PNG + 2 JPEG, 2.655.715 bytes no total | Uma publicação com três imagens na ordem do Trello; cartão distinto do “2” recuperado na 04.5B |
| Misto | `18` / `6abccd446e9f6ce120c3750c` / `WiHAk2RG` | Sem 3 | PNG + MP4, 4.638.807 bytes no total | Duas mídias com aviso de ajuste; nenhuma presunção de validade Instagram |

IDs dos anexos na ordem retornada/sanitizada:

- Vídeo: `6abccbdc77fe6c6b13ece0bb`.
- Imagens: `6abcf4e2456d0fe1454615be`, `6abcf4f716bb9989f94216af`, `6abcf4f73147ff3809cf6d80`.
- Misto: `6abccd446e9f6ce120c37521`, `6abccd446e9f6ce120c37523`.

O usuário autorizou explicitamente os três cartões e a sincronização repetida. A seleção padrão foi zerada; cada candidato foi conferido pela lista, anexos e posteriormente pelo vínculo remoto/publicação. Nenhum outro cartão foi importado.

### Resultados reais no Tauri Windows

Às 16:26, a operação concluiu 3/3 cartões, **3 novas publicações, 6 anexos baixados, 0 falhas de cartão e 0 falhas de anexo**. Todos passaram pelo downloader de produção, que exige HTTP 200 após aplicar a política restrita de redirecionamentos da 04.5B, preservando TLS. Nenhum domínio adicional foi liberado. O streaming existente de 64 KiB foi utilizado; tamanho final, validação de conteúdo/estrutura e SHA-256 foram conferidos no Rust e novamente no disco. Não houve conversão automática.

| Cartão | ID da publicação criada | Composição | Resultado nativo |
| --- | --- | --- | --- |
| `HeKKJ6t0` | `ac0d61fc-8a47-4708-a450-8672e0f71025` | 1 MP4; Reels | Vídeo renderizado e reproduzido; contador avançou até 0:13/0:58 |
| `Sh6EV1dH` | `fdcc229f-e318-44ec-9e1e-a02b3812223a` | 3 imagens; Carrossel | PNG e os dois JPEG exibidos individualmente, na ordem 0/1/2 |
| `WiHAk2RG` | `cc55e5b8-94cf-4b7e-b621-832811b2d01d` | PNG + MP4; Carrossel com ajuste | Capa exibida, segundo vídeo reproduzido até 0:32/1:26; aviso de ajuste preservado |

“Carrossel” é intenção editorial local. A mistura PNG/MP4 exige ajuste e esta homologação não valida requisitos da API Instagram. As duas reproduções reais funcionaram neste WebView2; não se identificou incompatibilidade de codec nos arquivos selecionados. Não se afirma suporte a todos os codecs MP4 nem certificação de áudio, que foi silenciado durante a inspeção.

Arquivos efetivamente baixados (53.717.460 bytes no total):

| Arquivo | MIME | Bytes | Dimensões / duração |
| --- | --- | ---: | --- |
| `corte_gmad.mp4` | video/mp4 | 46.422.938 | 1080 × 1920; 58,585 s |
| `Arauco_cozinhas_padrões.png` | image/png | 1.685.886 | 1122 × 1402 |
| `04-Cozinha-Pimenta-Rosa-matt-e-dueto-Cravo-e-Ginger-scaled.jpg` | image/jpeg | 499.402 | 2560 × 1920 |
| `01-Cozinha-Anis-Ginger-e-Cravo-imagem-01-scaled.jpg` | image/jpeg | 470.427 | 2560 × 1920 |
| `perrengue.png` | image/png | 1.142.104 | 1080 × 1920 |
| `snapins-ai_3430967810237650689.mp4` | video/mp4 | 3.496.703 | 360 × 640; 86,842 s |

Hashes dos vídeos: `eebe7736e0c629eb7daa7f11e12f7fe752b1c26f57a332a3ad28d108a6d4498e` (isolado) e `c3b910bc2168f17c10faf06cebc6b7e1496433f8ab8eaee1cec055ade3d022e3` (misto). Todos os sete arquivos locais, incluindo a imagem preexistente, tiveram tamanho e SHA-256 comparados aos MediaAssets. Total: 5 publicações, 7 MediaAssets, 7 vínculos e 7 arquivos físicos; nenhum arquivo redundante.

Às 16:30, a segunda operação selecionou somente esses três cartões, identificados pelos UUIDs criados: **3/3 processados, 0 novas publicações, 0 downloads, 0 origens alteradas, 0 falhas**. As tabelas `publications`, `media_assets` e `publication_media` ficaram integralmente idênticas ao snapshot posterior à primeira importação. Legendas, estados, versões e ordem das mídias preservados. As duas publicações anteriores e seus vínculos/asset permaneceram integralmente idênticos ao snapshot inicial. Novas publicações em `pending_review`, versão 1, com legendas iguais às descrições originais.

SQLite: `quick_check=ok`; nenhuma violação em `foreign_key_check`. Snapshots e script de conferência em `.tmp`, fora do Git; nenhuma gravação SQL corretiva foi feita. O aplicativo foi fechado normalmente para conferir esses resultados após reabertura.

### Apresentação da legenda

Implementada apresentação limitada de `**negrito**` e escapes comuns (`\\`, `\*`, `\#`, `\_`) nos detalhes de publicações Trello e na origem. React produz somente nós de texto e `strong`; HTML, scripts e links Markdown permanecem texto inerte. Sem `dangerouslySetInnerHTML`, dependência nova ou editor Markdown. Marcadores incompletos continuam visíveis. A classe existente conserva quebras de linha e emojis.

Nenhum texto persistido é reescrito. O editor continua recebendo a string integral. **Copiar legenda** copia a legenda editorial armazenada; **Copiar legenda original**, na origem Trello, copia a descrição original integral, incluindo marcadores, escapes, hashtags e quebras. Markdown completo fica fora desta etapa.

### Validações desta etapa

- `npm.cmd run check`: aprovado; typecheck, lint, **47 testes frontend**, **4 testes SQL**, build frontend.
- `cargo test --manifest-path src-tauri/Cargo.toml`: **55 aprovados**, 0 falhas, 4 opt-in ignorados na suíte padrão.
- `cargo check`, `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`: aprovados.
- Inspeção remota opt-in: **1 aprovada**, somente metadados. Total: **106 testes locais + 1 inspeção remota = 107 aprovações**. As importações e reproduções reais acima são verificações de aceitação adicionais, não testes com fixtures nem parte dessa contagem.
- Novas regressões: negrito/escapes, Unicode/quebras, HTML e links maliciosos inertes, delimitadores incompletos, vários trechos e cópia integral das duas versões sem gravação editorial.
- Build desktop: aprovado (`npm.cmd run build:desktop`, release sem bundle). A primeira tentativa encontrou o executável em uso; após o usuário disponibilizar a sessão, o GMAD foi fechado normalmente e o build repetido passou. O novo executável foi aberto no contexto operacional e a melhoria de legenda foi conferida visualmente: negrito, quebras e hashtags corretos, sem reescrever o banco.

### Persistência e aceite final

O usuário reabriu o executável release no contexto operacional. O novo processo iniciou às 16:33:45 (hora de São Paulo), após o fechamento normal do processo anterior. Dashboard e biblioteca confirmaram cinco publicações pendentes de revisão. As prévias das imagens e as contagens 1/2/3 das novas publicações foram preservadas; o vídeo isolado abriu novamente com quadro decodificado e duração 0:58. O texto formatado permaneceu correto.

A cópia somente leitura do SQLite aberto pelo novo processo foi verificada novamente contra os snapshots inicial e posterior à importação: IDs, legendas, estados, versões, vínculos e ordem idênticos; metadados e anexos preexistentes preservados. Sete arquivos físicos, sete MediaAssets e cinco publicações; tamanhos e SHA-256 dos sete arquivos conferidos novamente, sem duplicações. `quick_check=ok`, sem violações de chave estrangeira. **Persistência aprovada e aceite da Fase 04 concluído para o escopo implementado.**

Limites remanescentes: a composição mista exige decisão editorial antes de uso; os dois codecs selecionados foram reproduzidos, sem afirmar compatibilidade universal de MP4; áudio não foi certificado. Markdown completo/editor rico permanece fora do escopo. A publicação preexistente “1” permaneceu com zero mídias, conforme seu estado inicial, sem tentativa de recuperação fora da seleção autorizada. A diferença de contexto ao abrir pela automação continua documentada; a homologação utilizou o contexto operacional correto. Nenhum desses limites representa falha dos downloads, vínculos ou persistência homologados.

Não houve alterações no Trello remoto ou início da Fase 05.

Versionamento: `1ecc3b7 — test: finalize Trello media import validation` registra código, regressões e verificações anteriores à reabertura. O encerramento documental após a conferência final está em commit complementar (consultar histórico Git). Somente código, regressões e documentação são incluídos. Sem push ou tag; dados e snapshots permanecem fora do Git.
