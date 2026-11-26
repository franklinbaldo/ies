-- Resultado negativo explicitamente registrado para um período de busca processual.
-- Ausência da chave não demonstra ausência de processos.
CREATE TABLE pesquisa_processo_periodo_vazio (
    origem TEXT NOT NULL,
    cenario TEXT NOT NULL,
    unidade TEXT NOT NULL,
    data_inicio TEXT NOT NULL,
    data_fim TEXT NOT NULL,
    CHECK (data_inicio <> '' OR data_fim <> ''),
    PRIMARY KEY (origem, cenario, unidade, data_inicio, data_fim)
) STRICT;
PRAGMA user_version = 39;
