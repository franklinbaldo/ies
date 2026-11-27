-- 0040: catálogo sintético de texto padrão, por instalação, cenário e unidade.
-- A resposta original do autocomplete ainda não foi capturada.
CREATE TABLE sim_texto_padrao (
  origem TEXT NOT NULL,
  cenario TEXT NOT NULL,
  unidade TEXT NOT NULL,
  id_texto TEXT NOT NULL,
  nome TEXT NOT NULL,
  PRIMARY KEY (origem, cenario, unidade, id_texto)
) STRICT;
PRAGMA user_version = 40;
