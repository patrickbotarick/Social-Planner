# Guia de agendamento assistido — Meta Business Suite

Implementado na Fase 08, em 19/10/2026. O GMAD prepara arquivos e registra declarações do operador. O agendamento é feito por você no [Business Suite](https://business.facebook.com/); nenhum botão do GMAD envia ou agenda conteúdo na Meta.

## Preparar o material

1. Revise a publicação na Biblioteca e aprove a versão correta. No Calendário, mantenha um planejamento ativo com data futura, horário e fuso corretos.
2. Abra **Preparar para Meta**. Expanda **Filtrar preparação** para escolher mês, pesquisa, formato, estado, origem e quadro. Confira os motivos dos itens inelegíveis e os avisos de transferência.
3. Use **Escolher pasta de exportação** e selecione uma pasta local no diálogo Windows. A preferência será lembrada. Evite pastas sincronizadas ou compartilhadas se o material não puder ser compartilhado com terceiros; a pasta está sob seu controle.
4. Use **Preparar** individualmente ou selecione os itens elegíveis da página e **Preparar selecionadas**. Confirme a geração e confira o resultado de cada publicação. Falha de um item não desfaz os demais.
5. Abra a **Prévia**, confira ordem, legenda e avisos. Use **Abrir pasta** e **Copiar legenda**. Essas ações exigem um pacote atual e íntegro.

Cada pacote contém `media_01.<extensão>`, demais mídias na ordem editorial, `legenda.txt` UTF-8 e `informacoes.json`. A subpasta tem mês, data/hora, título seguro e um identificador único. Nenhuma mídia é convertida e nenhum texto é truncado. Emojis, quebras de linha e marcadores Markdown são preservados literalmente. O manifesto permite conferir IDs, versões, formato, data/hora/fuso/UTC e hashes; não contém credenciais nem URLs remotas.

PNG/WebP exigem conferir a aceitação na interface Meta; um aviso não significa que sejam sempre recusados. Vídeos exigem conferir codec, áudio, duração e enquadramento. Se a Meta exigir conversão ou edição, produza uma cópia manual, revise a composição no GMAD, aprove e planeje novamente. O pacote deve corresponder ao conteúdo realmente transferido. O fluxo local adota limites conservadores de 2.200 caracteres e dez mídias em carrossel; não são uma garantia universal das capacidades da interface Meta.

## Homologação operacional da GMAD Rio Preto — pendente

Execute o roteiro abaixo somente com autorização explícita para um agendamento real. A Fase 08 validou o mecanismo local em QA; não realizou estas operações na conta real.

1. Escolha **uma publicação real aprovada**, com autorização editorial, conta Instagram e data futura definidos. Identifique-a pelo ID, além do título.
2. Prepare seu pacote no GMAD e confira prévia, ordem, legenda e planejamento.
3. Clique em **Abrir Business Suite**. Entre diretamente na plataforma pelo navegador e confira a empresa, a Página e o Instagram **GMAD Rio Preto**. O GMAD não recebe sua senha ou cookies.
4. Crie o conteúdo manualmente no Business Suite e carregue as mídias do pacote na ordem numerada. Confira todas as prévias e mensagens de compatibilidade da plataforma.
5. Cole a legenda, conferindo emojis, quebras, hashtags e eventuais marcadores literais. Se precisar alterar conteúdo, mantenha a versão editorial local correspondente e prepare novamente antes da confirmação.
6. Defina data e horário na Meta. Confira o fuso exibido; compare o instante com o planejamento local, e não somente os números do relógio.
7. Confirme o agendamento **na Meta** e confira que consta no calendário da plataforma. Guarde uma referência não sensível, se disponível. Não assuma sucesso se a Meta mostrar erro ou resultado incerto; confira antes de tentar criar novamente.
8. Volte ao GMAD e use **Registrar confirmação**. Informe data/hora/fuso efetivos, referência e observação opcionais. Marque **Declaro que verifiquei o agendamento no Business Suite** somente após a conferência real. Salve a declaração manual.
9. Compare os dois calendários, conta/canal, conteúdo e horário. Registre evidências e limitações da homologação. Depois do agendamento efetivo na plataforma, o GMAD pode ser fechado e o PC desligado; a declaração local, isoladamente, não comprova esse agendamento.

Não registre tokens, senhas ou links assinados na referência ou observação. Links aceitos são HTTPS de hosts Meta permitidos, sem query, fragmento ou credenciais; uma referência textual curta também é aceita. O GMAD não abre nem consulta a referência.

## Divergências e revisões

Se o horário efetivo diferir do planejado, a declaração guarda ambos e mostra **Com divergência**. O calendário original continua intacto. Você pode mantê-lo ou escolher **Reconciliar calendário**, revisar a proposta e aplicar explicitamente o horário declarado ao calendário local. A ação não altera a Meta, desatualiza o pacote e exige regeneração e nova conferência antes de **Revisar confirmação**.

Editar conteúdo, alterar sua aprovação, reagendar ou cancelar localmente também desatualiza preparações e marca declarações como **Requer revisão na Meta**. Confira e ajuste manualmente o Business Suite, se necessário. Após regenerar, registre uma nova declaração explícita; a anterior permanece no histórico, substituída. Regenerar sozinho não reconfirma nada.

Preparar novamente um pacote atual reaproveita a preparação. **Regenerar pacote** cria outra pasta, preservando a anterior. Alterar a pasta preferida afeta novas gerações; pacotes atuais continuam na raiz original. Não apague ou edite arquivos do pacote que pretende usar; alterações, arquivos ausentes e hashes diferentes exigem regeneração. O aplicativo não remove automaticamente pacotes concluídos.

O registro assistido exige um planejamento ativo, aprovado e ainda futuro. Planejamentos vencidos são sinalizados; esta fase não oferece fechamento retrospectivo de publicação. Uma declaração existente continua no histórico, mesmo quando exige revisão. Não reagende um conteúdo vencido apenas para ocultar divergências sem antes conferir a plataforma.

## Acompanhamento e falhas

Na área assistida, contagens consideram mês, pesquisa, formato, origem e quadro; o filtro de estado seleciona as linhas sem alterar os totais desses outros filtros. Dashboard resume o mês atual no fuso da preferência. **Confirmada manualmente** significa declaração humana, nunca prova por API. Histórico → **Transferência assistida** mantém eventos locais com IDs de plano, pacote e declaração.

Se faltar espaço ou houver erro de escrita, preserve os originais, libere espaço/escolha outra pasta e tente novamente. A aplicação limpa staging conhecido quando seguro; após interrupção, marca a preparação como falha ao reabrir. Pastas finais são conservadas, inclusive se houver interrupção entre a promoção do pacote e o registro de sucesso no banco. Se a pasta foi movida, ficou indisponível ou ganhou links de diretório, escolha uma raiz válida e regenere.

Controles de preparação ficam indisponíveis na prévia web. Use a janela Tauri. Não há sincronização automática com a Meta, detecção automática de publicação, upload, conversão, backup ou instalador nesta fase.
