---
type: SEIObservation
title: Ensaio live de documento buscar na PEN3 5.0.4
description: PASS/FAIL/NOT_RUN de ies documento buscar na PEN3 (NUP com um e com dois documentos, parcial, --pagina 2 e última, fora do intervalo, inexistente, texto, --unidade divergente, saída humana), 04/11/2026
---

# Ensaio live de `ies documento buscar` — PEN3 5.0.4 — 04/11/2026

Primeiro ensaio do comando, na PEN3 (`sei.orgao3.tramita.processoeletronico.gov.br`,
SEI 5.0.4), com o binário release do ramo `slice-docbusca` (commit
`7ae0752`). Complementa o
[replay do acervo](../validacao-replay/2026-11-04-documento-buscar-acervo.md)
e segue a [decisão](../decisoes/documento-buscar.md).

## Condições

- **Sessão:** a de `ies login` com a conta pública de treinamento da PEN3
  ([credenciais](../interfaces/credenciais-de-treinamento.md)), unidade
  `TESTE`, arquivos `0600` numa pasta `0700` fora do repositório; não
  precisou relogar.
- **Efeito avaliado:** GET do Controle e da tela de Pesquisa são leitura;
  o POST da pesquisa lembra página e filtro (`PaginaOuFiltroLembrado`); o
  302 para `procedimento_trabalhar` **não** é seguido (nenhum processo ou
  documento aberto).
- **Orçamento declarado:** até 15 comandos, ≥ 25 s entre eles, sem retry;
  3 requisições na página 1 e 4 na página N. Usados: 10 comandos
  (17:54–17:59 UTC, 27 a 33 s entre o fim de um e o início do seguinte) e
  30 requisições, contadas por túnel num relay CONNECT local.
- **Termos.** Só termos com pontuação ou letras: o Cloudflare da PEN3
  bloqueou, mais cedo no mesmo dia, um POST de `processo buscar` com termo
  só de dígitos. Os NUPs e números SEI são conteúdo público da instalação
  de treinamento (aparecem na cassette `pesquisa-paginas-pen3`). Nenhum
  cookie, URL assinada, `infra_hash` ou HTML foi versionado; as saídas
  foram conferidas sem URL, hash nem cookie.

## Resultado

| # | Termo | Esperado | Saída do IES | Req. | Veredito |
| --- | --- | --- | --- | ---: | --- |
| 1 | `98390.000036/2026-99` (NUP de processo com um documento) | resposta observada | `documento_unico`, ids 1518 / 1517 (os da linha de `0001018` na cassette) | 3 | **PASS** |
| 2 | `98390.0000` (parcial) | `lista` | 10 linhas, total 46, faixa 1–10 | 3 | **PASS** |
| 3 | `98390.0000 --pagina 5` (última) | faixa 41–46 | 6 linhas, total 46, faixa 41–46 | 4 | **PASS** |
| 4 | `98390.0000 --pagina 2` | faixa 11–20 | 10 linhas, total 46, faixa 11–20 | 4 | **PASS** |
| 5 | `98390.0000 --pagina 6` | recusa antes do POST de página | `pagina_fora_do_intervalo`, saída 1 | 3 | **PASS** |
| 6 | `99999.999999/2026-00` | resposta observada | `sem_resultado`, saída 0 | 3 | **PASS** |
| 7 | `98390.000014/2026-29` (NUP com dois documentos) | resposta observada | `lista`, 2 de 2: `0001004` e `0001003`, os dois `anexo` | 3 | **PASS** |
| 8 | `Despacho` (texto no Protocolo) | resposta observada | `sem_resultado`, saída 0 | 3 | **PASS** |
| 9 | `--unidade OUTRA` | recusa depois do GET do Controle | `unidade_da_tela_diferente_da_solicitada`, saída 1 | 1 | **PASS** |
| 10 | `98390.000014/2026-29` sem `--json` | texto humano | 2 de 2, com número SEI, rótulo e NUP | 3 | **PASS** |
| — | número SEI exato ou parcial só de dígitos (`0001018`) | `documento_unico` | — | 0 | **NOT_RUN** |

## Observações

- **Pesquisa em documentos pelo Protocolo.** Na PEN3, o campo Protocolo
  com `D` casa os documentos pelo NUP do processo (inteiro ou parcial):
  com um só acerto o SEI responde 302 com `id_documento` (caso 1, a forma
  das cassettes `documento-numero-e-busca-*` para o número SEI exato), com
  mais de um, a lista (caso 7). `documento_unico` é "um acerto", não
  "o documento pedido": o comando publica só os ids do 302.
- **Linhas.** As 28 linhas lidas (casos 2, 3, 4 e 7) vieram na forma da
  decisão, com `unidade` (inclusive `TESTE_1_2`), `inclusao` e `anexo`
  (`documento_download_anexo`) observados; nenhuma recusa de linha.
- **Paginação.** A última página (caso 3) foi aceita com a faixa pedida,
  pelo mesmo POST de `navegar()`; o comando não grava o HTML, então o
  ensaio não registra se a barra veio "Exibindo 41 - 46 de 46" ou "46
  resultados" (a forma da cassette, coberta por teste sintético).
- **Texto no Protocolo** (caso 8) não é pesquisa por palavra: o campo
  `q` não é enviado. `sem_resultado` é resposta observada, não
  inexistência.

## Não executado

- **Número SEI só de dígitos** (exato `0001018` ou parcial): **NOT_RUN**
  de propósito, para não provocar de novo o bloqueio da borda da PEN3. A
  forma do 302 está nas sete cassettes `documento-numero-e-busca-*` e no
  replay; falta autorização para ensaiar o termo numérico.
- UnB, UFRJ, GO, PEN1, PEN2 e PEN4: **NOT_RUN** (fora do escopo desta
  rodada). A issue pede UnB e GO; GO publica menos linhas que a faixa e
  hoje seria recusado (ver a decisão).
