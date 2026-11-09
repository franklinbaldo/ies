-- 0027: banco derivado efemero; reconstruir listas com varias paginas observadas.
-- Na inicializacao os dados ainda nao foram ingeridos, portanto recriar v26.
DROP TABLE sim_bloco_linha;
DROP TABLE sim_bloco_pagina;
CREATE TABLE sim_bloco_pagina (
    origem TEXT NOT NULL,
    cenario TEXT NOT NULL,
    tipo TEXT NOT NULL CHECK(tipo IN ('assinatura','interno','reuniao')),
    pagina INTEGER NOT NULL CHECK(pagina >= 1),
    total INTEGER NOT NULL CHECK(total >= 0),
    inicio INTEGER,
    fim INTEGER,
    PRIMARY KEY (origem, cenario, tipo, pagina),
    CHECK((inicio IS NULL AND fim IS NULL) OR
          (inicio >= 1 AND fim >= inicio AND fim <= total))
) STRICT;
CREATE TABLE sim_bloco_linha (
    origem TEXT NOT NULL,
    cenario TEXT NOT NULL,
    tipo TEXT NOT NULL,
    pagina INTEGER NOT NULL CHECK(pagina >= 1),
    posicao INTEGER NOT NULL CHECK(posicao >= 0),
    numero TEXT NOT NULL,
    estado TEXT NOT NULL,
    descricao TEXT NOT NULL,
    PRIMARY KEY (origem, cenario, tipo, pagina, posicao),
    UNIQUE (origem, cenario, tipo, pagina, numero),
    FOREIGN KEY (origem, cenario, tipo, pagina)
        REFERENCES sim_bloco_pagina (origem, cenario, tipo, pagina)
) STRICT;
PRAGMA user_version = 27;
