-- 0037: estado explicito de recusa do servidor em validacao de acesso.
-- Banco efemero; nenhum segredo ou senha e persistido.
ALTER TABLE processo_validacao_acesso RENAME TO processo_validacao_acesso_anterior;
CREATE TABLE processo_validacao_acesso (
    origem TEXT NOT NULL,
    cenario TEXT NOT NULL,
    id_procedimento TEXT NOT NULL,
    situacao TEXT NOT NULL CHECK (situacao IN ('exigida', 'recusada', 'realizada')),
    PRIMARY KEY (origem, cenario, id_procedimento),
    FOREIGN KEY (origem, cenario, id_procedimento)
        REFERENCES sim_processo(origem, cenario, id_procedimento)
) STRICT;
INSERT INTO processo_validacao_acesso
    SELECT origem, cenario, id_procedimento, situacao
    FROM processo_validacao_acesso_anterior;
DROP TABLE processo_validacao_acesso_anterior;
PRAGMA user_version = 37;
