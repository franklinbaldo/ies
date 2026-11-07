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
