-- 0015: páginas efetivamente representadas por termo/unidade; ausência não é zero.
-- As linhas referenciam identidades documentais partilhadas com as versões.
CREATE TABLE pesquisa_documento_pagina (
    origem TEXT NOT NULL,
    cenario TEXT NOT NULL,
    unidade TEXT NOT NULL,
    termo TEXT NOT NULL,
    pagina INTEGER NOT NULL CHECK (pagina > 0),
    inicio INTEGER NOT NULL CHECK (inicio > 0),
    fim INTEGER NOT NULL CHECK (fim >= inicio),
    total INTEGER NOT NULL CHECK (total >= fim),
    PRIMARY KEY (origem, cenario, unidade, termo, pagina),
    UNIQUE (origem, cenario, unidade, termo, inicio)
) STRICT;
CREATE TABLE pesquisa_documento_pagina_linha (
    origem TEXT NOT NULL,
    cenario TEXT NOT NULL,
    unidade TEXT NOT NULL,
    termo TEXT NOT NULL,
    pagina INTEGER NOT NULL,
    posicao INTEGER NOT NULL CHECK (posicao >= 0),
    id_documento TEXT NOT NULL,
    rotulo TEXT NOT NULL,
    unidade_linha TEXT,
    inclusao TEXT,
    id_anexo TEXT,
    PRIMARY KEY (origem, cenario, unidade, termo, pagina, posicao),
    FOREIGN KEY (origem, cenario, unidade, termo, pagina)
        REFERENCES pesquisa_documento_pagina(origem, cenario, unidade, termo, pagina),
    FOREIGN KEY (origem, cenario, id_documento)
        REFERENCES documento_simulado(origem, cenario, id_documento)
) STRICT;
PRAGMA user_version = 15;
