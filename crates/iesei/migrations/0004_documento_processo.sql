-- 0004: NUP do processo para a árvore do documento do anti-IES.
-- Banco derivado efêmero; não é a migração do índice operacional.
ALTER TABLE documento_simulado ADD COLUMN numero_processo TEXT NOT NULL DEFAULT '';
PRAGMA user_version = 4;
