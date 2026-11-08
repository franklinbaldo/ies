-- 0008: recusa de acesso a versão, por cenário e instalação.
-- Sem cookies, credenciais ou sessão real; o restante do ensaio é acessível.
CREATE TABLE versao_inacessivel (
    origem TEXT NOT NULL,
    cenario TEXT NOT NULL,
    id_documento TEXT NOT NULL,
    versao INTEGER NOT NULL CHECK (versao > 0),
    PRIMARY KEY (origem, cenario, id_documento, versao)
) STRICT;
PRAGMA user_version = 8;
