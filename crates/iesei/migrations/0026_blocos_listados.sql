-- 0026: blocos de uma pagina observada, por instalacao, cenario e tipo.
-- O intervalo e o total sao independentes da contagem de linhas presentes.
CREATE TABLE sim_bloco_pagina (
    origem TEXT NOT NULL,
    cenario TEXT NOT NULL,
    tipo TEXT NOT NULL CHECK(tipo IN ('assinatura','interno','reuniao')),
    total INTEGER NOT NULL CHECK(total >= 0),
    inicio INTEGER,
    fim INTEGER,
    PRIMARY KEY (origem, cenario, tipo),
    CHECK((inicio IS NULL AND fim IS NULL) OR
          (inicio >= 1 AND fim >= inicio AND fim <= total))
) STRICT;
CREATE TABLE sim_bloco_linha (
    origem TEXT NOT NULL,
    cenario TEXT NOT NULL,
    tipo TEXT NOT NULL,
    posicao INTEGER NOT NULL CHECK(posicao >= 0),
    numero TEXT NOT NULL,
    estado TEXT NOT NULL,
    descricao TEXT NOT NULL,
    PRIMARY KEY (origem, cenario, tipo, posicao),
    UNIQUE (origem, cenario, tipo, numero),
    FOREIGN KEY (origem, cenario, tipo)
        REFERENCES sim_bloco_pagina (origem, cenario, tipo)
) STRICT;
PRAGMA user_version = 26;
