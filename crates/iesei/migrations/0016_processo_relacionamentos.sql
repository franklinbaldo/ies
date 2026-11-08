-- 0016: relações observadas entre processos, distintas de ausência de observação.
-- Identidade do processo e cenário já estão em sim_processo (0004).
CREATE TABLE processo_relacionamentos_observados (
    origem TEXT NOT NULL,
    cenario TEXT NOT NULL,
    id_procedimento TEXT NOT NULL,
    PRIMARY KEY (origem, cenario, id_procedimento),
    FOREIGN KEY (origem, cenario, id_procedimento)
        REFERENCES sim_processo(origem, cenario, id_procedimento)
) STRICT;
CREATE TABLE processo_relacao (
    origem TEXT NOT NULL,
    cenario TEXT NOT NULL,
    id_procedimento TEXT NOT NULL,
    id_destino TEXT NOT NULL,
    PRIMARY KEY (origem, cenario, id_procedimento, id_destino),
    FOREIGN KEY (origem, cenario, id_procedimento)
        REFERENCES processo_relacionamentos_observados(origem, cenario, id_procedimento),
    FOREIGN KEY (origem, cenario, id_destino)
        REFERENCES sim_processo(origem, cenario, id_procedimento)
) STRICT;
PRAGMA user_version = 16;
