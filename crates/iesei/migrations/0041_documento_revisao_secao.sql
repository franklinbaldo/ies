-- 0041: seções opacas da revisão do editor clássico simulado.
-- Nenhuma linha para uma revisão legada: não representa uma seção vazia.
CREATE TABLE documento_revisao_secao (
    origem TEXT NOT NULL,
    cenario TEXT NOT NULL,
    id_documento TEXT NOT NULL,
    versao INTEGER NOT NULL,
    posicao INTEGER NOT NULL CHECK (posicao >= 0),
    nome TEXT NOT NULL,
    principal INTEGER NOT NULL CHECK (principal IN (0, 1)),
    html TEXT NOT NULL,
    PRIMARY KEY (origem, cenario, id_documento, versao, nome),
    UNIQUE (origem, cenario, id_documento, versao, posicao),
    FOREIGN KEY (origem, cenario, id_documento, versao)
        REFERENCES documento_revisao(origem, cenario, id_documento, versao)
) STRICT;
CREATE UNIQUE INDEX documento_revisao_secao_principal
    ON documento_revisao_secao(origem, cenario, id_documento, versao)
    WHERE principal = 1;
-- O banco efêmero também é aberto por clientes SQLite com FK desativada.
-- A reimportação de uma revisão divergente remove sua estrutura antiga.
CREATE TRIGGER documento_revisao_limpar_secoes
    AFTER DELETE ON documento_revisao
    BEGIN
        DELETE FROM documento_revisao_secao
        WHERE origem=OLD.origem AND cenario=OLD.cenario
          AND id_documento=OLD.id_documento AND versao=OLD.versao;
    END;
PRAGMA user_version = 41;
