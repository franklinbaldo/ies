-- 0012: ausência explicitamente observada por termo e unidade na pesquisa documental.
-- Uma busca desconhecida NÃO equivale a resultado vazio.
CREATE TABLE pesquisa_documento_sem_resultado (
    origem TEXT NOT NULL,
    cenario TEXT NOT NULL,
    unidade TEXT NOT NULL,
    termo TEXT NOT NULL,
    PRIMARY KEY (origem, cenario, unidade, termo)
) STRICT;
PRAGMA user_version = 12;
