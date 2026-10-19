# Fase 07 — Viabilidade da integração Meta

Consulta em **19/10/2026**, sobre main `debb09b4554df2adfc331192a0ab2e30f90cb9e3`, inicialmente limpa e sem divergência com origin/main. Pesquisa e desenho documental concluídos. **Integração não implementada nem homologada na conta GMAD. Fase 08 não iniciada.**

## Resultado da investigação

Publicação imediata por API oficial é viável para contas profissionais, sujeita a autorização, mídia compatível e disponibilidade de transporte. **Não foi localizado endpoint público documentado que entregue à Meta um horário futuro de publicação Instagram.** As referências consultadas descrevem criação de contêiner, processamento e media_publish; não descrevem um contrato de agendamento remoto Instagram. Esta conclusão é uma inferência do conjunto público consultado, não uma afirmação sobre APIs privadas ou acordos com parceiros. [S03, S04, S05]

Um contêiner pronto não é um agendamento: expira sem publicação e ainda exige a chamada de publicação. O parâmetro de agendamento de Página Facebook não deve ser transferido para Instagram. Filas de terceiros ou um processo Windows chamando a API no horário são responsáveis por executar essa chamada; não tornam a execução independente do respectivo computador/servidor.

**Recomendação:** na Fase 08, preparar conteúdo localmente e orientar agendamento manual no Meta Business Suite. Depois que o operador efetivamente programa o conteúdo na interface da Meta, o GMAD não precisa ficar ligado para dispará-lo. A programação e suas falhas passam a depender da Meta. Essa independência é uma conclusão arquitetural do fluxo manual, ainda sem ensaio operacional autorizado na conta da loja. Não prometer transferência automática do calendário para o Business Suite. [S14]

## Evidência, versões e fontes

Somente fontes primárias sustentam os resultados. O leitor web retornou 429/indisponibilidade em páginas Meta; GET HTTPS público direto por PowerShell recuperou o conteúdo, sem cookies de sessão fornecidos nem credenciais. HTML e extrações temporárias ficaram em `.tmp/`, fora do Git. HTTP 200 foi acompanhado de inspeção do conteúdo, não tratado isoladamente como validação.

Graph API **v26.0**, publicada em 29/07/2026, aparece na tabela oficial e nos exemplos atuais. É a versão de referência desta análise, não uma versão chamada contra uma conta real. A referência de mídia/media_publish consultada indica atualização em 28/09/2026; o guia de publicação, em 30/06/2026. Revalidar versão e mudanças antes da implementação. [S01]

| ID | Fonte oficial consultada | Evidência principal |
| --- | --- | --- |
| S01 | [Versões Graph API](https://developers.facebook.com/docs/graph-api/changelog/versions/) | v26.0 e data de lançamento |
| S02 | [Visão geral Instagram Platform](https://developers.facebook.com/documentation/instagram-platform/overview) | Dois fluxos, ativos, tokens, acesso e papéis |
| S03 | [Content Publishing](https://developers.facebook.com/docs/instagram-platform/instagram-api-with-instagram-login/content-publishing/) | Processo, transporte, carrossel e limite geral |
| S04 | [IG User Media](https://developers.facebook.com/documentation/instagram-platform/instagram-graph-api/reference/ig-user/media) | Parâmetros e especificações de mídia; upload retomável |
| S05 | [IG User media_publish](https://developers.facebook.com/documentation/instagram-platform/instagram-graph-api/reference/ig-user/media_publish) | Publicação do contêiner e resultado |
| S06 | [IG Container](https://developers.facebook.com/documentation/instagram-platform/instagram-graph-api/reference/ig-container) | Estados de processamento/publicação e expiração |
| S07 | [Content Publishing Limit](https://developers.facebook.com/documentation/instagram-platform/instagram-graph-api/reference/ig-user/content_publishing_limit) | Uso e configuração de quota |
| S08 | [Instagram Login](https://developers.facebook.com/documentation/instagram-platform/instagram-api-with-instagram-login) | Conta profissional sem Página obrigatória |
| S09 | [Business Login for Instagram](https://developers.facebook.com/documentation/instagram-platform/instagram-api-with-instagram-login/business-login) | OAuth, segredo, troca e renovação |
| S10 | [Primeira chamada com Instagram Login](https://developers.facebook.com/documentation/instagram-platform/instagram-api-with-instagram-login/get-started) | Token administrativo do painel e consulta de identidade |
| S11 | [Facebook Login — Instagram](https://developers.facebook.com/documentation/instagram-platform/instagram-api-with-facebook-login) e [Get Started](https://developers.facebook.com/documentation/instagram-platform/instagram-api-with-facebook-login/get-started) | Página vinculada e descoberta de ativos |
| S12 | [App Review Instagram](https://developers.facebook.com/documentation/instagram-platform/app-review) | Standard/Advanced Access por cenário |
| S13 | [Criar app Instagram](https://developers.facebook.com/documentation/instagram-platform/create-an-instagram-app) | App Business, produto e configuração |
| S14 | [Criar e gerenciar posts no Business Suite](https://www.facebook.com/business/help/942827662903020) | Programar manualmente e gerenciar conteúdo programado |
| S15 | [Facebook Login manual](https://developers.facebook.com/docs/facebook-login/guides/advanced/manual-flow/) | Redirect, state e particularidades desktop |
| S16 | [Tokens Facebook de longa duração](https://developers.facebook.com/docs/facebook-login/guides/access-tokens/get-long-lived/) | Troca no servidor e expiração/revogação |
| S17 | [Meta oficial no Postman](https://www.postman.com/meta/) e [Instagram workspace](https://www.postman.com/meta/instagram) | Procedência oficial; coleção indexada corroborou fluxos, mas o leitor direto exibiu parcialmente o shell dinâmico |
| S18 | [Meta Business Suite — Meta Platforms no Google Play](https://play.google.com/store/apps/details?id=com.facebook.pages.app) | Aplicativo oficial e recursos de programação; não validação da conta GMAD |

As páginas gerais solicitadas de [Instagram Platform](https://developers.facebook.com/docs/instagram-platform/), [Content Publishing](https://developers.facebook.com/docs/instagram-platform/content-publishing/) e [Facebook Login](https://developers.facebook.com/docs/facebook-login/) foram tentadas; as referências específicas acima forneceram os conteúdos utilizados. Uma tentativa inicial de create-app devolveu página genérica; substituída pelo link oficial create-an-instagram-app encontrado na navegação. O Help Center de Instagram solicitado durante a pesquisa não entregou corpo legível; não sustenta conclusões. Nenhum bloqueio foi contornado com login, sessão privada ou API interna.

## Matriz de capacidades

Classificação refere-se à API pública/documentação, não a funcionalidades já disponíveis no GMAD. Restrição de acesso, formato e transporte continua aplicável a cada linha.

| Capacidade | Classificação | Restrições / fonte |
| --- | --- | --- |
| Feed, imagem única | Confirmada com restrições | Conta profissional; JPEG e URL acessível pela Meta. PNG/WebP locais não são payload pronto. S03/S04 |
| Carrossel | Confirmada com restrições | Até 10 itens imagem/vídeo, filhos + contêiner pai; ordem e recorte devem ser conferidos. GMAD aceita até 20 localmente. S03/S04 |
| Reels | Confirmada com restrições | Especificações de vídeo/áudio e processamento; caminho retomável documentado para Facebook Login for Business. S03/S04 |
| Stories | Confirmada com restrições | Referência Facebook Login restringe a conta Business; não presumir Creator elegível nem recursos de stickers. Elegibilidade do outro fluxo precisa ser confirmada na conta. S11/S04 |
| Vídeos | Confirmada com restrições | Rotas REELS/STORIES ou item de carrossel; não prometer vídeo avulso de feed legado como recurso independente. S04 |
| Legendas | Confirmada com restrições | Texto limitado e sem equivalência ao Markdown local; legenda do pai no carrossel. Capacidade varia por tipo; não prometer caption em Stories. S04 |
| Publicação imediata | Confirmada oficialmente | Fluxo documentado /media → processamento → /media_publish. Não executado nesta fase. S03/S05 |
| Agendamento remoto Instagram por API | Não suportada no contrato público consultado | Nenhum parâmetro/endpoint futuro identificado; conclusão delimitada ao conjunto pesquisado. S03/S04/S05 |
| Cancelamento remoto desse agendamento | Não suportada no contrato público consultado | Não existe contrato de agendamento identificado a cancelar; cancelar Schedule local não altera Meta. S04/S05/S06 |
| Verificação de publicação | Confirmada com restrições | Status do contêiner, ID retornado e consulta da mídia; FINISHED não significa publicado. S05/S06 |
| Recuperação de erros | Confirmada com restrições | Estados/erros consultáveis; deduplicação e reconciliação são responsabilidade do cliente, sem garantia genérica de exactly-once. S05/S06 |
| Conta, permissões e codecs GMAD reais | Não verificada | Não houve OAuth, token, leitura de conta, upload ou publicação |

## Pré-requisitos e impedimentos encontrados

Conta profissional Business/Creator; para Stories considerar Business. Facebook Login exige Página vinculada, usuário com tarefa adequada e descoberta do ID Instagram; Instagram Login dispensa Página. Propriedade/gestão e inclusão da conta no app determinam possibilidade de Standard Access sem App Review; atendimento a terceiros exige Advanced Access e revisão. A conta real, seu tipo, vínculos, papéis, restrições/PPA/2FA e eventual app existente não foram inspecionados. [S08–S13]

Duas limitações impedem assumir uma integração totalmente local como resolvida:

1. OAuth Instagram usa App Secret em trocas que a documentação manda realizar no servidor, sem expor em dispositivo/binário. Rust compilado ou Credential Manager não transforma um segredo compartilhado de app em segredo seguro de servidor. Há alternativa administrativa documentada: token de longa duração gerado manualmente no painel para a própria conta; isso não equivale a implementar OAuth desktop. [S09/S10]
2. Imagens pela API precisam de URL publicamente acessível. Caminho Windows, asset://, localhost e anexo Trello autenticado não servem como origem pública. Não foi comprovado upload binário de JPEG por esses endpoints. Upload retomável de vídeo é uma exceção documentada, não solução para todas as imagens. Hospedagem/CDN ou relay ampliaria as dependências e conexões hoje permitidas. [S03/S04]

Inconsistências oficiais registradas: guia geral cita 100 publicações/24h, enquanto sua seção de carrossel, media_publish e referência de quota ainda citam 50. Antes de enviar, futura implementação deverá consultar quota/configuração e confirmar os campos retornados; não fixar promessa de 100. Outra divergência é nomenclatura de token Facebook (Page no guia; User em referências): validar por endpoint/fluxo, sem intercambiar tokens automaticamente. App Review contém grafia divergente de permissão em uma tabela; usar os escopos do guia de login. [S03/S05/S07/S09/S12]

## Prova técnica e validação desta fase

Realizadas: leitura dos contratos locais; consulta pública HTTPS de documentos oficiais; comparação das referências de criação/publicação/status; inspeção de versão e parâmetros; matriz de compatibilidade entre armazenamento atual e payload externo; revisão dos contratos conceituais e links. Ausência de campos de horário nas referências foi conferida por leitura e busca textual, não por tentar POST inválido.

Não realizadas: login real, criação de app, leitura de conta autenticada, geração/importação de token, configuração de cofre, webhook, upload, criação de contêiner, media_publish ou agendamento. Não foram introduzidos clientes HTTP, fixtures executáveis, migrações, dependências ou código de produto. A prova é **documental**, não um teste ponta a ponta ou prova de publicação.

Validação documental concluída: oito arquivos Markdown alterados/criados, 70 referências locais resolvidas, 27 URLs externas de fontes oficiais identificadas no conjunto (incluindo referências preexistentes), 17 corpos oficiais recuperados e inspecionados com hashes de evidência em `.tmp/phase07-validation.json`. Conferida ausência de campos de agendamento futuro nas três referências centrais e presença das restrições de segredo/token e do fluxo manual. Essa busca negativa tem o limite das referências consultadas. Auditoria documental sem assinaturas de credenciais ou URLs credenciadas inesperadas; diff sem erros de whitespace. Testes/builds de aplicação não foram repetidos porque só documentação mudou. Os 152 testes pertencem à Fase 06, não são uma nova execução desta fase.

## Decisões pendentes e aceite

Pesquisa documental encerrada; integração continua não implementada. Recomendação registrada em [Decisão de agendamento](DECISAO_AGENDAMENTO_META.md), arquitetura em [Integração Meta](INTEGRACAO_META.md). Antes da Fase 08, o usuário deve escolher se aceita o fluxo assistido recomendado, confirmar os ativos da loja e autorizar separadamente qualquer conexão/publicação de teste. Se exigir API, decidir transporte de imagens e estratégia administrativa de token ou componente servidor; sem resolver isso, não prometer publicação automática nem independência do PC.
