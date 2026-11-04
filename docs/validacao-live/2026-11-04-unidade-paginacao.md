---
type: SEIObservation
title: Ensaio live da paginação e da troca de visão em listar-processos, recebidos e gerados
description: PASS/FAIL/NOT_RUN de unidade listar-processos, recebidos e gerados com e sem --pagina N, requisições contadas por túnel e efeito lembrado pelo SEI, UnB e PEN1, 04/11/2026
---

# Ensaio live da paginação da caixa — 04/11/2026

Ensaio da sintaxe nova de #57:
[`listar-processos [--pagina N]`](../../contracts/cli/unidade-listar-processos-visao-detalhada.md),
[`recebidos|gerados [--pagina N]`](../../contracts/cli/unidade-recebidos-gerados.md)
e as [páginas da caixa](../../contracts/cli/unidade-listar-processos-paginas.md),
no fluxo **sem restauração** (a tela fica onde o comando a deixou).
Binário em release do ramo `slice-pagina-live` (base `04a316e`), só
leitura, instalações de treinamento.

## Condições

- **Sessões:** as já existentes, em pastas privadas `0700`. UnB: a sessão
  tinha vencido e `ies login` relogou (7 req.). PEN1: `ies login`
  conferiu a sessão gravada (`sessao_existente_valida`, 1 req.). Um login
  por instalação, nenhum outro.
- **Contagem de requisições:** um relay `CONNECT` local, entre o `ies` e
  o proxy do ambiente, conta os túneis. O cliente HTTPS do IES abre uma
  conexão por pedido (`Connection: close`), então túnel = pedido. O relay
  vê só bytes cifrados; GET e POST se separam pelo volume enviado (GET
  1,0–1,3 KB com o handshake TLS; POST 2,8–4,2 KB, o formulário do
  Controle). Aferição: no login, túneis = `requisicoes` da saída (7 = 7 na
  UnB, 1 = 1 na PEN1).
- **Efeito:** um GET de conferência (`curl`, fora do relay, cookie num
  arquivo `0600`) imprime só `hdnTipoVisualizacao`, os índices
  `hdn{Recebidos,Gerados,Detalhado}PaginaAtual` e as legendas. Esses GETs
  não entram nas contagens abaixo.
- **Ritmo e orçamento:** 22 s ou mais entre execuções por instalação. UnB:
  12 comandos e 3 conferências; PEN1: 14 comandos e 6 conferências.
- Nenhuma saída trouxe URL, `infra_hash` ou cookie. Nenhuma escrita;
  `unidade colunas --definir` não foi executado.

## UnB 4.0.9 (Unidade1; 24 recebidos, 63 gerados, uma página cada)

Sessão encontrada na R (`R/0/0`); a D da UnB não publica
`hdnDetalhadoPaginaAtual` (sem paginação).

| Caso | Requisições (contrato) | Resultado |
| --- | --- | --- |
| `listar-processos` (R→D) | 1 GET + 1 POST (1 + 1) | **PASS**: 87 de 87, faixa 1–87, categoria por processo 24 recebidos e 63 gerados, 7 colunas, `colunas_ausentes` com 5 chaves |
| Conferência depois | — | `R/0/0`: a UnB volta à R no GET seguinte |
| `listar-processos --pagina 2` | 1 GET + 1 POST (1 + 1) | **PASS**: `pagina_fora_do_intervalo` depois do POST D (2 ≤ 87, o limite seguro antes de ver a D; D sem paginação só aceita a página 1) |
| `listar-processos --pagina 99999` | 1 GET (1) | **PASS**: `pagina_fora_do_intervalo` sem POST (99999 > 24 + 63) |
| `recebidos` | 1 GET (1) | **PASS**: 24 de 24, faixa 1–24 |
| `gerados` | 1 GET (1) | **PASS**: 63 de 63, faixa 1–63 |
| `recebidos --pagina 2`, `gerados --pagina 2` | 1 GET cada (1) | **PASS**: `pagina_fora_do_intervalo` sem POST |
| `recebidos --pagina 1` (a página mostrada) | 1 GET (1) | **PASS**: 24 de 24, nenhum POST |
| D depois de R: `listar-processos` de novo | 1 GET + 1 POST (1 + 1) | **PASS**: mesma saída da primeira |
| R depois de D: `gerados` logo depois | 1 GET (1) | **PASS**: a sessão já estava na R, sem POST de troca |
| Conferência final | — | `R/0/0` |
| Várias páginas (D ou R) | — | NOT_RUN: a unidade tem uma página em cada lista e a D sem paginação |
| Regime estável da D em 1 GET | — | NOT_RUN por natureza: a UnB 4.0.9 volta à R a cada GET, então `listar-processos` custa sempre 1 GET + 1 POST ali |

A primeira execução de `listar-processos` saiu com `falha_de_rede`: o
proxy do ambiente fechou o túnel depois do ClientHello (517 B enviados,
39 B recebidos, só a resposta do `CONNECT`; registrado no status do
proxy como `ws_closed_mid_exchange`). Nenhum pedido HTTP chegou ao SEI.
Foi a única repetição da rodada, registrada aqui como exceção à regra de
não repetir.

## PEN1 4.0.12 (TESTE; Recebidos 115→116, Gerados 218, D 333→334)

Sessão encontrada na **D**, página 1 (`hdnDetalhadoPaginaAtual=0`,
`Processos (333 registros - 1 a 100)`). As contas de treinamento são
compartilhadas: Recebidos passou de 115 a 116 entre dois comandos, sem
efeito nas verificações de cada comando.

| # | Caso (tela no GET) | Requisições (contrato) | Resultado |
| --- | --- | --- | --- |
| a | `listar-processos` (D, estável) | 1 GET (1) | **PASS**: 100 de 333, faixa 1–100, `categoria: null` nos 100, colunas `atribuicao`, `interessados` e `tipo` |
| b | `listar-processos --pagina 2` (D) | 1 GET + 1 POST (1 + 1) | **PASS**: faixa 101–200 |
| — | Conferência | — | `D`, `hdnDetalhadoPaginaAtual=1`, legenda `101 a 200`: página lembrada |
| d | `listar-processos` (D na página 2) | 1 GET (1) | **PASS**: faixa 101–200, a página lembrada, declarada pela faixa |
| e | `listar-processos --pagina 5` (D; 4 páginas) | 1 GET (1) | **PASS**: `pagina_fora_do_intervalo` sem POST |
| f | `recebidos` (D→R) | 1 GET + 1 POST (1 + 1) | **PASS**: 100 de 115, faixa 1–100 |
| — | Conferência | — | `R/0/0`: a sessão ficou na R |
| h | `gerados` (R, estável) | 1 GET (1) | **PASS**: 100 de 218, faixa 1–100 |
| i | `gerados --pagina 2` (R) | 1 GET + 1 POST (1 + 1) | **PASS**: faixa 101–200 |
| j | `recebidos --pagina 2` (R) | 1 GET + 1 POST (1 + 1) | **PASS**: 16 itens, 101–116 |
| — | Conferência | — | `R/1/1`: cada lista lembra a sua página |
| l | `listar-processos` (R→D, D lembrando a página 2) | 1 GET + 1 POST (1 + 1) | **PASS**: 100 de 334, faixa **101–200** (a página da D lembrada desde b); 16 com categoria `recebidos`, 84 `null` |
| m | `gerados --pagina 3` (D→R→página 3) | 1 GET + 2 POST (1 + 2) | **PASS**: 18 itens, 201–218 |
| n | `listar-processos --pagina 1` (R, D lembrando a página 2) | 1 GET + 2 POST (1 + 2) | **PASS**: faixa 1–100, `categoria: null` nos 100 |
| o | `recebidos --pagina 99` (D→R, 2 páginas) | 1 GET + 1 POST (1 + 1) | **PASS** (subcódigo): `pagina_fora_do_intervalo` depois do POST R; mensagem corrigida, ver Achados |
| — | Conferência | — | `R/1/2`: a sessão ficou na R, Recebidos na 2 e Gerados na 3 |
| p | `listar-processos --pagina 5` (R; 5 ≤ 116 + 218) | 1 GET + 1 POST (1 + 1) | **PASS**: `pagina_fora_do_intervalo` depois do POST D |
| — | Conferência final | — | `D`, `hdnDetalhadoPaginaAtual=0`, legenda `1 a 100` |

Os quatro regimes da tabela de orçamento da visão detalhada foram
medidos: D sem POST (a, d, e), D com POST de página (b), R com POST D
(l, p), R com POST D e de página (n). Os dois de `recebidos|gerados`
também: R com e sem POST de página (h, i, j) e D com POST R e de página
(f, m, o).

## Efeito observado (o que o GET seguinte mostra)

- **Visão:** PEN1 lembra a última visão pedida (D depois de
  `listar-processos`, R depois de `recebidos|gerados`); a UnB volta à R em
  todo GET. Nada foi restaurado, como decidido.
- **Página da D:** fica lembrada à parte das páginas da R. Na PEN1, a
  página 2 pedida em b continuou sendo a da D depois de três comandos na
  R (l devolveu 101–200 sem `--pagina`).
- **Páginas da R:** cada lista guarda a sua (`R/1/1`, depois `R/1/2`).
  Ao trocar de D para R, a R volta com essas páginas, e a `faixa_observada`
  as declara.
- **Categoria por processo na D:** sai só para os `id` que a R do mesmo
  GET mostrou. Como as páginas lembradas da R e da D são independentes, a
  sobreposição varia: 100 de 100 na UnB (uma página só), 16 de 100 em l,
  0 em n (a R mostrava Recebidos 101–116 e Gerados 201–218, a D a página
  1). `null` é "não observada", como contratado.
- A sessão da PEN1 terminou na D, página 1; a da UnB, na R. Nenhuma
  recusa `bloqueio_de_seguranca_da_instalacao` nem efeito inesperado.

## Achados

**Mensagem da recusa de página vinda da D (corrigido).** Em o, o
subcódigo estava certo, mas a mensagem era a genérica da troca de visão,
"a visão do Controle de Processos não foi obtida como esperado", embora
a troca para R tivesse passado. Com a sessão já na R, a mesma recusa
dizia "a página pedida do Controle não foi obtida". A mensagem passou a
vir do erro da página (`ErroPaginaDoControle::mensagem`), a mesma nos
dois caminhos. RED: 1 teste (`pagina_fora_do_intervalo_vista_na_r_fica_na_r`,
fixture sintética D→R com N fora) falhando antes da correção, verde
depois. A saída e o subcódigo não mudam. Essa mensagem não foi reensaiada
ao vivo (NOT_RUN).

**Limite seguro antes de ver a D.** Vindo da R, `listar-processos
--pagina N` com N ≤ recebidos + gerados faz o POST D antes de recusar
(UnB `--pagina 2`, PEN1 p). Isso está no contrato: o tamanho da página da
D só é conhecido depois de vê-la. Custa 1 POST e deixa a sessão na D.

## NOT_RUN

| Instalação | Motivo |
| --- | --- |
| GO 5.0.3 | WAF devolve 403 "Acesso Negado" em rajadas; fora desta rodada |
| PEN3, PEN4 5.0.4 | sem sessão configurada (nenhuma pasta com credencial) |
| UFRJ, PEN2 | fora desta rodada |
| UnB, várias páginas | a unidade tem uma página por lista e a D sem paginação |
