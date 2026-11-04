---
type: SEIObservation
title: Propriedades do processo na tela Alterar Processo
description: Link procedimento_alterar ou procedimento_consultar na barra do nó do processo e o formulário frmProcedimentoCadastro, de onde vêm tipo, especificação, assuntos, interessados, observações, nível de acesso e data de geração
---

# Propriedades do processo

Observado na UnB 4.0.9 (treinamento, 03 e 04/11/2026), em capturas
privadas `0600` lidas só pela estrutura. Nenhum valor real aparece aqui
ou nas fixtures. Uma instalação não prova as outras.

## O link na barra

A [árvore](arvore-do-processo.md) publica a barra do nó `PROCESSO` numa
linha `Nos[0].acoes = '<literal>';` (aspas simples, `\'` dentro de
`onclick`). O literal é uma sequência de `<a href="…" tabindex="451" ><img
…/></a>`; os `href` são relativos, com `&` cru, na forma
`controlador.php?acao=<acao>&acao_origem=arvore_visualizar&acao_retorno=arvore_visualizar&id_procedimento=N&arvore=1&infra_sistema=…&infra_unidade_atual=…&infra_hash=<64 hex>`.
Entre as ações: `procedimento_alterar`, `anotacao_registrar`,
`procedimento_atribuicao_cadastrar`, `procedimento_relacionar`,
`andamento_marcador_gerenciar`; algumas são `href="#"` com `onclick`.

Seis árvores capturadas: quatro com `procedimento_alterar`; as duas de
processos abertos só em outra unidade, com `procedimento_consultar` no
lugar (e sem alterar). O Todos, oráculo de comportamento, mediu o mesmo
`procedimento_consultar` na UFRJ 4.1 com o processo fechado na unidade:
"o mesmo formulário, com os níveis de acesso desabilitados".

## A tela

`GET procedimento_alterar` → 200, `iso-8859-1`, `<title>SEI - Alterar
Processo`, um `form#frmProcedimentoCadastro` (`method=post`, `action`
`acao=procedimento_alterar&acao_origem=procedimento_alterar` assinado).
Os dois `button#btnSalvar` (`type=button`, `onclick=confirmarDados()`) são
a escrita; o IES nunca os toca nem envia o formulário.

| Dado | Controle servido | Forma |
| --- | --- | --- |
| tipo | `select#selTipoProcedimento` | uma `option selected` (id decimal, texto); `hdnIdTipoProcedimento` igual ao `value`; `hdnNomeTipoProcedimento` com o mesmo texto |
| especificação | `input#txtDescricao` | `value` (máx. 100) |
| assuntos | `select#selAssuntos multiple` | uma `option` por assunto, sem `selected`; `value` id decimal; texto `<código> - <descrição>` (código com dígitos e ponto) |
| interessados | `select#selInteressadosProcedimento multiple` | uma `option` por interessado (na captura, nenhuma) |
| observações | `textarea#txaObservacoes` | o texto (na captura, vazio) |
| nível | `input[name=rdoNivelAcesso]` | `#optSigiloso` `value=2` (`disabled`), `#optRestrito` `1`, `#optPublico` `0`; um `checked` |
| hipótese legal | `select#selHipoteseLegal` | sem `option` com o nível público; preenchido por `infraAjaxMontarSelect` ao trocar o nível |
| geração | `hdnDtaGeracao` | `dd/mm/aaaa` |
| identidade | `hdnProtocoloFormatado`, `hdnIdProcedimento` | NUP formatado e id do processo |

`hdnAssuntos` e `hdnInteressadosProcedimento` vêm **vazios** no HTML: o
`infraLupaSelect('selAssuntos','hdnAssuntos',…)` os monta no navegador a
partir do `select`, e a validação do envio confere `hdnAssuntos`. Não
há duas fontes servidas para comparar; o `select` é a única. A forma
serializada nunca foi observada. `hdnStaNivelAcessoGlobal` é o nível
global (pode diferir do nível do processo) e não é publicado.
`selGrauSigilo` tem a `option value="null" selected` vazia fora do
sigiloso.

O `optSigiloso` traz `disabled="disabled"` antes de `type`: o `tl` 0.7.8
corrompe o atributo seguinte; o tokenizador de `html_atributos` lê certo.

## Efeito

GET de formulário de edição, sem envio. O Todos o usa como leitura; o IES
não mediu se a abertura grava algo no servidor (declarada leitura, nada a
restaurar).

## Não observado

Nível restrito ou sigiloso com hipótese marcada; a tela
`procedimento_consultar` no IES; interessados preenchidos; observação
preenchida. O leitor publica `null` onde não há valor marcado e recusa
formas novas.
