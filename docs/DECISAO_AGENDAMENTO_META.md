# Decisão de agendamento Meta — ADR da Fase 07

Data: 19/10/2026. **Recomendação técnica proposta ao usuário; implementação da Fase 08 não autorizada por este documento.** Fontes e limites em [Viabilidade](FASE_07_VIABILIDADE_META.md); contratos em [Integração Meta](INTEGRACAO_META.md).

## Problema e restrições

O calendário já registra intenção local aprovada. O requisito ideal é preparar o mês e desligar o PC depois de confirmar o agendamento, ficando a execução com a Meta. O produto deve continuar Windows/local-first, sem login próprio, backend remoto obrigatório, Supabase/Firebase, hospedagem própria inicial ou publicações sem decisão editorial. Somente Trello e Meta são conexões externas previstas.

Os contratos públicos de publicação Instagram pesquisados não oferecem agendamento futuro remoto. Criar contêiner não agenda: é preciso publicar e há expiração. OAuth com App Secret e distribuição de imagens por URL pública também introduzem dependências que o desktop sozinho não resolve com segurança. O token administrativo do painel é alternativa oficial para conta própria; não remove as limitações de mídia/agendamento. [S03–S10]

## Comparação dos caminhos

| Opção | Viabilidade e experiência | PC/internet | Custo e complexidade | Segurança, manutenção e local-first |
| --- | --- | --- | --- | --- |
| A — API oficial imediata | Viável com conta, token, transporte e mídia compatíveis; operador confirma envio naquele momento | PC/processo e internet durante preparação/envio/reconciliação; após publicação comprovada não depende do PC | Sem preço de API quantificado nesta pesquisa; engenharia moderada/alta, possíveis custos de armazenamento público/OAuth servidor | Token no cofre; segredo fora do cliente; versões, quotas e erros exigem manutenção. Local-first parcial, dependendo do transporte |
| B — Executor Windows no horário | Pode chamar A no instante local; não é agendamento remoto Meta | PC ligado, processo ativo, sessão/energia e internet; suspensão/desligamento impedem execução. Task Scheduler não garante despertar nem internet | Sem novo servidor, mas energia, suporte e scheduler/recovery elevam complexidade | Atrasos e falhas precisam de reconciliação; nunca enviar vencidos automaticamente. Não atende o ideal do PC desligado |
| C — Preparação local + Business Suite manual | Conteúdo/arquivos preparados no GMAD; operador escolhe conta, revisa e programa na interface Meta | PC e internet na preparação/transferência; depois do agendamento efetivo na Meta, GMAD pode estar fechado | Sem hospedagem/servidor próprio novo; custo principal é tempo do operador e internet. Produto oficial com experiência gratuita básica; recursos pagos/regionais não são requisito avaliado [S14/S18] | Melhor aderência às restrições; não armazena credenciais Meta no GMAD. Upload/cópia e conferência são manuais. Interface Meta pode mudar |
| D — Agendamento remoto oficial Instagram por API | Não suportado no contrato público consultado; não há arquitetura implementável comprovada | Independência do PC não demonstrada por API | Não estimar custo ou prazo de recurso não comprovado | Não inventar endpoint nem usar sessões/API privada. Reavaliar se a Meta publicar contrato oficial |
| E — Serviço remoto próprio | Tecnicamente poderia executar A no horário; serviço, não a Meta, assume a fila | PC pode desligar após transferir tudo; servidor e internet precisam operar | Hospedagem, mídia/egresso, monitoramento, patching, segurança e recuperação; sem orçamento nesta fase | Rompe a restrição de backend remoto obrigatório e amplia conexões. Apenas alternativa documental, não recomendada para o escopo inicial |

“Sem custo de hospedagem própria” não significa custo operacional zero ou promessa de gratuidade permanente da plataforma. Não se propõem SaaS, novos provedores, anúncios pagos ou assinaturas para atender o escopo inicial.

## Decisão recomendada

**Adotar C como primeira entrega da Fase 08, se o usuário aceitar a etapa manual.** Manter o GMAD como fonte editorial local e o Business Suite como interface em que o operador efetiva o agendamento. O contrato oficial documenta criação e programação nessa interface; a independência do processo GMAD decorre dessa transferência manual, não de chamada API do GMAD. [S14]

O usuário escolherá se aceita esse fluxo antes da implementação. A pesquisa não contém aceite dessa recomendação nem homologação da conta real. Se a exigência for “um clique no GMAD programa o mês na Meta e depois posso desligar o PC”, a combinação de requisitos **não está atendida pela API pública pesquisada**. Explicitar o trade-off; não alterar silenciosamente para B ou E.

A continua alternativa posterior para “Publicar agora”, caso haja autorização e decisões sobre token/transporte. Instagram Login tende a simplificar a descoberta da conta, mas não deve ser selecionado automaticamente se o requisito de upload de vídeo binário exigir Facebook Login. Priorizar um caminho por vez; não construir dois OAuth nesta fase.

## O que “confirmado” deve significar

| Situação | Rótulo futuro permitido | Evidência e limite |
| --- | --- | --- |
| Schedule salvo | Planejado localmente | SQLite; nenhuma ação Meta |
| Pacote exportado | Preparado localmente | Arquivos/manifesto; nenhuma ação Meta |
| Operador declara que programou na Meta | Programação informada pelo operador | Horário/fuso, conta e referência opcionais; não verificada por API |
| Container FINISHED | Pronto para publicar | Processamento concluído; não publicado |
| API retorna sucesso e objeto reconciliado | Publicação confirmada pela API | ID/estado/consulta associados à tentativa correta |
| Resposta perdida | Resultado incerto | Bloquear repetição cega até reconciliação |

Alterar/cancelar o plano local não reprograma/cancela conteúdo no Business Suite. O usuário deve realizar a ação também na Meta e registrar a reconciliação. Nenhum registro manual deve preencher uma confirmação de API. A simples abertura de Business Suite não é agendamento; concluir upload e clicar Programar é ação externa posterior, com autorização própria.

## Plano de implementação proposto para a Fase 08

1. Confirmar escolha C, conta Business/Creator, acesso de conteúdo no Business Suite, fuso e formatos realmente disponíveis. Fazer ensaio operacional isolado somente após autorização explícita para cada efeito externo necessário.
2. Definir pacote/manifesto local imutável por versão: publicationId, scheduleId/versões, conta escolhida, data/hora/fuso/UTC, legenda, ordem e hashes dos arquivos. Revalidar aprovação e mídia antes de exportar.
3. Pré-validar restrições do destino; solicitar edição consciente de legendas longas/carrosséis grandes e prévia das derivadas de imagem/vídeo. Preservar originais; não cortar, comprimir ou publicar silenciosamente.
4. Oferecer pasta/pacote para seleção manual e cópia explícita da legenda; orientar a interface oficial, sem DOM automation, scraping de sessão, preenchimento de credenciais ou chamada privada.
5. Criar registro separado de acompanhamento assistido com origem manual/evidência e aviso de divergência após mudanças locais. Migração aditiva e auditoria próprias; manter calendários/estados da Fase 06.
6. Testar exportação, versão desatualizada, integridade, falha parcial, cancelamento local sem efeito remoto, persistência, acessibilidade e clareza dos rótulos. Confirmar na Meta a programação de teste antes de investigar comportamento com GMAD fechado; não simular esse resultado.
7. Se A for solicitada, abrir etapa específica: escolher login, obter token administrativamente ou resolver OAuth servidor/callback, autorizar transporte de imagens, implementar adaptador Rust e reconciliação; começar com leituras consentidas antes de qualquer upload/publicação.

## Pendências e critérios de avanço

- Decisão humana sobre C e aceitação do trabalho manual; se rejeitada, renegociar dependência do PC ou servidor.
- Tipo/identidade da conta GMAD, Página vinculada, papéis e capacidade real do Business Suite ainda não verificados.
- API opcional exige decidir token seguro, callback/segredo e distribuição de mídia; não solicitar senhas ou tokens no chat.
- Quotas oficiais inconsistentes (100/50): revalidar resposta real quando leitura autenticada for autorizada, antes de fixar limites.
- Nenhuma publicação de teste, OAuth real, criação de app ou agendamento está autorizado implicitamente por esta pesquisa.

A Fase 07 encerra a pesquisa documental e a recomendação. A Fase 08 permanece não iniciada; sua implementação depende de solicitação e escolha do caminho.
