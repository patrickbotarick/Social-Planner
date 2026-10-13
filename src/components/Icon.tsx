const paths: Record<string, string> = {
  dashboard: 'M3 3h7v7H3z M14 3h7v7h-7z M3 14h7v7H3z M14 14h7v7h-7z',
  biblioteca: 'M3 6h7l2 2h9v12H3z M3 6V4h7l2 2',
  importacoes: 'M12 3v12 m-5-5 5 5 5-5 M4 15v6h16v-6',
  revisao: 'M9 4H5v17h14V4h-4 M9 3h6v4H9z m-1 11 3 3 5-6',
  calendario: 'M4 5h16v16H4z M4 10h16 M8 3v4 M16 3v4 M8 14h2 M14 14h2',
  'nova-publicacao': 'M12 5v14 M5 12h14',
  historico: 'M4 5v5h5 M4 10a8 8 0 1 1 1 8 M12 7v5l3 2',
  configuracoes: 'M4 7h16 M4 17h16 M8 4v6 M16 14v6',
  search: 'M10 3a7 7 0 1 0 0 14 7 7 0 0 0 0-14 M15 15l6 6',
  media: 'M3 4h18v16H3z M3 16l5-5 5 5 3-3 5 5 M16 8h.01',
  list: 'M9 5h12 M9 12h12 M9 19h12 M3 5h1 M3 12h1 M3 19h1',
  'chevron-left': 'M15 5l-7 7 7 7',
  'chevron-right': 'M9 5l7 7-7 7',
  close: 'M6 6l12 12 M6 18L18 6',
  help: 'M12 3a9 9 0 1 0 0 18 9 9 0 0 0 0-18 M9 9a3 3 0 0 1 6 0c0 2-3 2-3 4 M12 17h.01',
  copy: 'M8 8h13v13H8z M16 8V3H3v13h5',
  check: 'M5 12l4 4L19 6',
};
export function Icon({ name }: { name: string }) {
  return <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.6" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true"><path d={paths[name] ?? paths.biblioteca} /></svg>;
}
