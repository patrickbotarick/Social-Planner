// Valida o SQL real com SQLite nativo do Node. Não substitui os testes do repositório Rust.
import { DatabaseSync } from 'node:sqlite';
import { readFileSync, mkdtempSync, rmSync, rmdirSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { test } from 'node:test';
import assert from 'node:assert/strict';

const sql = readFileSync(new URL('../src-tauri/migrations/001_foundation.sql', import.meta.url), 'utf8');
const editorialSql = readFileSync(new URL('../src-tauri/migrations/002_editorial_library.sql', import.meta.url), 'utf8');

test('calendário migra banco existente, exige aprovação, evita duplicação e invalida por versão com auditoria atômica', () => {
  const db=new DatabaseSync(':memory:');db.exec('PRAGMA foreign_keys=ON');db.exec(sql);db.exec(editorialSql);
  for(const name of ['003_trello_import','004_editorial_review']) db.exec(readFileSync(new URL(`../src-tauri/migrations/${name}.sql`,import.meta.url),'utf8'));
  db.prepare('INSERT INTO publications(id,title,caption,search_text,format,editorial_status) VALUES(?,?,?,?,?,?)').run('original','Preservar','😊\n#GMAD','preservar','feed','approved');
  const before=db.prepare('SELECT * FROM publications').get();db.exec(readFileSync(new URL('../src-tauri/migrations/005_local_calendar.sql',import.meta.url),'utf8'));assert.deepEqual(db.prepare('SELECT * FROM publications').get(),before);
  const insert=db.prepare("INSERT INTO schedules(id,publication_id,publication_version,scheduled_date,scheduled_time,timezone,scheduled_at_utc,status) VALUES(?,'original',1,'2099-11-03','23:30','America/Sao_Paulo','2099-11-04T02:30:00Z','planned_local')");
  insert.run('planned');assert.throws(()=>insert.run('duplicate'));assert.equal(db.prepare('SELECT count(*) n FROM schedule_events').get().n,1);
  db.exec('UPDATE publications SET version=version+1');assert.equal(db.prepare('SELECT status FROM schedules').get().status,'invalidated_local');assert.equal(db.prepare('SELECT count(*) n FROM schedule_events').get().n,2);
  assert.throws(()=>db.exec('DELETE FROM schedules'));assert.throws(()=>db.exec('DELETE FROM schedule_events'));assert.throws(()=>db.exec("UPDATE schedule_events SET timezone='UTC'"));
  assert.equal(db.prepare('PRAGMA quick_check').get().quick_check,'ok');assert.deepEqual(db.prepare('PRAGMA foreign_key_check').all(),[]);db.close();
});

test('migração de revisão preserva dados anteriores e protege histórico imutável por versão', () => {
  const db = new DatabaseSync(':memory:');
  db.exec('PRAGMA foreign_keys=ON');db.exec(sql);db.exec(editorialSql);
  db.exec(readFileSync(new URL('../src-tauri/migrations/003_trello_import.sql', import.meta.url),'utf8'));
  db.prepare('INSERT INTO publications(id,title,caption,search_text,format) VALUES(?,?,?,?,?)').run('before','Preservar','😊\n#GMAD','preservar','feed');
  const before=db.prepare('SELECT * FROM publications').get();
  db.exec(readFileSync(new URL('../src-tauri/migrations/004_editorial_review.sql',import.meta.url),'utf8'));
  assert.deepEqual(db.prepare('SELECT * FROM publications').get(),before);
  db.exec("INSERT INTO editorial_reviews(id,publication_id,event,previous_status,next_status,reviewed_version,resulting_version) VALUES('event','before','submitted','draft','pending_review',1,2)");
  assert.throws(()=>db.exec("UPDATE editorial_reviews SET next_status='approved'"));
  assert.throws(()=>db.exec('DELETE FROM editorial_reviews'));
  assert.throws(()=>db.exec("INSERT INTO editorial_reviews(id,publication_id,event,previous_status,next_status,reviewed_version,resulting_version) VALUES('bad','before','approved','pending_review','approved',2,4)"));
  assert.equal(db.prepare('PRAGMA quick_check').get().quick_check,'ok');assert.deepEqual(db.prepare('PRAGMA foreign_key_check').all(),[]);db.close();
});

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
const trelloSql = readFileSync(new URL('../src-tauri/migrations/003_trello_import.sql', import.meta.url), 'utf8');
test('migração Trello é aditiva e garante uma origem e publicação inicial por cartão', () => {
  const db = new DatabaseSync(':memory:');
  try {
    db.exec('PRAGMA foreign_keys=ON'); db.exec(sql); db.exec(editorialSql);
    db.exec("INSERT INTO publications(id,title,caption,search_text,format) VALUES('manual','Manual','Preservar','manual','feed')");
    const before = db.prepare('SELECT * FROM publications').get();
    db.exec(trelloSql); assert.deepEqual(db.prepare('SELECT * FROM publications').get(), before);
    db.exec("INSERT INTO trello_boards(id,remote_id,name,url) VALUES('b','remote-board','Quadro','https://trello.com/b/Ab12Cd34')");
    db.exec("INSERT INTO imported_cards(id,remote_id,board_id,publication_id,original_title,original_description,source_list,metadata_json,source_hash) VALUES('c','remote-card','b','manual','Original','Descrição','Lista','{}','hash')");
    assert.throws(() => db.exec("INSERT INTO imported_cards SELECT 'c2',remote_id,board_id,publication_id,original_title,original_description,source_list,metadata_json,source_hash,source_updated_at,source_changed,source_missing,needs_adjustment,last_synced_at FROM imported_cards"));
    assert.throws(() => db.exec("DELETE FROM trello_boards WHERE id='b'"));
    assert.deepEqual(db.prepare('PRAGMA foreign_key_check').all(), []);
    assert.equal(db.prepare('PRAGMA quick_check').get().quick_check, 'ok');
  } finally { db.close(); }
});
