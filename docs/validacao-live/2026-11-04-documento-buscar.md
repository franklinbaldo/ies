---
type: SEIObservation
title: Ensaio live de documento buscar na PEN3 5.0.4
description: PASS/FAIL/NOT_RUN de ies documento buscar na PEN3 (NUP com um e com dois documentos, parcial, --pagina 2 e última, fora do intervalo, inexistente, texto, --unidade divergente, saída humana, número SEI e NUP só de dígitos), 04/11/2026
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
| 11 | `0001018` (número SEI exato, só dígitos) | `documento_unico` | `documento_unico`, ids 1518 / 1517 | 3 | **PASS** |
| 12 | `1018` (o mesmo sem zeros à esquerda) | resposta observada | `documento_unico`, ids 1518 / 1517 | 3 | **PASS** |
| 13 | `98390000036202699` (NUP do caso 1 só com dígitos) | resposta observada | `sem_resultado`, saída 0 | 3 | **PASS** |

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

## Termos só de dígitos (Cloudflare)

Rodada autorizada pelo proprietário, 18:24–18:41 UTC, binário do ramo
`slice-busca-compat` (commit `1cc78b2`), mesma sessão, **≥ 60 s** entre o
fim de um comando e o início do seguinte, sem retry, 7 comandos e 19
túneis contados no relay CONNECT. Comandos 1, 5 e 6 são os casos 11–13
acima; 2, 3, 4 e 5 são de `processo buscar` e estão no
[relatório dele](2026-11-04-processo-buscar.md#termos-só-de-dígitos-cloudflare).

| # | Início (UTC) | Comando | Túneis | Resposta |
| ---: | --- | --- | ---: | --- |
| 1 | 18:24:02 | `documento buscar 0001018` | 3 | 302 → `documento_unico` |
| 2 | 18:25:18 | `processo buscar 98390.000039/2026-22` (controle) | 3 | 302 → `processo_unico` |
| 3 | 18:26:37 | `processo buscar 98390000039202622` | 1 | **403 Cloudflare no GET do Controle** |
| — | — | espera de 10 min 53 s | — | — |
| 4 | 18:37:30 | `processo buscar 98390.000038/2026-88` (controle) | 3 | 302 → `processo_unico` |
| 5 | 18:38:46 | `processo buscar 98390000038202688` | 3 | 302 → `processo_unico` (o mesmo id do 4) |
| 6 | 18:40:01 | `documento buscar 1018` | 3 | 302 → `documento_unico` |
| 7 | 18:41:14 | `documento buscar 98390000036202699` | 3 | `sem_resultado` |

- **O termo não causa o bloqueio.** No comando 3 o 403 veio na
  **primeira** requisição, o GET do Controle (1 túnel, 1.286 bytes
  enviados, como em todos os GETs), que não leva o termo; o POST nem foi
  enviado. O IES publicou `bloqueio_de_seguranca_da_instalacao`, saída 1,
  sem repetir. Os outros quatro POSTs só de dígitos (7 e 17 dígitos)
  passaram. A borda da PEN3 bloqueia pela origem ou pelo ritmo agregado
  (a saída HTTPS é compartilhada com outras sessões, não medidas aqui),
  não pelo texto pesquisado.
- **Número SEI exato.** `0001018` e `1018` respondem o mesmo 302 com
  `id_documento=1518` (`id_procedimento=1517`): a PEN3 casa o número SEI
  sem exigir os zeros à esquerda. O IES envia o termo como digitado.
- **NUP só de dígitos em documentos** não casa (`sem_resultado`), embora
  o NUP formatado (caso 1) case; em processos, o NUP só de dígitos casa
  (comando 5). `sem_resultado` continua resposta observada.

## Não executado

- Número SEI **parcial** que case mais de um documento (lista por
  número): **NOT_RUN**.
- UnB, UFRJ, GO, PEN1, PEN2 e PEN4: **NOT_RUN** (fora do escopo desta
  rodada). A issue pede UnB e GO; GO publica menos linhas que a faixa e
  era recusado; agora sai com `linhas_ausentes_da_faixa` (ver a decisão),
  ainda não ensaiado ao vivo.
