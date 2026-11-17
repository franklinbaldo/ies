-- 0035: fotografia explícita dos acompanhamentos especiais por processo/unidade.
-- Não confundir ausência de observação com tabela observada vazia.
CREATE TABLE processo_acompanhamentos_observados (
    origem TEXT NOT NULL,
    cenario TEXT NOT NULL,
    id_procedimento TEXT NOT NULL,
    unidade TEXT NOT NULL,
    PRIMARY KEY (origem, cenario, id_procedimento, unidade),
    FOREIGN KEY (origem, cenario, id_procedimento)
        REFERENCES sim_processo(origem, cenario, id_procedimento)
) STRICT;
CREATE TABLE processo_acompanhamento_linha (
    origem TEXT NOT NULL,
    cenario TEXT NOT NULL,
    id_procedimento TEXT NOT NULL,
    unidade TEXT NOT NULL,
    posicao INTEGER NOT NULL CHECK (posicao >= 0),
    id_acompanhamento TEXT NOT NULL,
    grupo TEXT NOT NULL,
    observacao TEXT NOT NULL,
    PRIMARY KEY (origem, cenario, id_procedimento, unidade, posicao),
    UNIQUE (origem, cenario, id_procedimento, unidade, id_acompanhamento),
    FOREIGN KEY (origem, cenario, id_procedimento, unidade)
        REFERENCES processo_acompanhamentos_observados(origem, cenario, id_procedimento, unidade)
) STRICT;
PRAGMA user_version = 35;
