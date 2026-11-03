---
type: SEIObservation
title: Pesquisa de processo por protocolo
description: Menu, formulário, respostas 302/200 e paginação navegar() da pesquisa Web por protocolo
---

# Pesquisa de processo por protocolo (`protocolo_pesquisar`)

## Observação

UnB 4.0.9, treinamento, 03/11/2026, conta de aluno, sessão do `ies login`.
Capturas privadas `0600`, só para inspeção estrutural; nenhum valor real
(número, ID, hash, cookie) está neste repositório.

- **Menu.** O Controle publica, em qualquer visão, exatamente um
  `<a link="protocolo_pesquisar" href="controlador.php?acao=protocolo_pesquisar&…&infra_hash=<64 hex>">`
  ("Pesquisa"), com `&` cru no `href`. A mesma forma aparece nas capturas
  do Controle de PEN1 5.0.4 e GO 5.0.3, com `infra_item_menu` a mais. A barra
  superior tem outra ação, `protocolo_pesquisa_rapida`, que não é o menu.
- **Tela.** O GET desse link serve `frmPesquisaProtocolo` com
  `optDocumentos` marcado e `hdnInicio=0`.
- **POST aceito.** Ocultos como servidos, `rdoPesquisarEm` com o valor de
  `#optProcessos`, `selOrgaoPesquisa[]` com as opções servidas selecionadas,
  `sbmPesquisar` com o valor servido e `txtProtocoloPesquisa` com o termo.
- **NUP exato existente:** **302** com `Location` para
  `procedimento_trabalhar` (`id_procedimento`, sem o número formatado).
- **NUP inexistente:** **200** com `<div class="pesquisaSemResultado">`
  ("Nenhum resultado encontrado." e sugestões), sem `pesquisaResultado` e
  sem `txaInfraValidacao`. Significa que a pesquisa não devolveu nada para
  esta conta e unidade, não que o processo não exista.
- **Número parcial (5 dígitos):** **200** com
  `<div class="pesquisaBarraD">Exibindo 1 - 10 de N.NNN</div>` (total de quatro
  dígitos) e
  `<table class="pesquisaResultado">` de 10 linhas: o total usa `.` como
  separador de milhar. Não houve página com posição acima de 999, então a
  forma de `I` e `F` nessa faixa não foi vista.
- **Linha.** Cada `tr.pesquisaTituloRegistro` tem um `a.arvore` (ícone) e
  um `a.protocoloNormal` com o **mesmo** `href`
  (`acao=procedimento_trabalhar&acao_origem=protocolo_pesquisar&id_procedimento=…`);
  o `a.protocoloNormal` repete o atributo `class="protocoloNormal"`.
- **Tela de resultado.** O formulário volta com `optProcessos` marcado, o
  termo em `txtProtocoloPesquisa` e `hdnInicio=0`, e a tabela fica dentro
  dele.

## Paginação `navegar()`

O script servido define:

```js
function navegar(inicio) {
  document.getElementById('hdnInicio').value = inicio;
  if (typeof(window.onSubmitForm)=='function' && !window.onSubmitForm()) {
    return;
  }
  document.getElementById('frmPesquisaProtocolo').submit();
}
```

O paginador (`div.pesquisaPaginas`) publica a página atual em
`div.pesquisaPaginaSelecionada` e links `javascript:navegar('10')` a
`navegar('90')`, mais "Próxima" (`navegar('10')`). Logo, `inicio` é a
posição zero-based do primeiro registro da página, múltiplo do tamanho da
página (10). `submit()` por script não inclui `sbmPesquisar` no corpo.

## Efeito

`AlteraEstadoDaSessao::PaginaOuFiltroLembrado`: a busca e a página ficam
lembradas na sessão; o IES não restaura nada.

## Inferências e lacunas

- A resposta de `--pagina N` não foi capturada: o IES exige a faixa
  `inicio + 1 … min(inicio + 10, total)` e o mesmo total, e falha fechado em
  qualquer outra.
- Não foi visto 200 com uma única linha (para termo que casa um processo só
  sem ser exato), nem o efeito de filtros além do protocolo.
- UFRJ, GO e PEN não tiveram a pesquisa ensaiada nesta data.
