---
type: SEIObservation
title: Replay de conformidade do acervo público pelas peças de documento buscar
description: Lido, recusado e não aplicável das 52 cassettes com a tela de Pesquisa (sete instalações) pelas peças de ies documento buscar, offline, 04/11/2026
---

# Replay do acervo — documento buscar — 04/11/2026

Toda resposta gravada da tela de Pesquisa (`protocolo_pesquisar`) das 52
cassettes que a gravaram passou, offline e na ordem do fluxo, pelas
**mesmas funções** de `ies documento buscar` (Refs #61): o POST em
documentos derivado da tela gravada, o envio (a resposta gravada no
lugar do POST), o plano da página 2 e, nos POSTs de `navegar()` da tela
de resultado (gravados sem corpo), a página seguinte lida com a faixa
que o plano da página 1 espera e conferida. É o mesmo replay do
[acervo de processo buscar](2026-11-04-processo-buscar-acervo.md), com
quatro leitores a mais; as linhas daquele relatório não mudaram.
Decisão em [documento buscar](../decisoes/documento-buscar.md). Nenhuma
requisição ao SEI; os ZIPs vêm do cache local, conferidos pelo
`archive_sha1` dos cards.

**Como rodar:** `IES_ACERVO_SAIDA=saida.md cargo test --release --bin ies
pesquisa_web::acervo::replay_do_acervo_inteiro -- --ignored`.

**Classes.** *Lido*: a peça aceitou. *Recusado*: o subcódigo que o
comando publicaria. *Não aplicável*: a gravação é de uma pesquisa em
processos (`P` marcado na resposta, ou 302 com `id_documento` vazio).

**Antes e depois.** Na primeira execução do leitor, a página 1 de
`documento-numero-e-busca-pen3#13` foi recusada com
`atributo_repetido_na_tag` (link do processo visitado, PEN3 5.0.4) e a
página 2 de `pesquisa-paginas-pen3#9` com
`linhas_da_tabela_diferentes_da_faixa_da_barra` (barra "16 resultados" na
última página). Depois das duas correções da decisão, as duas são lidas;
o resto não mudou.

## Resultado

Cassettes lidas: 52; leituras registradas no replay inteiro: 505.

| Leitor | Instalação | Total | Lido | Recusado | Não aplicável |
|---|---|---:|---:|---|---|
| tela da pesquisa (POST em documentos) | unb | 21 | 21 | 0 | 0 |
| tela da pesquisa (POST em documentos) | ufrj | 17 | 17 | 0 | 0 |
| tela da pesquisa (POST em documentos) | go | 18 | 18 | 0 | 0 |
| tela da pesquisa (POST em documentos) | pen1 | 17 | 17 | 0 | 0 |
| tela da pesquisa (POST em documentos) | pen2 | 17 | 17 | 0 | 0 |
| tela da pesquisa (POST em documentos) | pen3 | 21 | 21 | 0 | 0 |
| tela da pesquisa (POST em documentos) | pen4 | 17 | 17 | 0 | 0 |
| tela da pesquisa (POST em documentos) | **todas** | 128 | 128 | 0 | 0 |
| resposta do envio em documentos | unb | 11 | 6 | 1 (`resposta_da_pesquisa_nao_reconhecida` 1) | 4 (`pesquisa_gravada_em_processos` 2; `redirecionamento_para_processo_sem_id_documento` 2) |
| resposta do envio em documentos | ufrj | 8 | 4 | 0 | 4 (`pesquisa_gravada_em_processos` 2; `redirecionamento_para_processo_sem_id_documento` 2) |
| resposta do envio em documentos | go | 10 | 4 | 2 (`linhas_da_tabela_diferentes_da_faixa_da_barra` 2) | 4 (`pesquisa_gravada_em_processos` 2; `redirecionamento_para_processo_sem_id_documento` 2) |
| resposta do envio em documentos | pen1 | 8 | 4 | 0 | 4 (`pesquisa_gravada_em_processos` 2; `redirecionamento_para_processo_sem_id_documento` 2) |
| resposta do envio em documentos | pen2 | 8 | 4 | 0 | 4 (`pesquisa_gravada_em_processos` 2; `redirecionamento_para_processo_sem_id_documento` 2) |
| resposta do envio em documentos | pen3 | 12 | 8 | 0 | 4 (`pesquisa_gravada_em_processos` 2; `redirecionamento_para_processo_sem_id_documento` 2) |
| resposta do envio em documentos | pen4 | 9 | 5 | 0 | 4 (`pesquisa_gravada_em_processos` 2; `redirecionamento_para_processo_sem_id_documento` 2) |
| resposta do envio em documentos | **todas** | 66 | 35 | 3 (`linhas_da_tabela_diferentes_da_faixa_da_barra` 2; `resposta_da_pesquisa_nao_reconhecida` 1) | 28 (`pesquisa_gravada_em_processos` 14; `redirecionamento_para_processo_sem_id_documento` 14) |
| plano da página 2 em documentos | unb | 5 | 4 | 1 (`assinante_oculto_preenchido_na_pesquisa_em_documentos` 1) | 0 |
| plano da página 2 em documentos | ufrj | 2 | 2 | 0 | 0 |
| plano da página 2 em documentos | go | 3 | 3 | 0 | 0 |
| plano da página 2 em documentos | pen1 | 3 | 3 | 0 | 0 |
| plano da página 2 em documentos | pen2 | 3 | 3 | 0 | 0 |
| plano da página 2 em documentos | pen3 | 4 | 4 | 0 | 0 |
| plano da página 2 em documentos | pen4 | 4 | 4 | 0 | 0 |
| plano da página 2 em documentos | **todas** | 24 | 23 | 1 (`assinante_oculto_preenchido_na_pesquisa_em_documentos` 1) | 0 |
| página seguinte em documentos | unb | 2 | 2 | 0 | 0 |
| página seguinte em documentos | ufrj | 2 | 2 | 0 | 0 |
| página seguinte em documentos | go | 3 | 1 | 2 (`linhas_da_tabela_diferentes_da_faixa_da_barra` 2) | 0 |
| página seguinte em documentos | pen1 | 2 | 2 | 0 | 0 |
| página seguinte em documentos | pen2 | 2 | 2 | 0 | 0 |
| página seguinte em documentos | pen3 | 1 | 1 | 0 | 0 |
| página seguinte em documentos | pen4 | 2 | 2 | 0 | 0 |
| página seguinte em documentos | **todas** | 14 | 12 | 2 (`linhas_da_tabela_diferentes_da_faixa_da_barra` 2) | 0 |

Formas lidas por instalação:

| Leitor | Forma | unb | ufrj | go | pen1 | pen2 | pen3 | pen4 |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| resposta do envio em documentos | `documento_unico` | 1 | 1 | 1 | 1 | 1 | 2 | 1 |
| resposta do envio em documentos | `lista` | 5 | 2 | 3 | 3 | 3 | 4 | 4 |
| resposta do envio em documentos | `sem_resultado` | 0 | 1 | 0 | 0 | 0 | 2 | 0 |
| plano da página 2 em documentos | `pagina_2_planejada` | 4 | 2 | 2 | 2 | 2 | 1 | 2 |
| plano da página 2 em documentos | `uma_pagina_so` | 0 | 0 | 1 | 1 | 1 | 3 | 2 |
| página seguinte em documentos | `pagina_seguinte_conferida` | 2 | 2 | 1 | 2 | 2 | 1 | 2 |

Totais declarados das listas lidas (`cassette#passo`, pesquisas em
documentos):

- go `documento-buscar-texto-go--724f88e4c47b#2`: 2
- go `documento-numero-e-busca-go--f26b4140acb8#11`: 27338
- go `pesquisa-paginas-go--c4018448aa04#3`: 27338
- pen1 `documento-numero-e-busca-pen1--f274a10a3ee0#11`: 2
- pen1 `documento-numero-e-busca-pen1--f274a10a3ee0#13`: 609
- pen1 `pesquisa-paginas-pen1--671f8ac44dd3#7`: 4490
- pen2 `documento-numero-e-busca-pen2--fac3b99dd330#11`: 2
- pen2 `documento-numero-e-busca-pen2--fac3b99dd330#13`: 124
- pen2 `pesquisa-paginas-pen2--fb240fd24b61#7`: 676
- pen3 `documento-busca-autocompletar-pen3--7dae7da4031b#3`: 6
- pen3 `documento-busca-autocompletar-pen3--7dae7da4031b#6`: 5
- pen3 `documento-numero-e-busca-pen3--2964627c9d6b#13`: 5
- pen3 `pesquisa-paginas-pen3--734d969bfe9c#8`: 16
- pen4 `documento-numero-e-busca-pen4--831054375369#11`: 2
- pen4 `documento-numero-e-busca-pen4--831054375369#13`: 55
- pen4 `pesquisa-paginas-pen4--ccbd16c8e872#3`: 2
- pen4 `pesquisa-paginas-pen4--ccbd16c8e872#5`: 238
- ufrj `documento-numero-e-busca-ufrj--d0b23f01b824#13`: 71
- ufrj `pesquisa-paginas-ufrj--4f3d961f8042#7`: 177
- unb `documento-busca-autocompletar-unb--3dfc2c884999#3`: 268
- unb `documento-busca-autocompletar-unb--3dfc2c884999#6`: 902
- unb `documento-busca-autocompletar-unb--3dfc2c884999#9`: 417
- unb `documento-numero-e-busca-unb--f1b386d8e7bc#13`: 1105
- unb `pesquisa-paginas-unb--b6147abfa4ab#8`: 4883

Recusas por subcódigo (leitores de documento buscar):

- `assinante_oculto_preenchido_na_pesquisa_em_documentos` (1): `documento-busca-autocompletar-unb--3dfc2c884999#9` (plano da página 2 em documentos)
- `linhas_da_tabela_diferentes_da_faixa_da_barra` (4): `documento-numero-e-busca-go--f26b4140acb8#13` (resposta do envio em documentos), `pesquisa-paginas-go--c4018448aa04#4` (página seguinte em documentos), `pesquisa-paginas-go--c4018448aa04#5` (página seguinte em documentos), `pesquisa-paginas-go--c4018448aa04#8` (resposta do envio em documentos)
- `resposta_da_pesquisa_nao_reconhecida` (1): `documento-numero-e-busca-unb--f1b386d8e7bc#11` (resposta do envio em documentos)

## Leitura

- **As 26 pesquisas em documentos** que o replay de `processo buscar`
  classificou como não aplicáveis foram lidas: 24 listas com total e
  faixa (inclusive "N resultados"), e as 2 de Goiás com menos linhas que
  a faixa recusadas; mais os 3 "sem resultado" e a recusa da UnB ("Erro
  realizando pesquisa" em `divInfraMensagens`, que o comando também não
  reconhece).
- **Número SEI exato.** Os 8 redirecionamentos com `id_documento` (7 de
  `documento-numero-e-busca`, 1 de `documento-busca-autocompletar-pen3`)
  viram `documento_unico`; os 14 de `processo-busca` (`id_documento`
  vazio) são não aplicáveis.
- **Paginação.** 23 planos da página 2 derivam o POST de `navegar()` em
  documentos (o da UnB com assinante preenchido na tela de resultado é
  recusado, como deve). Das 14 páginas seguintes gravadas, 12 batem com
  o plano feito da página 1 (total e faixa); as 2 de Goiás têm menos
  linhas que a faixa.
- **Tela.** As 128 telas de Pesquisa das sete instalações derivam o POST
  em documentos (rádio `D`, duas caixas de tipo marcadas, órgãos).
