---
type: SEIObservation
title: unidade colunas --definir todas --confirmar na PEN4
description: PASS da escrita das 12 colunas do Nível de detalhe na PEN4 5.0.4 em 04/11/2026 pelo roteiro reescrito, com requisições contadas por túnel, sinal de gravação reconhecido e leitura de conferência
---

# `--definir todas --confirmar` na PEN4 — 04/11/2026

## Condições

- Instalação: PEN4 (Tramita, homologação, 5.0.4 pela
  [tabela de instalações](../como-o-sei-funciona/estado-e-instalacoes.md)),
  unidade `TESTE`. Autorização do proprietário: **uma** escrita de
  preferência nesta instalação.
- Binário release de `main` em `62fb385`, que já classifica a resposta do
  salvar pelo sinal de gravação
  ([diagnóstico da PEN1](2026-11-04-definir-colunas-pen1.md)), copiado para
  uma pasta privada antes do ensaio.
- Sessão existente da PEN4 (aberta às 15:00 UTC do mesmo dia), arquivos
  `0600` em pasta `0700`, índice local da sessão.
- [Roteiro](roteiro-definir-colunas.md) executado uma vez, das 16:48 às
  16:50 UTC, com 23 a 29 s entre comandos. Saída JSON inteira de cada
  comando guardada em pasta privada; nenhuma traz URL, cookie ou
  `infra_hash` (conferido por busca).
- Requisições contadas por túnel `CONNECT` num relay local em frente ao
  proxy do ambiente, como nos ensaios anteriores (um túnel por requisição;
  GET de 1.286 B enviados, POST maior).

## Resultado

| Comando | Túneis | Saída e campos | Resultado |
| --- | --- | --- | --- |
| `ies --json login` | 1 | 0, `sessao_existente_valida`, `requisicoes: 1` | PASS |
| `ies --json unidade colunas` (`c0`) | 2 GET | 0; 12 opções disponíveis, 3 marcadas: `atribuicao`, `tipo`, `interessados` | PASS |
| `… colunas --definir todas` (prévia) | 2 GET | 0; `confirmado false`, `alterado false`, `marcar` com as 9 desmarcadas, `desmarcar` vazio, `indisponiveis` vazio; `campos` = `hdnInfraTipoPagina`, `sbmSalvar` e as 12 caixas na ordem do documento | PASS |
| `… colunas --definir todas --confirmar` | 3 GET + 1 POST | 0; `confirmado true`, `alterado true`, `reconciliado_por_leitura false`, sem `falha_do_salvar`; `depois` = as 12 | **PASS** |
| `ies --json unidade colunas` (`c1`, 24 s depois) | 2 GET | 0; 12 marcadas = 12 disponíveis | PASS |
| `ies --json unidade listar-processos` | 1 GET | 0; `total_declarado: 260`, `faixa_observada` 1–100, `colunas_ausentes: []` | PASS |

Total: 12 túneis (11 GET e 1 POST de salvar), seis comandos, nenhum
repetido; dentro do [orçamento](roteiro-definir-colunas.md#orçamento-por-instalação).
O POST foi o único túnel de envio maior (2.066 B); a resposta (19.138 B) é
do tamanho da tela do Nível de detalhe, como a resposta que gravou na PEN1
em 02/11.

## Efeito observado

- A preferência do usuário na PEN4 passou de 3 para **12 colunas**. O
  `c1`, na mesma sessão e 24 s depois, leu as 12 marcadas.
- O Controle servido na D cresceu de cerca de 222 KB para 262 KB entre o
  GET do `--confirmar` e o do `c1`: as colunas novas aparecem na tabela.
- Em `listar-processos`, cada um dos 100 processos traz as 12 chaves em
  `colunas`; com valor nesta página: `tipo` e `ultima_movimentacao` (100),
  `especificacao` (99), `interessados` (88), `anotacao` (3), `atribuicao`
  e `observacao` (2) e `marcadores` (1). `colunas_ausentes` veio vazia:
  nesta página as três colunas de ícone (`controle_de_prazo`,
  `para_devolver`, `aguardando_retorno`) foram reconhecidas pelo
  cabeçalho, e a divergência conhecida das colunas de ícone não apareceu.
- Nada foi restaurado: todas as colunas é o estado final desejado.

## O que isto mostra

- Na 5.0.4 da PEN4, a resposta do salvar trouxe o sinal de gravação
  (`inicializar()` com `'1'=='1'`), que o IES reconheceu: saída 0 sem
  `reconciliado_por_leitura`. É a mesma forma da resposta capturada na PEN1
  em 02/11.
- Com 22 s ou mais entre comandos, o POST e o GET de verificação seguidos
  tiveram resposta normal. A causa da saída 3 da PEN1 continua não
  determinada; este PASS não a esclarece, mas é o PASS que o roteiro
  pede antes de a PEN1 voltar.

Resultado nesta data: PEN4 **PASS** (12 colunas, 1 POST, sem
reconciliação). PEN3 fica para outro operador, com autorização própria.
