---
type: SEIObservation
title: Replay de conformidade do acervo público pelos leitores do Controle de Processos
description: Lido, recusado e não aplicável por leitor e instalação nas 757 respostas de procedimento_controlar das 495 cassettes públicas, páginas publicadas, continuidade entre páginas gravadas e casos negativos, offline, 04/11/2026
---

# Replay do acervo — Controle de Processos — 04/11/2026

Toda resposta gravada de `procedimento_controlar` (GET e POST) das 495
cassettes públicas passou pelos **mesmos leitores** de `ies unidade
listar-processos`, `recebidos` e `gerados`, offline (Refs #57). Decisão,
leitores e regras de identidade no
[apêndice da decisão](../decisoes/unidade-listar-processos/replay-do-acervo.md).
Nenhuma requisição ao SEI: os 29 ZIPs do cache, cada um conferido pelo
`archive_sha1` e cada membro pelo `cassette_sha256_lf` dos cards.

**Escopo.** 495 cassettes, 757 respostas (743 GET, 14 POST), 8.237
leituras, 1.397 páginas de lista. Replay offline não é ensaio live:
prova que cada tela gravada é lida ou recusada com subcódigo, não o
fluxo HTTP do IES. O ensaio live de paginação está em
[validação live](../validacao-live/).

**Classes.** *Lido*: o leitor aceitou. *Recusado*: o subcódigo tipado que
a CLI publicaria. *Não aplicável*: o comando não leria a tela com esse
leitor (outra visão: trocaria por POST; `tipo_filtro` na URL servida: lê
a URL guardada; lista R recusada antes) — com o motivo.

## Leitores por instalação

Recebidos e Gerados fora de GO: lido em todas as telas R lidas, sem
recusa (não aplicável: 4 por instalação, `lista_r_nao_lida`); linhas
omitidas abaixo.

| Leitor | Instalação | Total | Lido | Recusado | Não aplicável |
|---|---|---:|---:|---|---|
| transporte (GET/POST) | unb | 283 | 269 | 14 (`redirecionamento_recusado` 14) | 0 |
| transporte (GET/POST) | ufrj | 95 | 87 | 8 (`redirecionamento_recusado` 8) | 0 |
| transporte (GET/POST) | go | 90 | 84 | 6 (`redirecionamento_recusado` 6) | 0 |
| transporte (GET/POST) | pen1 | 72 | 72 | 0 | 0 |
| transporte (GET/POST) | pen2 | 69 | 69 | 0 | 0 |
| transporte (GET/POST) | pen3 | 76 | 76 | 0 | 0 |
| transporte (GET/POST) | pen4 | 72 | 72 | 0 | 0 |
| transporte (GET/POST) | **todas** | 757 | 729 | 28 (`redirecionamento_recusado` 28) | 0 |
| visão (hdnTipoVisualizacao) | **todas** | 729 | 729 | 0 | 0 |
| unidade (lnkInfraUnidade) | **todas** | 729 | 729 | 0 | 0 |
| --unidade da tela anterior | unb | 184 | 176 | 8 (`unidade_da_tela_diferente_da_solicitada` 8) | 0 |
| --unidade da tela anterior | ufrj | 14 | 6 | 8 (`unidade_da_tela_diferente_da_solicitada` 8) | 0 |
| --unidade da tela anterior | go | 12 | 6 | 6 (`unidade_da_tela_diferente_da_solicitada` 6) | 0 |
| --unidade da tela anterior | pen1 | 6 | 6 | 0 | 0 |
| --unidade da tela anterior | pen2 | 6 | 6 | 0 | 0 |
| --unidade da tela anterior | pen3 | 6 | 6 | 0 | 0 |
| --unidade da tela anterior | pen4 | 6 | 6 | 0 | 0 |
| --unidade da tela anterior | **todas** | 234 | 212 | 22 (`unidade_da_tela_diferente_da_solicitada` 22) | 0 |
| lista R (juntar) | unb | 269 | 265 | 1 (`filtro_de_marcador_ativo_no_controle` 1) | 3 (`filtro_na_url_servida` 3) |
| lista R (juntar) | ufrj | 87 | 83 | 1 (`filtro_de_marcador_ativo_no_controle` 1) | 3 (`filtro_na_url_servida` 3) |
| lista R (juntar) | go | 84 | 75 | 1 (`filtro_de_marcador_ativo_no_controle` 1) | 8 (`filtro_na_url_servida` 3; `tela_na_visao_d_o_comando_troca_para_r` 5) |
| lista R (juntar) | pen1 | 72 | 68 | 1 (`filtro_de_marcador_ativo_no_controle` 1) | 3 (`filtro_na_url_servida` 3) |
| lista R (juntar) | pen2 | 69 | 65 | 1 (`filtro_de_marcador_ativo_no_controle` 1) | 3 (`filtro_na_url_servida` 3) |
| lista R (juntar) | pen3 | 76 | 72 | 1 (`filtro_de_marcador_ativo_no_controle` 1) | 3 (`filtro_na_url_servida` 3) |
| lista R (juntar) | pen4 | 72 | 68 | 1 (`filtro_de_marcador_ativo_no_controle` 1) | 3 (`filtro_na_url_servida` 3) |
| lista R (juntar) | **todas** | 729 | 696 | 7 (`filtro_de_marcador_ativo_no_controle` 7) | 26 (`filtro_na_url_servida` 21; `tela_na_visao_d_o_comando_troca_para_r` 5) |
| recebidos | go | 84 | 75 | 0 | 9 (`lista_r_nao_lida` 4; `tela_na_visao_d_o_comando_troca_para_r` 5) |
| recebidos | **todas** | 729 | 696 | 0 | 33 (`lista_r_nao_lida` 28; `tela_na_visao_d_o_comando_troca_para_r` 5) |
| gerados | go | 84 | 75 | 0 | 9 (`lista_r_nao_lida` 4; `tela_na_visao_d_o_comando_troca_para_r` 5) |
| gerados | **todas** | 729 | 696 | 0 | 33 (`lista_r_nao_lida` 28; `tela_na_visao_d_o_comando_troca_para_r` 5) |
| página de recebidos | **todas** | 701 | 696 | 0 | 5 (`tela_na_visao_d_o_comando_troca_para_r` 5) |
| página de gerados | **todas** | 701 | 696 | 0 | 5 (`tela_na_visao_d_o_comando_troca_para_r` 5) |
| lista D (colunas) | **todas** | 729 | 5 | 0 | 724 (`tela_na_visao_r_o_comando_troca_para_d` 724) |
| página da D | **todas** | 729 | 5 | 0 | 724 (`tela_na_visao_r_o_comando_troca_para_d` 724) |
| linhas com link (colheita) | **todas** | 729 | 701 | 0 | 28 (`lista_r_nao_lida` 28) |
| tela de login (sip/login.php) | unb | 8 | 0 | 8 (`pagina_de_login_recebida_no_controle` 8) | 0 |
| tela de login (sip/login.php) | ufrj | 1 | 0 | 1 (`pagina_de_login_recebida_no_controle` 1) | 0 |
| tela de login (sip/login.php) | pen1 | 1 | 0 | 1 (`pagina_de_login_recebida_no_controle` 1) | 0 |
| tela de login (sip/login.php) | pen2 | 1 | 0 | 1 (`pagina_de_login_recebida_no_controle` 1) | 0 |
| tela de login (sip/login.php) | pen3 | 1 | 0 | 1 (`pagina_de_login_recebida_no_controle` 1) | 0 |
| tela de login (sip/login.php) | **todas** | 12 | 0 | 12 (`pagina_de_login_recebida_no_controle` 12) | 0 |

**Cabeçalho.** Visão identificada em 729 de 729 telas (724 R, 5 D, todas
D em GO); unidade em 729. Linhas publicadas: 38.310 de Recebidos e
36.998 de Gerados (até 100 por página), todas as faixas começando em 1.
Na D (GO, 5 telas, 10 linhas cada): 12 colunas configuradas,
`colunas_ausentes` vazio e uma célula de marcadores com marcador por
tela; lista e posição da D lidas nas 5. Colheita: 75.358 linhas em 701
telas, todas com `href`.

## Páginas publicadas (múltiplas páginas)

Listas lidas com `total_declarado` maior que as linhas, e o plano de
`--pagina 2` pelos mesmos leitores (`pagina_do_controle::planejar` na R,
`posicao`/`planejar` na D): a faixa pedida tem de começar logo depois da
exibida e terminar em `min(total, 2 × tamanho)`.

| Lista | Instalação | Lidas | Com mais de uma página | Maior total | Maior nº de páginas | Tamanhos | Página 2 logo após a exibida |
|---|---|---:|---:|---:|---:|---|---|
| recebidos | unb | 265 | 0 | 0 | 1 |  | 0 de 0 |
| recebidos | ufrj | 83 | 39 | 113 | 2 | 100 | 39 de 39 |
| recebidos | go | 75 | 73 | 402 | 41 | 10 | 73 de 73 |
| recebidos | pen1 | 68 | 32 | 106 | 2 | 100 | 32 de 32 |
| recebidos | pen2 | 65 | 65 | 493 | 5 | 100 | 65 de 65 |
| recebidos | pen3 | 72 | 72 | 168 | 2 | 100 | 72 de 72 |
| recebidos | pen4 | 68 | 68 | 157 | 2 | 100 | 68 de 68 |
| gerados | unb | 265 | 0 | 0 | 1 |  | 0 de 0 |
| gerados | ufrj | 83 | 0 | 0 | 1 |  | 0 de 0 |
| gerados | go | 75 | 73 | 7950 | 795 | 10 | 73 de 73 |
| gerados | pen1 | 68 | 68 | 218 | 3 | 100 | 68 de 68 |
| gerados | pen2 | 65 | 65 | 203 | 3 | 100 | 65 de 65 |
| gerados | pen3 | 72 | 60 | 123 | 2 | 100 | 60 de 60 |
| gerados | pen4 | 68 | 54 | 110 | 2 | 100 | 54 de 54 |
| lista D (colunas) | go | 5 | 5 | 8112 | 812 | 10 | 5 de 5 |

- **674 listas paginadas** (349 Recebidos, 320 Gerados, 5 D) em seis
  instalações (UnB: nenhuma), todas na página 1, e **674 de 674** com a
  página 2 planejada logo depois da exibida. Tamanho de página: 10 em GO
  (R e D), 100 em UFRJ e PEN1 a PEN4.
- Maiores: Gerados de GO com 7.950 declarados (795 páginas) e a D de GO
  com 8.112 (812 páginas); PEN2 Recebidos com 493 (5 páginas).
- **Paginador publicado ⇔ lista paginada**, sem exceção: 349 de
  Recebidos e 320 de Gerados com `lnk{T}ProximaPagina`, nenhum em lista
  de uma página. `lnk{T}UltimaPagina` e `sel{T}Paginacao` só em GO,
  PEN1 (Gerados) e PEN2; PEN3, PEN4 e UFRJ publicam só a próxima página.
- Lista de uma página: a página 2 é `pagina_fora_do_intervalo` (`paginas: 1`).

## Continuidade entre páginas gravadas

**Páginas gravadas além da primeira: 0.** Nenhuma faixa no acervo começa
depois de 1, nenhum `hdn*PaginaAtual` ou `hdnDetalhadoPaginaAtual`
diferente de 0, e os 14 POST de `procedimento_controlar` são de filtro
por marcador, não de paginação. As cassettes `unidade-processos-paginado-*`
gravam só a página 1. Logo, **a continuidade entre páginas reais não é
verificável no acervo**: fica com o ensaio live (`NOT_RUN` aqui).

O verificador roda assim mesmo sobre todas as páginas (mesma cassette,
lista, unidade, critério e total) e os testes sintéticos cobrem página
seguinte contínua, lacuna, sobreposição, id repetido e faixa repetida:

| Relação entre páginas consecutivas | Lista | Instalação | Pares | Exemplos |
|---|---|---|---:|---|
| mesma faixa, mesmos ids | gerados | unb | 169 | bloco-incluir-anotar-retirar-unb--c3232a87e814#0→1, bloco-incluir-anotar-retirar-unb--c3232a87e814#1→3 |
| mesma faixa, mesmos ids | gerados | ufrj | 1 | documento-incluir-ufrj--2ae847715992#25→29 |
| mesma faixa, mesmos ids | gerados | go | 2 | documento-incluir-go--fa4845e16ba9#25→29, unidade-troca-e-volta-go--adbfaefa1d78#0→11 |
| mesma faixa, mesmos ids | gerados | pen1 | 1 | documento-incluir-pen1--3d879b332aac#22→26 |
| mesma faixa, mesmos ids | gerados | pen2 | 1 | documento-incluir-pen2--d717b7e5a034#26→30 |
| mesma faixa, mesmos ids | gerados | pen3 | 1 | documento-incluir-pen3--afde2b1d5f4b#26→30 |
| mesma faixa, mesmos ids | gerados | pen4 | 1 | documento-incluir-pen4--ed9e81156eab#26→30 |
| mesma faixa, mesmos ids | recebidos | unb | 172 | bloco-incluir-anotar-retirar-unb--c3232a87e814#0→1, bloco-incluir-anotar-retirar-unb--c3232a87e814#1→3 |
| mesma faixa, mesmos ids | recebidos | ufrj | 2 | documento-incluir-ufrj--2ae847715992#0→25, documento-incluir-ufrj--2ae847715992#25→29 |
| mesma faixa, mesmos ids | recebidos | go | 4 | bloco-devolver-go--87741a72a8c9#0→51, documento-incluir-go--fa4845e16ba9#0→25 |
| mesma faixa, mesmos ids | recebidos | pen1 | 2 | documento-incluir-pen1--3d879b332aac#0→22, documento-incluir-pen1--3d879b332aac#22→26 |
| mesma faixa, mesmos ids | recebidos | pen2 | 2 | documento-incluir-pen2--d717b7e5a034#0→26, documento-incluir-pen2--d717b7e5a034#26→30 |
| mesma faixa, mesmos ids | recebidos | pen3 | 2 | documento-incluir-pen3--afde2b1d5f4b#0→26, documento-incluir-pen3--afde2b1d5f4b#26→30 |
| mesma faixa, mesmos ids | recebidos | pen4 | 2 | documento-incluir-pen4--ed9e81156eab#0→26, documento-incluir-pen4--ed9e81156eab#26→30 |
| mesma faixa, ids diferentes | gerados | unb | 1 | bloco-incluir-anotar-retirar-unb--c3232a87e814#135→140 |

As 362 repetições da página 1 com os mesmos ids mostram que releituras
da mesma página são estáveis; a única diferente
(`bloco-incluir-anotar-retirar-unb`, passos 135→140: mesmo total e
faixa, ids diferentes) é a caixa mudando entre as requisições do cenário.

## Casos negativos do acervo

| Caso | Telas | Recusa | Instalações |
|---|---:|---|---|
| Troca de unidade detectada (GET 302 para `procedimento_controlar`, `msg` da troca) | 28 | `redirecionamento_recusado` | UnB, UFRJ, GO |
| `--unidade` da tela anterior, unidade trocada na gravação | 22 | `unidade_da_tela_diferente_da_solicitada` | UnB, UFRJ, GO |
| Mesma unidade da tela anterior | 212 | lido (nenhuma divergência de `juntar`) | as sete |
| Tela de login (`sip/login.php`) pelo transporte do Controle | 12 | `pagina_de_login_recebida_no_controle` | UnB, UFRJ, PEN1 a PEN3 |
| Filtro de marcador ativo (POST, passo 2 de `marcador-listar-processos-*`) | 7 | `filtro_de_marcador_ativo_no_controle` | as sete |
| Unidade sem processos em GO (tabelas omitidas, prova estrutural) | 2 | lida: `total_declarado: 0`, sem faixa (achado corrigido 3) | GO |
| Tela de marcadores (`tipo_filtro=M`) | 21 | não aplicável | as sete |

Nenhuma tela de login gravada como resposta de `procedimento_controlar`
(sessão vencida não aparece no acervo); a recusa vem das telas de login
reais passadas pelo mesmo transporte.

## Achados

### Corrigidos nesta fatia

1. **Filtro de marcador ativo** (as sete instalações, 7 telas). Depois
   de "Listar processos com o marcador", o POST serve a visão R com
   `hdnIdMarcador{unidade}` preenchido e só as linhas do marcador; a
   tabela vazia some (UFRJ, GO, PEN2, PEN3) e a legenda vai ao singular.
   A leitura recusava **por acaso** (`tabela_ausente` 4,
   `legenda_sem_faixa_com_total_diferente_das_linhas` 3) e, corrigida a
   legenda, publicaria a lista filtrada como a da unidade. Agora recusa
   `filtro_de_marcador_ativo_no_controle` antes de publicar; as 715
   telas GET trazem o campo vazio. RED: 1 de 2 testes
   (`paginas::tests_filtro`).
2. **Legenda no singular na R** (`1 registro:`,
   `Processos recebidos (1 registro):`, as mesmas telas; a D já lia).
   RED: 1 de 1 teste. Sem efeito no acervo (as telas com singular têm
   filtro); evita recusar a lista de um processo só.
   [Contrato das páginas](../../contracts/cli/unidade-listar-processos-paginas.md).

3. **Unidade vazia em GO** (`unidade-troca-e-volta-go#6`,
   `bloco-devolver-go#38`, unidade `ABC/GECONT-18141`). As duas telas
   são idênticas na estrutura: visão R, unidade identificada, nenhuma
   `<table>`; `div{T}AreaTabela` e as duas `div{T}AreaPaginacao*` só
   com espaços; `hdn{T}NroItens` e `hdn{T}PaginaAtual` únicos e `0`;
   nenhuma mensagem de lista vazia. Antes `tabela_ausente` (2); agora
   lidas, Recebidos e Gerados com `total_declarado: 0` e
   `faixa_observada: null`, e `--pagina 2` é `pagina_fora_do_intervalo`
   (`paginas: 1`). Falta qualquer parte, ou repetida, ou diferente de 0:
   continua `tabela_ausente`. RED: 2 de 4 testes
   (`paginas::tests_vazia`). A visão D vazia não está no acervo e não
   foi inventada.

### Abertos

1. **Filtro na visão D**: não observado; a recusa nova é só da R.
2. **Página N real**: ver continuidade acima.

## Reproduzir

```sh
IES_ACERVO_SAIDA=/tmp/controle.md cargo test --release --bin ies \
  replay_do_controle_inteiro -- --ignored
```

Cerca de 90 s em `release` (10 min em `dev`). Falha se algum card não conferir.
