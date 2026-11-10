-- 0033: catálogo independente de marcadores oferecidos pela unidade.
-- Não derivar catálogo a partir das atribuições presentes no processo.
CREATE TABLE unidade_catalogo_marcadores (
    origem TEXT NOT NULL,
    cenario TEXT NOT NULL,
    unidade TEXT NOT NULL,
    total_declarado INTEGER NOT NULL CHECK (total_declarado >= 0),
    PRIMARY KEY (origem, cenario, unidade)
) STRICT;
CREATE TABLE unidade_catalogo_marcador_opcao (
    origem TEXT NOT NULL,
    cenario TEXT NOT NULL,
    unidade TEXT NOT NULL,
    posicao INTEGER NOT NULL CHECK (posicao >= 0),
    id_marcador TEXT NOT NULL,
    nome TEXT NOT NULL,
    cor TEXT NOT NULL,
    PRIMARY KEY (origem, cenario, unidade, posicao),
    UNIQUE (origem, cenario, unidade, id_marcador),
    FOREIGN KEY (origem, cenario, unidade)
        REFERENCES unidade_catalogo_marcadores(origem, cenario, unidade)
) STRICT;
PRAGMA user_version = 33;
