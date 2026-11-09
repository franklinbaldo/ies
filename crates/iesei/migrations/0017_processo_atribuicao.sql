-- 0017: atribuição observada por unidade e opções oferecidas da tela.
-- Registro explícito distingue ausência de observação de atribuição vazia.
CREATE TABLE processo_atribuicao_observada (
    origem TEXT NOT NULL,
    cenario TEXT NOT NULL,
    id_procedimento TEXT NOT NULL,
    unidade TEXT NOT NULL,
    login_atual TEXT NOT NULL,
    PRIMARY KEY (origem, cenario, id_procedimento, unidade),
    FOREIGN KEY (origem, cenario, id_procedimento)
        REFERENCES sim_processo(origem, cenario, id_procedimento)
) STRICT;
CREATE TABLE processo_atribuicao_opcao (
    origem TEXT NOT NULL,
    cenario TEXT NOT NULL,
    id_procedimento TEXT NOT NULL,
    unidade TEXT NOT NULL,
    id_usuario TEXT NOT NULL,
    login TEXT NOT NULL,
    nome TEXT NOT NULL,
    PRIMARY KEY (origem, cenario, id_procedimento, unidade, id_usuario),
    FOREIGN KEY (origem, cenario, id_procedimento, unidade)
        REFERENCES processo_atribuicao_observada(origem, cenario, id_procedimento, unidade)
) STRICT;
PRAGMA user_version = 17;
