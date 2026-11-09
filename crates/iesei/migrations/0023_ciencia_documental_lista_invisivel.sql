-- 0023: visibilidade eventual da Lista de Ciências no ensaio do BDD-271-03.
-- O GET de ciência pode ter gravado a atividade enquanto a lista ainda não a publica.
-- Só estado sintético; não representa uma instalação real nem apaga a ciência.
CREATE TABLE ciencia_documental_lista_invisivel (
    origem TEXT NOT NULL,
    cenario TEXT NOT NULL,
    id_documento TEXT NOT NULL,
    PRIMARY KEY (origem, cenario, id_documento),
    FOREIGN KEY (origem, cenario, id_documento)
        REFERENCES documento_simulado (origem, cenario, id_documento)
) STRICT;
PRAGMA user_version = 23;
