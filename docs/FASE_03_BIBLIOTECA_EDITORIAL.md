# Fase 03 — Biblioteca editorial local

Implementação: 13/10/2026. Validação final: 14/10/2026. Base: `ec54499`. Estado: Fase 03 concluída tecnicamente, com testes automatizados, fluxos nativos e persistência aprovados neste Windows. Fase 04 pronta para nova solicitação, ainda não iniciada.

## Funcionalidades e fluxo

Nova publicação → escolher formato → selecionar mídias pelo seletor Windows → escrever título/legenda → ordenar mídias → salvar → abrir detalhes na biblioteca. A edição reabre a versão persistida; duplicação cria novo UUID e título com “(cópia)”, mantendo legenda e arquivos compartilhados. A exclusão exige confirmação e oculta o registro, preservando os arquivos. A legenda pode ser copiada nos detalhes.

Biblioteca: busca em título/legenda, filtros por formato/origem/estado, ordenação por criação crescente/decrescente, título ou última edição e páginas de 24 registros. Busca ignora maiúsculas e acentos pela normalização Unicode no Rust. A consulta retorna apenas a mídia principal de cada publicação; nenhuma mídia completa atravessa o IPC. Imagens são carregadas de forma lazy nos cards, vídeos têm identificação no card e player local nos detalhes/formulário.

Dashboard consulta total, rascunhos, pendentes e aprovadas no SQLite real. Registros excluídos ficam fora das consultas e contagens. Não há métricas de agendamento/publicação inventadas. Na prévia web, salvamento e seleção ficam indisponíveis, sem mocks operacionais.

O formulário mostra alterações não salvas. Navegação por links, cancelamento, mudança de hash/histórico e fechamento nativo são protegidos; operações em andamento impedem saída até terminar. Confirmar descarte limpa somente as mídias temporárias da sessão. Um erro de gravação mantém o formulário para correção ou nova tentativa.

## Modelo e migração

`002_editorial_library.sql` acrescenta três tabelas, sem alterar `001_foundation.sql`, preferências ou identificador do aplicativo:

| Tabela | Dados e regras |
| --- | --- |
| publications | UUID Rust, título obrigatório até 200 caracteres, legenda até 20.000 caracteres, texto normalizado de busca, formato, origem, estado editorial, versão, datas UTC e deleted_at |
| media_assets | UUID, nome original, MIME/ extensão validados, bytes, arquivo interno relativo, dimensões/duração quando disponíveis, SHA-256 único e criação UTC |
| publication_media | FKs publicação/mídia, posição não negativa; par e posição únicos por publicação; índice de referências por asset |

Índices parciais para publicações ativas por criação, atualização e filtros. Listagem usa LIMIT/OFFSET com máximo de 100 por comando, limite de offset e ordem determinística por ID. A busca parcial de texto usa `instr` sobre coluna normalizada; não existe FTS nesta fase. Ordenação por título usa SQLite NOCASE, sem collation linguística portuguesa completa.

Save cria/edita publicação, registra assets e ordena vínculos numa transação IMMEDIATE. A versão esperada protege contra edição/exclusão com dados obsoletos. Edição permitida nesta fase somente para publicação manual em rascunho. O estado inicial oficial é `draft`; editar mantém rascunho e incrementa versão. Pendentes, aprovadas, ignoradas e prontidão são valores preparados no mesmo eixo editorial; nenhuma transição de revisão foi implementada.

## Arquivos e segurança

Raiz resolvida pelo Tauri `app_data_dir()`, sob o identificador estável `br.com.gmad.socialplanner`. No Windows:

```text
%APPDATA%/br.com.gmad.socialplanner/
  planner.sqlite3
  media/<sha256>.<extensão>
  temp/imports/<uuid-da-sessão>/<uuid-da-mídia>.<extensão>
```

O banco permanece na localização existente, sem mover dados da fundação. Nenhum caminho do desenvolvedor é usado em produção. A variável opt-in `GMAD_QA_DATA_DIR` funciona exclusivamente em builds debug para testes isolados; o release sempre usa o perfil real do Tauri.

O seletor é chamado pelo Rust com o plugin oficial dialog, ligado à janela principal. Não há pacote JS de dialog/fs, permissão genérica de leitura/escrita de arquivos ou comando que aceite caminho de origem pelo IPC. O plugin traz fs como dependência transitiva, sem permissões frontend concedidas.

Selecionar copia em blocos de 64 KiB para arquivo temporário exclusivo, calcula SHA-256 e valida o conteúdo copiado. Extensão deve concordar com o MIME identificado. O original fica intacto; salvar promove apenas a cópia validada. Em volume com hard links, a promoção cria o nome final atomicamente no mesmo volume; em outros volumes há fallback de cópia concluída/sincronizada e rename. Ambos preservam os bytes. Hash igual reutiliza o asset existente, inclusive na duplicação, sem criar outro arquivo físico. Uma publicação não aceita a mesma mídia duas vezes.

Prévia usa `convertFileSrc` e protocolo asset oficial do Tauri, com escopo inicial vazio e liberação Rust somente dos diretórios de mídia e staging controlados. CSP inclui apenas os endereços locais de asset em img-src/media-src. Nenhuma origem externa ou acesso arbitrário ao filesystem foi habilitado. O protocolo suporta requisições Range para vídeos; player usa preload metadata.

Temporários de importação inválida ou falha são removidos imediatamente; remoção no formulário e descarte alcançam apenas IDs conhecidos em staging. Save bem-sucedido limpa staging; fechamento normal tenta limpar a sessão. Falhas de permissão na limpeza não apagam o arquivo original e são tratadas como erro ou diagnóstico mínimo `TEMP_CLEANUP_PENDING`, conforme momento.

Quando a transação falha antes do commit, somente arquivos novos criados por ela são removidos; registros/vínculos revertem e staging permite tentar novamente. Se o resultado do commit for incerto ou o processo cair, arquivos completos são conservados para não causar perda de mídia referenciada. Não há limpeza destrutiva automática de arquivos definitivos. Interrupção abrupta pode deixar staging/arquivos completos sem referência; recuperação/limpeza explícita e backup pertencem à evolução futura. Arquivos retirados de uma edição permanecem no catálogo local; não são apagados automaticamente.

## Validação editorial

| Formato | Quantidade e tipos |
| --- | --- |
| Feed | Exatamente 1 imagem |
| Carrossel | 2–20 imagens e/ou vídeos, em ordem explícita |
| Reels | Exatamente 1 vídeo MP4 |
| Stories | Exatamente 1 imagem ou vídeo MP4 |

Imagens: JPEG/JPG, PNG ou WebP, até 64 MiB, lados até 8192 px e total até 40 milhões de pixels. Cabeçalho/dimensões e decodificação são verificados com limite de alocação de 192 MiB, sem reencode, compressão ou mudança de resolução.

Vídeos: MP4 até 512 MiB, contêiner legível e faixa de vídeo com dimensões válidas. Tamanhos de boxes, truncamento e excesso de metadados são verificados antes do parser; metadados moov limitados a 16 MiB. Não decodificar vídeo inteiro. Reprodução depende dos codecs suportados pelo WebView2; contêiner MP4 válido não garante todo codec disponível no Windows. Falha de prévia mostra mídia indisponível. Não existe transcodificação nesta fase.

Título e legenda preservam Unicode; legenda mantém emojis, hashtags e quebras de linha. Rust valida entradas, IDs e versão; não aceita campos adicionais para estado/origem ou SQL. Limites acima são editoriais locais, não afirmação de compatibilidade equivalente com a API Meta, a pesquisar na Fase 07.

## Exclusão e retenção

Exclusão **lógica**: preencher deleted_at/updated_at e incrementar versão numa transação. Listagem/get/contagens deixam de expor o registro. Vínculos e arquivos permanecem, preservando cópias e evitando exclusão física irreversível nesta fase. Não existe botão de restauração nem limpeza definitiva automática; isso não torna a exclusão uma prova de publicação/removal na Meta.

FKs impedem vínculo com asset inexistente e remoção de asset referenciado. Remover mídia de um formulário altera a composição daquela publicação ao salvar; nunca apaga um arquivo usado por outra. Nenhuma publicação, mídia ou credencial de demonstração é criada no banco operacional.

## Contratos IPC

`foundation_status` permanece sem argumentos e com o mesmo DTO; schemaVersion passa a 2 após a migração.

| Comando | Entrada / saída |
| --- | --- |
| publication_list | LibraryQuery (search, format/source/status opcionais, sort, offset, limit) → items resumidos e total |
| publication_get | id → publicação, versão e mídias ordenadas |
| publication_save | id/expectedVersion opcionais em pares, título, legenda, formato e mediaIds → publicação salva |
| publication_duplicate | id → nova publicação manual em rascunho |
| publication_delete | id + expectedVersion → exclusão lógica |
| editorial_counts | Sem argumentos → total/drafts/pendingReview/approved |
| media_select | formato → mídias temporárias e lista de arquivos rejeitados; seleção nativa Rust |
| media_discard | Até 20 IDs → limpar somente staging conhecido |

Todos os comandos específicos constam no AppManifest e capability main. Operações de banco/arquivos usam spawn_blocking e mutex; o frontend não executa SQL. Erros adicionais conhecidos: VALIDATION, MEDIA_INVALID, MEDIA_IO, NOT_FOUND e EDIT_CONFLICT; falhas desconhecidas continuam redigidas. Não registrar legendas, nomes originais, tokens ou payloads editoriais nos logs.

## Testes e evidências

Resultado automatizado atual: **53 testes aprovados** — 32 Vitest, 18 Rust, 3 SQL. Testes Rust/SQLite/arquivos executam operações reais em diretórios temporários isolados; mocks ficam na fronteira dos testes de componentes. Amostra MP4 mínima do repositório oficial mp4-rust em tests/fixtures, com licença MIT; imagens são geradas pelos testes. Nenhum fixture é conteúdo operacional.

Cobertura: criação, título/legenda/emoji, cópia fiel e independência da origem, persistência/reabertura, ordem, edição/versão/conflito, troca de formato/mídias, duplicação sem cópia física, exclusão lógica, FKs, deduplicação, inválidos/inexistentes/excesso de bytes, falha de SQL e arquivo, pesquisa sem acentos, filtros, ordenação/paginação, estado vazio, Dashboard, guardas de navegação e migração 1→2 preservando a preferência anterior.

Validação final de 14/10/2026:

| Comando | Resultado |
| --- | --- |
| npm.cmd run check | Typecheck, lint sem warnings, 32 Vitest, 3 SQL e build frontend aprovados |
| cargo check --locked (src-tauri) | Aprovado |
| cargo fmt -- --check (src-tauri) | Aprovado |
| cargo clippy --locked --all-targets -- -D warnings (src-tauri) | Aprovado, sem warnings |
| cargo test --locked (src-tauri) | 18 aprovados, nenhum reprovado |
| npm.cmd run build:desktop | Aprovado; release Rust em 47,22 s; executável sem bundle |
| npm.cmd run dev -- --window-size 1366x768 | Janela Tauri real; DATABASE_READY; fluxos abaixo aprovados |

Build frontend: 58 módulos; CSS 24,53 kB, JavaScript 288,40 kB, antes de gzip. O aviso experimental de node:sqlite nos testes SQL/inspeções é do runtime Node, sem falha de teste.

### Validação nativa realizada

Executada na janela debug Tauri/WebView2 com perfil isolado `.tmp/phase03-acceptance-20261014`, sem inserir dados de teste no perfil operacional. O operador auxiliou na seleção da primeira imagem pelo diálogo Windows; demais ações foram realizadas na janela do aplicativo. Não foi usada prévia de navegador para declarar aceite desktop.

| Fluxo | Evidência observada |
| --- | --- |
| Cadastro de imagem | Seleção nativa de PNG 512×512; prévia; Save por IPC; detalhes como Feed/Manual/Rascunho |
| Legenda | Três linhas com emoji, acentos e hashtags preservadas; ação Copiar legenda retornou confirmação |
| Edição | Título alterado; registro mantido; versão passou de 1 a 2; legenda e mídia preservadas |
| Duplicação | Novo UUID e título com (cópia); rascunho; mesma referência de mídia, sem segundo arquivo |
| Exclusão | Usuário autorizou excluir somente a cópia de QA; confirmação do produto; cópia saiu da biblioteca; original/arquivo preservados |
| Vídeo | MP4 320×240, 2591 bytes, cerca de 0,1 s; importação, metadados e Save; controle alternou reproduzir → pausar → reproduzir ao término, antes e após reabertura |
| Carrossel | Duas imagens selecionadas juntas; segunda.jpg movida para primeira posição; ordem persistida; galeria alternou para imagem.png |
| Busca | acao encontrou a legenda com ação; inexistente-QA produziu estado vazio |
| Filtros | Reels sem vídeo cadastrado produziu zero; origem Manual manteve os registros manuais; Aprovado produziu zero para registros em rascunho |
| Dashboard | 3 ativas e 3 rascunhos; 0 pendentes/aprovadas; cópia excluída fora das contagens |
| Fechar/reabrir | Encerramento normal e novo processo com o mesmo perfil; biblioteca, mídia, legenda, título e versão preservados |
| Independência da origem | Somente cópias de QA renomeadas para origem-preservada-*; reabertura continuou exibindo imagens e player; SHA-256 armazenado igual ao original preservado |
| Guarda de fechamento | Alteração não salva seguida de Alt+F4 abriu confirmação; Cancelar manteve formulário; restabelecer título salvo retirou estado dirty |
| Layout | Cadastro, detalhes e biblioteca em conteúdo 1366×768 (moldura 1368×800); biblioteca/Dashboard maximizado no monitor Full HD, sem sobreposição de controles |
| Release | Executável incorporado abriu em http://tauri.localhost; armazenamento conectado e contagens operacionais vazias, sem mocks ou QA |

Inspeção somente leitura antes/depois da reabertura: histórico de migrações em versão 2, `quick_check=ok`, `foreign_key_check` sem violações, 3 publicações ativas/1 excluída e 3 assets/arquivos físicos íntegros. Ordem do carrossel e legenda exata conferidas no SQLite. No perfil operacional, preferências, timestamps e registro completo da migração 001 ficaram idênticos ao snapshot anterior; nenhuma publicação QA foi inserida. O release aplicou 002 incrementalmente.

A amostra MP4 é mínima e o player arredonda a duração para 0:00. O teste comprova a importação, controles e reprodução dessa amostra; não certifica todos os codecs, vídeos longos, áudio ou cargas de 512 MiB. Não houve transcodificação. Paginação em massa, falhas de escrita e conflitos foram cobertos pelos testes automatizados em diretórios temporários, não por criação em massa no perfil operacional.

### Evidências visuais

- [Biblioteca 1366×768](evidencias/fase03/biblioteca-1366x768.jpg).
- [Carrossel e ordem](evidencias/fase03/carrossel-detalhes-1366x768.jpg).
- [Vídeo no editor](evidencias/fase03/video-editor-1366x768.jpg).
- [Biblioteca Full HD](evidencias/fase03/biblioteca-fullhd.jpg).
- [Dashboard com contagens reais de QA](evidencias/fase03/dashboard-real-fullhd.jpg).
- [Biblioteca após reabertura](evidencias/fase03/biblioteca-apos-reabrir.jpg).
- [Release operacional sem QA](evidencias/fase03/release-operacional.jpg).

Full HD contém uma sobreposição externa do companheiro do Codex; não integra a interface GMAD. Evidências mostram exclusivamente conteúdo próprio/de teste, sem salvar o seletor com arquivos pessoais.

### Problemas tratados

- Tipos inteiros do rusqlite ajustados na leitura/gravação de bytes; cargo check e testes confirmaram compatibilidade.
- Importação valida dimensões/alocação de imagens e boxes MP4 antes do parser; promoção ganhou fallback para volumes sem hard links e verificação de checksum.
- Assets reutilizados e falhas transacionais foram testados sem invalidar publicações compartilhadas.
- Seletor ligado à janela principal; protocolo asset/CSP e permissões específicas ajustados para prévias e guarda de fechamento.
- Build inicial encontrou o release anterior em uso; somente esse processo foi encerrado e builds seguintes passaram. Banco não foi removido ou recriado.
- Sessão Windows bloqueada e controle do diálogo nativo limitaram a primeira inspeção. Com desbloqueio/seleção assistida e coordenadas do diálogo, a validação foi completada em 14/10. Um PNG de Downloads rejeitado na inspeção anterior não teve a causa específica determinada; não foi classificado como defeito corrigido. O PNG/JPEG conhecidos de QA foram importados corretamente.

## Arquivos relevantes e pendências

Rust: editorial_domain.rs, editorial_repository.rs, media_store.rs, editorial_tests.rs, comandos em lib.rs, migração 002, AppManifest/capability/CSP, dependências e lockfile. Frontend: tipos/serviço editorial, useEditorialQuery, features/editorial (editor e prévia), Library, PublicationDetails, NewPublication, Dashboard, navegação/guardas no App e editorial.css. Testes: editorial.test.tsx, app.test.tsx, migrations.test.mjs e fixture MP4 licenciada.

Trello, revisão completa, calendário persistente, Meta, publicação externa, instalador e backup permanecem fora desta fase. Limitações de codec, retenção sem restauração/limpeza física e recuperação após interrupção abrupta estão documentadas acima; não impedem o escopo local validado. Não foi executada publicação externa.

Critérios 1–13 de aceite da solicitação atendidos: criar/importar, persistir, listar/pesquisar/filtrar, editar/duplicar/excluir com segurança, Dashboard real, integridade, Tauri, documentação e testes/builds. Critério 14: entrega registrada no commit local `feat: implement local editorial library` (consultar `git log -1` para o hash). A revisão automática inicialmente rejeitou o fixture MP4 pela regra de mídias fora do Git; em 14/10/2026, o usuário autorizou explicitamente somente a amostra MIT de 2.591 bytes como exceção de teste. Bancos, mídias operacionais e builds não entram no commit. Não houve push ou tag. Fase 04 tecnicamente liberada para uma nova solicitação; nenhuma integração Trello foi iniciada.
