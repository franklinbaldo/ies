-- 0030: sobrestamento e motivo como estado do processo no anti-IES.
-- Nenhuma URL/cookie: transição reversível da mesma identidade.
CREATE TABLE sim_sobrestamento (
    origem TEXT NOT NULL,
    cenario TEXT NOT NULL,
    id_procedimento TEXT NOT NULL,
    unidade TEXT NOT NULL,
    motivo TEXT NOT NULL CHECK (length(trim(motivo)) > 0),
    PRIMARY KEY (origem, cenario, id_procedimento),
    FOREIGN KEY (origem, cenario, id_procedimento)
        REFERENCES processo_simulado(origem, cenario, id_procedimento)
) STRICT;
PRAGMA user_version = 30;
