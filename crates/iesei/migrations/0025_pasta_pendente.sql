-- 0025: cenário sintético de pasta ainda não carregada para BDD-342-01.
-- Não representa falha observada nos ZIPs UnB, cuja abertura teve sucesso.
CREATE TABLE sim_pasta_pendente (
    origem TEXT NOT NULL,
    cenario TEXT NOT NULL,
    id_procedimento TEXT NOT NULL,
    indice INTEGER NOT NULL CHECK (indice > 0),
    protocolo_declarado TEXT NOT NULL CHECK (length(protocolo_declarado) > 0),
    PRIMARY KEY (origem, cenario, id_procedimento),
    FOREIGN KEY (origem, cenario, id_procedimento)
        REFERENCES sim_processo (origem, cenario, id_procedimento)
) STRICT;
PRAGMA user_version = 25;
