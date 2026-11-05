---
type: SEIObservation
title: Replay de conformidade do acervo público pelas peças de processo buscar
description: Lido, recusado e não aplicável por peça e instalação nos 52 cards públicos com a tela de Pesquisa (UnB, UFRJ, GO, PEN1 a PEN4), offline, 04/11/2026
---

# Replay do acervo — processo buscar — 04/11/2026

Toda resposta gravada do Controle (`procedimento_controlar`, GET) e da
tela de Pesquisa (`protocolo_pesquisar`, GET e POST) dos cards que
gravaram a Pesquisa passou pelas **mesmas funções** de
`ies processo buscar`, offline, na ordem do fluxo. Decisão e regras em
[replay do acervo](../decisoes/processo-buscar/replay-do-acervo.md).
Nenhuma requisição ao SEI; os ZIPs vêm do cache local, cada um conferido
pelo `archive_sha1` dos cards.

**Como rodar:** `IES_ACERVO_SAIDA=saida.md scripts/replay
pesquisa_web::acervo::replay_do_acervo_inteiro` (o
[script](../../scripts/replay) roda `cargo test --bin ies -- --ignored`).

**Classes.** *Lido*: a peça aceitou. *Recusado*: o subcódigo que o
comando publicaria. *Não aplicável*: a gravação não estabelece o que a
peça precisa (unidade, tela derivada, corpo do POST) ou a pesquisa foi em
documentos, que o comando não envia.

**Antes e depois da correção desta rodada.** Na primeira execução, as duas
listas de `processo-busca-go` foram recusadas com
`linha_de_pesquisa_com_mais_de_um_link_do_processo` (linha com acerto em
documento). Depois da correção, o passo 2 é lido e o passo 4 continua
recusado, agora por publicar 9 linhas para a faixa 1–10.

**Linhas ausentes da faixa (Goiás, decisão do proprietário de 04/11/2026).**
O HTML do passo 4 tem 9 registros (27 `tr`, três por registro; nenhum
`tr` ou link fora dos registros lidos): a linha não existe na resposta,
não é uma forma que o leitor pule. Antes, o passo 4 era recusado com
`linhas_da_tabela_diferentes_da_faixa_da_barra`; depois, é lido como
`lista_com_linhas_ausentes` (9 processos, faixa 1–10 da barra,
`linhas_ausentes_da_faixa: 1`) e o plano da página 2 sai dele, como no
passo 2. Regra em [processo buscar](../decisoes/processo-buscar.md#linhas-ausentes-da-faixa).

## Resultado

Cassettes lidas: 52; leituras registradas: 274. **Instantâneo de
04/11/2026**, só com as peças de `processo buscar`: a tabela abaixo é dessa
rodada. O mesmo teste hoje também passa as peças de `documento buscar` e da
pesquisa rápida e lê **109 cassettes, com 688 leituras**; a tabela atual
está no [replay da pesquisa de 05/11/2026](2026-11-05-pesquisa-acervo.md).

| Leitor | Instalação | Total | Lido | Recusado | Não aplicável |
|---|---|---:|---:|---|---|
| Controle (unidade e link) | unb | 8 | 8 | 0 | 0 |
| Controle (unidade e link) | ufrj | 7 | 7 | 0 | 0 |
| Controle (unidade e link) | go | 8 | 8 | 0 | 0 |
| Controle (unidade e link) | pen1 | 7 | 7 | 0 | 0 |
| Controle (unidade e link) | pen2 | 7 | 7 | 0 | 0 |
| Controle (unidade e link) | pen3 | 8 | 8 | 0 | 0 |
| Controle (unidade e link) | pen4 | 7 | 7 | 0 | 0 |
| Controle (unidade e link) | **todas** | 52 | 52 | 0 | 0 |
| tela da pesquisa (POST derivado) | unb | 21 | 21 | 0 | 0 |
| tela da pesquisa (POST derivado) | ufrj | 17 | 17 | 0 | 0 |
| tela da pesquisa (POST derivado) | go | 18 | 18 | 0 | 0 |
| tela da pesquisa (POST derivado) | pen1 | 17 | 17 | 0 | 0 |
| tela da pesquisa (POST derivado) | pen2 | 17 | 17 | 0 | 0 |
| tela da pesquisa (POST derivado) | pen3 | 21 | 21 | 0 | 0 |
| tela da pesquisa (POST derivado) | pen4 | 17 | 17 | 0 | 0 |
| tela da pesquisa (POST derivado) | **todas** | 128 | 128 | 0 | 0 |
| resposta do envio | unb | 11 | 5 | 1 (`resposta_da_pesquisa_nao_reconhecida` 1) | 5 (`lista_da_pesquisa_em_documentos` 5) |
| resposta do envio | ufrj | 8 | 6 | 0 | 2 (`lista_da_pesquisa_em_documentos` 2) |
| resposta do envio | go | 10 | 5 | 0 | 5 (`lista_da_pesquisa_em_documentos` 5) |
| resposta do envio | pen1 | 8 | 5 | 0 | 3 (`lista_da_pesquisa_em_documentos` 3) |
| resposta do envio | pen2 | 8 | 5 | 0 | 3 (`lista_da_pesquisa_em_documentos` 3) |
| resposta do envio | pen3 | 12 | 8 | 0 | 4 (`lista_da_pesquisa_em_documentos` 4) |
| resposta do envio | pen4 | 9 | 5 | 0 | 4 (`lista_da_pesquisa_em_documentos` 4) |
| resposta do envio | **todas** | 66 | 39 | 1 (`resposta_da_pesquisa_nao_reconhecida` 1) | 26 (`lista_da_pesquisa_em_documentos` 26) |
| plano da página 2 | unb | 2 | 2 | 0 | 0 |
| plano da página 2 | ufrj | 2 | 2 | 0 | 0 |
| plano da página 2 | go | 2 | 2 | 0 | 0 |
| plano da página 2 | pen1 | 2 | 2 | 0 | 0 |
| plano da página 2 | pen2 | 2 | 2 | 0 | 0 |
| plano da página 2 | pen3 | 2 | 2 | 0 | 0 |
| plano da página 2 | pen4 | 2 | 2 | 0 | 0 |
| plano da página 2 | **todas** | 14 | 14 | 0 | 0 |
| POST da tela de resultado | unb | 2 | 0 | 0 | 2 (`post_da_tela_de_resultado_sem_corpo_gravado` 2) |
| POST da tela de resultado | ufrj | 2 | 0 | 0 | 2 (`post_da_tela_de_resultado_sem_corpo_gravado` 2) |
| POST da tela de resultado | go | 3 | 0 | 0 | 3 (`post_da_tela_de_resultado_sem_corpo_gravado` 3) |
| POST da tela de resultado | pen1 | 2 | 0 | 0 | 2 (`post_da_tela_de_resultado_sem_corpo_gravado` 2) |
| POST da tela de resultado | pen2 | 2 | 0 | 0 | 2 (`post_da_tela_de_resultado_sem_corpo_gravado` 2) |
| POST da tela de resultado | pen3 | 1 | 0 | 0 | 1 (`post_da_tela_de_resultado_sem_corpo_gravado` 1) |
| POST da tela de resultado | pen4 | 2 | 0 | 0 | 2 (`post_da_tela_de_resultado_sem_corpo_gravado` 2) |
| POST da tela de resultado | **todas** | 14 | 0 | 0 | 14 (`post_da_tela_de_resultado_sem_corpo_gravado` 14) |

Formas lidas por instalação:

| Leitor | Forma | unb | ufrj | go | pen1 | pen2 | pen3 | pen4 |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| resposta do envio | `lista` | 2 | 2 | 1 | 2 | 2 | 2 | 2 |
| resposta do envio | `lista_com_linhas_ausentes` | 0 | 0 | 1 | 0 | 0 | 0 | 0 |
| resposta do envio | `processo_unico` | 3 | 3 | 3 | 3 | 3 | 4 | 3 |
| resposta do envio | `sem_resultado_em_documentos` | 0 | 1 | 0 | 0 | 0 | 2 | 0 |
| plano da página 2 | `pagina_2_planejada` | 2 | 2 | 2 | 2 | 2 | 1 | 2 |
| plano da página 2 | `uma_pagina_so` | 0 | 0 | 0 | 0 | 0 | 1 | 0 |

Totais declarados das listas lidas (`cassette#passo`):

- go `processo-busca-go--a7182b2ab8d7#2`: 1220
- go `processo-busca-go--a7182b2ab8d7#4`: 966 (9 linhas na faixa 1–10)
- pen1 `processo-busca-pen1--c06d64484496#2`: 2140
- pen1 `processo-busca-pen1--c06d64484496#4`: 937
- pen2 `processo-busca-pen2--d577a6abc93e#2`: 286
- pen2 `processo-busca-pen2--d577a6abc93e#4`: 174
- pen3 `processo-busca-pen3--08651f97ae2f#2`: 21
- pen3 `processo-busca-pen3--08651f97ae2f#4`: 5
- pen4 `processo-busca-pen4--1538da3f0602#2`: 123
- pen4 `processo-busca-pen4--1538da3f0602#4`: 89
- ufrj `processo-busca-ufrj--afdd24d769a4#2`: 24
- ufrj `processo-busca-ufrj--afdd24d769a4#4`: 44
- unb `processo-busca-unb--67c8f2ed35b6#2`: 794
- unb `processo-busca-unb--67c8f2ed35b6#4`: 208

Recusas por subcódigo:

- `resposta_da_pesquisa_nao_reconhecida` (1): `documento-numero-e-busca-unb--f1b386d8e7bc#11` (resposta do envio)

## Leitura

- **Sete instalações, mesmas peças.** Os 52 Controles publicam o link do
  menu na forma aceita e a unidade é reconhecida; as 128 telas de
  Pesquisa derivam o POST (regras do `onsubmit`, `maxlength`, órgãos
  selecionados, unidade). Isso inclui UFRJ e PEN2–PEN4, que nunca
  tinham passado pelo comando.
- **302.** Os 22 redirecionamentos para `procedimento_trabalhar` são
  conferidos contra a URL derivada da tela gravada (7 deles saem de
  `documento-numero-e-busca`, pesquisa pelo número de um documento).
- **Listas de processos.** 14 páginas 1 lidas (uma de Goiás com uma
  linha ausente da faixa), com total e faixa da
  barra, inclusive "N resultados" (PEN3, 5); o plano da página 2 deriva
  o POST de `navegar()` em todas as que têm mais de uma página.
- **Sem cobertura gravada:** "sem resultado" e páginas N > 1 da pesquisa
  de **processos**; ficam com o ensaio live.
- **UnB, `documento-numero-e-busca#11`:** a resposta do POST é só o
  formulário, sem tabela nem aviso. O comando falha fechado
  (`resposta_da_pesquisa_nao_reconhecida`) e não diz "sem resultado".
