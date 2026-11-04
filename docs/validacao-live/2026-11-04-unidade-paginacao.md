---
type: SEIObservation
title: Ensaio live da paginação e da troca de visão em listar-processos, recebidos e gerados
description: PASS/FAIL/NOT_RUN de unidade listar-processos, recebidos e gerados com e sem --pagina N, requisições contadas por túnel e efeito lembrado pelo SEI, UnB, PEN1, PEN3, PEN4 e UFRJ (PEN2 NOT_RUN), 04/11/2026
---

# Ensaio live da paginação da caixa — 04/11/2026

Ensaio da sintaxe nova de #57:
[`listar-processos [--pagina N]`](../../contracts/cli/unidade-listar-processos-visao-detalhada.md),
[`recebidos|gerados [--pagina N]`](../../contracts/cli/unidade-recebidos-gerados.md)
e as [páginas da caixa](../../contracts/cli/unidade-listar-processos-paginas.md),
no fluxo **sem restauração** (a tela fica onde o comando a deixou).
Binário em release do ramo `slice-pagina-live` (base `04a316e`) para UnB e
PEN1 e do ramo `slice-pagina-pen34` (base `5735017`) para PEN3 e PEN4
(Tramita, homologação) e do ramo `slice-pagina-ufrj-pen2` (base `ec02d2f`)
para UFRJ e PEN2, só leitura, instalações de treinamento.

## Condições

- **Sessões:** as já existentes, em pastas privadas `0700`. UnB: a sessão
  tinha vencido e `ies login` relogou (7 req.). PEN1: `ies login`
  conferiu a sessão gravada (`sessao_existente_valida`, 1 req.). Um login
  por instalação, nenhum outro. PEN3 e PEN4: sessões abertas por `ies
  login` minutos antes da rodada (6 req. cada, fora das contagens);
  nenhum login durante a rodada. UFRJ e PEN2: pastas novas, um `ies login`
  cada antes da rodada (UFRJ 6 req.; PEN2 recusado, ver abaixo).
- **Contagem de requisições:** um relay `CONNECT` local, entre o `ies` e
  o proxy do ambiente, conta os túneis. O cliente HTTPS do IES abre uma
  conexão por pedido (`Connection: close`), então túnel = pedido. O relay
  vê só bytes cifrados; GET e POST se separam pelo volume enviado (GET
  1,0–1,3 KB com o handshake TLS; POST 2,8–4,2 KB, o formulário do
  Controle; na PEN3 e na PEN4, todo GET 1.286 B e todo POST 2,7–3,8 KB). Aferição: no login, túneis = `requisicoes` da saída (7 = 7 na
  UnB, 1 = 1 na PEN1).
- **Efeito:** um GET de conferência (`curl`, fora do relay, cookie num
  arquivo `0600`) imprime só `hdnTipoVisualizacao`, os índices
  `hdn{Recebidos,Gerados,Detalhado}PaginaAtual` e as legendas. Esses GETs
  não entram nas contagens abaixo.
- **Ritmo e orçamento:** 22 s ou mais entre execuções por instalação. UnB:
  12 comandos e 3 conferências; PEN1: 14 comandos e 6 conferências; PEN3:
  16 comandos e 5 conferências; PEN4: 16 comandos e 4 conferências; UFRJ: 9 comandos e 4 conferências.
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

## PEN3 e PEN4 5.0.4 (TESTE; PEN3 171 + 123 = D 294, PEN4 157 + 103 = D 260)

As duas sessões começaram na **R**, páginas `R/0/0`; totais estáveis
durante toda a rodada. As duas listas da R têm duas páginas (paginador só
com `lnk{T}ProximaPagina*` ou `lnk{T}PaginaAnterior*`, sem `select`); a D
tem três, com `hdnDetalhadoPaginaAtual`, `lnkDetalhado{Proxima,Ultima}Pagina*`
e `selDetalhadoPaginacao{Superior,Inferior}`. Colunas da D:
Processo, Atribuição, Tipo, Interessados e três colunas sem título;
`colunas_ausentes` com 6 chaves (`anotacao`, `especificacao`, `prioridade`,
`observacao`, `ultima_movimentacao`, `marcadores`). Mesmos casos,
mesmos resultados nas duas; requisições medidas iguais ao contrato em
todos.

| # | Caso (tela no GET) | Requisições (contrato) | PEN3 | PEN4 |
| --- | --- | --- | --- | --- |
| a | `recebidos` (R, estável) | 1 GET (1) | **PASS**: 100 de 171, 1–100 | **PASS**: 100 de 157, 1–100 |
| b | `gerados --pagina 2` (R) | 1 GET + 1 POST (1 + 1) | **PASS**: 23 itens, 101–123 | **PASS**: 3 itens, 101–103 |
| — | Conferência | — | `R/0/1` | — |
| d | `listar-processos` (R→D) | 1 GET + 1 POST (1 + 1) | **PASS**: 100 de 294, 1–100; 57 `recebidos`, 43 `null` | **PASS**: 100 de 260, 1–100; 45 `recebidos`, 55 `null` |
| — | Conferência | — | `D`, índice 0, `1 a 100` | idem |
| e | `listar-processos` (D, estável) | 1 GET (1) | **PASS**: 1–100, `null` nos 100 | **PASS**: idem |
| f | `listar-processos --pagina 2` (D) | 1 GET + 1 POST (1 + 1) | **PASS**: 101–200 | **PASS**: 101–200 |
| g | `listar-processos --pagina 3` (D, última) | 1 GET + 1 POST (1 + 1) | **PASS**: 94 itens, 201–294 | **PASS**: 60 itens, 201–260 |
| h | `listar-processos` (D na página 3) | 1 GET (1) | **PASS**: 201–294, a página lembrada | **PASS**: 201–260 (rodado depois de i) |
| i | `listar-processos --pagina 4` (D; 3 páginas) | 1 GET (1) | **PASS**: `pagina_fora_do_intervalo` sem POST | **PASS**: idem |
| j | `recebidos` (D→R) | 1 GET + 1 POST (1 + 1) | **PASS**: 1–100 | **PASS**: 1–100 |
| — | Conferência | — | `R/0/1`: Gerados lembra a página 2 | idem |
| k | `recebidos --pagina 2` (R) | 1 GET + 1 POST (1 + 1) | **PASS**: 71 itens, 101–171 | **PASS**: 57 itens, 101–157 |
| l | `listar-processos` (R→D, D lembrando a página 3) | 1 GET + 1 POST (1 + 1) | **PASS**: 201–294; 64 `recebidos`, 23 `gerados`, 7 `null` | **PASS**: 201–260; 41, 3 e 16 |
| m | `gerados --pagina 1` (D→R→página 1) | 1 GET + 2 POST (1 + 2) | **PASS**: 1–100 de 123 | **PASS**: 1–100 de 103 |
| n | `listar-processos --pagina 1` (R, D lembrando a 3) | 1 GET + 2 POST (1 + 2) | **PASS**: 1–100; 43 `gerados`, 57 `null` | **PASS**: 1–100; 55 `gerados`, 45 `null` |
| o | `recebidos --pagina 99` (D→R) | 1 GET + 1 POST (1 + 1) | **PASS**: `pagina_fora_do_intervalo`, mensagem "a página pedida do Controle não foi obtida" | **PASS**: idem |
| p | `listar-processos --pagina 99999` (R) | 1 GET (1) | **PASS**: recusa sem POST | **PASS**: idem |
| q | `listar-processos --pagina 4` (R; 4 ≤ recebidos + gerados) | 1 GET + 1 POST (1 + 1) | **PASS**: recusa depois do POST D | **PASS**: idem |
| — | Conferência final | — | `D`, índice 0, `1 a 100` | idem |

Os quatro regimes da visão detalhada e os dois de `recebidos|gerados`
foram medidos nas duas, como na PEN1. Várias páginas, antes provadas só
na PEN1, agora também na 5.0.4: D com três páginas e R com duas por lista.

## UFRJ 4.1.5 (PR-P; 1 recebido, 9 gerados, D 10; uma página cada)

Versão lida na tela. Sessão nova na R (`R/0/0`). A R publica
`hdn{Recebidos,Gerados}PaginaAtual` sem paginador; a D publica
`hdnDetalhadoPaginaAtual=0`, nenhum paginador e a legenda
`Processos (10 registros):`, sem intervalo. Todo GET 1.060 B, todo POST
2,1–2,3 KB; requisições medidas iguais ao contrato em todos os casos.

| # | Caso (tela no GET) | Requisições (contrato) | Resultado |
| --- | --- | --- | --- |
| a | `recebidos` (R) | 1 GET (1) | **FAIL**: `legenda_sem_faixa_com_total_diferente_das_linhas`; legenda `(1 registro):` no singular (corrigido, ver Achados) |
| a′ | `recebidos` (R), binário corrigido | 1 GET (1) | **PASS**: 1 de 1, faixa 1–1 |
| b | `gerados --pagina 2` (R) | 1 GET (1) | **PASS**: `pagina_fora_do_intervalo` sem POST |
| d | `listar-processos` (R→D) | 1 GET + 1 POST (1 + 1) | **FAIL**: `faixa_detalhada_sem_intervalo_com_paginacao` depois do POST D (ver Achados) |
| — | Conferência | — | `D`, índice 0, `10 registros`: a UFRJ lembra a D |
| o | `recebidos --pagina 99` (D→R) | 1 GET + 1 POST (1 + 1) | **PASS**: `pagina_fora_do_intervalo`, mensagem "a página pedida do Controle não foi obtida" |
| — | Conferência | — | `R/0/0` |
| j | `recebidos` (R, estável) | 1 GET (1) | **PASS**: 1 de 1 |
| k | `recebidos --pagina 2` (R) | 1 GET (1) | **PASS**: recusa sem POST |
| m | `gerados --pagina 1` (R, a página mostrada) | 1 GET (1) | **PASS**: 9 de 9, 1–9, nenhum POST |
| p | `listar-processos --pagina 99999` (R) | 1 GET (1) | **PASS**: recusa sem POST |
| — | Conferência final | — | `R/0/0` |
| e–i, l, n, q | `listar-processos` com a D | — | NOT_RUN: a leitura da D falha como em d; repetir seria repetir o comando que falhou |
| — | Várias páginas | — | NOT_RUN: a unidade tem uma página em cada lista e na D |

## PEN2 (Tramita)

`ies login` recusado: `fonte_invalida` / `status_http_de_erro_do_servidor`
(503 do nginx no segundo GET, 2 túneis). Sem repetição, sem sessão: casos
a–q **NOT_RUN**, como no ensaio de 02/11. A segunda tentativa do dia
(GO e PEN2) está no [relatório próprio](2026-11-04-unidade-paginacao-go-pen2.md).

## Efeito observado (o que o GET seguinte mostra)

- **Visão:** PEN1, PEN3 e PEN4 lembram a última visão pedida (D depois
  de `listar-processos`, R depois de `recebidos|gerados`); a UFRJ também
  (D depois do POST de d, R depois de o); a UnB volta à R em todo GET. Nada foi restaurado, como decidido.
- **Página da D:** fica lembrada à parte das páginas da R. Na PEN1, a
  página 2 pedida em b continuou sendo a da D depois de três comandos na
  R (l devolveu 101–200 sem `--pagina`); na PEN3 e na PEN4, a página 3
  pedida em g, depois de três comandos na R (l devolveu 201–N).
- **Páginas da R:** cada lista guarda a sua (`R/1/1`, depois `R/1/2`).
  Ao trocar de D para R, a R volta com essas páginas, e a `faixa_observada`
  as declara.
- **Categoria por processo na D:** sai só para os `id` que a R do mesmo
  GET mostrou. Como as páginas lembradas da R e da D são independentes, a
  sobreposição varia: 100 de 100 na UnB (uma página só), 16 de 100 em l,
  0 em n (a R mostrava Recebidos 101–116 e Gerados 201–218, a D a página
  1); na PEN3, d e n se complementam (57 recebidos da R página 1 + 43
  gerados da página 1 formam a D página 1). `null` é "não observada",
  como contratado.
- As sessões da PEN1, PEN3 e PEN4 terminaram na D, página 1; a da UnB,
  na R, como a da UFRJ. Nenhuma
  recusa `bloqueio_de_seguranca_da_instalacao` nem efeito inesperado.

## Achados

**Legenda no singular na R (corrigido).** A UFRJ publica
`Processos recebidos (1 registro):`. O total era lido, mas a faixa só
procurava ` registros` e `recebidos` recusava (caso a). A faixa passou a
aceitar o singular, como a D já aceitava. RED: 1 teste
(`legenda_no_singular_comprova_a_faixa_de_um_registro`) falhando antes da
correção; reensaio a′ **PASS**.

**D de uma página com índice oculto (corrigido na fatia seguinte).** A D
da UFRJ publica `hdnDetalhadoPaginaAtual=0` sem paginador e legenda sem
intervalo, e o parser da D tratava o índice oculto como paginação. Agora
o índice `0` sozinho não é paginação (como `hdn{T}PaginaAtual` na R);
índice diferente de `0`, os paginadores `lnkInfra*`/`selInfra*` e os
`lnkDetalhado*`/`selDetalhado*` (que antes não eram reconhecidos)
continuam exigindo `- inicio a fim`. RED: 1 teste
(`indice_oculto_zero_sem_paginador_nao_e_paginacao`) falhando antes da
correção. Replay do acervo sem regressão (as 5 D de GO seguem lidas).
Reensaio d′ na UFRJ: **PASS**, 10 de 10, `colunas_ausentes` com 5 chaves
(requisições não contadas: rodada sem o relé).

**Mensagem da recusa de página vinda da D (corrigido).** Em o, o
subcódigo estava certo, mas a mensagem era a genérica da troca de visão,
"a visão do Controle de Processos não foi obtida como esperado", embora
a troca para R tivesse passado. Com a sessão já na R, a mesma recusa
dizia "a página pedida do Controle não foi obtida". A mensagem passou a
vir do erro da página (`ErroPaginaDoControle::mensagem`), a mesma nos
dois caminhos. RED: 1 teste (`pagina_fora_do_intervalo_vista_na_r_fica_na_r`,
fixture sintética D→R com N fora) falhando antes da correção, verde
depois. A saída e o subcódigo não mudam. Reensaiada ao vivo na PEN3 e na
PEN4 (caso o): **PASS**, a mensagem nova.

**SEI 5.0.4 sem diferença estrutural para o fluxo.** PEN3 e PEN4 publicam
os mesmos campos e paginadores que a PEN1 4.0.12 usa; nenhum FAIL,
nenhuma correção nesta rodada.

**Limite seguro antes de ver a D.** Vindo da R, `listar-processos
--pagina N` com N ≤ recebidos + gerados faz o POST D antes de recusar
(UnB `--pagina 2`, PEN1 p, PEN3 e PEN4 q). Isso está no contrato: o tamanho da página da
D só é conhecido depois de vê-la. Custa 1 POST e deixa a sessão na D.

## NOT_RUN

| Instalação | Motivo |
| --- | --- |
| GO 5.0.3 | WAF devolve 403 "Acesso Negado" em rajadas; fora desta rodada; na segunda tentativa, login interrompido pelo proxy do ambiente ([GO e PEN2](2026-11-04-unidade-paginacao-go-pen2.md)) |
| PEN2 | login recusado com 503 (`status_http_de_erro_do_servidor`); na segunda tentativa, 403 do Cloudflare ([GO e PEN2](2026-11-04-unidade-paginacao-go-pen2.md)) |
| UFRJ, várias páginas e casos e–i, l, n, q | uma página por lista e na D; casos da D não rodados na rodada (o d′ passou depois da correção) |
| UnB, várias páginas | a unidade tem uma página por lista e a D sem paginação |
