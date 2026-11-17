---
type: SEIObservation
title: "Ensaio #245 — interessados explícitos na criação de processo"
description: "RED sintético, conferência de interessados na prévia/POST e pendência de validação real"
operation: processo.criar
date: 2026-11-07
status: not-run
---

# 🦉 #245 — interessado explícito no ensaio

## Observação e lacuna

A CLI Web já recebe `ies processo criar --interessado NOME` (repetível)
e publica `interessados` nas saídas JSON de prévia e confirmação
(`src/cli_processo.rs`, `src/processo_criar_cli/apresentacao.rs`).
O fluxo Rust resolve os nomes observados e o classificador pós-POST
compara o multiconjunto, sem atribuir identidade à ordem
(`src/processo_criar/desfechos.rs`).

O ensaio manual `scripts/ensaio_processo_criar.py` não recebia esse
argumento: não havia como exercer a capacidade opcional pelo roteiro
seguro de prévia → uma confirmação → releitura independente.

## Testes e correção

- RED test-first no commit `16ed8e0`, CI
  [42372535834](https://github.com/franklinbaldo/ies/actions/runs/42372535834):
  replay Python com **30 testes, 3 ERROR novos** (o roteiro rejeitava
  `interessados`).
- `--interessado` repetível agora é transmitido **igualmente** à prévia
  e à confirmação. A resposta JSON deve publicar os mesmos nomes
  explícitos, com multiplicidade preservada, qualquer que seja a ordem.
- Interessado divergente na prévia impede o POST. Divergência após
  confirmação conserva o NUP e realiza a releitura independente antes de
  devolver saída 3, sem declarar `PASS`.
- O CI oficial do head final da PR é a prova de GREEN. Estes testes
  usam respostas sintéticas, não sessões autenticadas do SEI.

## Live

**NOT_RUN:** nenhuma prévia autenticada executada nesta fatia, nenhum
`--confirmar`, nenhuma releitura real. **0 requisições reais ao SEI,
0 POST, 0 escritas e 0 resíduos novos**. Sem sessão autenticada segura
disponível neste ambiente para completar o ciclo em uma segunda
instalação de TREINAMENTO. Não houve acesso à produção nem repetição da
criação histórica inconclusiva da UnB.

A #245 permanece WIP até prova causal independente de um POST único,
com NUP/resíduo e contagem **real** de requisições observadas.
