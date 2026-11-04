---
type: SEIObservation
title: Varredura live na UnB 4.0.9 e em GO 5.0.3
description: Rodada só de leitura de 04/11/2026 (Refs #176, #57, #60, #61, #88) com documento externo PDF, versões e documento buscar na UnB, e caixa, processo NUP e documento buscar em GO; requisições contadas por túnel, ritmo espaçado, nenhum bloqueio do WAF, nenhuma falha do IES
---

# Varredura live na UnB e em GO — 04/11/2026

Rodada só de leitura para os critérios live ainda abertos de #176 (caixa
em GO), #57, #60 (externo PDF), #61 (`documento buscar` na UnB e em GO) e
#88 (versões na UnB). UFRJ, PEN3 e PEN4 ficaram com outra rodada.

## Método

- Binário em release do ramo `slice-varredura-b` (base `7a8b332`), cópia
  fora da árvore de build.
- Sessões existentes em arquivos privados `0600`; `ies login` uma vez por
  instalação.
- Relay `CONNECT` local, uma porta por instalação, contando túneis e
  bytes (um túnel por requisição).
- Nenhuma escrita, nenhum `--confirmar`, nenhuma repetição de comando.
- Nenhuma saída trouxe URL assinada, `infra_hash` ou cookie (conferido
  nas 23 saídas); o único `https://` é a raiz pública em `ies login`.

Orçamento declarado:

| Instalação | Comandos | Intervalo | Usado |
| --- | --- | --- | --- |
| UnB 4.0.9 | até 20 | 22 s ou mais | 14 comandos, 37 requisições |
| GO 5.0.3 | até 12 | 90 s ou mais | 9 comandos, 27 requisições |

## UnB 4.0.9 (unidade `Unidade1`)

| # | Comando | Req. | Resultado |
| --- | --- | --- | --- |
| u01 | `login` | 1 | **PASS**: `sessao_existente_valida` |
| u02 | `processo 23106.001739/2026-01` | 5 | **PASS**: 3 documentos; `0011862` com `icone: documento_pdf` |
| u03 | `documento 0011862 --processo …` | 2 | **PASS**: `fonte: documento_download_anexo`, `mime: application/pdf`, 193 bytes, `disposicao: inline`, `nome_original: anexo_teste_todos.pdf`, `texto: null`, `resolvido_por: indice_link` |
| u04 | idem `--saida DIR` (0700) | 2 | **PASS**: `0011862.pdf` 0600, 193 bytes, SHA-256 `c896da11…0ab5b` igual ao de u03, `file`: "PDF document, version 1.4, 1 page(s)", nenhum parcial restante |
| u14 | `documento 0011862` (sem `--processo`) | 2 | **PASS**: índice aprendido na árvore, mesmo SHA-256 |
| u05 | `documento 0011906 --processo 23106.001765/2026-21` | 4 | **PASS**: `versao: 2`, `versao_atual: 2` pelo rodapé, pesquisa rápida |
| u06 | idem `--versao 1` | 1 | **PASS** (recusa tipada): `lista_de_versoes_nao_publicada_na_arvore`; processo concluído (ver Leitura) |
| u07 | `documento 0011863 --processo 23106.001739/2026-01 --versao 1` | 1 | **PASS** (recusa tipada): idem; processo aberto só na `Unidade2` (u02) |
| u08 | `documento 0011826 --processo 23106.001682/2026-32 --versao 1` | 3 | **PASS** (recusa tipada): idem; processo aberto só na `Unidade2` (u11) |
| u11 | `processo 23106.001682/2026-32` | 3 | **PASS**: `aberto_em` só `Unidade2` |
| u12 | `documento 0011870 --processo 23106.001748/2026-94 --versao 1` | 4 | **PASS**: `versao: 1`, `versao_atual: 3` pela lista; trabalhar → árvore → lista → versão |
| u09 | `documento buscar 23106.001739/2026` | 3 | **PASS**: `lista`, `total_declarado: 2`, faixa 1–2, `linhas_ausentes_da_faixa: 0`; `0011863` (`anexo: false`) e `0011862` (`anexo: true`) |
| u10 | `documento buscar 0011862` | 3 | **PASS**: 302 → `documento_unico` (ids 51311/51309) |
| u13 | `processo buscar 23106.001748/2026-94` (controle) | 3 | **PASS**: `processo_unico` |

## GO 5.0.3 (unidade `SEAD/CDE-09516`)

| # | Início | Comando | Req. | Resultado |
| --- | --- | --- | --- | --- |
| g01 | 21:21:15 | `login` | 7 | **PASS**: `login_realizado`; a sessão de 03/11 tinha vencido (conferência + 3 GET + POST + 2 redirecionamentos) |
| g02 | 21:23:04 | `unidade recebidos` | 1 | **PASS**: R, faixa 1–10 de 131 |
| g03 | 21:24:38 | `unidade gerados --pagina 2` | 2 | **PASS**: 1 GET + 1 POST, faixa 11–20 de 8.021 |
| g04 | 21:26:16 | `unidade listar-processos` | 2 | **PASS**: R→D (1 GET + 1 POST), faixa 1–10 de 8.152, 12 colunas, `colunas_ausentes: []`, categoria `recebidos` só no id visto em g02 |
| g05 | 21:27:55 | `unidade listar-processos --pagina 2` | 2 | **PASS**: D lembrada, 1 GET + 1 POST, faixa 11–20 |
| g06 | 21:29:35 | `unidade recebidos --pagina 99` | 2 | **PASS** (recusa): D→R (1 GET + 1 POST R), `pagina_fora_do_intervalo` antes do POST de página (14 páginas) |
| g07 | 21:31:16 | `processo 202400013000003` | 5 | **PASS**: `indice_trabalhar`, 8 documentos, `aberto_em` com 3 unidades, `marcadores: []` com `marcadores_total_declarado: 0`, propriedades de `procedimento_alterar` |
| g08 | 21:32:56 | `documento buscar 2024000130000` | 3 | **PASS**: `lista`, 2 de 2, `linhas_ausentes_da_faixa: 0` |
| g09 | 21:34:35 | `documento buscar 2026000050` | 3 | **PASS**: `lista`, faixa 1–10 de 9.090 com **9 linhas**, `linhas_ausentes_da_faixa: 1` |

O processo de g07 já estava `visualizado` na caixa: abri-lo não registrou
recebimento novo.

## WAF de GO

Nenhum 403 e nenhum `bloqueio_de_seguranca_da_instalacao` em 27
requisições. Intervalo entre comandos de 90 s ou mais, medido do fim de um
ao início do seguinte. Dentro de um comando, os pedidos saem em sequência:
7 em 14 s (g01) e 5 em 9 s (g07), cada túnel com 1,1 a 5,4 s. O túnel
do ambiente não caiu (na rodada anterior, `ws_closed_mid_exchange`). O
limiar do WAF segue não medido: só sabemos que esse ritmo passa.

## Leitura

- **PDF ao vivo (#60).** Primeiro `application/pdf` ao vivo, antes coberto
  só por fixture sintética. Custo de 2 requisições pelo link guardado da
  árvore (árvore e anexo), como o interno. O `Content-Disposition`
  `inline` com nome do servidor não vira caminho: o arquivo é `DOC.pdf`.
- **Versões na UnB (#88).** A lista só existe na barra quando o processo
  está aberto na unidade da sessão. Concluído (u06) ou aberto só na
  `Unidade2` (u07, u08), a barra não publica `documento_versao_listar`;
  na cassette pública `liberacoes-e-versoes-leitura-unb` a árvore depois
  de `procedimento_concluir` traz o mesmo documento sem o link. A recusa
  sai em G requisições, sem GET a mais. O replay da barra lê 1.547 barras
  da UnB, 1.351 com a lista.
- **`documento buscar` (#61).** Um pedaço de NUP casa documentos pelo
  número do processo, mas não todos: na UnB, 2 dos 3 da árvore; em GO, 2
  dos 8. `total_declarado` é o que a tela diz, não o processo inteiro.
  Em GO o NUP é servido só com dígitos (`202400013000003`), então o
  "fragmento formatado" é um prefixo de dígitos. Com 9.090 acertos, a
  página de GO veio com 9 linhas para a faixa 1–10: primeira observação
  ao vivo de `linhas_ausentes_da_faixa`.
- **Caixa em GO (#176, #57).** Página de 10 linhas nas três listas, com os
  mesmos regimes da PEN1, PEN3 e PEN4: R depois do login, D lembrada
  depois de `listar-processos`, troca D→R em `recebidos`.

## Achados

Nenhuma falha do IES; nenhuma correção nesta rodada.

## NOT_RUN

| Caso | Motivo |
| --- | --- |
| `--versao N` em GO, PDF em GO | fora do escopo; GO não tem PDF no acervo (só `documento_txt`) |
| `--saida` com cópia `DOC.vN.html` na UnB | não pedido nesta rodada (só leitura de versão) |
| Limiar do WAF de GO | não provocado: rodada só no ritmo seguro |
