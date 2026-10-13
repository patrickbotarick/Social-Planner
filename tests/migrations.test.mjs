// Valida o SQL real com SQLite nativo do Node. Não substitui os testes do repositório Rust.
import { DatabaseSync } from 'node:sqlite';
import { readFileSync, mkdtempSync, rmSync, rmdirSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { test } from 'node:test';
import assert from 'node:assert/strict';

const sql = readFileSync(new URL('../src-tauri/migrations/001_foundation.sql', import.meta.url), 'utf8');

test('migração cria preferências reais, preserva alterações após reabrir e mantém integridade', () => {
  const directory = mkdtempSync(join(tmpdir(), 'gmad-sql-test-'));
  const file = join(directory, 'test.sqlite3');
  let db;
  try {
    db = new DatabaseSync(file);
    db.exec('BEGIN IMMEDIATE');
    db.exec(sql);
    db.exec('COMMIT');
    const settings = db.prepare('SELECT * FROM app_settings').get();
    assert.equal(settings.timezone, 'America/Sao_Paulo');
    assert.match(settings.created_at, /^\d{4}-\d{2}-\d{2}T.*Z$/);
    db.prepare('UPDATE app_settings SET timezone = ? WHERE id = 1').run('UTC');
    db.close();
    db = new DatabaseSync(file);
    assert.equal(db.prepare('SELECT timezone FROM app_settings WHERE id = 1').get().timezone, 'UTC');
    assert.equal(db.prepare('PRAGMA quick_check').get().quick_check, 'ok');
    assert.throws(() => db.exec("INSERT INTO app_settings (id, timezone) VALUES (2, 'UTC')"));
    assert.throws(() => db.exec("UPDATE app_settings SET timezone = ' '"));
  } finally {
    db?.close();
    // Somente o arquivo criado pelo teste; não faz remoção recursiva.
    rmSync(file, { force: true });
    rmdirSync(directory);
  }
});

test('falha transacional não deixa esquema parcialmente aplicado', () => {
  const db = new DatabaseSync(':memory:');
  try {
    db.exec('BEGIN IMMEDIATE');
    assert.throws(() => db.exec(`${sql}\nINVALID SQL;`));
    db.exec('ROLLBACK');
    assert.equal(db.prepare("SELECT COUNT(*) AS count FROM sqlite_master WHERE name = 'app_settings'").get().count, 0);
  } finally { db.close(); }
});
