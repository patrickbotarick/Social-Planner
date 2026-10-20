# Windows — candidata 1.0.0-rc.1

**Candidata gerada em 20/10/2026. Não é release estável publicado.** Backup/restauração nativos e execução release QA validados. Instalação adiada pelo usuário. Evidências e pendências em [Fase 09A](FASE_09A_AUDITORIA_RELEASE.md). Não há tag v1.0.0 nem release definitivo.

## Gerar

No ambiente de desenvolvimento Windows x64 com Node 24, Rust MSVC, Build Tools/SDK e WebView2:

```powershell
npm.cmd ci
npm.cmd run check
npm.cmd run test:rust
npm.cmd run check:rust
npm.cmd run build:installer
Get-FileHash -Algorithm SHA256 -LiteralPath 'src-tauri/target/release/bundle/nsis/GMAD Social Planner_1.0.0-rc.1_x64-setup.exe'
```

Executável: `src-tauri/target/release/gmad-social-planner.exe`.
Instalador: `src-tauri/target/release/bundle/nsis/GMAD Social Planner_1.0.0-rc.1_x64-setup.exe`.
Os caminhos são relativos à raiz do checkout. Artefatos ficam fora do Git. Build final `npm.cmd run build:installer` aprovado; produto/identificador originais conferidos no executável e no NSIS. Não reutilizar hashes de builds anteriores com o mesmo nome/versão.

| Artefato final | Bytes / SHA-256 |
| --- | --- |
| Instalador NSIS x64 | 5.113.885 bytes; `7F2D34F197BB41B5B22D394235C3619CA55A71C216E4CDC80A6DC54286957ABF` |
| Executável release | `24127BA8B38E0813B687B6A637AD001BB82FB07B6669FB70374D0D951B3189BE` |

Assinatura do instalador: `NotSigned`. O hash corresponde a este build local, não é assinatura digital. A variante QA foi aberta sem instalação, em identidade separada, com biblioteca/IPC/SQLite/prévia de imagem e persistência após reabertura. A candidata operacional não foi aberta para evitar tocar o banco real.

## Instalar, atualizar e remover

O instalador NSIS é para o usuário atual, sem exigir o Visual Studio IDE. O aplicativo release inicia sem terminal e não exige Rust, Node ou Codex no computador de destino. Windows x64 e WebView2 são necessários; o bootstrapper oficial do WebView2 pode precisar de internet quando o runtime não estiver presente. Teste em Windows limpo permanece pendente.

Antes de atualizar, crie e verifique backup e feche todas as instâncias. Mantenha o mesmo usuário e identificador de aplicativo. A instalação fica separada do perfil de dados; o diretório efetivo pode ser conferido em Configurações → Armazenamento. Migrações existentes são aditivas, não reescritas. Não apague o banco para efetuar update. O NSIS foi configurado para impedir downgrade, mas a candidata não oferece compatibilidade futura universal entre versões.

Ao desinstalar, mantenha desmarcada a opção de remover dados. O script gerado não apaga silenciosamente o perfil em atualização e deixa a remoção de dados como opção explícita em desinstalação. Preserve backups antes de qualquer remoção. Credenciais pertencem ao cofre Windows e não são um arquivo da instalação.

O instalador não está assinado (`NotSigned`); o publisher de metadados não equivale a assinatura digital. Podem aparecer avisos do Windows ou bloqueios corporativos. Não contorne barreiras de segurança; solicite avaliação/autorização humana e use apenas artefato com origem e hash conferidos.

## Homologação antes da versão estável

- Validar instalador em perfil isolado, atalhos, atualização e retenção de dados na desinstalação. Execução portátil release QA e reabertura já aprovadas; instalação não executada por decisão do usuário.
- Testar Windows limpo sem ferramentas de desenvolvimento e execução após reinício do sistema.
- Backup/restauração em QA aprovados: IDs, legenda, estados editoriais, vínculos, sete hashes e SQLite preservados; pacotes/declaração desatualizados conforme contrato. Repetir no ambiente de destino antes da adoção operacional.
- Executar um agendamento real autorizado pelo [roteiro Meta](GUIA_AGENDAMENTO_META.md); conferir conta/canal, instante/fuso, calendário remoto e independência do GMAD fechado.
- Resolver bloqueios, apreciar interface, considerar assinatura de código e autorizar separadamente o release estável.

Limites: [guia de backup](GUIA_BACKUP_RESTAURACAO.md), [guia do usuário](GUIA_USUARIO_V1.md), [changelog](../CHANGELOG.md). Não iniciar 09B nem publicar conteúdo por efeito desta candidata.
