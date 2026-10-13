# Instruções persistentes — GMAD Social Planner

1. Ler `docs/PROJETO_MESTRE.md` antes de alterações relevantes.
2. Respeitar o roadmap oficial de nove fases.
3. Não implementar fases futuras sem solicitação do usuário.
4. Preservar dados locais; nunca apagar/recriar banco operacional para corrigir erros.
5. Evitar dependências desnecessárias; verificar versões e compatibilidade antes de instalar.
6. Manter TypeScript estrito e separar interface, domínio e persistência.
7. Executar as validações aplicáveis após alterações; registrar bloqueios, sem alegar testes não executados.
8. Atualizar a documentação ao concluir cada fase e registrar o resultado real.
9. Não modificar contratos estabelecidos sem necessidade justificada e documentada; nunca reescrever migrações aplicadas.
10. Nunca executar publicação real sem autorização explícita.

SQLite e integrações são exclusivos do Rust. Segredos externos pertencem ao armazenamento seguro do sistema operacional. Não habilitar SQL, shell ou acesso arbitrário ao filesystem pelo frontend. Dados, mídias, tokens e builds não entram no Git. Não fazer push automático.
