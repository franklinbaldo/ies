-- 0003: modelo sintético revisável de documento interno (#246).
-- Estado compartilhado entre lista de tipos, cadastro e árvore local.
CREATE TABLE sim_processo (
    origem TEXT NOT NULL,
    cenario TEXT NOT NULL,
    id_procedimento TEXT NOT NULL,
    nup TEXT NOT NULL,
    unidade TEXT NOT NULL,
    PRIMARY KEY (origem, cenario, id_procedimento)
) STRICT;
CREATE TABLE sim_tipo (
    origem TEXT NOT NULL,
    cenario TEXT NOT NULL,
    id_procedimento TEXT NOT NULL,
    id_serie TEXT NOT NULL,
    nome TEXT NOT NULL,
    PRIMARY KEY (origem, cenario, id_procedimento, id_serie)
) STRICT;
CREATE TABLE sim_documento (
    id_documento INTEGER PRIMARY KEY AUTOINCREMENT,
    origem TEXT NOT NULL,
    cenario TEXT NOT NULL,
    id_procedimento TEXT NOT NULL,
    id_serie TEXT NOT NULL,
    descricao TEXT NOT NULL,
    nivel INTEGER NOT NULL CHECK (nivel = 0)
) STRICT;
PRAGMA user_version = 3;
