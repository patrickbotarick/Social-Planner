export interface AppSettings {
  timezone: string;
  createdAt: string;
  updatedAt: string;
}

export interface FoundationStatus {
  schemaVersion: number;
  databasePath: string;
  integrity: 'ok';
  settings: AppSettings;
}

export type FoundationState =
  | { kind: 'loading' }
  | { kind: 'browser' }
  | { kind: 'ready'; data: FoundationStatus }
  | { kind: 'error'; message: string };
