-- 0013: efeito de cancelamento de documento no estado sintético compartilhado (#273).
-- Nunca guarda cookie, token, URL assinada ou conteúdo privado do SEI.
CREATE TABLE documento_cancelamento (
    origem TEXT NOT NULL,
    cenario TEXT NOT NULL,
    id_documento TEXT NOT NULL,
    id_procedimento TEXT NOT NULL,
    motivo TEXT NOT NULL,
    PRIMARY KEY (origem, cenario, id_documento),
    FOREIGN KEY (origem, cenario, id_documento)
        REFERENCES documento_simulado (origem, cenario, id_documento)
) STRICT;
PRAGMA user_version = 13;
