-- 0016: tipos e assuntos do cadastro de processos observados no cenário.
-- Lista conhecida somente quando há entradas explícitas; não inferir lista vazia.
CREATE TABLE tipo_processo_simulado (
  origem TEXT NOT NULL,
  cenario TEXT NOT NULL,
  id_tipo TEXT NOT NULL,
  nome TEXT NOT NULL,
  id_assunto TEXT NOT NULL,
  assunto TEXT NOT NULL,
  PRIMARY KEY (origem, cenario, id_tipo)
) STRICT;
PRAGMA user_version = 16;
