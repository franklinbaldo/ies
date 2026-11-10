-- 0031: observação explícita dos marcadores por processo e unidade.
-- Ausência da linha de observação NÃO é lista vazia. Sem escrever no SEI.
CREATE TABLE processo_marcadores_observados (
    origem TEXT NOT NULL,
    cenario TEXT NOT NULL,
    id_procedimento TEXT NOT NULL,
    unidade TEXT NOT NULL,
    total_declarado INTEGER NOT NULL CHECK (total_declarado >= 0),
    PRIMARY KEY (origem, cenario, id_procedimento, unidade),
    FOREIGN KEY (origem, cenario, id_procedimento)
        REFERENCES sim_processo(origem, cenario, id_procedimento)
) STRICT;
CREATE TABLE processo_marcador_linha (
    origem TEXT NOT NULL,
    cenario TEXT NOT NULL,
    id_procedimento TEXT NOT NULL,
    unidade TEXT NOT NULL,
    posicao INTEGER NOT NULL CHECK (posicao >= 0),
    id_marcador TEXT NOT NULL,
    nome TEXT NOT NULL,
    cor TEXT NOT NULL,
    texto TEXT,
    usuario TEXT,
    data_hora TEXT NOT NULL,
    PRIMARY KEY (origem, cenario, id_procedimento, unidade, posicao),
    UNIQUE (origem, cenario, id_procedimento, unidade, id_marcador),
    FOREIGN KEY (origem, cenario, id_procedimento, unidade)
        REFERENCES processo_marcadores_observados(origem, cenario, id_procedimento, unidade)
) STRICT;
PRAGMA user_version = 31;
