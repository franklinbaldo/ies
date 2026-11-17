-- 0001: Controle resumido R observado; nenhum cookie, URL ou hash.
CREATE TABLE controle_resumido (
    origem TEXT NOT NULL,
    cenario TEXT NOT NULL,
    unidade TEXT NOT NULL,
    recebidos_total INTEGER CHECK (recebidos_total >= 0),
    recebidos_inicio INTEGER CHECK (recebidos_inicio > 0),
    recebidos_fim INTEGER CHECK (recebidos_fim >= recebidos_inicio),
    gerados_total INTEGER CHECK (gerados_total >= 0),
    gerados_inicio INTEGER CHECK (gerados_inicio > 0),
    gerados_fim INTEGER CHECK (gerados_fim >= gerados_inicio),
    PRIMARY KEY (origem, cenario)
) STRICT;
CREATE TABLE controle_linha (
    origem TEXT NOT NULL,
    cenario TEXT NOT NULL,
    categoria TEXT NOT NULL CHECK (categoria IN ('recebidos', 'gerados')),
    posicao INTEGER NOT NULL CHECK (posicao >= 0),
    id TEXT NOT NULL,
    numero TEXT NOT NULL,
    visualizado INTEGER NOT NULL CHECK (visualizado IN (0, 1)),
    sigiloso INTEGER NOT NULL CHECK (sigiloso IN (0, 1)),
    PRIMARY KEY (origem, cenario, categoria, posicao)
) STRICT;
PRAGMA user_version = 1;
