# Backup e restauração — versão candidata 1.0

Disponível em **Configurações → Backup**, na janela desktop. O pacote é local, contém conteúdo editorial privado e **não é criptografado**. Guarde cópias em destino protegido e mantenha mais de uma geração. Não envie banco, mídias ou backups ao GitHub.

## Criar e conferir

1. Feche outras instâncias do GMAD. Aguarde importações e exportações terminarem.
2. Clique em **Criar backup local** e escolha uma pasta local no seletor Windows, fora do diretório de dados e de seus ancestrais. Não escolha a própria pasta do aplicativo, a raiz do disco ou uma pasta de rede.
3. Aguarde a mensagem **Backup concluído e verificado**. Anote caminho, contagens e SHA-256 do manifesto. Não feche o aplicativo durante a operação.
4. Preserve a pasta `GMAD-Backup-<data UTC>-<UUID>` inteira, incluindo `manifesto.json`, `planner.sqlite3` e `media/`. Não renomeie arquivos internos nem altere o manifesto.

O SQLite é capturado pela Online Backup API, incluindo transações já confirmadas no WAL; não é uma cópia cega do banco aberto. O pacote é autossuficiente, sem depender de WAL/SHM. Operações locais são serializadas durante a captura e verificação. Cada geração usa uma pasta nova e nunca sobrescreve outra. O manifesto completo é escrito após os arquivos; somente a inspeção de todos os hashes, estrutura SQLite e mídias permite declarar sucesso. Uma pasta deixada por interrupção não é automaticamente um backup válido.

Inclui banco, mídias físicas locais por checksum, vínculos, revisões, calendários, histórico e preferências persistentes. Não inclui credenciais do Windows Credential Manager, tokens, senhas, caches temporários, executáveis ou pacotes Meta exportados em outras pastas. O módulo de backup não acessa o cofre. Em outro computador, configure a conexão Trello novamente pelo formulário próprio.

## Restaurar

1. Faça uma cópia de segurança atual e feche outras instâncias, inclusive versões antigas.
2. Use **Inspecionar backup para restaurar** e escolha a pasta exata que contém `manifesto.json`, e não sua pasta pai.
3. Confira origem, data, versão, esquema e contagens. Pacotes incompletos, adulterados ou incompatíveis são bloqueados antes da substituição.
4. Leia o aviso. A restauração substitui **todo o perfil editorial local**, não mescla publicações. Marque a confirmação e use **Restaurar e recarregar dados** somente para o perfil desejado.
5. O GMAD verifica novamente o pacote e cria um backup de recuperação do estado atual antes de ativar o restaurado. Também preserva o diretório anterior completo. Os dois caminhos aparecem no resultado; registre-os e não os exclua antes de conferir os dados.
6. Confira Biblioteca, mídias, Calendário e Histórico. Feche e reabra o aplicativo. Se houver aviso sobre prévias, reabra para reativá-las.
7. Escolha novamente a pasta de exportação e regenere os pacotes. Preparações são desatualizadas e declarações manuais passam a exigir revisão. Confira o Business Suite antes de reconfirmar qualquer agendamento.

Credenciais do cofre permanecem intactas. Restaurar o SQLite **não desfaz ações no Trello ou na Meta**. Uma declaração restaurada não comprova o estado remoto atual. Para desfazer uma restauração concluída, inspecione o backup de recuperação informado e execute outra restauração explícita; não copie arquivos sobre um banco aberto.

## Falhas e limites

- Falta de espaço ou permissão interrompe a operação com erro. A reserva inicial é estimada e não garante espaço até o fim; mantenha folga para banco, mídias, staging e recuperação.
- Formato de pacote 1 e esquema SQLite 6 nesta candidata. O esquema e o histórico de migrações devem corresponder exatamente ao modelo conhecido. A versão textual do aplicativo é informativa; compatibilidade é decidida pelo formato/esquema/estrutura, não apenas pelo número de versão.
- Apenas disco local, sem UNC, links simbólicos ou reparse points. Mídias aceitas: JPEG, PNG, WebP e MP4 nos limites editoriais existentes. Limites adicionais do pacote: manifesto até 16 MiB, 100.001 arquivos, 2 GiB por arquivo e 64 GiB no total.
- SHA-256 detecta alterações, mas não é assinatura de autoria. Restaure apenas backups de origem confiável. Não há criptografia nem defesa garantida contra um processo malicioso do mesmo usuário trocando diretórios simultaneamente.
- A troca de banco e mídias usa staging no mesmo volume e diário de recuperação. Se houver interrupção antes da confirmação final, a próxima abertura tenta recuperar o diretório anterior. O protocolo é testado por injeção de falhas; não promete atomicidade de todos os componentes em qualquer perda de energia ou falha física.
- Não exclua manualmente arquivos `.gmad-*`, staging ou diretórios anteriores durante uma recuperação. Em erro persistente, preserve tudo e solicite suporte. Se o banco atual estiver inválido e não puder gerar o ponto de recuperação verificado, a restauração é bloqueada; não existe reparo destrutivo automático.
- O bloqueio de instância vale para esta versão. Builds antigos não participam dele e precisam ser fechados. Pacotes concluídos, cópias anteriores e resíduos de interrupções não têm limpeza automática indiscriminada.

Evidências e pendências: [auditoria da Fase 09A](FASE_09A_AUDITORIA_RELEASE.md).
