-- 0006: trilhas de processos, unidades abertas e destinos oferecidos no anti-IES.
-- Dados sintéticos por instalação/cenário, sem credenciais nem índice operacional.
CREATE TABLE processo_simulado (
    origem TEXT NOT NULL,
    cenario TEXT NOT NULL,
    id_procedimento TEXT NOT NULL,
    anotacao_origem TEXT,
    PRIMARY KEY (origem, cenario, id_procedimento),
    FOREIGN KEY (origem, cenario, id_procedimento)
        REFERENCES sim_processo(origem, cenario, id_procedimento)
) STRICT;
CREATE TABLE processo_abertura (
    origem TEXT NOT NULL,
    cenario TEXT NOT NULL,
    id_procedimento TEXT NOT NULL,
    posicao INTEGER NOT NULL CHECK (posicao >= 0),
    unidade TEXT NOT NULL,
    PRIMARY KEY (origem, cenario, id_procedimento, posicao),
    UNIQUE (origem, cenario, id_procedimento, unidade),
    FOREIGN KEY (origem, cenario, id_procedimento)
        REFERENCES processo_simulado(origem, cenario, id_procedimento)
) STRICT;
CREATE TABLE processo_destino (
    origem TEXT NOT NULL,
    cenario TEXT NOT NULL,
    id_procedimento TEXT NOT NULL,
    id_unidade TEXT NOT NULL,
    sigla TEXT NOT NULL,
    descricao TEXT NOT NULL,
    PRIMARY KEY (origem, cenario, id_procedimento, id_unidade),
    UNIQUE (origem, cenario, id_procedimento, sigla),
    FOREIGN KEY (origem, cenario, id_procedimento)
        REFERENCES processo_simulado(origem, cenario, id_procedimento)
) STRICT;
PRAGMA user_version = 6;
