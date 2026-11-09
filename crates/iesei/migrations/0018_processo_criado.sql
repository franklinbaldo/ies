-- 0018: identidade e propriedades da criacao confirmada no banco descartavel.
-- Dados de ensaio, nunca persistidos no indice operacional do IES.
CREATE TABLE processo_criado (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  origem TEXT NOT NULL,
  cenario TEXT NOT NULL,
  id_procedimento TEXT UNIQUE,
  id_tipo TEXT NOT NULL,
  especificacao TEXT NOT NULL,
  nivel INTEGER NOT NULL CHECK(nivel = 0),
  FOREIGN KEY (origem, cenario, id_tipo)
    REFERENCES tipo_processo_simulado(origem, cenario, id_tipo)
) STRICT;
PRAGMA user_version = 18;
