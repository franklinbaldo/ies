-- 0024: ciência de processo separada da ciência documental no SQLite efêmero.
CREATE TABLE ciencia_processo (
    origem TEXT NOT NULL,
    cenario TEXT NOT NULL,
    id_procedimento TEXT NOT NULL,
    id_atividade TEXT NOT NULL,
    usuario TEXT,
    PRIMARY KEY (origem, cenario, id_procedimento),
    FOREIGN KEY (origem, cenario, id_procedimento)
        REFERENCES sim_processo (origem, cenario, id_procedimento)
) STRICT;
CREATE TABLE ciencia_processo_lista_invisivel (
    origem TEXT NOT NULL,
    cenario TEXT NOT NULL,
    id_procedimento TEXT NOT NULL,
    PRIMARY KEY (origem, cenario, id_procedimento),
    FOREIGN KEY (origem, cenario, id_procedimento)
        REFERENCES sim_processo (origem, cenario, id_procedimento)
) STRICT;
PRAGMA user_version = 24;
