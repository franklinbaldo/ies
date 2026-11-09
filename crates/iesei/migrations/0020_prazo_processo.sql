-- 0020: prazo explicitamente observado no processo simulado (#272).
-- Ausência de linha significa não observado; NULL em prazo observado significa sem prazo.
CREATE TABLE prazo_processo (
    origem TEXT NOT NULL,
    cenario TEXT NOT NULL,
    id_procedimento TEXT NOT NULL,
    vence_em TEXT,
    concluido INTEGER NOT NULL CHECK (concluido IN (0, 1)),
    PRIMARY KEY (origem, cenario, id_procedimento),
    CHECK (concluido = 0 OR vence_em IS NULL),
    FOREIGN KEY (origem, cenario, id_procedimento)
        REFERENCES sim_processo (origem, cenario, id_procedimento)
) STRICT;
PRAGMA user_version = 20;
