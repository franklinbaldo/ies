-- 0007: oferta de edição observada pelo simulador; versões ficam em documento_revisao.
-- Assinatura não é autorização: a oferta de edição é um fato independente.
CREATE TABLE documento_editor (
    origem TEXT NOT NULL,
    cenario TEXT NOT NULL,
    id_documento TEXT NOT NULL,
    editavel INTEGER NOT NULL CHECK (editavel IN (0, 1)),
    assinaturas INTEGER NOT NULL CHECK (assinaturas >= 0),
    PRIMARY KEY (origem, cenario, id_documento),
    FOREIGN KEY (origem, cenario, id_documento)
        REFERENCES documento_simulado(origem, cenario, id_documento)
) STRICT;
PRAGMA user_version = 7;
