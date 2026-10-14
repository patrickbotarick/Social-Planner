# Roadmap oficial

Fonte normativa: [Projeto mestre](PROJETO_MESTRE.md). Fases 01–03 concluídas tecnicamente e validadas neste Windows; Fase 04 implementada e validada com mocks/desktop local, com validação externa pendente; fases 05–09 **não iniciadas**. Evidências nos relatórios das fases.

Validação complementar: [Fase 01.5 — Windows](FASE_01_5_VALIDACAO_WINDOWS.md). Em 13/10/2026, os pré-requisitos foram instalados com autorização e os testes, builds, janela nativa, navegação, IPC, SQLite e persistência após reabertura foram comprovados. A Fase 01.5 é uma etapa de aceite da fundação, sem alterar o roadmap de nove fases.

| Fase | Objetivo | Escopo | Dependências | Conclusão verificável | Estado atual |
| --- | --- | --- | --- | --- | --- |
| 01 — Fundação e documentação | Base local testável | Mestre, arquitetura, React/Tauri, navegação, SQLite mínimo, testes, Git | Node 24; Rust MSVC; C++ Build Tools/SDK; WebView2 | Documentação consistente, janela inicia, navegação e IPC funcionam, banco inicializa e persiste, verificações registradas | Concluída tecnicamente; validada no Windows na Fase 01.5 |
| 02 — Identidade visual e interface | Consolidar experiência GMAD | Tokens definitivos, componentes, estados e acessibilidade | Aceite 01 e referências de identidade | Interface aprovada e consistente em resoluções suportadas | Implementada e validada tecnicamente no Windows; evidências disponíveis para apreciação visual do usuário |
| 03 — Biblioteca editorial | Gerir conteúdo local real | Publicações manuais, mídias, busca/listagem, edição e migrações editoriais | 02 | Conteúdo criado/editado persiste; integridade e arquivos validados | Concluída tecnicamente em 14/10/2026; 53 testes, builds, fluxos Tauri e reabertura aprovados |
| 04 — Integração Trello | Receber conteúdo da matriz | Credenciais seguras, quadro por URL, cartões, anexos, importação incremental | 03 e autorização Trello | Reimportação sem duplicação ou perda de edições; falhas recuperáveis | Implementada; testes e desktop local validados; conta/quadro reais pendentes de autorização |
| 05 — Revisão e aprovação | Selecionar conteúdo da loja | Aprovar, ignorar, editar, validar prontidão, histórico por versão | 03 e 04 | Transições e reversões explícitas testadas; nenhuma aprovação de conteúdo desatualizado | Não iniciada |
| 06 — Calendário mensal | Planejamento local | Visão mensal, horário/fuso, replanejamento, conflitos | 05 | Distribuição mensal persistente com distinção explícita de intenção local | Não iniciada |
| 07 — Pesquisa e integração Meta | Validar caminho oficial | Pesquisa documentada, autorização, permissões, prova técnica ou fluxo assistido | 06, ativos profissionais e autorização externa | Evidência oficial do comportamento, limites e dependência do PC; decisão arquitetural registrada | Não iniciada |
| 08 — Agendamento e acompanhamento | Executar o caminho validado | Envio ou assistência, reconciliação, retentativas, histórico de plataforma | 07 | Confirmação/publicação comprovada; falhas e resultados incertos tratados sem duplicidade | Não iniciada |
| 09 — Testes e distribuição | Entregar operação Windows confiável | Regressão completa, instalação/atualização, recuperação e backup, documentação de uso | 01–08 | Instalação em Windows limpo e recuperação de dados testadas; critérios de release atendidos | Não iniciada |

## Portão entre fases

Registrar escopo realizado, verificações realmente executadas e pendências. Nenhuma fase seguinte inicia automaticamente. Nesta Fase 01, build/testes Rust e janela Tauri com SQLite/IPC reais foram verificados, além dos testes SQL auxiliares. Os limites específicos da inspeção estão registrados no relatório da Fase 01.5.

A Fase 02 foi implementada em 13/10/2026. A Fase 03 foi validada em 14/10/2026: 53 testes aprovados, verificações e builds sem falhas finais, cadastro de imagem/vídeo/carrossel, edição, duplicação, exclusão lógica, pesquisa/filtros, contagens e persistência na janela Tauri. QA ficou isolado; dados operacionais preservados. Detalhes e limites em [Fase 03](FASE_03_BIBLIOTECA_EDITORIAL.md). A Fase 04 foi solicitada e implementada em 14/10/2026; veja [Fase 04](FASE_04_INTEGRACAO_TRELLO.md). A Fase 05 tem base técnica preparada, mas não foi iniciada; o aceite externo do Trello permanece pendente. A apreciação estética final do usuário não deve ser confundida com aprovação já fornecida.
