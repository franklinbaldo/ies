---
type: SEIObservation
title: Ensaio live da leitura do Controle em seis instalações de treinamento
description: PASS/FAIL/NOT_RUN de status, unidade atual e listar-processos com GET único, 02/11/2026
---

# Ensaio live da leitura do Controle — 02/11/2026

Primeiro ensaio **autenticado** do IES em instalações reais. Só
leitura: login + 3 GETs do IES por instalação, **zero POST, zero
escrita**. Binário `ies` em release a partir de `main` + #144.

## Condições

- **Login:** feito pelo Todos (`SEISession.caixa()`), usado como
  arnês privado fora do repositório; o IES não implementa login.
  Contas públicas de treinamento do
  [inventário do Todos](https://github.com/franklinbaldo/todos/blob/323eb013a1e7fac42c50336f838373a2cbe23e4b/docs/sei-multi-instalacao.md)
  (`aluno1`, `servidor1`, `orgao1`, `orgao3`, `orgao4`, `aluno05`).
- **Sessão para o IES:** origem, cookie e URL assinada do
  `procedimento_controlar` da **mesma geração**, em arquivos `0600`
  numa pasta `0700` fora do repositório (`IES_ORIGEM_FILE`,
  `IES_COOKIE_FILE`, `IES_URL_CONTROLE_FILE`). Nada disso, nem o HTML
  autenticado, foi versionado.
- **Comandos:** `ies status --verificar-controle-web --json`,
  `ies unidade atual --json`, `ies unidade listar-processos --json`.
- **Orçamento:** 3 GETs do IES ao Controle por instalação, além das
  requisições de login do Todos (≈3–5). Sem nova tentativa após falha,
  exceto uma única na PEN2 (503 no login) e, em GO, uma segunda rodada
  após a correção (login + 3 GETs + 2 GETs de diagnóstico só com
  status/tipo + 1 repetição manual).

## Resultado

| Instalação | SEI | Unidade exibida | `status --verificar-controle-web` | `unidade atual` | `unidade listar-processos` |
| --- | --- | --- | --- | --- | --- |
| UnB | 4.0.9 | Unidade1 | PASS | PASS | **PASS**: 12 recebidos, 47 gerados = legendas da tela |
| UFRJ | 4.1.5 | PR-P | PASS | PASS | **PASS**: 2 recebidos, 7 gerados = legendas |
| PEN1 | 4.0.12 | TESTE | PASS | PASS | **PASS (recusa)**: 114/218 registros, tela com 1–100 → `paginas_do_controle_nao_obtidas` |
| PEN3 | 5.0.4 | TESTE | PASS | PASS | **PASS (recusa)**: 171/123 registros, 1–100 → `paginas_do_controle_nao_obtidas` |
| PEN4 | 5.0.4 | TESTE | PASS | PASS | **PASS (recusa)**: 157/103 registros, 1–100 → `paginas_do_controle_nao_obtidas` |
| GO (1ª rodada) | 5.0.3 | — | **FAIL** `controle_web_nao_reconhecido` | **FAIL** `link_do_processo_nao_identificado_na_linha` | **FAIL** `resposta_inesperada` |
| GO (após a correção) | 5.0.3 | SEAD/CDE-09516 | PASS | PASS na 2ª execução; a 1ª deu `resposta_inesperada` | **PASS (recusa)**: 131/7990 registros, 1–10 → `paginas_do_controle_nao_obtidas` |
| PEN2 | 4.1.5 | — | NOT_RUN | NOT_RUN | NOT_RUN: HTTP 503 no login (2 tentativas) |

"PASS (recusa)" é o comportamento do contrato: com páginas não obtidas,
o IES falha fechado em vez de publicar lista parcial. Comprova, com
dados reais, a lacuna da paginação Web ativa (#57).

## Achados

**GO — link de processo sigiloso.** No Controle de Goiás, uma linha de
`tblProcessosGerados` tem o link `procedimento_trabalhar` com a classe
`processoVisualizadoSigiloso`; as outras 19 linhas usam
`processoVisualizado`. O leitor da caixa reconhecia só
`processoVisualizado`/`processoNaoVisualizado` e recusava a caixa
inteira. O protocolo de Goiás tem 15 dígitos sem pontuação; o IES já o
trata como opaco e não é a causa. A caixa de Goiás é paginada de 10 em
10 (131 recebidos, 7990 gerados).

Corrigido no mesmo dia: o leitor reconhece `processoVisualizadoSigiloso`
e expõe `sigiloso` no JSON v2. A nova rodada usou sessão nova e o
binário corrigido.

**GO — resposta intermitente.** Em 8 GETs do IES ao mesmo Controle
(duas sessões), 2 receberam `resposta_inesperada`. Dois GETs de
diagnóstico logo depois devolveram 200 com `text/html; charset=iso-8859-1`,
e a repetição do comando passou. O subcódigo atual não distingue status
HTTP, `Content-Type` ou decodificação; a causa fica indeterminada. Não
houve nova tentativa automática: a repetição foi manual e única.

**Lista igual à tela.** Na UnB e na UFRJ, os totais devolvidos
coincidem com as legendas `N registros` das duas tabelas.

## Limites

- Uma conta e uma unidade por instalação; outras contas, perfis ou
  unidades podem divergir.
- O GET do Controle pode registrar efeitos administrativos; nenhum foi
  verificado ou observado neste ensaio.
- Não valida login do IES, paginação, pesquisa, POST nem escrita.
- O HTML de Goiás foi inspecionado só estruturalmente (nomes de
  atributos, classes e forma dos textos), sem copiar valores.
