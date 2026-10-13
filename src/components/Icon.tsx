const paths: Record<string, string> = {
  dashboard: 'M3 3h7v7H3z M14 3h7v7h-7z M3 14h7v7H3z M14 14h7v7h-7z',
  biblioteca: 'M3 6h7l2 2h9v12H3z M3 6V4h7l2 2',
  importacoes: 'M12 3v12 m-5-5 5 5 5-5 M4 15v6h16v-6',
  revisao: 'M9 4H5v17h14V4h-4 M9 3h6v4H9z m-1 11 3 3 5-6',
  calendario: 'M4 5h16v16H4z M4 10h16 M8 3v4 M16 3v4 M8 14h2 M14 14h2',
  'nova-publicacao': 'M12 5v14 M5 12h14',
  historico: 'M4 5v5h5 M4 10a8 8 0 1 1 1 8 M12 7v5l3 2',
  configuracoes: 'M4 7h16 M4 17h16 M8 4v6 M16 14v6',
};
export function Icon({ name }: { name: string }) {
  return <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.6" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true"><path d={paths[name] ?? paths.biblioteca} /></svg>;
}
