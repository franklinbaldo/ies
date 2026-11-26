-- Catálogo de tipos efetivamente oferecidos pela pesquisa (não pelo cadastro).
CREATE TABLE pesquisa_processo_tipo (
    origem TEXT NOT NULL,
    cenario TEXT NOT NULL,
    unidade TEXT NOT NULL,
    id_tipo TEXT NOT NULL CHECK (id_tipo <> ''),
    nome TEXT NOT NULL CHECK (nome <> ''),
    PRIMARY KEY (origem, cenario, unidade, id_tipo)
) STRICT;

-- Resultado negativo explicitamente registrado para os critérios da busca.
-- Ausência da chave não demonstra ausência de processos.
CREATE TABLE pesquisa_processo_periodo_vazio (
    origem TEXT NOT NULL,
    cenario TEXT NOT NULL,
    unidade TEXT NOT NULL,
    id_tipo TEXT NOT NULL,
    data_inicio TEXT NOT NULL,
    data_fim TEXT NOT NULL,
    CHECK (id_tipo <> '' OR data_inicio <> '' OR data_fim <> ''),
    PRIMARY KEY (origem, cenario, unidade, id_tipo, data_inicio, data_fim)
) STRICT;
PRAGMA user_version = 39;
