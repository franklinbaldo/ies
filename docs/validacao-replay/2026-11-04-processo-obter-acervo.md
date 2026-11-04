---
type: SEIObservation
title: Replay de conformidade do acervo público pelos leitores de processo NUP
description: Lido, recusado e não aplicável por leitor e instalação nas 495 cassettes públicas (UnB, UFRJ, GO, PEN1 a PEN4), offline, 04/11/2026
---

# Replay do acervo — processo NUP — 04/11/2026

Toda resposta gravada de tela do processo (`procedimento_trabalhar`),
árvore (`procedimento_visualizar`), histórico
(`procedimento_consultar_historico`, GET e POST), propriedades
(`procedimento_alterar`/`procedimento_consultar`) e marcadores
(`andamento_marcador_gerenciar`, 200 e 302) das cassettes públicas passou
pelos **mesmos leitores** de `ies processo NUP`, offline. Decisão e
regras de identidade em
[replay do acervo](../decisoes/processo-obter/replay-do-acervo.md).
Nenhuma requisição ao SEI; só o download dos 29 ZIPs do Internet Archive,
cada um conferido pelo `archive_sha1` dos cards.

**Escopo.** 280 cards com alguma dessas operações (de 495); 13.092
leituras. Replay offline não é ensaio live: prova que cada tela gravada
é lida ou recusada com subcódigo, não o fluxo HTTP do IES.

**Classes.** *Lido*: o leitor aceitou. *Recusado*: o subcódigo tipado
que a CLI publicaria (em situação e natureza, que nunca falham o comando,
é o campo que sairia `null`). *Não aplicável*: a gravação não estabelece
a identidade, ou o próprio fluxo não pediria a tela (motivo da CLI).

## Leitores por instalação

| Leitor | Instalação | Total | Lido | Recusado | Não aplicável |
|---|---|---:|---:|---|---|
| tela do processo | unb | 129 | 127 | 2 (`endereco_da_arvore_de_outro_processo` 2) | 0 |
| tela do processo | ufrj | 102 | 101 | 1 (`endereco_da_arvore_de_outro_processo` 1) | 0 |
| tela do processo | go | 50 | 49 | 1 (`endereco_da_arvore_de_outro_processo` 1) | 0 |
| tela do processo | pen1 | 50 | 47 | 3 (`arvore_ausente_na_tela_do_processo` 2; `endereco_da_arvore_de_outro_processo` 1) | 0 |
| tela do processo | pen2 | 44 | 43 | 1 (`endereco_da_arvore_de_outro_processo` 1) | 0 |
| tela do processo | pen3 | 47 | 46 | 1 (`endereco_da_arvore_de_outro_processo` 1) | 0 |
| tela do processo | pen4 | 45 | 44 | 1 (`endereco_da_arvore_de_outro_processo` 1) | 0 |
| tela do processo | **todas** | 467 | 457 | 10 (`arvore_ausente_na_tela_do_processo` 2; `endereco_da_arvore_de_outro_processo` 8) | 0 |
| árvore | unb | 397 | 396 | 1 (`no_do_processo_ausente_na_arvore` 1) | 0 |
| árvore | ufrj | 386 | 385 | 1 (`no_do_processo_ausente_na_arvore` 1) | 0 |
| árvore | go | 276 | 267 | 9 (`documento_movido_presente_entre_os_documentos` 8; `no_do_processo_ausente_na_arvore` 1) | 0 |
| árvore | pen1 | 249 | 240 | 9 (`documento_movido_presente_entre_os_documentos` 8; `no_do_processo_ausente_na_arvore` 1) | 0 |
| árvore | pen2 | 253 | 244 | 9 (`documento_movido_presente_entre_os_documentos` 8; `no_do_processo_ausente_na_arvore` 1) | 0 |
| árvore | pen3 | 254 | 245 | 9 (`documento_movido_presente_entre_os_documentos` 8; `no_do_processo_ausente_na_arvore` 1) | 0 |
| árvore | pen4 | 258 | 257 | 1 (`no_do_processo_ausente_na_arvore` 1) | 0 |
| árvore | **todas** | 2073 | 2034 | 39 (`documento_movido_presente_entre_os_documentos` 32; `no_do_processo_ausente_na_arvore` 7) | 0 |
| pastas | unb | 24 | 24 | 0 | 0 |
| pastas | ufrj | 18 | 18 | 0 | 0 |
| pastas | **todas** | 42 | 42 | 0 | 0 |
| andamentos | unb | 19 | 19 | 0 | 0 |
| andamentos | ufrj | 18 | 18 | 0 | 0 |
| andamentos | go | 17 | 17 | 0 | 0 |
| andamentos | pen1 | 19 | 19 | 0 | 0 |
| andamentos | pen2 | 17 | 17 | 0 | 0 |
| andamentos | pen3 | 13 | 13 | 0 | 0 |
| andamentos | pen4 | 17 | 17 | 0 | 0 |
| andamentos | **todas** | 120 | 120 | 0 | 0 |
| propriedades | unb | 21 | 21 | 0 | 0 |
| propriedades | ufrj | 21 | 21 | 0 | 0 |
| propriedades | go | 20 | 20 | 0 | 0 |
| propriedades | pen1 | 11 | 11 | 0 | 0 |
| propriedades | pen2 | 33 | 33 | 0 | 0 |
| propriedades | pen3 | 33 | 33 | 0 | 0 |
| propriedades | pen4 | 20 | 20 | 0 | 0 |
| propriedades | **todas** | 159 | 159 | 0 | 0 |
| marcadores | unb | 8 | 8 | 0 | 0 |
| marcadores | ufrj | 9 | 9 | 0 | 0 |
| marcadores | go | 7 | 7 | 0 | 0 |
| marcadores | pen1 | 9 | 9 | 0 | 0 |
| marcadores | pen2 | 10 | 9 | 0 | 1 (`tabela_de_marcadores_nao_publicada_na_tela` 1) |
| marcadores | pen3 | 9 | 9 | 0 | 0 |
| marcadores | pen4 | 9 | 9 | 0 | 0 |
| marcadores | **todas** | 61 | 60 | 0 | 1 (`tabela_de_marcadores_nao_publicada_na_tela` 1) |
| pastas | `aberta_por_abrir_pastas` | 15 | ufrj, unb |
| pastas | `fechada_com_acao_de_abrir` | 27 | ufrj, unb |
| pastas | `mista` | 4 | ufrj, unb |
| pastas | `por_identidade` | 71 | ufrj, unb |
| andamentos | `get_completo` | 2 | unb |
| andamentos | `get_resumido` | 110 | go, pen1, pen2, pen3, pen4, ufrj, unb |
| andamentos | `paginador_vazio` | 120 | go, pen1, pen2, pen3, pen4, ufrj, unb |
| andamentos | `post_completo` | 7 | go, pen1, pen2, pen3, pen4, ufrj, unb |
| andamentos | `post_resumido` | 1 | unb |
| propriedades | `procedimento_alterar` | 158 | go, pen1, pen2, pen3, pen4, ufrj, unb |
| propriedades | `procedimento_consultar` | 1 | ufrj |
| propriedades | `publico` | 130 | go, pen1, pen2, pen3, pen4, ufrj, unb |
| propriedades | `restrito` | 29 | go, pen2, pen3, pen4, ufrj, unb |
| marcadores | `302_sem_marcadores` | 32 | go, pen1, pen2, pen3, pen4, ufrj, unb |
| marcadores | `tabela` | 28 | go, pen1, pen2, pen3, pen4, ufrj, unb |
| tela do processo | `arvore_ausente_na_tela_do_processo` | pen1 | 2 | processo-excluir-pen1--fc4c8d59a2b2#12, processo-excluir-pen1--fc4c8d59a2b2#15 |
| tela do processo | `endereco_da_arvore_de_outro_processo` | go | 1 | documento-numero-e-busca-go--f26b4140acb8#2 |
| tela do processo | `endereco_da_arvore_de_outro_processo` | pen1 | 1 | documento-numero-e-busca-pen1--f274a10a3ee0#2 |
| tela do processo | `endereco_da_arvore_de_outro_processo` | pen2 | 1 | documento-numero-e-busca-pen2--fac3b99dd330#2 |
| tela do processo | `endereco_da_arvore_de_outro_processo` | pen3 | 1 | documento-numero-e-busca-pen3--2964627c9d6b#2 |
| tela do processo | `endereco_da_arvore_de_outro_processo` | pen4 | 1 | documento-numero-e-busca-pen4--831054375369#2 |
| tela do processo | `endereco_da_arvore_de_outro_processo` | ufrj | 1 | documento-numero-e-busca-ufrj--d0b23f01b824#2 |
| tela do processo | `endereco_da_arvore_de_outro_processo` | unb | 2 | documento-numero-e-busca-unb--f1b386d8e7bc#2, documento-propriedades-unb--4f50b7e8fb79#2 |
| árvore | `documento_movido_presente_entre_os_documentos` | go | 8 | documento-mover-cancelar-go--a152c0306b45#53, documento-mover-cancelar-go--a152c0306b45#55, documento-mover-cancelar-go--a152c0306b45#64 |
| árvore | `documento_movido_presente_entre_os_documentos` | pen1 | 8 | documento-mover-cancelar-pen1--0c59af5df1ba#46, documento-mover-cancelar-pen1--0c59af5df1ba#49, documento-mover-cancelar-pen1--0c59af5df1ba#58 |
| árvore | `documento_movido_presente_entre_os_documentos` | pen2 | 8 | documento-mover-cancelar-pen2--2375a0354d05#53, documento-mover-cancelar-pen2--2375a0354d05#56, documento-mover-cancelar-pen2--2375a0354d05#65 |
| árvore | `documento_movido_presente_entre_os_documentos` | pen3 | 8 | documento-mover-cancelar-pen3--60eb468849de#53, documento-mover-cancelar-pen3--60eb468849de#56, documento-mover-cancelar-pen3--60eb468849de#65 |
| árvore | `no_do_processo_ausente_na_arvore` | go | 1 | processo-excluir-go--1588cdeda71d#11 |
| árvore | `no_do_processo_ausente_na_arvore` | pen1 | 1 | processo-excluir-pen1--fc4c8d59a2b2#10 |
| árvore | `no_do_processo_ausente_na_arvore` | pen2 | 1 | processo-excluir-pen2--85da3ad2ec15#11 |
| árvore | `no_do_processo_ausente_na_arvore` | pen3 | 1 | processo-excluir-pen3--b7b4154fa995#11 |
| árvore | `no_do_processo_ausente_na_arvore` | pen4 | 1 | processo-excluir-pen4--2b679bbd6be3#11 |
| árvore | `no_do_processo_ausente_na_arvore` | ufrj | 1 | processo-excluir-ufrj--30bc65e0e270#11 |
| árvore | `no_do_processo_ausente_na_arvore` | unb | 1 | processo-excluir-unb--160fadda318c#10 |

Leituras derivadas da mesma árvore (todas as instalações; por instalação
no teste):

| Leitor | Instalação | Total | Lido | Recusado | Não aplicável |
|---|---|---:|---:|---|---|
| situação (aberto_em) | **todas** | 2034 | 2013 | 21 (`html_da_raiz_fora_da_forma_observada` 21) | 0 |
| natureza | **todas** | 2034 | 2034 | 0 | 0 |
| link do histórico | **todas** | 2034 | 2034 | 0 | 0 |
| link das propriedades | **todas** | 2034 | 2034 | 0 | 0 |
| link dos marcadores | **todas** | 2034 | 1743 | 0 | 291 (`tela_de_marcadores_nao_publicada_na_barra` 291) |

## Destaques

Maiores leituras de árvore por `nos` (uma por cassette):

| Cassette | Instalação | Passo | nos | documentos | pastas | anexados | movidos | aguarde | Formas |
|---|---|---:|---:|---:|---:|---:|---:|---:|---|
| documento-lote-pelo-zip-unb--05402e0f931b | unb | 9 | 166 | 157 | 8 | 0 | 0 | 0 |  |
| processo-arquivo-apenas-alem-de-100-unb--dff8c4a20a0d | unb | 9 | 166 | 157 | 8 | 0 | 0 | 0 |  |
| processo-em-pastas-unb--0bc73a70deac | unb | 9 | 166 | 157 | 8 | 0 | 0 | 0 |  |
| processo-pasta-mista-ufrj--65861ad4af07 | ufrj | 435 | 25 | 22 | 2 | 0 | 0 | 0 |  |
| processo-pasta-mista-unb--dffe2d9379eb | unb | 433 | 25 | 22 | 2 | 0 | 0 | 0 |  |
| processo-arquivo-documentos-unb--4462ca5487d7 | unb | 3 | 16 | 15 | 0 | 0 | 0 | 0 |  |
| cadastro-listas-go--beb707305040 | go | 10 | 8 | 7 | 0 | 0 | 0 | 0 |  |
| cadastro-listas-ufrj--d9711bdddb18 | ufrj | 10 | 7 | 6 | 0 | 0 | 0 | 0 |  |

**Árvore maior: 166 nós** (157 documentos, 8 pastas) na UnB, a resposta
de "Abrir todas as Pastas" de `processo-em-pastas-unb` (a mesma árvore
nas cassettes `processo-arquivo-apenas-alem-de-100` e
`documento-lote-pelo-zip`). Anexados e movidos: no máximo 1 por árvore
(cenários `processo-anexar-*` nas sete instalações; `documento-mover-cancelar`
em GO e PEN1 a PEN3).

Maiores leituras de árvore por `aguarde` (uma por cassette):

| Cassette | Instalação | Passo | aguarde | nos | documentos | pastas | Formas |
|---|---|---:|---:|---:|---:|---:|---|
| processo-pasta-mista-ufrj--65861ad4af07 | ufrj | 413 | 1 | 4 | 1 | 2 |  |
| documento-lote-pelo-zip-unb--05402e0f931b | unb | 8 | 1 | 26 | 17 | 8 |  |
| processo-arquivo-apenas-alem-de-100-unb--dff8c4a20a0d | unb | 8 | 1 | 26 | 17 | 8 |  |
| processo-em-pastas-unb--0bc73a70deac | unb | 8 | 1 | 26 | 17 | 8 |  |
| processo-pasta-mista-unb--dffe2d9379eb | unb | 411 | 1 | 4 | 1 | 2 |  |

**Pastas.** `AGUARDE` só na UnB e na UFRJ; todas as 27 árvores fechadas
publicam a ação `ABRIR_PASTAS` conferida, e as 15 respostas de
`abrir_pastas=1` vêm sem `AGUARDE`. Verificação `mista` vista na UnB e na
UFRJ (antes da correção abaixo, a UFRJ era recusada).

Maiores leituras de andamentos por `linhas` (uma por cassette):

| Cassette | Instalação | Passo | linhas | total_declarado | Formas |
|---|---|---:|---:|---:|---|
| processo-andamento-completo-ufrj--6154b2b9d7c9 | ufrj | 15 | 57 | 57 | paginador_vazio, post_completo |
| processo-andamento-ufrj--6aa848bf537c | ufrj | 10 | 28 | 28 | paginador_vazio, get_resumido |
| processo-nup-inexistente-ufrj--e88e670e57a8 | ufrj | 11 | 28 | 28 | paginador_vazio, get_resumido |
| processo-andamento-completo-unb--ddd49eff108e | unb | 30 | 26 | 26 | paginador_vazio, post_completo |
| processo-andamento-completo-pen4--083571aeb3aa | pen4 | 15 | 23 | 23 | paginador_vazio, post_completo |
| processo-andamento-completo-go--8b8110e1b484 | go | 11 | 10 | 10 | paginador_vazio, post_completo |
| processo-andamento-completo-pen1--2418e3a42844 | pen1 | 15 | 8 | 8 | paginador_vazio, post_completo |
| processo-andamento-completo-pen2--cc1197c62434 | pen2 | 15 | 6 | 6 | paginador_vazio, post_completo |

**Histórico maior: 57 linhas** (UFRJ, visão `P` pelo POST
`verHistorico('P')`). Em **todas** as 120 páginas a
`divInfraAreaPaginacaoSuperior` está vazia e as linhas lidas igualam o
`total_declarado` da legenda; nenhuma legenda fora das duas formas. O
acervo não tem histórico maior que os 75 vistos ao vivo em GO: acima
disso, paginação continua **NOT_RUN**.

## Formas vistas

| Leitor | Forma | Ocorrências | Instalações |
|---|---|---:|---|
| pastas | `aberta_por_abrir_pastas` | 15 | ufrj, unb |
| pastas | `fechada_com_acao_de_abrir` | 27 | ufrj, unb |
| pastas | `mista` | 4 | ufrj, unb |
| pastas | `por_identidade` | 71 | ufrj, unb |
| situação (aberto_em) | `nenhum_andamento_aberto` | 231 | go, pen1, pen2, pen3, pen4, ufrj, unb |
| situação (aberto_em) | `uma_unidade` | 1659 | go, pen1, pen2, pen3, pen4, ufrj, unb |
| situação (aberto_em) | `uma_unidade_com_atribuicao` | 41 | go, pen1, pen2, pen3, pen4, ufrj, unb |
| situação (aberto_em) | `varias_unidades` | 81 | go, pen1, pen2, pen3, pen4, ufrj, unb |
| situação (aberto_em) | `varias_unidades_com_atribuicao` | 1 | go |
| natureza | `icone:documento_cancelado` | 6 | pen4, ufrj |
| natureza | `icone:documento_email` | 30 | go, pen1, pen2, pen4, ufrj |
| natureza | `icone:documento_html` | 81 | pen1, pen2, pen3, pen4, ufrj |
| natureza | `icone:documento_interno` | 2965 | go, pen1, pen2, pen3, pen4, ufrj, unb |
| natureza | `icone:documento_pdf` | 75 | pen3, pen4, ufrj, unb |
| natureza | `icone:documento_txt` | 19 | go, pen1, pen3, pen4 |
| natureza | `icone:documento_zip` | 6 | pen4 |
| andamentos | `get_completo` | 2 | unb |
| andamentos | `get_resumido` | 110 | go, pen1, pen2, pen3, pen4, ufrj, unb |
| andamentos | `paginador_vazio` | 120 | go, pen1, pen2, pen3, pen4, ufrj, unb |
| andamentos | `post_completo` | 7 | go, pen1, pen2, pen3, pen4, ufrj, unb |
| andamentos | `post_resumido` | 1 | unb |
| propriedades | `procedimento_alterar` | 158 | go, pen1, pen2, pen3, pen4, ufrj, unb |
| propriedades | `procedimento_consultar` | 1 | ufrj |
| propriedades | `publico` | 130 | go, pen1, pen2, pen3, pen4, ufrj, unb |
| propriedades | `restrito` | 29 | go, pen2, pen3, pen4, ufrj, unb |
| marcadores | `302_sem_marcadores` | 32 | go, pen1, pen2, pen3, pen4, ufrj, unb |
| marcadores | `tabela` | 28 | go, pen1, pen2, pen3, pen4, ufrj, unb |

Ícones: sete radicais, todos na forma `svg/<radical>.svg`; nenhum
documento com ícone não lido. `aberto_em`: quatro formas lidas e uma
forma nova (abaixo).

## Achados

### Corrigido nesta fatia

1. **Barra concatenada em documento de pasta** (UFRJ;
   `barra_de_no_fora_da_forma_observada`, 16 árvores e 6 leituras de
   pastas de `processo-pasta-mista-ufrj`). Depois do `.html` de cada
   documento a UFRJ e GO servem
   `Nos[i].acoes = Nos[i].acoes.concat('');` (1.104 linhas no acervo,
   sempre com o literal vazio). Fora de pasta a linha já era ignorada;
   dentro, a árvore inteira era recusada. RED: 1 de 2 testes
   (`arvore_processo::tests_pastas_concatenacao`), GREEN com a linha exata
   aceita depois da barra lida; concatenação com conteúdo continua
   recusada. Depois: UFRJ 385/386 árvores e 18/18 pastas lidas.
   [Contrato das pastas](../../contracts/cli/processo-obter-pastas.md).

### Abertos (próximas fatias)

1. **Documento movido de volta** (GO, PEN1, PEN2, PEN3;
   `documento_movido_presente_entre_os_documentos`, 32 árvores de
   `documento-mover-cancelar-*`). O documento foi movido para outro
   processo e depois devolvido: a árvore traz o `DOCUMENTO_MOVIDO`
   (`<id>-<rel>`, "Documento movido para o processo …") **e** o
   `DOCUMENTO` de mesmo id. A recusa é a regra contratada; aceitar exige
   decidir o que `documentos_movidos` publica quando o documento voltou.
2. **Processo sobrestado** (todas as sete instalações, 21 árvores de
   `processo-sobrestar-retomar-*`): o `.html` da raiz é
   `Processo sobrestado na unidade <a …>SIGLA</a>.<br />`. Hoje
   `aberto_em: null` com `html_da_raiz_fora_da_forma_observada`, como o
   contrato manda; publicar o sobrestamento é decisão de contrato.
3. **Tela de marcadores sem tabela** (PEN2, 1 leitura de
   `marcador-escritas-pen2`, depois de remover marcadores): GET 200 com
   `divInfraAreaTabela` vazia e `hdnInfraNroItens` 0, não o 302 de zero
   marcadores. Hoje `tabela_de_marcadores_nao_publicada_na_tela` (não
   lido); afirmar `[]` exige uma segunda observação.

### Esperados (não são achados)

- `no_do_processo_ausente_na_arvore` (7) e
  `arvore_ausente_na_tela_do_processo` (2, PEN1): telas gravadas depois de
  `processo-excluir-*`; a árvore "Árvore Montar" vem sem nós. No passo 1
  o fluxo trata como link envelhecido.
- `endereco_da_arvore_de_outro_processo` (8): a pesquisa rápida abriu a
  tela por outro protocolo (número de documento em
  `documento-numero-e-busca-*`; também `documento-propriedades-unb`):
  `id_protocolo` não é o do processo da tela. `ies processo NUP` só
  pesquisa NUP, e confere o mesmo.
- Link dos marcadores não publicado na barra (291): a barra sem
  `andamento_marcador_gerenciar`; a CLI publica o motivo, sem pedir a tela.

## Reproduzir

```sh
mkdir -p ~/.cache/ies   # os 29 ZIPs, nomes originais, por curl -fL
IES_ACERVO_SAIDA=/tmp/acervo.md cargo test --bin ies \
  replay_do_acervo_inteiro -- --ignored
```

Cerca de 50 s em `dev`. O teste falha se algum card não conferir.
