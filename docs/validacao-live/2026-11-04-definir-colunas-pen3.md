---
type: SEIObservation
title: unidade colunas --definir todas --confirmar na PEN3
description: PASS da escrita das 12 colunas do Nível de detalhe na PEN3 5.0.4 em 04/11/2026 pelo mesmo roteiro da PEN4, com requisições contadas por túnel, sinal de gravação reconhecido e leitura de conferência
---

# `--definir todas --confirmar` na PEN3 — 04/11/2026

## Condições

- Instalação: PEN3 (Tramita, homologação, 5.0.4 pela
  [tabela de instalações](../como-o-sei-funciona/estado-e-instalacoes.md)),
  unidade `TESTE`. Autorização do proprietário: **uma** escrita de
  preferência nesta instalação.
- Binário release do ramo de integração em `ce8dfb7` (o mesmo código do
  comando que passou na [PEN4](2026-11-04-definir-colunas-pen4.md)),
  copiado para uma pasta privada antes do ensaio.
- Sessão existente da PEN3 (aberta às 14:59 UTC do mesmo dia), arquivos
  `0600` em pasta `0700`, índice local da sessão.
- [Roteiro](roteiro-definir-colunas.md) executado uma vez, das 16:59 às
  17:01 UTC, com 25 a 27 s entre comandos. Saída JSON inteira de cada
  comando guardada em pasta privada; nenhuma traz URL, cookie ou
  `infra_hash` (conferido por busca).
- Requisições contadas por túnel `CONNECT` num relay local em frente ao
  proxy do ambiente, como na PEN4 (um túnel por requisição; GET de
  1.286 B enviados, POST maior).
- Mais cedo no mesmo dia, a borda da PEN3 bloqueou um POST de pesquisa
  com termo só de dígitos. Neste ensaio nenhum comando saiu com
  `bloqueio_de_seguranca_da_instalacao`.

## Resultado

| Comando | Túneis | Saída e campos | Resultado |
| --- | --- | --- | --- |
| `ies --json login` | 1 | 0, `sessao_existente_valida`, `requisicoes: 1` | PASS |
| `ies --json unidade colunas` (`c0`) | 2 GET | 0; 12 opções disponíveis, 3 marcadas: `atribuicao`, `tipo`, `interessados` | PASS |
| `… colunas --definir todas` (prévia) | 2 GET | 0; `confirmado false`, `alterado false`, `marcar` com as 9 desmarcadas, `desmarcar` vazio, `indisponiveis` vazio; `campos` = `hdnInfraTipoPagina`, `sbmSalvar` e as 12 caixas na ordem do documento, idênticos aos da PEN4 | PASS |
| `… colunas --definir todas --confirmar` | 3 GET + 1 POST | 0; `confirmado true`, `alterado true`, `reconciliado_por_leitura false`, sem `falha_do_salvar`; `depois` = as 12 | **PASS** |
| `ies --json unidade colunas` (`c1`, 25 s depois) | 2 GET | 0; 12 marcadas = 12 disponíveis | PASS |
| `ies --json unidade listar-processos` | 1 GET | 0; `total_declarado: 294`, `faixa_observada` 1–100, `colunas_ausentes: []` | PASS |

Total: 12 túneis (11 GET e 1 POST de salvar), seis comandos, nenhum
repetido; dentro do [orçamento](roteiro-definir-colunas.md#orçamento-por-instalação).
O POST foi o único túnel de envio maior (2.066 B, o mesmo tamanho da
PEN4); a resposta (19.127 B) é do tamanho da tela do Nível de detalhe,
como na PEN4.

## Efeito observado

- A preferência do usuário na PEN3 passou de 3 para **12 colunas**. O
  `c1`, na mesma sessão e 25 s depois, leu as 12 marcadas.
- O Controle servido na D cresceu de cerca de 214 KB para 255 KB entre o
  GET do `--confirmar` e o do `c1`: as colunas novas aparecem na tabela.
- Em `listar-processos`, cada um dos 100 processos traz as 12 chaves em
  `colunas`; com valor nesta página: `tipo`, `especificacao` e
  `ultima_movimentacao` (100), `interessados` (67), `atribuicao` (8) e
  `anotacao` (2). `colunas_ausentes` veio vazia: as três colunas de ícone
  foram reconhecidas pelo cabeçalho, como na PEN4.
- Nada foi restaurado: todas as colunas é o estado final desejado.

## O que isto mostra

- Na 5.0.4 da PEN3, como na PEN4, a resposta do salvar trouxe o sinal de
  gravação que o IES reconhece: saída 0 sem `reconciliado_por_leitura`.
- Com 22 s ou mais entre comandos, o POST e o GET de verificação seguidos
  tiveram resposta normal, e a borda da PEN3 não bloqueou nenhuma das 12
  requisições. A causa da saída 3 da PEN1 continua não determinada.

Resultado nesta data: PEN3 **PASS** (12 colunas, 1 POST, sem
reconciliação). Com PEN3 e PEN4 em PASS, a próxima etapa do roteiro é GO e
UnB 4.0.9, cada uma com autorização própria.
