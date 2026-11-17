-- 0006: visão detalhada da página ativa, com cobertura parcial explícita.
CREATE TABLE controle_detalhado (
    origem TEXT NOT NULL,
    cenario TEXT NOT NULL,
    unidade TEXT NOT NULL,
    total INTEGER NOT NULL CHECK (total >= 0),
    inicio INTEGER CHECK (inicio > 0),
    fim INTEGER CHECK (fim >= inicio AND fim <= total),
    CHECK ((inicio IS NULL AND fim IS NULL) OR
           (inicio IS NOT NULL AND fim IS NOT NULL)),
    PRIMARY KEY (origem, cenario)
) STRICT;
CREATE TABLE controle_detalhado_linha (
    origem TEXT NOT NULL,
    cenario TEXT NOT NULL,
    posicao INTEGER NOT NULL CHECK (posicao >= 0),
    id TEXT NOT NULL,
    numero TEXT NOT NULL,
    visualizado INTEGER CHECK (visualizado IN (0, 1)),
    sigiloso INTEGER CHECK (sigiloso IN (0, 1)),
    PRIMARY KEY (origem, cenario, posicao)
) STRICT;
PRAGMA user_version = 6;
