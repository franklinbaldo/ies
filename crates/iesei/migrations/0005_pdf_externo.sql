-- Shape v5: documento externo sintético com duas transições separadas.
-- Banco exclusivamente efêmero do anti-IES, sem sessão/cookie/URL assinada.
-- Reusa sim_processo da migração 0004: uma identidade por processo.
CREATE TABLE externo_tipo (
  origem TEXT NOT NULL, cenario TEXT NOT NULL, id_procedimento TEXT NOT NULL,
  id_serie TEXT NOT NULL, nome TEXT NOT NULL, limite_bytes INTEGER NOT NULL CHECK (limite_bytes > 0),
  PRIMARY KEY (origem, cenario, id_procedimento, id_serie)
) STRICT;
CREATE TABLE externo_upload (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  origem TEXT NOT NULL, cenario TEXT NOT NULL, id_procedimento TEXT NOT NULL,
  nome TEXT NOT NULL, tamanho INTEGER NOT NULL CHECK (tamanho > 0)
) STRICT;
CREATE TABLE externo_documento (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  origem TEXT NOT NULL, cenario TEXT NOT NULL, id_procedimento TEXT NOT NULL,
  id_serie TEXT NOT NULL, id_upload INTEGER NOT NULL UNIQUE,
  numero_sei TEXT NOT NULL, nome TEXT NOT NULL, tamanho INTEGER NOT NULL,
  FOREIGN KEY (id_upload) REFERENCES externo_upload(id)
) STRICT;
PRAGMA user_version = 5;
