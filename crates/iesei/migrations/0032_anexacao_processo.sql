-- 0032: anexações simuladas, dirigidas pelo mesmo estado de processos.
-- Ninguém pode ser anexado a si próprio nem pertencer a dois principais.
CREATE TABLE sim_anexacao (
    origem TEXT NOT NULL,
    cenario TEXT NOT NULL,
    id_principal TEXT NOT NULL,
    id_anexado TEXT NOT NULL,
    PRIMARY KEY (origem, cenario, id_anexado),
    CHECK (id_principal != id_anexado),
    FOREIGN KEY (origem, cenario, id_principal)
        REFERENCES sim_processo(origem, cenario, id_procedimento),
    FOREIGN KEY (origem, cenario, id_anexado)
        REFERENCES sim_processo(origem, cenario, id_procedimento)
) STRICT;
CREATE INDEX sim_anexacao_principal ON sim_anexacao (origem, cenario, id_principal);
PRAGMA user_version = 32;
