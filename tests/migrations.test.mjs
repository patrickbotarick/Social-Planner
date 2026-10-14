// Valida o SQL real com SQLite nativo do Node. Não substitui os testes do repositório Rust.
import { DatabaseSync } from 'node:sqlite';
import { readFileSync, mkdtempSync, rmSync, rmdirSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { test } from 'node:test';
import assert from 'node:assert/strict';

const sql = readFileSync(new URL('../src-tauri/migrations/001_foundation.sql', import.meta.url), 'utf8');
const editorialSql = readFileSync(new URL('../src-tauri/migrations/002_editorial_library.sql', import.meta.url), 'utf8');

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

test('migração editorial preserva preferências e restringe referências, ordem e caminhos', () => {
  const db = new DatabaseSync(':memory:');
  try {
    db.exec('PRAGMA foreign_keys=ON'); db.exec(sql); db.exec("UPDATE app_settings SET timezone='UTC'");
    const before = db.prepare('SELECT * FROM app_settings').get();
    db.exec('BEGIN IMMEDIATE'); db.exec(editorialSql); db.exec('COMMIT');
    assert.deepEqual(db.prepare('SELECT * FROM app_settings').get(), before);
    db.prepare('INSERT INTO publications (id,title,caption,search_text,format) VALUES (?,?,?,?,?)').run('p1','Título','Legenda 😊\n#GMAD','titulo legenda','feed');
    const checksum='a'.repeat(64);
    db.prepare('INSERT INTO media_assets (id,filename,mime_type,extension,size_bytes,storage_path,checksum) VALUES (?,?,?,?,?,?,?)').run('m1','imagem.png','image/png','png',100,checksum+'.png',checksum);
    db.prepare('INSERT INTO publication_media VALUES (?,?,?)').run('p1','m1',0);
    assert.throws(() => db.prepare('INSERT INTO publication_media VALUES (?,?,?)').run('p1','missing',1));
    assert.throws(() => db.prepare('INSERT INTO publication_media VALUES (?,?,?)').run('p1','m1',1));
    assert.throws(() => db.exec("DELETE FROM media_assets WHERE id='m1'"));
    assert.throws(() => db.exec("UPDATE media_assets SET storage_path='../private'"));
    assert.throws(() => db.exec("UPDATE publications SET editorial_status='published'"));
    assert.equal(db.prepare('SELECT caption FROM publications').get().caption,'Legenda 😊\n#GMAD');
    assert.deepEqual(db.prepare('PRAGMA foreign_key_check').all(),[]);
    assert.equal(db.prepare('PRAGMA quick_check').get().quick_check,'ok');
  } finally { db.close(); }
});
