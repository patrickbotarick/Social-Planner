# Fase 05 — Revisão, aprovação e triagem editorial

Implementada em 16/10/2026, sobre `97a7a99`. Escopo exclusivamente editorial local. Calendário persistente, agendamento e integração Meta permanecem nas fases 06–09.

## Fluxo entregue

Revisão abre a fila de pendentes com contagens reais de pendentes, aprovadas e ignoradas. O progresso considera `(aprovadas + ignoradas) / (pendentes + aprovadas + ignoradas)` do acervo ativo; rascunhos ainda não enviados ficam fora desse denominador. Os indicadores não mudam de significado ao filtrar a lista. Dashboard inclui total, rascunhos, pendentes, aprovadas e ignoradas, consultados novamente ao entrar na página.

Pesquisa normalizada por título/legenda; filtros combináveis de quadro, lista, formato, estado, origem, presença de mídia e erros/avisos. Paginação de 30 registros, ordenação estável por criação/ID. A seleção múltipla abrange a página visível; mudar filtro/página limpa a seleção. Até 10.000 IDs por consulta permitem navegação entre páginas sem pular itens quando uma decisão os remove do filtro. Acima disso, é necessário refinar a consulta.

A análise individual mostra imagem, vídeo com controles WebView2, galeria ordenada, posição da mídia, título, legenda, origem, quadro/lista, link canônico do cartão, formato, estado, versão, avisos e histórico. A seleção aberta preserva sua ordem durante decisões; anterior/próxima continuam disponíveis. Voltar mantém filtros e página da listagem durante a sessão da área.

O editor da Fase 03 é reutilizado para título, legenda, formato, ordem e inclusão/remoção de mídias. Markdown Trello continua com interpretação limitada e segura, mantendo o texto original armazenado. Emojis, hashtags, acentuação e quebras de linha não são normalizados na gravação editorial.

## Estados e decisões

| Ação explícita | Estado de origem | Resultado |
| --- | --- | --- |
| Enviar para revisão | `draft` | `pending_review` |
| Aprovar | `pending_review`, `editing` | `approved`, após validação |
| Ignorar | `pending_review`, `editing` | `ignored`, preservando conteúdo/mídias/origem |
| Retornar à revisão | `approved`, `ignored`, `editing` | `pending_review` |
| Salvar edição | `draft` | Continua `draft`, nova versão |
| Salvar edição | `pending_review`, `editing`, `approved`, `ignored` | `pending_review`, nova versão |

Abrir o editor não grava um estado transitório: o formulário só persiste ao salvar explicitamente. O estado legado `editing` é aceito nas transições; não foi criada outra enumeração de estados. Salvar edição de uma publicação aprovada ou ignorada invalida a decisão anterior e exige nova decisão. A UI não oferece transições de `ready_to_schedule`, reservado à evolução futura. Aprovar não agenda nem publica.

Cada decisão exige a versão esperada; conflito mantém o registro intacto. Aprovação individual não tem confirmação redundante. Ignorar aceita, sem obrigatoriedade, motivo e observação de até 2.000 caracteres. Motivos: produto não comercializado, campanha não participante, conteúdo não relevante, duplicação editorial ou outro. Ausência de motivo é válida. A observação não é enviada ao Trello.

Alterações não salvas ativam diálogo com **Permanecer**, **Descartar e sair** e **Salvar e continuar**, tanto na navegação da revisão quanto na navegação global/fechamento. Falha de salvamento mantém o formulário e o diálogo. Operações em andamento impedem saída até concluir. Não há salvamento automático.

## Ações em lote

Aprovar, ignorar e retornar à revisão exigem confirmação com quantidade e ação. O Rust aceita de 1 a 100 alvos únicos por chamada; a interface envia somente os selecionados da página, com suas versões. Cada item tem transação própria, com atualização de estado/versão e evento indivisíveis. Resultado parcial informa sucessos e erros por ID; falhas ficam selecionadas para correção. Os itens não selecionados permanecem intactos. Um item inelegível não impede os demais e não ganha evento de sucesso.

## Mídias e limites de validação

- Impedem aprovação: composição vazia; quantidade/tipo incompatível com o formato editorial existente; arquivo ausente, tamanho divergente, checksum alterado ou conteúdo inválido.
- Análise individual e aprovação verificam bytes, SHA-256, parser de imagem/estrutura MP4 e metadados. Os limites locais existentes continuam em 64 MB por imagem e 512 MB por MP4. Não são apresentados como limites oficiais Meta.
- A listagem faz checagem leve de caminho/tamanho e formato. Alterações com mesmo tamanho são detectadas na análise individual/aprovação; o filtro de pendências não substitui essa checagem completa.
- Composição mista é aviso de decisão humana, não falha automática. Legenda vazia é aviso. Vídeos têm aviso sobre verificação da reprodução local; estrutura MP4 válida não certifica codecs.
- Falha de apresentação WebView2 em imagem/vídeo bloqueia o botão Aprovar naquela visualização, com explicação separada de falha de download. Reabrir/editar permite nova tentativa. O backend não decodifica codecs de vídeo e não pode certificar reprodução em operações em lote: confira os vídeos individualmente antes de confirmar o lote.
- Não há transcodificação, publicação, requisitos Meta presumidos ou limpeza física de mídias.

## Persistência e segurança

Migração aditiva **004_editorial_review.sql**, sem modificar 001–003, cria `editorial_reviews`. Guarda UUID do evento/publicação, tipo (`approved`, `ignored`, `returned`, `submitted`, `edited`), estados anterior/novo, versões revisada/resultante, motivo/observação opcionais e timestamp UTC. A versão resultante é sempre a anterior + 1. Índices por publicação/data e histórico recente; foreign key RESTRICT e triggers proíbem UPDATE/DELETE. Não guarda cópias de legendas, arquivos, credenciais ou payloads remotos.

Comandos específicos `review_list`, `review_get`, `review_decide`, `review_history`, declarados no manifesto/capability da janela. DTOs tipados, validações Rust, parâmetros SQL vinculados e conexão sincronizada. Nenhum SQL, shell, HTTP ou caminho arbitrário foi exposto ao frontend. Nenhuma dependência foi acrescentada.

Sincronização Trello continua atualizando apenas a origem. Teste de regressão aprova um cartão, ignora outro, altera snapshots remotos e sincroniza ambos: estados, versões, legendas/títulos locais, mídia e histórico permanecem; mudanças remotas continuam sinalizadas. Não foram realizadas novas chamadas ao Trello real nesta fase, nem escrita remota ou importação de novos cartões.

## Atalhos e acessibilidade

Seta esquerda/direita: anterior/próxima, sem modificadores ou repetição, fora de campos editáveis, links, botões, controles de vídeo e diálogos. Campos de legenda mantêm suas setas de edição. Escape cancela/fecha diálogos sem confirmar mudanças. Não há atalhos de aprovação/ignorar que possam alterar estado por acidente. Tab, foco visível, rótulos e mensagens acessíveis reutilizam o design system.

## Validações automatizadas

| Comando | Resultado final |
| --- | --- |
| `npm.cmd run typecheck` | Aprovado, TypeScript estrito |
| `npm.cmd run lint` | Aprovado, zero warnings |
| `npm.cmd test` | 61 aprovados |
| `npm.cmd run test:sql` | 5 aprovados |
| `cargo test --manifest-path src-tauri/Cargo.toml` | 62 aprovados; 5 opt-in ignorados |
| `cargo check --manifest-path src-tauri/Cargo.toml` | Aprovado |
| `cargo fmt --manifest-path src-tauri/Cargo.toml --check` | Aprovado |
| `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings` | Aprovado |
| `npm.cmd run build` | Aprovado |
| `npm.cmd run build:desktop` | Aprovado, executável Windows release sem bundle |

Total: **128 testes locais aprovados**, zero falhas finais. Os cinco ignorados são preparadores QA (2), inspeção/download remotos (2) e teste de cofre Windows opt-in (1); não são contados como aprovados. Node emite aviso experimental do módulo SQLite usado apenas pelos testes auxiliares, sem falha.

Cobertura: transições individuais/reversíveis, motivo opcional, edição/versionamento, persistência após reabrir, histórico imutável, rollback quando auditoria falha, conflitos, resultados parciais, itens não selecionados, mídias ausentes/alteradas/inválidas, composição mista, aprovação bloqueada, filtros/pesquisa/paginação/ordem, contagens, reprodução indisponível, lote com confirmação, guarda salvar/descartar/permanecer, navegação por teclado, editor manual e preservação após ressincronização Trello.

## Homologação Windows isolada

Perfil debug explícito `.tmp/phase05-qa-20261016`, iniciado por `GMAD_QA_DATA_DIR` + `npm.cmd run dev -- --window-size 1366x768`. Preparado a partir do snapshot consistente de aceite da Fase 04, com cinco publicações e sete arquivos reais; checksums conferidos antes da cópia. Dados demonstrativos não foram introduzidos no produto operacional. Release ignora a variável QA, conforme contrato já existente.

Inspeções realizadas na janela Tauri, usando a skill [Computer Use](C:/Users/patrick.silva/.codex/plugins/cache/openai-bundled/computer-use/26.1007.71326/skills/computer-use/SKILL.md):

1. Dashboard: cinco pendentes inicialmente; contagens reais atualizadas após decisões.
2. Publicação com PNG: imagem, origem, legenda Markdown e metadados; aprovação, retorno, edição com emoji/quebras de linha e diálogo Salvar e continuar funcionando.
3. Carrossel: navegação da primeira imagem PNG para a segunda JPEG, renderização e indicador 2/3. Ignorar com observação opcional preservou três mídias e gerou histórico.
4. Composição mista: aviso editorial e prévia PNG; MP4 local abriu com duração 1:26 e controle de reprodução acionado. Vídeo Reels de 46 MB reproduziu com quadros mudando e tempo avançando a 0:13/0:58.
5. Lote com quatro pendentes: três aprovações e uma rejeição por ausência de mídia. A publicação sem mídia permaneceu na versão 1 e o carrossel não selecionado permaneceu ignorado. Nenhuma falha apagou dados.
6. Ignorar em lote sem motivo e retorno em lote do carrossel filtrado aprovados. Filtro Ignorado recuperou duas publicações; pesquisa `cozinha` restringiu a uma. Após retorno, esse item saiu corretamente do filtro.
7. Layout de conteúdo 1366×768 (moldura externa 1368×800) e janela maximizada em monitor 1920×1080: mídia, formulários, filtros, navegação e diálogos utilizáveis, sem scroll horizontal observado. Legenda e mídia foram compactadas após a primeira inspeção para facilitar acesso às ações.
8. Histórico global exibiu nove eventos reais, versões e observação. Fechamento normal e reabertura pelo mesmo perfil QA confirmaram no Dashboard cinco publicações: uma pendente, três aprovadas e uma ignorada. Comparação SQLite antes/depois preservou registros, legenda com emoji/quebras de linha, nove eventos, sete assets e sua ordem/checksums. `quick_check = ok`, sem falhas em `foreign_key_check`.

Evidências locais, fora do Git: perfil QA, baseline `.tmp/phase05-operational-baseline.json`, verificador `.tmp/phase05-verify.mjs` e resultados `.tmp/phase05-qa-verification.json`. Não publicar esses arquivos, textos ou mídias operacionais no repositório.

## Limitações e pendências

Validação de codecs depende do WebView2 instalado; não equivale à homologação Meta. As regras editoriais de composição continuam sendo as da biblioteca, sem adicionar limites de plataforma. Consultas muito grandes exigem filtros; a inspeção completa de arquivos ocorre no worker Rust e pode ocupar brevemente a conexão local. Filtros/posição são contexto da sessão da área, não preferências persistentes entre reinícios.

O contexto operacional aberto pelo usuário na fase anterior difere do contexto aparente de armazenamento acessível ao processo de automação, conforme registrado na homologação 04.5. Por isso esta fase fez todas as decisões/edições em cópia QA explícita; não abriu release para alterar o acervo operacional, não restaurou, apagou ou recriou seu banco. O snapshot de aceite e os sete arquivos operacionais permaneceram intactos, com hashes conferidos. A versão nova aplicará a migração aditiva quando o usuário abrir seu contexto habitual.

A Fase 06 depende de solicitação própria; nenhuma implementação de calendário, agendamento ou Meta foi iniciada nesta fase.

## Aceite

Critérios técnicos da Fase 05 atendidos no perfil Windows isolado: fila/decisões/edição/lotes/filtros/histórico/Dashboard, integridade, persistência, regressão Trello, testes e builds. Sem bloqueio técnico remanescente no escopo implementado. A apreciação do fluxo visual pelo usuário continua disponível e não é apresentada como aprovação já fornecida. A Fase 06 está tecnicamente habilitada para solicitação, sem início automático. Commit previsto: `feat: implement editorial review workflow`; o hash é informado na entrega Git, sem push/tag.
