-- 0021: prazo por dias é distinto de data de calendário (#272).
ALTER TABLE prazo_processo
ADD COLUMN dias INTEGER CHECK (
    dias IS NULL OR (dias BETWEEN 1 AND 999 AND vence_em IS NULL AND concluido = 0)
);
PRAGMA user_version = 21;
