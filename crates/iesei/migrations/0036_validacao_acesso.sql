-- 0036: estado tipado da exigencia de revalidacao de senha por processo.
-- Ausencia da linha e desconhecimento nao equivalem a acesso liberado.
-- Hipotese sintetica BDD-633: formulario/POST ainda nao capturados.
CREATE TABLE processo_validacao_acesso (
    origem TEXT NOT NULL,
    cenario TEXT NOT NULL,
    id_procedimento TEXT NOT NULL,
    situacao TEXT NOT NULL CHECK (situacao IN ('exigida', 'realizada')),
    PRIMARY KEY (origem, cenario, id_procedimento),
    FOREIGN KEY (origem, cenario, id_procedimento)
        REFERENCES sim_processo(origem, cenario, id_procedimento)
) STRICT;
PRAGMA user_version = 36;
