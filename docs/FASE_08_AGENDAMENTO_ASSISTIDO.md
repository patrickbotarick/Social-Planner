# Fase 08 — Agendamento assistido pelo Meta Business Suite

Data: 19/10/2026. Implementação local solicitada após a decisão da Fase 07. Base Git inicial limpa: `main`, `e68cbda2c87bf89c3b4540f346b5f0cd535e226c`. O pedido desta fase autoriza commit e push normal após validação/auditoria. Sem tag e sem início da Fase 09.

## Entrega e fronteiras

Nova área **Preparar para Meta**: seleção individual/lote, filtros por mês/pesquisa/formato/estado/origem/quadro, elegibilidade com motivos, prévia ordenada, exportação, abertura da pasta, cópia fiel de legenda e abertura da URL fixa do Business Suite no navegador padrão. Registro explícito de declaração manual, revisão otimista, divergência de horário, reconciliação local e regeneração preservando pacotes anteriores. Dashboard e Histórico consultam registros reais; estados editorial, planejamento, preparação e declaração permanecem separados.

Não há API de publicação Meta, OAuth, token Meta, leitura de cookies, automação de navegador, upload, scheduler Windows, servidor remoto ou Supabase. Abrir o site e preparar o material não criam confirmação nem envio. A declaração não comprova agendamento ou publicação na plataforma. A homologação de agendamento real na GMAD está **pendente para a Fase 09**, mediante autorização explícita e execução manual do [roteiro operacional](GUIA_AGENDAMENTO_META.md).

## Persistência e estados

Migração aditiva **006_assisted_planning.sql**, preservando 001–005. Tabelas `export_roots`, `assisted_settings`, `preparations`, `assisted_confirmations` e `assisted_events`. Foreign keys RESTRICT, auditoria imutável e índices únicos parciais impedem múltiplos pacotes atuais por plano e declarações ativas contraditórias por plano/publicação. Snapshots de versões, hashes, formato e instantes são preservados; revisões substituem a declaração anterior sem apagá-la.

Preparação: ausência = `not_prepared`; `preparing` interno → `prepared` ou `preparation_failed`; alterações → `outdated`. Regenerar cria novo registro ligado ao anterior. Declaração: ausência = `not_confirmed`; confirmação explícita → `confirmed_manually`; alteração posterior → `requires_review`; revisão substitui a anterior por `superseded` e cria outro snapshot confirmado. Divergência de instante e exigência de revisão são sinalizadas separadamente da declaração manual.

A declaração inclui IDs, versões editorial/planejamento, assinatura, data/hora/fuso/UTC planejados e efetivos, Instagram, formato, referência/observação opcionais, estado, versão otimista e instante UTC da declaração. Mudanças editoriais/de calendário invalidam preparações e declarações na mesma transação dos contratos existentes. Reimportar somente a origem Trello não altera esses contratos. Histórico registra início, conclusão/falha, regeneração, desatualização, declaração, revisão e divergência, sem reescrever eventos editoriais antigos.

## Pacotes e integridade

```text
<raiz escolhida no diálogo nativo>/
  2026-10/
    2026-10-23_11-00_post-titulo_<UUID>/
      media_01.png
      media_02.mp4
      legenda.txt
      informacoes.json
```

Nomes Windows seguros e UUID evitam colisão. Mídias são copiadas em blocos, sem conversão, na ordem editorial; tamanho e SHA-256 são conferidos. Legenda UTF-8 mantém texto, emojis, Markdown e quebras. Manifesto com esquema 1 registra IDs/versões, assinatura, formato, calendário explícito e arquivos ordenados com hash/tamanho/MIME. Não inclui credenciais, URLs de origem ou caminhos originais. A legenda e mídias exportadas são conteúdo editorial e devem ficar em uma pasta com acesso apropriado.

Assinatura SHA-256 inclui plano/versão/estado, publicação/versão/título/legenda/formato e IDs/ordem/hashes/tipos/tamanhos das mídias. Consulta e ações de transferência verificam os arquivos e o manifesto; conteúdo modificado ou incompleto deixa de ser atual. Nova preparação do mesmo snapshot é idempotente. Alterar raiz preferida não move pacotes existentes.

Staging `.tmp-<UUID>` na mesma pasta mensal do destino, arquivos `create_new`, sincronização de arquivos e promoção por rename. Pré-checagem de espaço livre Windows com reserva de 1 MiB; erros de disco cheio/escrita são tratados. Limpeza limita-se ao staging conhecido e sem links. Falha/interrupção nunca apaga original ou pacote final. SQLite e filesystem não formam uma transação única: preparação é persistida antes da cópia e concluída depois; ao reabrir, `preparing` pendente é recuperado como falha, com pacote final eventualmente existente preservado. Renomeação atômica no mesmo volume não equivale a garantia contra perda de energia de todo o conjunto SQLite/arquivos.

## Segurança e validação

Nove comandos IPC específicos, permissões próprias e CSP preservada. Rust controla SQLite, seletor, cópia e validação; frontend passa IDs/versões, sem SQL, shell ou paths arbitrários. Raízes são escolhidas no seletor nativo, canônicas, existentes, em unidade local; rejeição de UNC, traversal, componentes inesperados e links/reparse points nos destinos verificados. Banco, pasta de mídias e staging interno não são destinos de exportação. Caminho de raiz limitado a 180 bytes UTF-8. Não há garantia contra um processo malicioso do mesmo usuário trocando diretórios durante a operação.

Abertura usa `ShellExecuteW` na thread principal: somente pasta resolvida de preparação atual ou `https://business.facebook.com/` fixa. Nenhum comando genérico de abertura é exposto. Referências externas não são consultadas; URLs aceitas exigem HTTPS e host exato instagram.com/www.instagram.com/facebook.com/www.facebook.com/business.facebook.com, sem credenciais, query ou fragmento. Referência até 500 caracteres, observação até 2.000; sem NUL. Datas/fuso/DST são validados pelo contrato Chrono existente.

Elegibilidade exige publicação aprovada na versão exata, planejamento ativo futuro e composição/arquivos íntegros. O fluxo adota limite conservador de legenda 2.200 caracteres e carrossel dez itens; não afirma que sejam limites universais do Business Suite. PNG/WebP e vídeos exibem avisos para conferência/conversão manual quando necessária. Codec/áudio/duração/enquadramento e aceitação efetiva na Meta não são garantidos pela validação básica. Conteúdo vencido não pode ser novamente preparado/confirmado pelo fluxo atual; fechamento retrospectivo não foi implementado.

## Testes automatizados e builds

| Validação | Comando | Resultado |
| --- | --- | --- |
| TypeScript, ESLint, Vitest, SQL, frontend | `npm.cmd run check` | Aprovado: 89 frontend e 7 SQL, zero falhas |
| Rust | `cargo test --manifest-path src-tauri/Cargo.toml --quiet` | 80 aprovados; cinco opt-in ignorados |
| Compilação Rust | `cargo check --manifest-path src-tauri/Cargo.toml` | Aprovado |
| Formatação | `cargo fmt --manifest-path src-tauri/Cargo.toml --check` | Aprovado |
| Clippy | `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings` | Aprovado |
| Desktop release | `npm.cmd run build:desktop` | Aprovado, executável sem bundle |

**176 testes aprovados**. Adicionados 11 frontend, 12 Rust e um SQL; a suíte anterior foi executada. Cinco testes opt-in de integração/fixtures continuam não executados nesta fase; não são contados como aprovados. Falta de espaço e falha de escrita são injetadas em testes Rust, não provocadas enchendo o disco do usuário. Testes de UI simulam a fronteira IPC; testes Rust/SQL usam banco e disco reais temporários.

Cobertura: preparação individual/lote com resultados parciais, carrossel ordenado/vídeo, emojis/UTF-8, mídias ausentes/modificadas, destinos inválidos/traversal/colisão, escrita/espaço, staging interrompido, regeneração/idempotência, alteração de conteúdo/ordem/data/hora, checkbox, declaração repetida, revisão otimista, reconciliação, preservação de snapshots, histórico, Dashboard, migração, rollback e reabertura. Um novo plano após editar/reaprovar também revisa a declaração anterior sem duplicação ativa por publicação. Nenhum teste exige chamada Meta.

## Evidências Windows

QA debug isolado em `.tmp/phase08-qa-20261019`, derivado de um perfil QA anterior fechado, com cinco publicações e sete mídias verificadas por hash. Dois planos adicionais foram criados somente nessa cópia, resultando em três planos ativos para Feed, Reels e carrossel misto. O banco operacional não foi aberto ou modificado pelos testes.

Na janela Tauri real, dimensões de conteúdo solicitadas 1366×768 e maximização no monitor Full HD: seletor nativo da raiz, preparação individual do carrossel imagem/vídeo, lote com três itens (reuso do primeiro pacote), pasta aberta no Explorador com quatro arquivos esperados, cópia/colagem da legenda no formulário QA e abertura do Business Suite no navegador padrão conferidas. Nenhum controle do navegador Meta foi automatizado. Pesquisa por vídeo reduziu a listagem/contagens a um item; prévia MP4 reproduziu no WebView2. Filtros recolhíveis mantêm a tela compacta e formulário mantém rodapé acessível por rolagem/teclado.

Declaração QA explicitamente identificada como simulação: plano 10:00, efetivo 11:00 no mesmo fuso → uma declaração/uma divergência, sem alterar o calendário. Reconciliação explícita → plano v3 às 11:00, pacote desatualizado e declaração exigindo revisão. Regeneração preservou a pasta antiga; nova declaração explícita manteve apenas uma ativa e resolveu a divergência. Conferidos quatro pacotes físicos (três originais e uma regeneração), com duas declarações históricas, uma substituída.

Reabertura do mesmo QA aprovada: Dashboard manteve cinco publicações, três preparadas, uma confirmada manualmente e nenhuma divergência; Histórico apresentou os 15 eventos assistidos. Inspeção somente leitura comparou integralmente as cinco publicações, sete assets e sete vínculos ao baseline: IDs, versões, estados, legendas e ordem preservados. Todos os sete arquivos internos mantiveram SHA-256; os seis arquivos de mídia exportados nos quatro pacotes (incluindo regeneração) totalizaram 57.048.518 bytes e coincidiram com os originais. Legendas/manifestos conferidos. Três pacotes atuais, um antigo preservado, duas declarações históricas/uma ativa, zero staging pendente, schema 6, quick_check=ok e foreign_key_check vazio. Evidência local ignorada pelo Git: `.tmp/phase08-verification.json`. Nenhum conteúdo foi agendado na conta operacional.

## Limitações e aceite

Validação local não comprova conta/permissões/aceitação ou execução da Meta. Homologação real permanece pendente, com roteiro no guia, para eventual Fase 09 solicitada separadamente. Instalador, atualização, backup/restauração e testes em Windows limpo continuam fora desta fase. Não há polling de plataforma ou estado “publicado” automático.

Preparações e inspeções completas fazem leitura de hashes no worker Rust sob o mutex editorial; vídeos grandes e meses volumosos podem levar tempo e bloquear outras consultas locais. Lote aceita até 50 alvos; UI seleciona os elegíveis da página de 20; consulta mensal herda limite de 10.000 itens e histórico usa páginas de 30. Use uma instância do aplicativo por perfil; recuperação de operações interrompidas não é um protocolo multi-instância de longa duração.

Auditoria pré-commit: 46 arquivos de fonte, testes, permissões e documentação; nenhum banco, mídia, build, arquivo binário ou segredo reconhecido nos padrões de chaves/tokens revisados. Migrações 001–005 intactas; links locais dos documentos conferidos; sem novas permissões genéricas de filesystem/shell/SQL/HTTP. CSP e lockfiles preservados. QA, pacotes e evidências privadas permanecem em `.tmp`, fora do Git. Inspeção por padrões complementa a revisão do diff e não substitui uma garantia matemática de ausência de segredos.

Aceite técnico do escopo assistido aprovado: persistência, integridade, preservação, suíte e builds confirmados. Commit desta entrega: `feat: implement assisted Meta scheduling workflow`; hash final deve ser consultado no histórico Git/relatório de entrega para evitar autorreferência dentro do commit. Push normal solicitado pelo usuário, sujeito à auditoria final. Aceite operacional de agendamento real não concedido por esta execução. A base está pronta para uma Fase 09 solicitada separadamente, incluindo essa pendência.
