# Roadmap oficial

Fonte normativa: [Projeto mestre](PROJETO_MESTRE.md). Todas as fases além da 01 estão **não iniciadas**. Concluir código da fundação não equivale a validar sua execução nativa; acompanhar evidências em [Fase 01](FASE_01_FUNDACAO.md).

| Fase | Objetivo | Escopo | Dependências | Conclusão verificável | Estado atual |
| --- | --- | --- | --- | --- | --- |
| 01 — Fundação e documentação | Base local testável | Mestre, arquitetura, React/Tauri, navegação, SQLite mínimo, testes, Git | Node 24; Rust MSVC; C++ Build Tools/SDK; WebView2 | Documentação consistente, janela inicia, navegação e IPC funcionam, banco inicializa e persiste, verificações registradas | Implementada; aceite desktop pendente por pré-requisitos ausentes |
| 02 — Identidade visual e interface | Consolidar experiência GMAD | Tokens definitivos, componentes, estados e acessibilidade | Aceite 01 e referências de identidade | Interface aprovada e consistente em resoluções suportadas | Não iniciada |
| 03 — Biblioteca editorial | Gerir conteúdo local real | Publicações manuais, mídias, busca/listagem, edição e migrações editoriais | 02 | Conteúdo criado/editado persiste; integridade e arquivos validados | Não iniciada |
| 04 — Integração Trello | Receber conteúdo da matriz | Credenciais seguras, quadro por URL, cartões, anexos, importação incremental | 03 e autorização Trello | Reimportação sem duplicação ou perda de edições; falhas recuperáveis | Não iniciada |
| 05 — Revisão e aprovação | Selecionar conteúdo da loja | Aprovar, ignorar, editar, validar prontidão, histórico por versão | 03 e 04 | Transições e reversões explícitas testadas; nenhuma aprovação de conteúdo desatualizado | Não iniciada |
| 06 — Calendário mensal | Planejamento local | Visão mensal, horário/fuso, replanejamento, conflitos | 05 | Distribuição mensal persistente com distinção explícita de intenção local | Não iniciada |
| 07 — Pesquisa e integração Meta | Validar caminho oficial | Pesquisa documentada, autorização, permissões, prova técnica ou fluxo assistido | 06, ativos profissionais e autorização externa | Evidência oficial do comportamento, limites e dependência do PC; decisão arquitetural registrada | Não iniciada |
| 08 — Agendamento e acompanhamento | Executar o caminho validado | Envio ou assistência, reconciliação, retentativas, histórico de plataforma | 07 | Confirmação/publicação comprovada; falhas e resultados incertos tratados sem duplicidade | Não iniciada |
| 09 — Testes e distribuição | Entregar operação Windows confiável | Regressão completa, instalação/atualização, recuperação e backup, documentação de uso | 01–08 | Instalação em Windows limpo e recuperação de dados testadas; critérios de release atendidos | Não iniciada |

## Portão entre fases

Registrar escopo realizado, verificações realmente executadas e pendências. Nenhuma fase seguinte inicia automaticamente. Na Fase 01 deste ambiente, ainda faltam build/testes Rust e observação da janela Tauri com SQLite/IPC reais. Os testes SQL via Node complementam, mas não substituem essa comprovação.

A Fase 02 está planejada e tem a estrutura técnica preparada para evolução; seu início depende de solicitação do usuário e resolução/aceite explícito da pendência desktop da Fase 01.
