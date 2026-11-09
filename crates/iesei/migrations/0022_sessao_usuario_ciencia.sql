-- 0022: autoria observável quando o cenário conhece o usuário da sessão.
-- Sem cookies ou login reais. Ausência de sessão conhecida permanece NULL.
CREATE TABLE sessao_usuario (
    origem TEXT NOT NULL,
    cenario TEXT NOT NULL,
    usuario TEXT NOT NULL CHECK (length(trim(usuario)) > 0),
    PRIMARY KEY (origem, cenario)
) STRICT;
ALTER TABLE ciencia_documental ADD COLUMN usuario TEXT;
PRAGMA user_version = 22;
