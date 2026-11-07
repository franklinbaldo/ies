-- 0002: retornos mensais extraídos do SEI, com mês e situação observados.
-- Banco descartável; recriado do ZIP imutável em cada ensaio.
CREATE TABLE retorno_mes (
    origem TEXT NOT NULL,
    cenario TEXT NOT NULL,
    data_inicial TEXT NOT NULL,
    data_final TEXT NOT NULL,
    PRIMARY KEY (origem, cenario, data_inicial, data_final)
) STRICT;
CREATE TABLE retorno_linha (
    origem TEXT NOT NULL,
    cenario TEXT NOT NULL,
    data_inicial TEXT NOT NULL,
    data_final TEXT NOT NULL,
    posicao INTEGER NOT NULL CHECK (posicao >= 0),
    situacao TEXT NOT NULL CHECK (situacao IN ('aguardando', 'devolver')),
    numero TEXT NOT NULL,
    unidade TEXT NOT NULL,
    data_programada TEXT NOT NULL,
    retornado_em TEXT,
    prazo_restante TEXT,
    PRIMARY KEY (origem, cenario, data_inicial, data_final, posicao),
    FOREIGN KEY (origem, cenario, data_inicial, data_final)
        REFERENCES retorno_mes(origem, cenario, data_inicial, data_final)
) STRICT;
PRAGMA user_version = 2;
