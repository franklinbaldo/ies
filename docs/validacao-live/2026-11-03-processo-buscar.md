---
type: SEIObservation
title: Ensaio live de processo buscar em três instalações de treinamento
description: PASS/FAIL/NOT_RUN de ies processo buscar (NUP exato, inexistente, parcial, --pagina 2 e fora do intervalo), 03/11/2026
---

# Ensaio live de `ies processo buscar` — 03/11/2026

Primeiro ensaio do comando pela sessão de `ies login`, binário em release
do ramo de #58. Só pesquisa: o efeito é `PaginaOuFiltroLembrado` (a
pesquisa e a página ficam lembradas na conta); sem restauração. Nenhum
NUP, id, cookie ou HTML foi versionado; abaixo só formas e contagens.

## Condições

- **Sessão:** `ies login` com as contas públicas de treinamento do
  [roteiro de login](roteiro-login.md), arquivos `0600` numa pasta `0700`
  temporária fora do repositório.
- **Termos:** o NUP do primeiro processo de `ies unidade recebidos`
  (exato), `99999.999999/2026-00` (inexistente) e os 5 primeiros dígitos
  desse NUP (parcial).
- **Orçamento declarado:** 3 requisições na página 1, 4 na página N,
  como o [fluxo](../decisoes/processo-buscar.md) contrata; o JSON do
  comando confirmou `requisicoes` em todos os casos.

## Resultado

| Caso | UnB 4.0.9 | GO | PEN1 4.0.12 |
| --- | --- | --- | --- |
| NUP exato | **PASS**: `processo_unico` com `id`, 3 req. | **PASS**: idem | **FAIL do servidor**: `sem_resultado` |
| Inexistente | **PASS**: `sem_resultado`, saída 0 | **PASS** | **PASS** |
| Parcial | **PASS**: `lista`, 10 linhas, total 3.441, faixa 1–10 | **PASS**: total 15.835, faixa 1–10 | `sem_resultado` |
| `--pagina 2` | **PASS**: faixa 11–20, mesmo total, 4 req. | **PASS**: idem | **PASS (recusa)**: `pagina_fora_do_intervalo` |
| `--pagina 100000` | **PASS (recusa)**: `pagina_fora_do_intervalo` | **PASS (recusa)** | **PASS (recusa)** |

Depois da revisão (UnB, mesmo dia): termo de 51 caracteres →
`termo_acima_do_maxlength_da_tela` sem POST; termo de 50 → `sem_resultado`
em 3 requisições (**PASS**).

UFRJ, PEN2, PEN3 e PEN4: NOT_RUN (fora do orçamento desta rodada).

## Observações

- **Lacunas fechadas.** A resposta de `--pagina N` foi capturada ao vivo
  (mesmo total, faixa `inicio + 1 … inicio + 10`), e a barra acima de 999
  (`3.441`, `15.835`) usa o separador `.` previsto.
- **PEN1 não acha nada.** O NUP saiu dos recebidos da própria unidade, e
  a resposta trouxe `div.pesquisaSemResultado` mesmo assim. O POST montado
  pelo arnês Python, independente do IES, recebeu a mesma resposta, então
  o comportamento é da instalação (índice de pesquisa da instalação de
  treinamento, hipótese não verificada), não do IES. Por isso
  `sem_resultado` nunca vira `processo_inexistente`.
- **Rede intermitente.** Uma vez na UnB e uma no login de GO, o
  transporte falhou (`falha_de_rede`), sem nova tentativa, como o contrato
  exige; a repetição manual seguinte passou. O erro não diz se foi
  timeout, reset ou TLS, então a causa de transporte ainda não é exposta.
