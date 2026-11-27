-- Catálogo de tipos efetivamente oferecidos pela pesquisa (não pelo cadastro).
CREATE TABLE pesquisa_processo_tipo (
    origem TEXT NOT NULL,
    cenario TEXT NOT NULL,
    unidade TEXT NOT NULL,
    id_tipo TEXT NOT NULL CHECK (id_tipo <> ''),
    nome TEXT NOT NULL CHECK (nome <> ''),
    PRIMARY KEY (origem, cenario, unidade, id_tipo)
) STRICT;

-- Uma página observada por chave de busca. Zero é uma observação explícita.
CREATE TABLE pesquisa_processo_pagina (
    origem TEXT NOT NULL,
    cenario TEXT NOT NULL,
    unidade TEXT NOT NULL,
    id_tipo TEXT NOT NULL,
    data_inicio TEXT NOT NULL,
    data_fim TEXT NOT NULL,
    pagina INTEGER NOT NULL CHECK (pagina >= 1),
    inicio INTEGER NOT NULL CHECK (inicio >= 1),
    fim INTEGER NOT NULL,
    total INTEGER NOT NULL,
    CHECK (id_tipo <> '' OR data_inicio <> '' OR data_fim <> ''),
    CHECK ((total = 0 AND pagina = 1 AND inicio = 1 AND fim = 0)
           OR (total >= fim AND fim >= inicio)),
    PRIMARY KEY (origem, cenario, unidade, id_tipo, data_inicio, data_fim, pagina),
    UNIQUE (origem, cenario, unidade, id_tipo, data_inicio, data_fim, inicio)
) STRICT;

CREATE TABLE pesquisa_processo_pagina_linha (
    origem TEXT NOT NULL,
    cenario TEXT NOT NULL,
    unidade TEXT NOT NULL,
    id_tipo TEXT NOT NULL,
    data_inicio TEXT NOT NULL,
    data_fim TEXT NOT NULL,
    pagina INTEGER NOT NULL,
    posicao INTEGER NOT NULL CHECK (posicao >= 0),
    id_procedimento TEXT NOT NULL,
    PRIMARY KEY (origem, cenario, unidade, id_tipo, data_inicio, data_fim, pagina, posicao),
    UNIQUE (origem, cenario, unidade, id_tipo, data_inicio, data_fim, pagina, id_procedimento),
    FOREIGN KEY (origem, cenario, unidade, id_tipo, data_inicio, data_fim, pagina)
      REFERENCES pesquisa_processo_pagina
) STRICT;
PRAGMA user_version = 41;
