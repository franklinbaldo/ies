-- 0019: uma ciência documental comprovável por documento no estado simulado (#271).
-- Banco de ensaio efêmero, sem login, URL assinada, cookies ou sessão real.
CREATE TABLE ciencia_documental (
    origem TEXT NOT NULL,
    cenario TEXT NOT NULL,
    id_documento TEXT NOT NULL,
    id_procedimento TEXT NOT NULL,
    id_atividade TEXT NOT NULL,
    PRIMARY KEY (origem, cenario, id_documento),
    FOREIGN KEY (origem, cenario, id_documento)
        REFERENCES documento_simulado (origem, cenario, id_documento)
) STRICT;
PRAGMA user_version = 19;
