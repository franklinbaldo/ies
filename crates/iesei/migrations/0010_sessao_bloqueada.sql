-- 0010: acesso negado explicitamente à sessão sintética inteira.
-- Não armazenar credenciais de autenticação ou sessões reais.
CREATE TABLE sessao_bloqueada (
    origem TEXT NOT NULL,
    cenario TEXT NOT NULL,
    PRIMARY KEY (origem, cenario)
) STRICT;
PRAGMA user_version = 10;
