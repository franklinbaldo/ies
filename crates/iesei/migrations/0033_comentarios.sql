-- 0033: comentários explicitamente observados; ausência NÃO significa lista vazia.
-- A observação pode ser parcial; nenhuma tabela declara completude por inferência.
CREATE TABLE processo_comentarios_observados (
    origem TEXT NOT NULL,
    cenario TEXT NOT NULL,
    id_procedimento TEXT NOT NULL,
    visao TEXT NOT NULL CHECK (visao IN ('P', 'T')),
    total_declarado INTEGER NOT NULL CHECK (total_declarado >= 0),
    PRIMARY KEY (origem, cenario, id_procedimento),
    FOREIGN KEY (origem, cenario, id_procedimento)
        REFERENCES sim_processo(origem, cenario, id_procedimento)
) STRICT;
CREATE TABLE processo_comentario_linha (
    origem TEXT NOT NULL,
    cenario TEXT NOT NULL,
    id_procedimento TEXT NOT NULL,
    posicao INTEGER NOT NULL CHECK (posicao >= 0),
    id_comentario TEXT NOT NULL,
    data_hora TEXT NOT NULL,
    unidade TEXT NOT NULL,
    usuario TEXT NOT NULL,
    texto TEXT NOT NULL,
    protocolo TEXT,
    tipo TEXT,
    PRIMARY KEY (origem, cenario, id_procedimento, posicao),
    UNIQUE (origem, cenario, id_procedimento, id_comentario),
    FOREIGN KEY (origem, cenario, id_procedimento)
        REFERENCES processo_comentarios_observados(origem, cenario, id_procedimento)
) STRICT;
PRAGMA user_version = 33;
