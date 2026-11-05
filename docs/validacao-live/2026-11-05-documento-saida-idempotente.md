---
type: SEIObservation
title: Ensaio live de documento DOC --saida com a foto DOC.vN.html idempotente na UnB
description: PASS/FAIL/NOT_RUN da foto DOC.vN.html (#207) na UnB 4.0.9 em 05/11/2026, com leitura repetida sem gravação, recusa tipada de --versao 1 e prévia local de documento salvar sobre a foto lida
---

# Ensaio live da foto `DOC.vN.html` — 05/11/2026

[Contrato de versões](../../contracts/cli/documento-obter-versoes.md),
[decisão](../decisoes/documento-obter/saida-idempotente.md), binário de
depuração do ramo `slice-207` (commit `94635aa`), UnB 4.0.9
(`https://sei.treinamento.unb.br/sei`), conta pública `aluno1`
([credenciais](../interfaces/credenciais-de-treinamento.md)).

## Método

- Só leitura. Sessão, senha e índice em arquivos `0600` numa pasta
  privada `0700` fora do repositório; `--saida` noutra pasta `0700`.
- Requisições contadas por túnel num relay local (`relay.py`), um
  túnel por requisição; o campo `requisicoes` do IES confere.
- Pelo menos 30 s entre comandos; nenhum retry.
- Nenhuma saída trouxe URL assinada, `infra_hash` ou cookie (conferido
  nas cinco saídas); a foto gravada não contém `infra_hash`.

Orçamento declarado: até 6 comandos e 20 requisições. Usado: 5 comandos,
13 requisições.

## Resultados

Documento `0011870` do processo `23106.001748/2026-94` (3 versões no
ensaio de 04/11).

| # | Comando | Túneis | Resultado |
| --- | --- | --- | --- |
| L0 | `login` | 5 | **PASS**: `login_realizado`, `unidade_exibida: Unidade2` |
| L1 | `documento 0011870 --processo NUP --saida DIR` | 4 | **PASS**: `versao: 3` pelo rodapé, G + 1; grava só `0011870.v3.html` (0600, 24.955 bytes); `arquivo.gravado: true` |
| L2 | o mesmo comando de novo | 2 | **PASS** (no-op): `gravado: false`, `motivo: igual`, mesmo SHA-256 `202f41d6…25f15`; `mtime` e tamanho iguais; pelo link guardado da árvore |
| L3 | `… --versao 1 --saida DIR` | 1 | **PASS** (recusa tipada): `lista_de_versoes_nao_publicada_na_arvore`; nada gravado |
| L4 | `… --versao 1 --saida DIR --unidade Unidade1` | 1 | **PASS** (recusa tipada): `unidade_da_tela_diferente_da_solicitada`; nada gravado |

Em `DIR` ficou só `0011870.v3.html`: nenhum `.txt`, nenhum arquivo
oculto, nenhum parcial.

Prévia local de `documento salvar` sobre a foto lida, sem rede (0
túneis com o mesmo relay como proxy), com o índice do ensaio:

| Caso | Resultado |
| --- | --- |
| `0011870.v4.html` = v3 com um parágrafo a mais | **PASS**: `versao_base: 3`, `versao_editada: 4`, `trecho_divergente` na linha 61 |
| `salvar --listar-copias` | **PASS**: v3 `foto: true`, v4 `foto: false` |
| v3 com um byte a mais | **PASS**: `base_versao_divergente` |

## Leitura

- **Idempotência.** O `documento_visualizar` da versão atual serviu os
  mesmos bytes nas duas leituras (SHA-256 igual), o que sustenta o no-op.
  Uma instalação não prova as outras: a PEN e a UFRJ não foram lidas
  duas vezes.
- **Unidade.** A conta compartilhada caiu na `Unidade2` no login; o
  processo está aberto na `Unidade1` (ensaio de 04/11), então a barra
  não publica a lista e `--versao 1` recusa em G requisições (L3). O
  `--unidade Unidade1` só confere a unidade, não troca (L4). O IES não
  troca a unidade da conta (princípio de não restaurar a tela).

## NOT_RUN

| Caso | Bloqueio |
| --- | --- |
| gravar `DOC.v1.html` de versão histórica ao vivo | a sessão estava na `Unidade2`, sem a lista na barra (L3, L4); coberto por teste sintético |
| foto sem rodapé pela lista (G + 2) | nenhum documento sem rodapé lido; coberto por teste sintético |
| recusa `foto_local_divergente` depois da rede | não provocada ao vivo; coberta por testes sintéticos |
