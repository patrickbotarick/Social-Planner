# Guia do usuário — GMAD Social Planner 1.0 candidata

O GMAD organiza conteúdo neste computador. Não publica nem agenda automaticamente no Instagram. **Planejado localmente**, **Preparado** e **Confirmado manualmente** têm significados distintos; nenhum é confirmação por API da Meta.

## Primeiro uso

Use o instalador Windows x64 candidato indicado nas [notas de distribuição](RELEASE_CANDIDATE_V1.md). O usuário final não precisa de Node, Rust ou Codex; precisa de Windows compatível e WebView2. Abra pelo atalho do GMAD. Configurações → Armazenamento informa o diretório efetivo de dados; ele fica no perfil Windows, separado da instalação. Não o apague para resolver erros.

## Receber e criar conteúdo

Em **Configurações → Trello**, configure a conexão usando suas credenciais autorizadas. Segredos ficam no cofre do Windows. Em **Importações**, informe a URL do quadro, consulte e selecione somente os cartões desejados. Confira o resultado por cartão e anexo. Em falha parcial, sincronize o mesmo cartão para retomar; não recrie publicações para contornar anexos pendentes. A origem é identificada pelo ID remoto e edições locais são preservadas. O GMAD não modifica cartões remotos.

Em **Criar publicação**, informe os dados editoriais e selecione mídias pelo diálogo Windows. Os arquivos são validados e copiados para o armazenamento local. Vídeos dependem dos codecs disponíveis no WebView2; um arquivo baixado corretamente ainda pode não reproduzir. Confira a ordem e o formato antes de salvar.

## Biblioteca e revisão

Use pesquisa e filtros para localizar a publicação. Abra os detalhes para conferir legenda, galeria, origem e histórico. Edite a cópia editorial local e salve explicitamente. Duplicação cria uma cópia; exclusão é lógica e não deve ser usada para reparar sincronização. Ao sair de alterações não salvas, escolha salvar, descartar ou permanecer.

Em **Revisão**, confira composição e mídia e decida aprovar ou ignorar, individualmente ou em lote. Lotes mostram resultado por publicação. Alterar conteúdo exige revisar a nova versão; histórico permanece preservado. Aprovar não envia conteúdo à Meta.

## Calendário e transferência

No **Calendário**, escolha uma aprovada, data futura, horário e fuso. Confira conflitos e confirme alterações por formulário ou arraste. Reagendar ou cancelar muda a intenção local; não altera um agendamento existente no Business Suite.

Em **Preparar para Meta**, escolha a pasta de exportação, prepare os itens elegíveis e confira a prévia. Cada pacote mantém mídias ordenadas, legenda UTF-8 e informações de integridade. Copiar legenda ou abrir pasta não registra agendamento. Alterações posteriores tornam o pacote desatualizado; regenere preservando o anterior.

Siga o [guia de agendamento assistido](GUIA_AGENDAMENTO_META.md) para transferir manualmente ao Business Suite, conferir conta/canal/fuso e só depois registrar a declaração no GMAD. Divergência mantém ambos os horários até reconciliação explícita. Confira sempre a plataforma ao revisar uma declaração. A homologação real na conta GMAD continua pendente.

## Acompanhar e proteger

**Visão geral** mostra contagens reais; **Histórico** separa eventos editoriais, planejamento e transferência assistida. Mensagens de falha não significam sucesso remoto. Não repita uma operação externa incerta antes de verificar a plataforma.

Crie backups periódicos por **Configurações → Backup**, especialmente antes de atualizar ou restaurar. Consulte o [guia de backup e restauração](GUIA_BACKUP_RESTAURACAO.md). Backups incluem conteúdo privado e não são criptografados. Nunca compartilhe tokens, senhas ou URLs assinadas em observações, capturas, chamados ou Git.

Para suporte, informe versão, código/mensagem do erro e ação realizada, sem credenciais ou mídia privada. Preserve o banco e os arquivos; não reinstale apagando dados como primeira tentativa de correção.
