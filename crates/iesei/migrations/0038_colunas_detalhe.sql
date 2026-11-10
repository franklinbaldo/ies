-- 0038: configuração de colunas oferecida na tela do Nível de detalhe.
-- A falta de linhas significa observação ausente, nunca configuração vazia.
CREATE TABLE colunas_detalhe (
    origem TEXT NOT NULL,
    cenario TEXT NOT NULL,
    unidade TEXT NOT NULL,
    posicao INTEGER NOT NULL CHECK (posicao >= 0),
    coluna TEXT NOT NULL,
    marcada INTEGER NOT NULL CHECK (marcada IN (0, 1)),
    PRIMARY KEY (origem, cenario, posicao),
    UNIQUE (origem, cenario, coluna)
) STRICT;
PRAGMA user_version = 38;
