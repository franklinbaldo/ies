-- Catálogo de gestão: uma testemunha por ordinal, não opções de atribuição.
CREATE TABLE unidade_gestao_marcadores (
    origem TEXT NOT NULL,
    cenario TEXT NOT NULL,
    unidade TEXT NOT NULL,
    ordinal INTEGER NOT NULL CHECK (ordinal >= 0),
    total_declarado INTEGER NOT NULL CHECK (total_declarado >= 0),
    inicio INTEGER,
    fim INTEGER,
    CHECK (
        (total_declarado = 0 AND inicio IS NULL AND fim IS NULL)
        OR (total_declarado > 0 AND inicio IS NOT NULL AND fim IS NOT NULL
            AND inicio >= 1 AND fim >= inicio AND fim <= total_declarado)
    ),
    PRIMARY KEY (origem, cenario, unidade, ordinal)
) STRICT;
CREATE TABLE unidade_gestao_marcador_linha (
    origem TEXT NOT NULL,
    cenario TEXT NOT NULL,
    unidade TEXT NOT NULL,
    ordinal INTEGER NOT NULL,
    posicao INTEGER NOT NULL CHECK (posicao >= 0),
    id_marcador TEXT NOT NULL CHECK (id_marcador <> '' AND id_marcador NOT GLOB '*[^0-9]*'),
    nome TEXT NOT NULL CHECK (length(trim(nome)) > 0),
    cor TEXT NOT NULL CHECK (length(cor) > 0),
    estado TEXT NOT NULL CHECK (estado IN ('ativo', 'inativo')),
    PRIMARY KEY (origem, cenario, unidade, ordinal, posicao),
    UNIQUE (origem, cenario, unidade, ordinal, id_marcador),
    FOREIGN KEY (origem, cenario, unidade, ordinal)
        REFERENCES unidade_gestao_marcadores(origem, cenario, unidade, ordinal)
) STRICT;
PRAGMA user_version = 43;
