---
type: SEIObservation
title: Saída 3 de unidade colunas --definir todas --confirmar na PEN1
description: Diagnóstico offline do salvar do Nível de detalhe que não aplicou na PEN1 em 04/11/2026, o que foi descartado, o que mudou e o reensaio proposto
---

# `--definir todas --confirmar` na PEN1 — 04/11/2026

## Fato

Às 01:11 UTC, com sessão existente válida e o binário release do ramo da
época, o mesmo script rodou na UnB 4.0.9 e na PEN1 (treinamento):
`ies login`, prévia `--definir todas`, `--definir todas --confirmar` e
`listar-processos`, com 2 a 3 s entre comandos.

| Instalação | Prévia | `--confirmar` | Leitura seguinte |
| --- | --- | --- | --- |
| UnB 4.0.9 | saída 0 | **PASS** (todas as 11) | marcadores visíveis na D |
| PEN1 | saída 0 | **FAIL**: saída 3, `efeito_incerto_na_configuracao_de_colunas` | `ies unidade colunas` ~10 s depois: as marcas de antes (atribuição e tipo, sem anotação nem especificação) |

O script guardou só o `subcodigo`: `falha_do_salvar` e `verificacao`, que
a saída publica, se perderam. Não houve nova tentativa. A mesma escrita
tinha passado na PEN1 em 02/11/2026 (12 marcadas, conferido por leitura,
[validação](2026-11-02-leitura-controle.md)).

Versão: a [tabela de instalações](../como-o-sei-funciona/estado-e-instalacoes.md)
registra a PEN1 como 4.0.12, mas as telas capturadas em 02/11 e em 03 e
04/11 carregam os recursos com `?5.0.4-2.45.1-…`.

## O que a saída 3 prova

Pela tabela de reconciliação do
[contrato](../../contracts/cli/unidade-definir-colunas.md), saída 3 sai em
dois casos: o GET de verificação falhou, ou a resposta do salvar não foi a
esperada e o GET leu marcas que não são nem o alvo nem as de antes. Como a
leitura seguinte mostrou as de antes e nada escreveu nesse intervalo, o
caso é o primeiro: **o GET de verificação, logo depois do POST, falhou** no
transporte ou trouxe tela irreconhecível. A mesma URL tinha sido lida
segundos antes, e a mesma sessão leu de novo ~10 s depois (o IES não
reloga sozinho). Duas requisições seguidas, POST e GET, não tiveram o
resultado normal.

## O que o IES enviou (reconstruído offline)

Derivado da tela capturada da PEN1 (`exp-colunas.html`, privada) pelo
mesmo código do comando, sem rede:

- `POST` no `action` do `frmAcessoExternoCancelar`:
  `controlador.php?acao=procedimento_configurar_detalhe&acao_origem=procedimento_configurar_detalhe&infra_sistema=…&infra_unidade_atual=…&infra_hash=…`.
  O link `ancNivelDetalhe` da D (a URL do GET) traz
  `acao_origem=procedimento_controlar&acao_retorno=procedimento_controlar`;
  o `action` não traz `acao_retorno`, e o navegador também não o envia.
- Corpo, na ordem do documento:
  `hdnInfraTipoPagina=2&sbmSalvar=Salvar&chkSinAtribuicao=on&…&chkSinMarcadores=on`
  (14 campos; caixas sem `value` vão como `on`).
- `Content-Type: application/x-www-form-urlencoded`, `Origin` da
  instalação e `Referer` = URL do GET.

É, byte a byte, o corpo do POST que gravou na PEN1 em 02/11/2026 (arnês do
ensaio, resposta `exp-salvar-todas.html`) e o do `ies` que passou no mesmo
dia. O teste `post_do_salvar_repete_os_campos_do_navegador_na_tela_da_pen1`
(`src/nivel_de_detalhe/tests_salvar.rs`) fixa esse corpo numa tela
sintética com a estrutura da PEN1. A derivação não mudou desde o PASS; o
transporte passou a usar o cliente HTTPS único pelo proxy do ambiente em
03/11/2026.

## O que a tela serve

- O JS embutido tem só `OnSubmitForm()` (devolve `true`) e
  `inicializar()`. Nenhum script preenche oculto, monta lista de colunas
  ou troca o botão antes do envio. `validarNumeroCriterios(this)`, de
  `sei.js` (não capturado), roda no `onchange` das caixas, não no envio.
- Único oculto no formulário: `hdnInfraTipoPagina`. `hdnInfraPrefixoCookie`
  fica fora do `<form>` e não é enviado. Não há token além do `infra_hash`
  do `action`.
- PEN1 5.0.4, GO 5.0.3 e UnB 4.0.9 servem o mesmo formulário (a UnB sem
  Prioridade) e o mesmo `inicializar()`, com `if (''=='1'){`.

## A resposta que gravou

`exp-salvar-todas.html` (PEN1, 02/11/2026): HTML 200, sem redirect, a
**mesma tela** com as 12 caixas `checked` e `inicializar()` em
`if ('1'=='1'){`, seguido de `window.parent.location` para
`procedimento_controlar` e `infraFecharJanelaModal()`. É um sinal de
gravação que o contrato dizia não existir. `exp-2-get-depois.html` não é
desse salvar: é o Controle depois do POST D do experimento da visão
(16:26 de 02/11). Nenhum dos 29 ZIPs de cassettes tem interação
`procedimento_configurar_detalhe`.

## Causa

**Não determinada pela evidência guardada.** Descartados: campos, valores,
ordem, botão, `action`, oculto preenchido por JS e token. Hipóteses
compatíveis com "POST sem efeito e GET seguinte falho, leituras normais
segundos depois":

1. Recusa transitória na borda por rajada: as telas da PEN1 carregam
   `/cdn-cgi/challenge-platform/scripts/jsd/main.js` (detecção de bots de
   CDN); GO já respondeu 403 de WAF depois de rajadas; o comando fez cerca
   de seis requisições em oito segundos.
2. Falha de rede no proxy de saída do ambiente nas duas requisições.
3. O SEI devolveu 200 sem gravar e o GET falhou: antes desta mudança, essa
   resposta passava por esperada e não deixava rastro.

## O que mudou

- A resposta do salvar só é a esperada com o sinal de gravação; `''=='1'`
  vira `nivel_de_detalhe_devolvido_sem_sinal_de_gravacao` e outra forma,
  `resposta_do_salvar_nao_reconhecida`, ambas em `falha_do_salvar`. A
  verificação por um GET e o POST único não mudam (RED 5 falhas, GREEN).
- O [roteiro](roteiro-definir-colunas.md) guarda a saída JSON inteira,
  espaça 22 s entre comandos e não restaura nada.

## Reensaio proposto

Autorização: **uma** escrita de preferência por instalação, em treinamento.

1. PEN3 5.0.4, com a sessão nova de 04/11, pelo
   [roteiro](roteiro-definir-colunas.md); se as 12 já estiverem marcadas,
   NOT_RUN do salvar ali.
2. PEN4 5.0.4, igual. A PEN1 só depois de um PASS.
3. Orçamento: login, cerca de 10 GET e **1 POST de salvar** por instalação
   (mais 1 POST D se a sessão estiver na R), seis comandos com 22 s entre
   eles, sem repetição.
4. PASS: `aplicar` com saída 0, `alterado true`, `reconciliado false`, sem
   `falha_do_salvar`, `depois` = todas as disponíveis, e o `c1` 22 s depois
   igual. Saída 3: parar e registrar `falha_do_salvar` e `verificacao`.
5. Nada a restaurar: todas as colunas é o estado final desejado.

Resultado nesta data: PEN1 **FAIL** (saída 3, sem efeito, causa não
registrada); reensaio NOT_RUN.
