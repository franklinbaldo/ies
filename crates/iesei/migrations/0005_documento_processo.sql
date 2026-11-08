-- 0005: NUP do documento no estado do simulador, depois da 0004 de #246.
-- Banco derivado efêmero; não modificar o índice operacional.
ALTER TABLE documento_simulado ADD COLUMN numero_processo TEXT NOT NULL DEFAULT '';
PRAGMA user_version = 5;
