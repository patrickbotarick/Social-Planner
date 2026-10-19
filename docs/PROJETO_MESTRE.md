# GMAD Social Planner — Projeto mestre

Fonte oficial de escopo e contratos. Criado em 13/10/2026. Alterações relevantes devem preservar os dados locais e atualizar este documento e os registros complementares.

## 1. Visão geral

Aplicativo Windows desktop para um único operador da GMAD Rio Preto organizar o conteúdo editorial recebido mensalmente da matriz pelo Trello. Resolve a dispersão de textos e mídias e permite selecionar campanhas aplicáveis à loja antes de planejar publicações no Instagram.

O produto completo permitirá importar, revisar, editar, criar, aprovar, planejar e acompanhar publicações. A Fase 01 entrega a fundação Windows validada; a Fase 02 consolida identidade e layouts. A Fase 03 implementa biblioteca manual, cópia local de mídias, edição, detalhes, pesquisa, duplicação e exclusão lógica; foi validada no Tauri Windows em 14/10/2026, com 53 testes aprovados, builds e persistência após reabertura. A Fase 04 acrescenta integração Trello somente leitura, snapshots independentes, download validado e jobs locais. A homologação real foi autorizada: conexão/consulta aprovadas; redirecionamento do anexo corrigido e PNG real validado em 16/10/2026. A publicação original foi recuperada com imagem, sem duplicações; segunda sincronização e persistência após reabertura aprovadas, conforme [Fase 04.5](FASE_04_5_HOMOLOGACAO_TRELLO.md). A Fase 05 implementa revisão individual e em lote, decisões reversíveis, edição com proteção de saída e histórico imutável por versão; evidências em [Fase 05](FASE_05_REVISAO_EDITORIAL.md). A Fase 06 implementa calendário, fila de aprovadas, data/hora/fuso, arraste com confirmação, conflitos e histórico locais, com 152 testes e persistência no Tauri; cancelamento nativo e preservação homologados no QA. Agendamento e publicação externa continuam nas fases próprias.

### Restrições permanentes

- Local-first, sem login interno, Supabase, Firebase, servidor próprio, banco em nuvem ou serviço pago obrigatório.
- Funcionalidades locais operam offline. Somente integrações externas exigirão internet.
- Dados editoriais e arquivos de mídia permanecem no computador, fora da instalação.
- Credenciais externas ficam no cofre do sistema operacional, nunca no SQLite editorial, repositório, logs ou armazenamento persistente do frontend. O formulário recebe o token transitoriamente para enviá-lo ao Rust.
- Sem automação de navegador para contornar limitações da Meta. Sem publicação real sem autorização explícita.
- Nenhuma confirmação local representa confirmação da Meta.

## 2. Fluxo operacional planejado

1. Receber a URL do quadro mensal da matriz.
2. Autorizar acesso externo e importar cartões com identidade de origem preservada.
3. Extrair textos e baixar mídias com tratamento de limites, erros e arquivos incompletos.
4. Selecionar campanhas relevantes; aprovar ou ignorar com registro de revisão.
5. Editar uma cópia editorial e criar publicações manuais sem alterar a origem.
6. Planejar o mês, respeitando fuso horário e requisitos de mídia.
7. Agendar pelo método oficial validado na Fase 07, ou usar fluxo assistido no Business Suite.
8. Acompanhar evidências de envio, confirmação, publicação e falhas separadamente.

Fluxo: Trello → importação → biblioteca → revisão → aprovação → calendário → integração Meta → publicação.

## 3. Arquitetura

React + TypeScript estrito + Vite compõem a interface. Tauri 2 hospeda a janela e expõe comandos Rust específicos, tipados e de privilégio mínimo. SQLite é controlado exclusivamente pelo Rust, com repositório testável e migrações SQL versionadas. CSS usa tokens e módulos. Vitest valida a interface.

Frontend: apresentação, navegação, estados de carregamento/erro/vazio e validações de usabilidade. Recebe chave/token apenas no formulário transitório de configuração, limpa o token ao enviar e nunca recupera credenciais persistentes; não executa SQL, não controla caminhos arbitrários e não chama Trello/Meta diretamente.

Rust: validação de comandos, caminhos confiáveis, persistência, migrações, integridade e integração Trello. Erros retornam códigos e mensagens úteis sem conteúdo sensível. Logs operacionais serão locais, mínimos e sem textos editoriais ou credenciais.

Diretório: `app.path().app_data_dir()` do Tauri, que no Windows resolve o diretório de dados do usuário sob o identificador estável `br.com.gmad.socialplanner`. Banco `planner.sqlite3`, mídias por checksum em `media/` e cópias temporárias em `temp/imports/<sessão>/`. Nunca usar a pasta de instalação. Não renomear o identificador sem plano explícito de migração de dados.

Uma conexão SQLite sincronizada atende os comandos locais; inicialização usa transações, foreign keys, busy timeout e verificação `quick_check`. Migrações desconhecidas ou banco inválido bloqueiam acesso sem apagar ou recriar dados. Versão 1 mantém histórico/preferências; migração incremental 002 acrescenta publications, media_assets e publication_media sem dados pré-carregados. A migração 003 acrescenta trello_boards, imported_cards, trello_attachments e import_jobs, preservando 001/002. A migração aditiva 004 cria editorial_reviews, com histórico imutável e decisões versionadas, preservando 001–003. A 005 cria schedules, schedule_settings e schedule_events, com unicidade ativa, auditoria e invalidação editorial transacionais, preservando 001–004.

Permissões: janela local única, CSP restritiva, comandos específicos e sem plugins genéricos de shell, filesystem, HTTP ou SQL expostos ao frontend. Seletor de arquivos no Rust; protocolo asset limitado aos diretórios controlados de mídia/staging. Eventos e destroy da janela permitem a guarda de fechamento. A prévia web identifica a indisponibilidade do desktop, sem simular persistência.

## 4. Modelo de dados planejado

Identificadores locais das entidades editoriais serão UUIDs gerados no Rust; IDs externos são strings opacas e separados. Datas técnicas em UTC, com fuso IANA preservado onde houver planejamento. Relacionamentos têm foreign keys e índices explícitos; exclusão de origem não deve apagar edições do usuário em cascata.

| Entidade | Responsabilidade, relacionamentos e restrições |
| --- | --- |
| TrelloBoard | Origem do quadro; ID local, `external_board_id` único, URL canônica, nome e última sincronização bem-sucedida. |
| ImportedCard | Snapshot da origem; pertence ao quadro; ID Trello único, texto original, hash e timestamps externos. Importações atualizam o snapshot, nunca o texto editorial. |
| MediaAsset | Arquivo local e metadados; origem/anexo opcional, hash de conteúdo, MIME, tamanho, caminho relativo validado e estado do download. Anexo externo único por origem; compartilhamento por hash exige contagem de referências. |
| Publication | Documento editável do usuário; cartão de origem opcional, título, legenda, formato, versão e estado editorial. Uma publicação inicial por cartão; futuras variantes precisam de ação explícita e vínculo próprio. |
| PublicationMedia | Associação N:N entre publicação e mídia; posição não negativa, única por publicação, e par publicação/mídia único. |
| EditorialReview | Histórico imutável de revisão, motivo, estado anterior e novo, data e versão da publicação. Pertence à publicação. |
| Schedule | Fase 06: intenção local, publicação/versão aprovada, data/hora local, UTC, fuso IANA, estado e versão otimista. No máximo um ativo por publicação. Modo oficial/assistido e evidências externas permanecem futuros. |
| ScheduleEvent | Fase 06: auditoria local imutável de criação, reagendamento, cancelamento e invalidação, com instantes anteriores/novos e versões. |
| PublicationEvent | Eventos imutáveis de integração/publicação; publicação e agendamento opcionais conforme tipo, data, código, identificador externo e chave de deduplicação. Sem tokens ou payloads sensíveis. |
| AppSettings | Preferências locais versionadas. Fase 01: registro único com fuso `America/Sao_Paulo` e datas de criação/atualização; sem segredos. |

Camadas independentes: origem importada (`ImportedCard`), conteúdo do usuário (`Publication`), decisão editorial (`EditorialReview`), intenção/estado de agendamento (`Schedule`) e evidência real (`PublicationEvent`).

Reimportação: usar IDs externos e upsert transacional, nunca título como identidade. Repetir a mesma importação não cria outra publicação inicial ou anexo. Cartão alterado sinaliza atualização de origem; edições do usuário são preservadas. Cartões removidos da origem são marcados, não excluídos silenciosamente. Falhas parciais não avançam a marca de sincronização completa. Implementado na Fase 04; falhas de anexo não apagam resultados anteriores.

## 5. Máquinas de estados planejadas

Os estados abaixo pertencem a eixos distintos; não serão condensados num único campo de status.

**Editorial:** `draft` é o rascunho manual; editar mantém draft e duplicar cria outro draft independente. Enviar explicitamente à revisão produz `pending_review`; Trello já cria nesse estado. Na Fase 05, `pending_review`/`editing` → `approved` ou `ignored`; `approved`/`ignored`/`editing` → `pending_review` por retorno explícito. Salvar edição de conteúdo em approved/ignored/editing retorna diretamente a pending_review, em uma transação e com nova versão. Abrir o editor não grava estado transitório; esta decisão evita mutação implícita sem salvar. Cada decisão e edição gera histórico imutável. Aprovação se refere à versão revisada e exige integridade/composição de mídia válida. `ready_to_schedule` permanece reservado e sem ação; o planejamento local usa seu próprio eixo e conserva approved. Não há segunda lógica concorrente de estados.

**Planejamento local implementado na Fase 06:** `planned_local` → `cancelled_local` por confirmação ou `invalidated_local` por mudança de versão/aprovação/exclusão lógica da publicação. Reagendar conserva o estado ativo e incrementa versão; auditoria e alterações são transacionais. Encerrados não reativam: após revisão, novo plano exige ação explícita. Aprovação continua editorial, sem promover a `ready_to_schedule` nem implicar envio. Data/hora local e UTC com fuso IANA explícito; um ativo por publicação, com mídia validada.

**Agendamento externo conceitual, ainda não implementado:** submissão, recebimento, confirmação, falha e resultado incerto serão definidos após a pesquisa da Fase 07. Confirmação exigirá evidência oficial; resultado incerto exigirá reconciliação antes de reenviar. Cancelamento/reagendamento confirmado externamente requer reconciliação oficial; nenhuma operação local deve afirmar que cancelou na plataforma.

**Publicação real:** `not_published` → `published` apenas com identificador/evidência consultada da plataforma. Uma falha de envio não apaga uma publicação já comprovada. Falhas registram escopo, instante, código e possibilidade de nova tentativa; não se repetem envios cegamente.

Fluxo assistido registra intenção local e ação humana; abrir Business Suite ou marcar uma tarefa como feita não equivale a confirmação pela API. Evidência manual deve ser identificada como tal. Depois de confirmação oficial efetiva, deseja-se independência do computador ligado; viabilidade permanece em aberto até Fase 07.

## 6. Interface

Na Fase 04.5C, detalhes e origem Trello receberam apresentação limitada de negrito e escapes comuns, sem interpretar HTML ou alterar o texto armazenado. O editor mantém a string integral; os botões de cópia distinguem legenda editorial e descrição original. Vídeo isolado, três imagens e composição mista reais foram importados mediante confirmação, exibidos/reproduzidos e sincronizados novamente sem duplicações. A persistência após reabertura foi aprovada, com cinco publicações e sete arquivos íntegros; Fase 04 homologada em 16/10/2026 para o escopo implementado. Evidências em FASE_04_5_HOMOLOGACAO_TRELLO.md.

Dashboard consulta contagens editoriais reais, aprovadas disponíveis, planejadas/conflitos do mês e próximos planos locais. Menu: Dashboard, Biblioteca editorial, Importações Trello, Revisão de publicações, Calendário mensal, Nova publicação, Histórico e Configurações. Biblioteca/cadastro/revisão mantêm suas operações. Calendário oferece mês/lista/dia, fila de aprovadas, filtros, formulário data/hora/fuso, arraste com confirmação, reagendamento e cancelamento lógico. Mudança editorial invalida o plano atomicamente. Histórico separa auditoria editorial e de planejamento. Preferências gerais permanecem em leitura; proximidade e intervalo sugerido têm configuração específica persistente. Recursos das fases 07–09 não foram implementados.

Verde profundo, branco, cinza e detalhe laranja formam a identidade do software consolidada na Fase 02, documentada em DESIGN_SYSTEM.md, sem afirmar códigos oficiais da marca. Inter local licenciada, layout adaptável, foco visível, links semânticos, títulos e estados acessíveis. Nenhum dado demonstrativo deve se passar por dado real.

## 7. Integrações

**Trello (Fase 04, implementada e homologada nas etapas 04.5B/04.5C):** chave/token do fluxo oficial 1/authorize com scope=read, guardados no Windows Credential Manager; leitura de quadros, cartões e anexos; paginação, limites de taxa, timeout, retentativas limitadas, cancelamento e importação incremental idempotente. Validar URLs e destinos de download no Rust, tamanho/tipo de arquivo e integridade. Nenhuma integração ativa na Fase 01.

**Meta (Fase 07 — pesquisa documental concluída em 19/10/2026):** referências oficiais v26.0 confirmam publicação imediata com restrições. Não foi localizado contrato público de agendamento Instagram futuro por API. Recomendado preparar localmente e programar manualmente no Business Suite, sujeito à escolha do usuário. A conta GMAD, OAuth e publicação reais não foram homologados; integração não implementada. Fontes em FASE_07_VIABILIDADE_META.md, contratos em INTEGRACAO_META.md e recomendação em DECISAO_AGENDAMENTO_META.md.

Prova documental registra versão, fontes, formatos, quotas inconsistentes, expiração e estados de contêineres. Upload de vídeo retomável não comprova upload binário de imagem; imagens por API dependem de URL pública. OAuth com App Secret exige troca no servidor, sem embutir segredo no cliente; token administrativo do painel para conta própria é alternativa documentada, ainda não configurada. Não tornar hospedagem/relay/servidor obrigatório sem nova decisão. No caminho assistido, o GMAD pode ser desligado depois que o operador efetivamente programa na interface Meta; isso não é sincronização automática nem confirmação pela API. Nenhuma autenticação/publicação real sem autorização específica.

## 8. Roadmap oficial

| Fase | Objetivo e escopo | Dependências | Critérios de conclusão | Estado |
| --- | --- | --- | --- | --- |
| 01 — Fundação e documentação | Documentos, shell desktop, navegação e SQLite mínimo | Node, Rust MSVC, C++ Build Tools, WebView2 | Documentos consistentes, inicialização desktop/banco e navegação verificadas; validações e limitações registradas | Concluída tecnicamente e validada no Windows; evidências em FASE_01_5_VALIDACAO_WINDOWS.md |
| 02 — Identidade visual e interface | Identidade definitiva e componentes acessíveis | 01 aceita, referências visuais | Interface consistente, responsiva e aprovada | Implementada e validada tecnicamente no Windows; evidências em FASE_02_IDENTIDADE_INTERFACE.md; apreciação visual final disponível ao usuário |
| 03 — Biblioteca editorial | Persistência editorial, mídias e criação manual | 02; migrações das entidades necessárias | Criar/editar/consultar conteúdo real sem perda de dados | Concluída tecnicamente e validada no Windows em 14/10/2026; evidências e limites em FASE_03_BIBLIOTECA_EDITORIAL.md |
| 04 — Integração Trello | Autorização, cartões, anexos e importação incremental | 03; acesso oficial ao Trello | Importação repetível sem duplicação; falhas e limites tratados | Homologada em 16/10/2026: imagem, vídeos e múltiplas mídias reais; repetição idempotente, preservação editorial e persistência após reabertura aprovadas |
| 05 — Revisão e aprovação | Decisões, histórico e validação editorial | 03 e 04 | Transições válidas e rastreáveis por versão | Implementada e validada no Windows em perfil isolado; 128 testes locais aprovados, verificações e builds; evidências em FASE_05_REVISAO_EDITORIAL.md |
| 06 — Calendário mensal | Planejamento local com fuso e conflitos | 05 | Agendar/replanejar localmente sem alegar confirmação Meta | Concluída tecnicamente em 19/10/2026: 152 testes, builds, Tauri em três resoluções, arraste, cancelamento e persistência aprovados em QA isolado |
| 07 — Pesquisa e integração Meta | Prova técnica oficial e decisão de viabilidade | 06; conta profissional e autorização externa | Evidências oficiais e estratégia validada, inclusive alternativa assistida | Pesquisa documental concluída em 19/10/2026; recomendado Business Suite manual assistido; escolha do usuário e homologação da conta pendentes. Integração não implementada |
| 08 — Agendamento e acompanhamento | Executar estratégia validada e reconciliar eventos | 07 | Estados reais separados, retentativas seguras e confirmação auditável | Não iniciada |
| 09 — Testes e distribuição | Endurecimento, recuperação e distribuição Windows | 01–08 | Testes de ponta a ponta, instalação/atualização e recuperação validados | Não iniciada |

Não iniciar outra fase sem solicitação. Detalhes de aceite e acompanhamento em ROADMAP.md.

## 9. Qualidade e evolução

- TypeScript estrito; typecheck, lint, Vitest e build frontend. Rust: formatação, testes de migração/persistência e build desktop quando o ambiente permitir.
- Migrações sequenciais e transacionais; não editar migração aplicada. Falha deve preservar o arquivo e permitir diagnóstico, sem reset automático.
- Inicialização idempotente e teste de persistência após reabertura. Integridade básica não substitui estratégia de recuperação.
- Tratamento explícito de carregamento, indisponibilidade do desktop e falhas de armazenamento.
- Acessibilidade por teclado, contraste e responsividade. Sem fontes/CDNs remotos para funcionamento local.
- Logs mínimos sem segredos; futura rotação e exportação de diagnóstico dependem de escopo próprio.
- Backup futuro consistente via SQLite Backup API ou mecanismo equivalente validado, com banco, manifesto e mídias; não copiar somente o arquivo de um banco aberto em WAL. Restauração deve validar versão/integridade, manter cópia anterior e exigir decisão consciente. Backup ainda não implementado.
- Git com commits incrementais, lockfiles e exclusão de bancos, mídias operacionais, builds e segredos. Sem push automático.
- Aceite da fase depende de evidência real; testes não executados nunca são considerados aprovados.

## 10. Documentos complementares

- [Fase 07](FASE_07_VIABILIDADE_META.md): fontes oficiais, capacidades e limites de viabilidade.
- [Integração Meta](INTEGRACAO_META.md): autenticação, mídia, segurança e contratos futuros.
- [Decisão de agendamento Meta](DECISAO_AGENDAMENTO_META.md): alternativas e recomendação para avaliação do usuário.
- [Fase 06](FASE_06_CALENDARIO_EDITORIAL.md): calendário, modelo local, fuso, conflitos, histórico, testes e homologação Windows.
- [Fase 05](FASE_05_REVISAO_EDITORIAL.md): revisão, decisões, histórico, testes e homologação Windows isolada.
- [Integração Trello](INTEGRACAO_TRELLO.md): conexão, segurança, API, importação e limites.
- [Fase 04](FASE_04_INTEGRACAO_TRELLO.md): testes e pendência de validação externa.

- [Arquitetura](ARQUITETURA.md): fronteiras técnicas e operação.
- [Modelo de dados](MODELO_DADOS.md): contratos e esquema implementado.
- [Decisões técnicas](DECISOES_TECNICAS.md): justificativas e fontes de compatibilidade.
- [Roadmap](ROADMAP.md): entregas e critérios por fase.
- [Fase 01](FASE_01_FUNDACAO.md): evidências, limitações e aceite.
- [Fase 01.5](FASE_01_5_VALIDACAO_WINDOWS.md): instalação autorizada e validação nativa Windows.
- [Design system](DESIGN_SYSTEM.md): identidade do software, componentes, acessibilidade e responsividade.
- [Fase 02](FASE_02_IDENTIDADE_INTERFACE.md): implementação, testes e evidências visuais nativas.
- [Fase 03](FASE_03_BIBLIOTECA_EDITORIAL.md): biblioteca, mídias, contratos, testes e estado do aceite.

Em conflito, este documento define o escopo; divergências de implementação devem ser corrigidas ou justificadas antes de ampliar contratos.
