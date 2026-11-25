---
type: Guide
title: Oráculo sintético para unidade atual — visões R e D
description: Matriz de entradas e hipóteses falsificáveis do leitor do Controle
---

# Oráculo estrutural de unidade.atual

Esta fatia da [#353](https://github.com/franklinbaldo/ies/issues/353)
acrescenta **testes sintéticos** ao leitor de produção já existente
(`src/controle_reconhecido/tests.rs`). Os testes não são prova de
equivalência HTTP com o SEI, nem substituem hashes de cassettes originais.

## Cobertura inicial

- 64 combinações **determinísticas** de atributos e espaços
  cosméticos na visão R completa e vazia, mantendo a unidade exibida.
- Verificação da unidade na visão D em páginas com início diferente.
- Exclusão da tabela obrigatória, retirada do marcador da unidade e
  tela de login disfarçada: **a leitura deve recusar**.

Os exemplos partem da estrutura R/D já observada e dos cenários
em `src/controle_reconhecido/tests.rs`. O oráculo independente
permanece em [cassettes](../cassettes.md) e no
[contrato](../../contracts/cli/unidade-atual.md).

O teste de 64 combinações não é `proptest` e não faz shrinking:
é uma regressão estrutural pequena. O harness futuro
[#361](https://github.com/franklinbaldo/ies/issues/361)
terá estratégias geradoras, contraexemplos mínimos e resposta HTTP
do anti-IES. Antes de marcar ORÁCULO concluído, recuperar os ZIPs
originais e validar hashes e formas R/D; isso continua **NOT_RUN**
nesta fatia. Não há GET/POST a SEI, credenciais, nem escrita.
