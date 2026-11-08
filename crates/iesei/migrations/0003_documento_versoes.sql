-- 0003: versões de documento de um cenário de treinamento simulado.
-- Nada de sessão, URLs assinadas ou índice operacional do IES.
CREATE TABLE documento_simulado (
    origem TEXT NOT NULL,
    cenario TEXT NOT NULL,
    id_documento TEXT NOT NULL,
    numero TEXT NOT NULL,
    tipo TEXT NOT NULL,
    PRIMARY KEY (origem, cenario, id_documento)
) STRICT;
CREATE TABLE documento_revisao (
    origem TEXT NOT NULL,
    cenario TEXT NOT NULL,
    id_documento TEXT NOT NULL,
    versao INTEGER NOT NULL CHECK (versao > 0),
    usuario TEXT NOT NULL,
    unidade TEXT NOT NULL,
    modificacao TEXT NOT NULL,
    html_corpo TEXT NOT NULL,
    PRIMARY KEY (origem, cenario, id_documento, versao),
    FOREIGN KEY (origem, cenario, id_documento)
        REFERENCES documento_simulado(origem, cenario, id_documento)
) STRICT;
PRAGMA user_version = 3;
