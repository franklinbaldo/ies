-- Histórico observado por processo: existência da observação é distinta de ausência.
CREATE TABLE andamento_historico (
    origem TEXT NOT NULL,
    cenario TEXT NOT NULL,
    id_procedimento TEXT NOT NULL,
    total_declarado INTEGER NOT NULL CHECK (total_declarado > 0),
    PRIMARY KEY (origem, cenario, id_procedimento)
) STRICT;

CREATE TABLE andamento_linha (
    origem TEXT NOT NULL,
    cenario TEXT NOT NULL,
    id_procedimento TEXT NOT NULL,
    posicao INTEGER NOT NULL CHECK (posicao >= 0),
    ocorrido_em TEXT NOT NULL,
    unidade TEXT NOT NULL,
    usuario TEXT NOT NULL,
    descricao TEXT NOT NULL,
    PRIMARY KEY (origem, cenario, id_procedimento, posicao),
    FOREIGN KEY (origem, cenario, id_procedimento)
        REFERENCES andamento_historico (origem, cenario, id_procedimento)
        ON DELETE CASCADE
) STRICT;
PRAGMA user_version = 27;
