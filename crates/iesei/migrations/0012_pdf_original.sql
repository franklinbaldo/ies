-- Shape v12: bytes do PDF original enviados ao anti-IES, só no SQLite efêmero.
-- Sem cookies, credenciais nem URLs reais de sessão.
ALTER TABLE externo_upload ADD COLUMN conteudo BLOB;
PRAGMA user_version = 12;
