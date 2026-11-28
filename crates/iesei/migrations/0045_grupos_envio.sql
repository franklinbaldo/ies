-- Hipótese do anti-IES: catálogo integral explicitamente semeado por unidade.
-- Ausência do catálogo não representa catálogo vazio.
CREATE TABLE sim_grupo_envio_catalogo (
    origem TEXT NOT NULL,
    cenario TEXT NOT NULL,
    unidade TEXT NOT NULL,
    proximo_id INTEGER NOT NULL CHECK (proximo_id > 0),
    PRIMARY KEY (origem, cenario, unidade)
) STRICT;
CREATE TABLE sim_grupo_envio (
    origem TEXT NOT NULL,
    cenario TEXT NOT NULL,
    unidade TEXT NOT NULL,
    id_grupo TEXT NOT NULL CHECK (id_grupo <> ''),
    nome TEXT NOT NULL CHECK (nome <> ''),
    descricao TEXT NOT NULL,
    PRIMARY KEY (origem, cenario, unidade, id_grupo),
    FOREIGN KEY (origem, cenario, unidade) REFERENCES sim_grupo_envio_catalogo
) STRICT;
CREATE TABLE sim_grupo_envio_membro (
    origem TEXT NOT NULL,
    cenario TEXT NOT NULL,
    unidade TEXT NOT NULL,
    id_grupo TEXT NOT NULL,
    id_unidade TEXT NOT NULL CHECK (id_unidade <> ''),
    nome TEXT NOT NULL CHECK (nome <> ''),
    posicao INTEGER NOT NULL CHECK (posicao >= 0),
    PRIMARY KEY (origem, cenario, unidade, id_grupo, id_unidade),
    UNIQUE (origem, cenario, unidade, id_grupo, posicao),
    FOREIGN KEY (origem, cenario, unidade, id_grupo) REFERENCES sim_grupo_envio
) STRICT;
-- A oferta do autocomplete é uma observação separada dos membros dos grupos.
CREATE TABLE sim_grupo_envio_unidade (
    origem TEXT NOT NULL,
    cenario TEXT NOT NULL,
    unidade TEXT NOT NULL,
    id_unidade TEXT NOT NULL CHECK (id_unidade <> ''),
    nome TEXT NOT NULL CHECK (nome <> ''),
    PRIMARY KEY (origem, cenario, unidade, id_unidade),
    FOREIGN KEY (origem, cenario, unidade) REFERENCES sim_grupo_envio_catalogo
) STRICT;
PRAGMA user_version = 45;
