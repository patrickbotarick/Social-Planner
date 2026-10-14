export const sections = [
  { id: 'dashboard', label: 'Dashboard', short: 'Visão geral', phase: '01', description: 'Seu espaço de planejamento editorial.' },
  { id: 'biblioteca', label: 'Biblioteca editorial', short: 'Biblioteca', phase: '03', description: 'Textos e mídias das campanhas, reunidos em um só lugar.' },
  { id: 'importacoes', label: 'Importações Trello', short: 'Importações', phase: '04', description: 'Receba os cartões e anexos do quadro mensal da matriz.' },
  { id: 'revisao', label: 'Revisão de publicações', short: 'Revisão', phase: '05', description: 'Selecione as campanhas que fazem sentido para a GMAD Rio Preto.' },
  { id: 'calendario', label: 'Calendário mensal', short: 'Calendário', phase: '06', description: 'Planeje a distribuição das publicações ao longo do mês.' },
  { id: 'nova-publicacao', label: 'Nova publicação', short: 'Criar publicação', phase: '03', description: 'Prepare conteúdos próprios da unidade.' },
  { id: 'historico', label: 'Histórico', short: 'Histórico', phase: '08', description: 'Acompanhe eventos e confirmações reais de publicação.' },
  { id: 'configuracoes', label: 'Configurações', short: 'Configurações', phase: '01', description: 'Informações do aplicativo e armazenamento neste computador.' },
] as const;

export type Section = (typeof sections)[number];
export function sectionFromHash(hash: string): Section {
  return sections.find((section) => `#/${section.id}` === hash || section.id === 'biblioteca' && /^#\/biblioteca\/[0-9a-f-]{36}(\/editar)?$/i.test(hash)) ?? sections[0];
}
