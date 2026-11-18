---
type: Policy
title: Integração sem revisão prévia e revisão retrospectiva do 🦅
description: Parte do AGENTS.md que define quando uma PR entra com CI verde sem parecer prévio, a ronda retrospectiva do Gavião sobre a main e as exceções com revisão antes do merge
---

# Integração sem revisão prévia e revisão retrospectiva

Parte normativa do [AGENTS.md](../AGENTS.md), com a mesma força das
regras que ficaram lá. Onde as fichas [🦅](../team/gaviao.md),
[🐘](../team/elefante.md) ou [🦡](../team/texugo.md) condicionarem o
merge a parecer prévio do 🦅, prevalece esta página.

## Regra do proprietário (18/11/2026)

1. **Padrão:** a PR com os gates oficiais verdes no head exato (CI
   completo 7/7 e replays aplicáveis, conforme o item 5 do AGENTS.md)
   entra sem esperar revisão. A falta de parecer do 🦅 deixa de ser
   motivo de HOLD. A passagem Draft → Ready que dispara o CI completo
   não depende de LGTM do 🦅.
2. **Revisão retrospectiva:** o 🦅 faz uma ronda diária sobre o que
   entrou na `main`, priorizada por risco, a partir do último merge que
   já revisou. Cada defeito vira issue ou PR de correção com um
   responsável nomeado. O 🦅 deixa de ser porteiro da fila e passa a
   auditar o que está na `main`.

## Exceções com revisão antes do merge

**Exceções determinadas expressamente pelo proprietário em 18/11/2026.**
Nessas classes sensíveis, a PR exige revisão independente do 🦅 **antes**
do merge, no diff e nos arquivos efetivamente integrados:

- credenciais, sessão e escrita no SEI;
- mudança de contrato do projeto (`AGENTS.md` e suas partes extraídas).

## Memórias

Commit que só adiciona arquivo em `team/memories/` na `main` não é
motivo de HOLD nem exige nova sincronização da branch ou novo CI.

## O que não muda

Os gates oficiais, o replay de cassettes no head final, o merge por
squash e os limites de produção, segredos e SEI real continuam como
estão. Integrar sem parecer prévio não autoriza escrita fora do
treinamento.
