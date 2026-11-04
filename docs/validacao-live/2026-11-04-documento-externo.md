---
type: SEIObservation
title: Ensaio live de documento DOC externo na PEN1
description: PASS/FAIL/NOT_RUN da leitura do anexo de documento externo pelo GET binário do link documento_download_anexo da árvore e das recusas tipadas, PEN1 5.0.4, 04/11/2026
---

# Ensaio live do documento externo — 04/11/2026

[Documento externo](../../contracts/cli/documento-obter-externo.md),
binário em release do ramo `slice-externos`, PEN1 5.0.4 (unidade `TESTE`),
sessão existente validada por `ies login` (1 requisição) e índice da
sessão com o `action` da pesquisa rápida já colhido. Orçamento declarado:
até 20 comandos, ≥ 22 s entre eles, só GET e o POST da pesquisa rápida,
nenhuma repetição; requisições contadas por um relay `CONNECT` local
(um túnel por requisição). Usados: 11 comandos, 28 requisições. O
ensaio parou no primeiro bloqueio do Cloudflare (caso `k`).

## Resultado (PEN1 5.0.4)

| Caso | Pedido | Req. | Resultado |
| --- | --- | --- | --- |
| a. Sessão | `login` | 1 | **PASS**: `sessao_existente_valida` |
| b. TXT do acervo | `0012184` | 1 | **PASS** (recusa tipada): `pesquisa_rapida_nao_abriu_o_documento`; o processo `043393/2026-93` hoje só mostra `0012183` (caso `i`) |
| c. HTML recebido, JSON | `0009006` | 4 | **PASS**: `fonte: documento_download_anexo`, `tipo: null`, `anexo.mime: text/html`, 67.913 bytes, `disposicao: inline`, `nome_original: 0506871.html` (número do órgão de origem), texto visível publicado |
| d. `--saida DIR`, texto humano | `0009006` | 4 | **PASS**: `0009006.html` 0600, 67.913 bytes, SHA-256 igual ao do caso c, nenhum parcial restante |
| e. `--saida` de novo | `0009006` | 4 | **PASS** (recusa tipada): `arquivo_do_anexo_ja_existe`, antes de ler o corpo (13 KB recebidos no túnel contra 73 KB no caso d) |
| f. Limite de 100 bytes | `0009006` | 4 | **PASS** (recusa tipada): `anexo_acima_do_limite` |
| g. Limite inválido | `0009006` | 0 | **PASS**: `limite_de_anexo_invalido`, saída 2, sem rede |
| h. Interno (fatia 1) | `0009007` | 4 | **PASS**: `Despacho 0009007`, `fonte: documento_visualizar`, sem `anexo` |
| i, j. Árvores para achar PDF/TXT | `043393/2026-93`, `01350.001013/2026-36` | 2 + 2 | só `documento_interno` e `documento_html` (`0012169`, `0012170`) |
| k. Árvore de processo recebido | `52710.122783/2026-16` | 2 | **parada**: `bloqueio_de_seguranca_da_instalacao` (403 do Cloudflare de `processoeletronico.gov.br`); nenhuma requisição depois |
| PDF e TXT pelo GET binário | — | — | **NOT_RUN**: nenhum PDF ou TXT achado na PEN1 dentro do orçamento (o acervo da PEN1 também não tem PDF) e o ensaio parou no bloqueio; cobertos por fixture sintética |
| Exceção do SEI no anexo, `chunked`, nome em ISO-8859-1 | — | — | **NOT_RUN**: não provocados ao vivo; fixture sintética |

## Observado

- HTML recebido com tipo principal `text/html` (o valor inteiro do
  cabeçalho não é publicado) e `Content-Disposition: inline;
  filename="…"`, que as cassettes não gravaram; o nome é o do arquivo no
  órgão de origem, não o número da árvore.
- Custo igual ao do interno: 4 requisições com o `action` guardado
  (POST, tela, árvore, anexo). O GET do anexo é o quarto túnel; nenhum
  redirect.
- O texto do HTML recebido traz URLs públicas de conferência do órgão de
  origem; nenhuma URL assinada, `infra_hash` ou cookie nas saídas.
