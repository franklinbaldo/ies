---
type: SEIObservation
title: "UnB treinamento: candidato de atribuição observado na unidade ativa"
description: "A CLI real encontrou um processo de treinamento atualmente sem atribuição na unidade ativa, após login e leitura única; prévia e escrita não foram executadas"
---

# Processo atribuir — descoberta somente-leitura na UnB (09/11/2026)

**Referência:** [issue #264](https://github.com/franklinbaldo/ies/issues/264), BDD-264-01/02/03 já especificados, [Actions #42690546239](https://github.com/franklinbaldo/ies/actions/runs/42690546239), commit `24ae3751`.

## Observação

O runner `ubuntu-22.04` compilou o binário real do IES com Rust 1.99.1. A origem foi conferida **por igualdade exata** com `https://sei.treinamento.unb.br/sei` e com a tabela oficial de credenciais públicas; a senha e os arquivos de sessão permaneceram privados (`0600`) no runner. Não foram impressos cookies, URLs assinadas, valores de credencial ou HTML.

| Etapa | Resultado observado | Requisições |
| --- | --- | --- |
| Compilação Rust release | PASS | N/A |
| `ies --json login` (um login) | `login_realizado`, saída 0 | 6 |
| `ies unidade recebidos` | Leitura da primeira página aceita pelo executor; contagem não publicada | `null` |
| `ies processo NUP` (um candidato da caixa) | Identidade e unidade conferidas, ausência de atribuição observada na unidade ativa | 3 |
| Descoberta `--descobrir` | `CANDIDATO_OBSERVADO`, uma consulta individual | `[null, 3]` |
| Prévia `ies processo atribuir`, confirmação, releitura e remoção | **NOT_RUN** | Nenhuma dessas etapas solicitada |

**Candidato observado naquela leitura:** `23106.001544/2026-53`. A consulta independente de `ies processo NUP` coincidiu com o ID apresentado na caixa da unidade ativa e publicou ausência de atribuição nessa unidade, segundo os predicados do executor. Isto não é prova de que o processo continuará elegível, não substitui a prévia Web de atribuição, e não autoriza um POST sem nova verificação.

`null` em requisições da caixa significa **contagem não observada**, jamais zero. O relatório comprova uma leitura da caixa e uma leitura individual por comando da CLI; não prova um orçamento HTTP total. GETs podem ter efeito administrativo no SEI de treinamento, mesmo sem alterar atribuição.

## Limites e próximo ensaio

Nenhuma chamada `--confirmar`, POST de atribuição, remoção, replay de escrita ou alteração comprovada da atribuição. Uma tentativa de preparar um segundo workflow temporário com confirmação foi **bloqueada antes da criação**; por isso nenhuma requisição adicional ou escrita dessa etapa ocorreu. Não contornar esse bloqueio nem tratar a descoberta como prévia PASS.

- **IES real ↔ anti-IES ↔ SQLite:** PASS sintético de fatias anteriores (#547); **NOT_RUN neste SHA**.
- **IES e anti-IES ↔ cassettes:** verificações de integridade/menus GET em rodadas anteriores, sem comprovação causal de POST; **NOT_RUN neste SHA**.
- **Treinamento:** login PASS e **descoberta read-only PASS restrita**; prévia da atribuição e efeito de escrita **NOT_RUN**.

O próximo passo permitido é uma **prévia independente, única e atual** no NUP observado, se o acesso autorizado à instalação estiver disponível e o bloqueio de execução não persistir. Apenas depois de prévia PASS e com execução autorizada seria possível confirmar uma vez, reler e remover uma vez, com checkpoint de resíduo. Sem retry de resposta incerta. Os dois NUPs antes recusados não foram revisitados.

O workflow temporário usado exclusivamente nesta leitura foi removido da PR antes de sua apresentação para integração.
