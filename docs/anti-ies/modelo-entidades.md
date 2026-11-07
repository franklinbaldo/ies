---
type: Guide
title: Modelo inicial de entidades do anti-IES
description: Migração de identidade e representação tipada da cobertura observada
---

# Modelo inicial — anti-IES

- **Identidade:** `iesei::instalacao::Instalacao` e `VersaoSei`
  foram movidas de `ies-core` sem alterar a interpretação de configurações.
- **Observação:** `UnidadeExibida`, `FormaDoControle` R/D,
  `Cobertura<T>` e `TabelaObservada<T>` distinguem dados observados,
  desconhecidos, total declarado e faixa conhecida.
- **Proveniência:** `Proveniencia::LeituraDaUnidadeAtual` distingue a
  operação observadora de `Proveniencia::Cassette { cartao }`.
- **Ausência:** `None` é não observado, não total zero. Tabelas
  não verificáveis não podem ser oferecidas como Controle válido.
- **Próxima fatia:** #358 converte somente fatos que o IES já leu
  para SQLite, isolando dados de sessão. #354 fará SELECT tipado e
  renderização somente a partir de fatos comprovados.

Casos AIES-01..04 definidos previamente na [#355](https://github.com/franklinbaldo/ies/issues/355).
Esta PR não implementa nem executa o E2E: `NOT_RUN`. Não cria flags,
rotas, consultas Web ou dependências de transporte.

## Projeção de unidade observada (#358, primeira fatia)

A migração SQLite 3→4 acrescenta `controle_unidade` com chave por
instalação e **geração de sessão já derivada**, sem gravar cookie nem URL.
O leitor do Controle só alimenta essa tabela após reconhecer a estrutura R/D.
O `iesei::sqlite` oferece registro e consulta tipados; ausência de registro
retorna `Option::None`, sem inventar lista, total ou unidade.

**Limite intencional:** a migração ainda não persiste as listas de processos,
faixas e totais para renderização completa pelo anti-IES. A etapa #354 deve
recusar dados insuficientes, não apresentar 200 fictício. BDD AIES-01..04
permanecem `NOT_RUN` no E2E.

## Próxima etapa — banco derivado para compatibilidade

O banco de teste reconstruído das cassettes **não é** o índice
operacional `ies.sqlite3`. O primeiro tem esquema SQL explícito,
versionado e evolutivo; o segundo conserva suas migrações próprias.
Ao acrescentar entidades ou tabelas, evoluir tipos e SQL junto com
o parser e a resposta do anti-IES na mesma PR de ciclo fechado.
Ver o [contrato do ciclo de cassettes](ciclo-cassettes.md).


## Primeira migração do banco efêmero para Controle R

O esquema declarado \`crates/iesei/migrations/0001_controle_resumido.sql\`
é independente do índice operacional \`ies.sqlite3\`: tabelas
\`controle_resumido\` (unidade, totais e faixas conhecidos) e
\`controle_linha\` (processos observados, categoria e posição).
\`PRAGMA user_version=1\`; um novo ensaio recria o banco.
O IES extrai o snapshot pelo parser de produção da cassette conferida,
sem gravar URLs ou cookies. O anti-IES só consulta por origem e cenário
e serve Controle R quando os dados são suficientes.

O replay UnB verifica de forma offline
\`cassette → leitura IES → SQLite → HTTP anti-IES → leitura IES\`.
A integração do **processo CLI completo** por sessão HTTPS segue
pendente em #355. Não confundir leitor de produção exercitado no
replay com E2E de CLI concluído.
