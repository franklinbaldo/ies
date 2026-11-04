---
type: SEIObservation
title: Ensaio live da paginação da caixa em GO e na PEN2
description: Segunda tentativa, 04/11/2026, de listar-processos, recebidos e gerados em Goiás 5.0.3 e na PEN2; os dois logins falharam (GO por túnel do proxy do ambiente, PEN2 por bloqueio do Cloudflare), casos NOT_RUN, requisições contadas e correção do reconhecimento do bloqueio
---

# Paginação da caixa em GO e na PEN2 — 04/11/2026

Complementa o [ensaio de 04/11/2026](2026-11-04-unidade-paginacao.md)
(UnB, PEN1, PEN3, PEN4 e UFRJ), onde GO e PEN2 ficaram **NOT_RUN**. Mesmo
método e mesmos casos a–q: binário em release do ramo `slice-go-pen2`
(base `baed0e2`, com a [lista vazia sem tabela](../../contracts/cli/unidade-listar-processos-paginas.md)),
relay `CONNECT` local contando túneis (túnel = pedido), só leitura,
instalações de treinamento. Nenhuma saída trouxe URL, `infra_hash` ou
cookie.

## Orçamento e ritmo declarados

- **GO 5.0.3** (WAF: rajadas recebem 403 "Acesso Negado"): no máximo 12
  comandos, 90 s ou mais entre eles, sem GET de conferência (também
  contaria para o WAF). Ordem: `login` (sessão de 03/11), `recebidos`,
  `gerados --pagina 2` (página de 10), `listar-processos` (R→D),
  `listar-processos --pagina 2`, uma recusa fora do intervalo. O primeiro
  `bloqueio_de_seguranca_da_instalacao` ou 403 encerra GO.
- **PEN2** (Tramita): um único `ies login`, nova tentativa horas depois do
  503 da rodada anterior (não é repetição automática); se passasse, os
  casos a–q da PEN3/PEN4, 22 s entre comandos, no máximo 25.
- Nenhuma repetição de comando que falhou.

## Resultado

| Instalação | Caso | Túneis (pedidos ao SEI) | Resultado |
| --- | --- | --- | --- |
| PEN2 | `ies login` | 2 (2) | **FAIL** do ambiente: 403 do Cloudflare no 2º GET; saída `sessao_nao_autorizada` (corrigido para `bloqueio_de_seguranca_da_instalacao`, ver Achados) |
| PEN2 | a–q | — | **NOT_RUN**: sem sessão |
| GO | `ies login` | 3 (2) | **FAIL** do ambiente: `falha_de_rede`; o 3º túnel fechou no proxy do ambiente depois do ClientHello |
| GO | `recebidos`, `gerados --pagina 2`, `listar-processos [--pagina 2]`, recusa | — | **NOT_RUN**: sem sessão; o login não foi repetido |

**PEN2.** Túnel 1: 799 B enviados, 4.269 B recebidos (o primeiro GET);
túnel 2: 803 B, 9.971 B, HTTP 403 com a página do Cloudflare
(`Attention Required! | Cloudflare`, "Sorry, you have been blocked",
"You are unable to access processoeletronico.gov.br"). Mesmo passo que
recebeu 503 do nginx na rodada anterior. Nenhum POST, nenhuma credencial
enviada.

**GO.** Túnel 1 (GET do Controle com o cookie de 03/11): 1.408 B / 5.882 B;
túnel 2 (GET da tela de login): 864 B / 5.326 B; o login seguiu, então a
sessão de 03/11 tinha vencido. Túnel 3: 517 B enviados (só o ClientHello),
39 B recebidos (só a resposta do `CONNECT`), 10,9 s; o status do proxy do
ambiente registrou `ws_closed_mid_exchange` para `seitreinamento.go.gov.br`.
O POST de login não chegou ao SEI. Nenhum 403 de GO.

## Comportamento de bloqueio observado

- **GO:** nenhum bloqueio em 2 pedidos espaçados de 2 s; o limiar segue
  não medido. A falha foi do túnel do ambiente, não do WAF.
- **PEN2:** bloqueio no 2º pedido, depois de horas sem pedidos da sessão;
  não é rajada no sentido de GO. A causa (origem do proxy do ambiente,
  regra do site) não foi determinada.

## Achados

**Bloqueio do Cloudflare saía como sessão vencida (corrigido).** O 403
da PEN2 virou `sessao_nao_autorizada`, "a sessão pode estar vencida",
convidando a um relogin que só faria mais pedidos ao bloqueio. Agora a
página do Cloudflare (título exato e a frase no texto visível) é
`bloqueio_de_seguranca_da_instalacao`, como a de GO; outro título ou
outra frase continuam `sessao_nao_autorizada`. RED: 1 de 6 testes
(`transporte::tests_bloqueio_de_seguranca`, página sintética com os
textos observados) falhando antes da correção.
[Erros tipados](../erros-tipados.md).

## NOT_RUN

| Instalação | Motivo | Próximo passo |
| --- | --- | --- |
| GO 5.0.3 | login interrompido pelo proxy do ambiente (`falha_de_rede`) | nova rodada com `ies login` uma vez, mesmo orçamento |
| PEN2 | 403 do Cloudflare no login (antes, 503 do nginx) | tentar de outra origem de rede ou mais tarde; sem repetição na mesma rodada |
