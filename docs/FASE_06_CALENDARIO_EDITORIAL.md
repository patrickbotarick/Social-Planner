# Fase 06 — Calendário editorial e planejamento mensal

Implementação em 19/10/2026, sobre `9afbe6a937b6cba5d111e4b44f9bbdc952bc86ea`. Escopo exclusivamente local. Nenhum comando, credencial, chamada ou confirmação da Meta foi implementado. Fases 07–09 não iniciadas.

## Entrega e interface

Calendário mensal com 42 células, dia atual, seleção de dia, mês anterior/próximo, Hoje e seleção direta de mês/ano entre 1900 e 9998. Resumo real de planejadas no mês, dias sem posts, distribuição por formato e posts com conflitos. Visão alternativa em lista e detalhes do dia ordenados pelo instante UTC, com miniatura, título, formato, origem, estado, versão planejada e link para a biblioteca.

Fila real de aprovadas sem planejamento ativo, paginada em 20 itens. Pesquisa e filtros por formato, origem, quadro e presença de imagem/vídeo. Pendentes, ignoradas, rascunhos e excluídas ficam fora da fila. Filtros independentes no calendário por pesquisa, formato, origem, quadro e estado. Filtros recolhíveis preservam espaço em 1366×768. Nenhuma contagem ou publicação fictícia na operação.

Agendar abre formulário com miniatura, data, horário de 24 horas e fuso IANA explícito. Horários sugeridos usam intervalo configurável de 5, 10, 15, 30 ou 60 minutos; edição manual aceita qualquer minuto. Apenas confirmação grava. Erros preservam o formulário; alterações não salvas têm opções de salvar, descartar ou permanecer, inclusive na navegação/fechamento. Reagendar usa o mesmo formulário; remover do planejamento pede confirmação e conserva conteúdo, aprovação e arquivos.

O Dashboard consulta aprovadas disponíveis, planejadas/conflitos do mês e até seis próximos itens locais. Histórico tem abas separadas para decisões editoriais e planejamento, com 30 eventos por página. Nenhum desses indicadores representa envio ou confirmação externa.

## Persistência e migração 005

Migração aditiva `005_local_calendar.sql`; 001–004 preservadas. `schedules` referencia publicação e versão aprovada e guarda data local canônica, hora local, fuso IANA, instante UTC, versão otimista, timestamps UTC e cancelamento opcional. Não copia mídia. Índice único parcial impede dois planejamentos ativos da mesma publicação. Foreign keys RESTRICT e trigger impedem exclusão física de planejamentos.

Estados implementados:

| Estado | Significado | Saídas permitidas |
| --- | --- | --- |
| planned_local | Intenção local vinculada à versão aprovada | Reagendar, cancelar ou invalidar |
| cancelled_local | Cancelamento explícito; registro retido | Novo planejamento cria outro registro |
| invalidated_local | Publicação mudou, perdeu aprovação ou foi excluída logicamente | Revisar/aprovar e planejar novamente explicitamente |

`schedule_settings` singleton guarda proximidade, intervalo sugerido e versão. `schedule_events` registra criação, reagendamento (data/hora/fuso anteriores e novos), cancelamento e invalidação, com versões e timestamps. Triggers de auditoria pertencem à mesma transação da alteração; UPDATE/DELETE do histórico são bloqueados.

Um trigger em `publications` invalida qualquer planejamento ativo na mesma transação que modifica versão, aprovação ou exclusão lógica. Falha de auditoria reverte também a mudança editorial. Invalidado não volta a ativo automaticamente. Reimportação Trello que só atualiza snapshot não invalida nem altera planejamento, conteúdo ou aprovação; regressão cobre esse contrato.

## Elegibilidade, concorrência e integridade

Rust valida ID, publicação existente/não excluída, estado approved, versão exata, composição editorial e integridade completa dos arquivos (hash e conteúdo, reutilizando revisão). Data/hora/fuso inválidos, instantes passados, duplicação e versões obsoletas são bloqueados. Reagendar repete a validação. Constraints/triggers reconferem aprovação/versão na gravação, protegendo a fronteira transacional. Cancelar altera apenas o planejamento ativo na versão esperada.

Listagens usam inspeção leve de caminho/tamanho, sinalizando mídia indisponível; criar/reagendar usam inspeção completa. Falhas não são tratadas como sucesso. Campos textuais/enum/IDs são validados; SQL é parametrizado. Nenhuma API SQL, shell ou filesystem genérica é exposta ao frontend. Os sete comandos específicos usam o worker bloqueante e o mutex do repositório existente.

## Datas, fuso e conflitos

Padrão America/Sao_Paulo, registrado em cada planejamento. A grade usa o fuso explícito das preferências do aplicativo; o formulário permite outro fuso IANA. Conversão é feita no Rust por Chrono 0.4.45 e chrono-tz 0.10.4, com TZDB incorporada. A resolução `.single()` rejeita horários ambíguos ou inexistentes em transições de horário de verão. Data/hora originais e UTC são preservados; mudança do fuso de visualização converte o instante, sem regravar o planejamento. O fuso do sistema/navegador não decide a programação silenciosamente.

Conflito exato: mesmo instante UTC. Proximidade: diferença maior que zero e estritamente menor que a janela configurada (0–180 minutos; padrão 30). Zero desabilita somente proximidade. Backend considera outros planejamentos ativos, inclusive fora do filtro e atravessando meia-noite; avisos não bloqueiam a gravação. Contadores mensais consideram somente ativos no mês/filtros exibidos. Cancelados ficam ocultos por padrão; inválidos são visíveis e não contam como prontos. Horários decorridos continuam identificados como locais, sem envio.

Referências primárias: [chrono-tz 0.10.4](https://docs.rs/chrono-tz/0.10.4/chrono_tz/) e [Chrono LocalResult](https://docs.rs/chrono/latest/chrono/offset/enum.LocalResult.html). Nenhuma dependência frontend nova; Cargo.lock fixa as versões.

## Arraste e acessibilidade

Arrastar aprovada para um dia abre confirmação de data/horário. Mover item existente preserva o horário no fuso de visualização. O destino recebe destaque; nenhuma persistência ocorre no drop. Payload interno tipado só é aceito para IDs já consultados na fila/grade. Há alternativa integral via botões, teclado e formulário.

Tentativas iniciais com HTML5 drag/drop não acionaram o formulário no Tauri Windows. A configuração `dragDropEnabled: false` evita a interceptação nativa de arquivos do Tauri (requisito documentado no schema da CLI instalada). Acrescentado arraste interno por Pointer Events com captura, limiar de oito pixels e validação do destino; não depende do transporte OLE do WebView2. HTML5 permanece como alternativa. Regressões cobrem drop sem gravação e confirmação. Na janela nativa, arrastar a publicação 18 da fila para 22/10 abriu o formulário; confirmar criou um único plano; arrastar para 23/10 preservou 10:00 e só gravou após confirmação.

## Validação automatizada

| Comando | Resultado |
| --- | --- |
| npm.cmd run check | Typecheck, lint sem warnings, 78 testes frontend, 6 testes SQL e build frontend aprovados |
| cargo test --manifest-path src-tauri/Cargo.toml | 68 aprovados, zero falhas; cinco testes opt-in ignorados |
| cargo check --manifest-path src-tauri/Cargo.toml | Aprovado |
| cargo fmt --manifest-path src-tauri/Cargo.toml --check | Aprovado |
| cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings | Aprovado |
| npm.cmd run build:desktop | Executável release final compilado, sem bundle/instalador |

Total: **152 testes aprovados**, cinco opt-in não executados. Cobertura nova: meses 28/29/30/31, virada de ano, calendário/fila/filtros, formulários e guarda, erro sem perda do rascunho, arraste, edição/cancelamento, configurações, Dashboard/histórico, aprovação/versões, passado, duplicação, fuso/DST, meia-noite, conflitos, invalidação editorial, rollback de auditoria, integridade/FKs, reimportação e reabertura. Testes frontend simulam somente a fronteira IPC; testes Rust/SQL usam SQLite real e arquivos temporários.

## Homologação Windows e preservação

QA debug exclusivo em `.tmp/phase06-qa-20261019`, criado de snapshot fechado do QA da fase anterior e cópias das sete mídias daquele QA. Banco operacional não foi aberto, apagado, recriado ou alterado nesta homologação. Dados de teste não entram no Git.

Tauri real executado por `npm.cmd run dev -- --window-size 1366x768` e `1600x900`, depois maximizado em monitor Full HD 1920×1080. Dimensões do script referem-se ao conteúdo; a moldura aumenta o tamanho externo. Grade legível, filtros compactos, diálogo com confirmação acessível por rolagem vertical, sem rolagem horizontal observada. Criação, reagendamento, arraste da fila/entre dias, navegação mensal, Hoje, lista, miniaturas, Dashboard e quatro eventos locais conferidos nativamente.

Publicação 2: criada em 20/10/2026 10:00 e reagendada para 21/10 11:00. Publicação 18: criada em 22/10 10:00 por arraste, movida para 23/10 10:00. Dois IDs de planejamento, ambos versão 2, quatro eventos. Fechamento normal e reabertura em 1600×900 recuperaram planos e contagens; consulta readonly confirmou schema 5, quick_check ok, zero violações de FK, cinco publicações e sete assets/vínculos exatamente iguais ao baseline e checksums dos sete arquivos preservados. Sem cópias de mídia por plano.

Cancelamento nativo autorizado explicitamente pelo usuário e concluído somente no plano QA da publicação 2 (21/10 11:00): mesmo ID, status cancelled_local, versão 3, timestamp de cancelamento e quinto evento. A publicação reapareceu na fila de disponíveis, mantendo aprovação/versão 6 e imagem. O plano da publicação 18 permaneceu ativo, versão 2. Novo fechamento/reabertura em 1366×768 confirmou Dashboard com duas aprovadas disponíveis, um plano ativo e zero conflitos; Histórico exibiu os cinco eventos, incluindo o cancelamento. Verificação readonly após o encerramento confirmou novamente cinco publicações/sete mídias e vínculos inalterados, todos os hashes, schema 5, quick_check ok e zero violações de FK. Aceite técnico da Fase 06 concluído para o escopo local.

## Git e auditoria

Branch main e origin `https://github.com/patrickbotarick/Social-Planner.git` conferidos. Status inicial limpo em 9afbe6a; fetch confirmou zero divergências. Auditoria dos 38 arquivos preparados: somente fontes/configuração, testes, lockfile e documentação; zero bancos, mídias privadas, builds, arquivos de credenciais, assinaturas de tokens ou URLs credenciadas inesperadas. Migrações 001–004 intactas e diff sem erro de whitespace. Artefatos QA/auditoria ficam em `.tmp/`, ignorada. Entrega com commit `feat: implement local editorial calendar` e push normal para origin/main autorizados pelo pedido após as validações; hash e resultado remoto constam no relatório final da execução. Sem force push ou tag.

## Limitações e próxima fase

Consulta mensal limitada a 10.000 registros para evitar resposta ilimitada; fila/histórico paginados. Sem recorrência, preenchimento automático, visão semanal complexa, serviço em segundo plano ou entrega com computador desligado. Alterações políticas futuras de fuso exigem atualizar TZDB e avaliar os instantes já armazenados explicitamente. Revalidação completa de vídeos pode ocupar temporariamente o mutex local. Sem novo teste Trello remoto nesta fase; regressões offline preservam contratos existentes.

Pesquisa oficial, permissões, autenticação e provas da Meta pertencem à Fase 07; envio, reconciliação e confirmação externa à Fase 08; instalador/recuperação/distribuição à Fase 09. A base técnica do calendário está validada e pronta para solicitar a Fase 07; nenhuma fase futura foi iniciada.
