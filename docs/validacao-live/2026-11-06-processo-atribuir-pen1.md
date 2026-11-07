---
type: SEIObservation
title: Ensaio de prévia da atribuição na PEN1 de treinamento
description: Tentativa única de login PEN1 para segunda instalação da issue 264, impedida por falha de rede; não houve prévia nem escrita
---

# Atribuição — segunda instalação PEN1, 06/11/2026

Issue #264, PR #316. Ensaio exclusivamente na **PEN1 5.0.4 de
treinamento**, via [workflow temporário 42293214245](https://github.com/franklinbaldo/ies/actions/runs/42293214245).
O código do workflow foi removido do branch após a execução.

## Procedimento e resultado

O executor preparou os arquivos de origem, credenciais públicas da instalação,
cookie e Controle em diretório privado do runner (permissões `0600`). Nenhum
segredo foi publicado; nenhuma senha foi passada como argumento. A
tentativa foi planejada para fazer login, aguardar **65 segundos** e então
executar **uma única prévia de leitura** de
`ies processo atribuir 043484/2026-29 --usuario orgao1 --json`,
sem `--confirmar`.

| Etapa | Resultado | Saída | Requisições reportadas |
| --- | --- | --- | --- |
| Build Rust release | **PASS** | 0 | não aplicável |
| Login PEN1 | **FAIL** | 1, `falha_de_rede` | `null`, não observadas |
| Prévia PEN1 | **NOT_RUN** | não executada, bloqueada pelo login | 0 de atribuição |
| Confirmação e remoção | **NOT_RUN** | `--confirmar` público fechado | 0 de atribuição |

O job GitHub Actions terminou com **success** porque registra falhas
de login como dado de diagnóstico; **não** transforma o login em PASS.
A resposta `falha_de_rede` não prova indisponibilidade global da PEN1
nem determina qual etapa de autenticação falhou. Como a contagem de
requisições do login é `null`, não substituímos por zero; não se pode
afirmar quantos GETs ou POSTs de autenticação foram tentados.

**Não houve requisição de prévia de atribuição, POST de atribuição,
confirmação, remoção, retry ou resíduo de atribuição.** Não houve
acesso a produção. A autorização de escrita só vale em treinamento,
e a CLI permanece fechada antes da sessão/rede.

## Estado da prova

A prévia histórica UnB permanece **PASS (4 GET, 0 POST de atribuição)**.
As duas tentativas históricas UFRJ permanecem **FAIL**, com unidade e
contagem não comprovadas. Esta tentativa PEN1 não satisfaz o critério
de **prévia PASS numa segunda instalação**. Antes de habilitar a escrita
pública, é necessário obter outra prévia válida e a sequência autorizada
prévia → uma escrita → releitura → uma remoção → releitura, registrando
resíduo e saída 3 quando incerto.

Integração de `main` nesta execução: merge de dois pais
[`8b150192`](https://github.com/franklinbaldo/ies/commit/8b150192d3e6f5ae12b2d7f8c86f292de381c5d5),
preservando a matriz das issues #245 e #264; [CI 42292792436](https://github.com/franklinbaldo/ies/actions/runs/42292792436)
**5/5 jobs PASS** no SHA do merge, 0 commits atrás da `main`.
