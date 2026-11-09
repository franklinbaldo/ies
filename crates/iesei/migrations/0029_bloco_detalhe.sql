-- 0029: itens de um bloco cuja linha foi observada na listagem da unidade.
-- Detalhe nao observado nao equivale a bloco vazio; nao armazenar hrefs de sessao.
CREATE TABLE sim_bloco_detalhe (
    origem TEXT NOT NULL,
    cenario TEXT NOT NULL,
    tipo TEXT NOT NULL CHECK(tipo IN ('assinatura','interno','reuniao')),
    numero TEXT NOT NULL,
    total INTEGER NOT NULL CHECK(total >= 0),
    inicio INTEGER,
    fim INTEGER,
    PRIMARY KEY(origem,cenario,tipo,numero),
    CHECK((inicio IS NULL AND fim IS NULL) OR
          (inicio >= 1 AND fim >= inicio AND fim <= total))
) STRICT;
CREATE TABLE sim_bloco_item (
    origem TEXT NOT NULL,
    cenario TEXT NOT NULL,
    tipo TEXT NOT NULL,
    numero TEXT NOT NULL,
    posicao INTEGER NOT NULL CHECK(posicao >= 0),
    processo TEXT NOT NULL,
    documento TEXT,
    tipo_documento TEXT,
    anotacoes TEXT NOT NULL,
    PRIMARY KEY(origem,cenario,tipo,numero,posicao),
    FOREIGN KEY(origem,cenario,tipo,numero)
        REFERENCES sim_bloco_detalhe(origem,cenario,tipo,numero)
) STRICT;
PRAGMA user_version = 29;
