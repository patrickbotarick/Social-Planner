# Integração Trello — operação e contratos

Implementada na Fase 04 em 14/10/2026. A comunicação ocorre exclusivamente no Rust e é somente leitura. SQLite e arquivos locais continuam sendo a fonte do conteúdo editorial. Não há backend remoto próprio, login interno, scraping ou publicação externa.

## Configurar uma conta

1. Abra **Configurações → Trello** na janela desktop.
2. Acesse [Trello Apps Admin](https://trello.com/apps/admin), crie um Power-Up pessoal se necessário, abra **Trello Auth** e gere uma chave de API. Não é necessário distribuir/instalar esse Power-Up nos quadros para usar a API pessoal.
3. Informe a chave no aplicativo e use **Copiar endereço de autorização**. Abra o endereço no navegador. O endereço contém apenas a chave pública, `scope=read`, `expiration=30days` e `response_type=token`.
4. Confira a permissão de leitura e autorize no próprio Trello. Copie o token recebido para o campo protegido do aplicativo e clique em **Conectar Trello**. Nunca envie senhas ou tokens por chat, Git ou arquivos de configuração.
5. O Rust consulta `members/me`; somente uma resposta válida permite guardar o par chave/token no **Windows Credential Manager**, via keyring/windows-native. O formulário limpa o token ao enviar e nunca recupera o segredo salvo.
6. **Testar conexão** consulta o serviço e mostra nome/username quando disponíveis. “Credencial salva” significa apenas presença no cofre; não é prova de validade atual. HTTP 401 solicita renovação; não existe renovação automática neste fluxo.
7. **Desconectar** remove somente a entrada local. Para revogar o acesso no Trello, remova a autorização nas configurações da conta. Publicações, mídias e históricos permanecem locais.

O usuário realiza a autorização no navegador. O aplicativo não recebe senha Trello. Tokens não entram no SQLite nem no localStorage. Não há client secret distribuído. O par no cofre usa service `br.com.gmad.socialplanner.trello` e usuário lógico `personal-read-only`.

Escolha: fluxo oficial `1/authorize` para instalação pessoal, ainda documentado pela Atlassian. Não se mistura o token desse fluxo com Bearer/OAuth 2.0. OAuth 2.0 também foi consultado; o cadastro/configuração de cliente e callbacks desse fluxo não foram implementados nesta fase. A escolha do fluxo pessoal não afirma que OAuth 2.0 público exige client secret.

Fontes oficiais consultadas em 14/10/2026: [introdução e chave API](https://developer.atlassian.com/cloud/trello/guides/rest-api/api-introduction/), [autorização, escopo e cabeçalho](https://developer.atlassian.com/cloud/trello/guides/rest-api/authorization/), [configuração OAuth 2.0](https://developer.atlassian.com/cloud/trello/guides/rest-api/oauth-2-client-configuration/).

## Consulta e confirmação

Importações → colar URL → Consultar quadro → pesquisar/filtrar por lista → selecionar cartões → confirmar importação. Todos os cartões não arquivados começam selecionados, com ações Selecionar todos/Desmarcar todos. Filtros não apagam seleções fora da lista visível. A consulta não grava publicações nem quadros; a prévia fica somente em memória por até 15 minutos. Trocar conta/URL ou concluir importação invalida a prévia.

Aceita HTTPS com host exato trello.com ou www.trello.com e caminho /b/{shortLink}, com slug opcional, barra final, query/fragmento de compartilhamento. O identificador tem oito caracteres alfanuméricos. Rejeita credenciais na autoridade, portas alternativas, outros hosts/protocolos e caminhos desconhecidos. O destino HTTP é sempre construído no Rust; a URL digitada nunca vira destino de requisição.

A prévia apresenta nomes/contagens retornados, listas, descrição completa, anexos, cartões novos/importados/alterados/arquivados e última sincronização local. Falha de consulta não gera metadados fictícios. URLs de cartões são reconstruídas a partir do shortLink retornado.

## API e limites

Somente GET com `Authorization: OAuth oauth_consumer_key="…", oauth_token="…"`. Cabeçalho marcado sensível. Nenhum segredo em query, resposta IPC ou mensagem de erro.

| Endpoint | Uso |
| --- | --- |
| /1/members/me?fields=fullName,username | Testar a autorização e identificar a conta, sem pedir email |
| /1/boards/{shortLink}?fields=name,shortLink | Confirmar quadro e acesso |
| /1/boards/{id}/lists?filter=all | Listas, nomes e arquivamento |
| /1/boards/{id}/cards | Cartões com campos selecionados e anexos aninhados |
| /1/cards/{id}/attachments/{id}/download/{filename} | Download autenticado de upload real |

[Referência Boards](https://developer.atlassian.com/cloud/trello/rest/api-group-boards/) e [Cards](https://developer.atlassian.com/cloud/trello/rest/api-group-cards/). Quadros privados exigem permissão da conta; o aplicativo também exige conexão para quadros públicos, mantendo um único fluxo de identidade/download. 403/404 não são usados para presumir exclusão de dados locais.

Paginação usa limit=1000 e cursor before pelo menor ID de criação da página, nunca pela posição editorial. Interrompe ao receber página menor; repetição de ID/cursor ou atingir dez páginas cheias bloqueia a consulta, sem alegar snapshot completo. Limite de até 16 MiB por resposta JSON e até 1000 anexos por cartão. O número de páginas é um limite de segurança, não uma quota da plataforma. Ordenação visual por lista/posição/ID. A [introdução oficial](https://developer.atlassian.com/cloud/trello/guides/rest-api/api-introduction/#paging) documenta paginação por criação; a API não oferece snapshot transacional entre as páginas. Mudanças concorrentes podem exigir nova consulta.

O Trello informa 300 requisições por chave/10 segundos, 100 por token/10 segundos e 100 para members/900 segundos. O cliente serializa operações e espaça GETs em pelo menos 120 ms. HTTP 429, 5xx e falhas de conexão antes da resposta permitem até três tentativas, backoff 1/2 segundos ou Retry-After numérico limitado a 10 segundos. Depois retorna erro e exige nova ação; não há loop infinito. [Limites oficiais](https://developer.atlassian.com/cloud/trello/guides/rest-api/rate-limits/).

## Arquivos

Somente uploads do Trello com extensão JPEG/JPG, PNG, WebP ou MP4 seguem para download. Links externos e formatos não aceitos ficam como referências, sem buscar seus hosts. Referências persistidas removem query/fragmento para não conservar URLs assinadas ou tokens; links que dependam desses parâmetros devem ser acessados pelo cartão original. Credenciais embutidas/protocolos não web são rejeitados.

O endpoint de download é reconstruído com IDs validados e nome codificado como um segmento. Somente api.trello.com por HTTPS; TLS verificado, timeout de conexão de 10 segundos e redirecionamentos desabilitados. Respostas 3xx viram falha individual, nunca seguem host desconhecido nem encaminham credenciais. Se um anexo real exigir redirecionamento, essa compatibilidade deve ser investigada antes de ampliar a lista de destinos. O anúncio oficial apresenta download com esse cabeçalho e cópia local: [Authenticated Access to S3](https://community.developer.atlassian.com/t/update-authenticated-access-to-s3/43681).

Download em blocos de 64 KiB para nome UUID temporário, limite efetivo de bytes e conferência de Content-Length quando presente. UI permite reduzir limites: imagens 1–64 MiB; MP4 1–512 MiB; timeout por requisição 10–300 segundos (padrão 120). Metadados da API nunca substituem a inspeção do conteúdo. Reutiliza o MediaStore da Fase 03, incluindo SHA-256, decodificação limitada de imagens, validação estrutural MP4 e promoção de cópia completa para media/<hash>.<ext>. Vídeos não são carregados inteiros em memória e não são transcodificados.

Não abre/executa arquivos baixados. Downloads inválidos/interrompidos limpam os temporários conhecidos; cancelamento é verificado entre requisições, blocos e esperas de retry. Uma leitura de rede já bloqueada pode aguardar o timeout configurado. Falta de espaço/permissão é erro de arquivo, preservando o original e itens anteriores. Interrupção do processo pode conservar temporários ou arquivos completos sem vínculo; não há coletor destrutivo automático.

## Mapeamento e sincronização

Cada cartão confirmado cria no máximo uma publicação inicial `trello/pending_review`. Título/descrição originais ficam completos no snapshot. Título editorial limitado a 200 caracteres e legenda a 20.000; excedentes permanecem na origem e sinalizam ajuste. Datas due/start/atividade são metadados, nunca agendamentos.

Uma imagem sugere Feed; um MP4 sugere Reels; 2–20 mídias distintas sugerem Carrossel. Mistura imagem/vídeo sinaliza ajuste. Sem mídia, excesso de itens, conteúdo duplicado ou anexos indisponíveis exigem ajuste; todos os anexos ficam referenciados, mesmo quando não entram na composição inicial. Publicações sem mídia podem ter o texto editado, permanecendo pendentes.

ID remoto único identifica a origem. Hash do snapshot detecta mudanças de textos, etiquetas, datas, posições e anexos; metadados ficam separados da Publication. Reimportação atualiza o snapshot e o indicador persistente de origem alterada, sem atualizar título, legenda, formato, estado, versão, composição ou deleted_at da publicação existente. Não restaura automaticamente publicações excluídas. Cópias manuais não adquirem vínculo de sincronização.

Novos anexos são baixados; uploads já disponíveis são reutilizados por ID e os bytes deduplicados por SHA-256. Anexos removidos recebem removed=1, mas assets/composições ficam intactos. Arquivado/ausente no resultado completo marca source_missing, sem excluir publicação. Falha/incompletude de leitura não é evidência de remoção. A data do quadro avança somente ao concluir a seleção sem falhas de cartão/anexo; uma seleção parcial representa essa operação confirmada, não importação de todos os cartões.

**Origem Trello**, nos detalhes/editor, mostra snapshot, referências, arquivos novos e necessidade de ajuste. O operador pode adicionar um arquivo disponível à composição e salvar explicitamente. Recuperar um anexo que falhou não o insere silenciosamente numa composição já criada. Comparação visual lado a lado, resolução de conflitos/aceite da mudança e revisão/aprovação completa ficam fora da Fase 04.

## Jobs, recuperação e IPC

Uma operação Trello por vez. Worker blocking mantém rede/download fora do mutex do banco; a transação por cartão reconfere o checksum físico antes de criar vínculos e preserva resultados parciais. Essa conferência pode bloquear brevemente consultas locais com arquivos grandes. UI consulta o progresso a cada 700 ms durante a importação. Fechamento/navegação ficam protegidos enquanto a operação desta tela está em andamento; Cancelar preserva cartões já gravados. Ao reiniciar, jobs ainda running viram interrupted. Para repetir, consulte o quadro e confirme a seleção novamente. Falhas de anexo aparecem na origem da publicação; falhas de cartão são contadas e a mensagem final é redigida.

IPC específico: trello_status, trello_authorization_url, trello_connect, trello_test, trello_disconnect, trello_preview, trello_import, trello_cancel, trello_history, trello_source. Todos declarados no AppManifest/capability main. Não há HTTP/SQL/shell/filesystem genérico exposto ao frontend. CSP e escopos asset anteriores permanecem restritos.

Dependências: reqwest 0.12.28 (blocking, rustls, sem features padrão), keyring 3.6.3 (windows-native), serde_json e url já presentes na árvore Tauri. Compatibilidade verificada por Cargo/check/clippy/build; lockfile versionado. Fontes: [reqwest](https://docs.rs/reqwest/0.12.28/reqwest/), [keyring](https://docs.rs/crate/keyring/3.6.3).
