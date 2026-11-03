---
type: SEIObservation
title: Histórico do processo e troca de visão
description: Link do histórico na árvore (duas formas), página procedimento_consultar_historico e a troca R/P por verHistorico
---

# Histórico do processo e troca de visão

Observado na UnB 4.0.9 e em Goiás 5.0.3 (treinamento, 03/11/2026), em
capturas privadas `0600` lidas só pela estrutura. Nenhum valor real
aparece aqui ou nas fixtures. Uma instalação não prova as outras.

## O link na árvore

A [árvore](arvore-do-processo.md) publica o link assinado do histórico
(`acao=procedimento_consultar_historico&acao_origem=arvore_visualizar&acao_retorno=arvore_visualizar&id_procedimento=N&arvore=1&infra_sistema=…&infra_unidade_atual=…&infra_hash=<64 hex>`,
relativo, `&` cru) num `div#divConsultarAndamento` com um único `<a
href="#">` ("Consultar Andamento"). O CSS do mesmo `id` também aparece
no `head`, sem ser elemento. Duas formas:

- **UnB 4.0.9.** O `onclick` é `consultarAndamento();`. O `script` inline
  define a função em duas linhas: `function consultarAndamento(){` e
  `parent.document.getElementById('ifrVisualizacao').src = '<URL>';`
  (aspas simples). As outras atribuições de `ifrVisualizacao` na página
  (`"about:blank"`, `no.href`) não são literais de URL.
- **GO 5.0.3.** O `onclick` é `consultarAndamento('<URL>');` e a função
  não é definida na página da árvore.

## A página do histórico

O GET do link devolve 200, `iso-8859-1`:

- `<title>SEI - Histórico do Processo <NUP>&nbsp;&nbsp;</title>`;
- `form#frmProcedimentoHistorico method="post"` com `action` assinado
  (`acao=procedimento_consultar_historico&acao_origem=procedimento_consultar_historico&arvore=1&id_procedimento=N&…&infra_hash=…`);
- dentro do formulário, o botão `button#btnAtualizarAndamento`
  (`type="button"`, `onclick` com `location.href` para
  `procedimento_atualizar_andamento`: **escrita**), o link
  `a#ancTipoHistorico`, os paginadores `divInfraAreaPaginacaoSuperior` e
  `…Inferior` vazios, a `table#tblHistorico.infraTable` e os ocultos
  `hdnInfraTipoPagina`, `hdnInfraNroItens`, `hdnInfraItemId`,
  `hdnInfraItens`, `hdnInfraItensHash`, `hdnInfraItensSelecionados`,
  `hdnInfraSelecoes`, `hdnInfraPaginaAtual`, `hdnInfraHashCriterios` e
  `hdnTipoHistorico`; `hdnInfraPrefixoCookie` fica fora do formulário;
- a tabela começa por
  `<caption class="infraCaption">Lista de Andamentos (N registro):</caption>`
  (`registros` no plural) e uma linha de `th` (Data/Hora, Unidade,
  Usuário, Descrição); cada andamento é um `tr.andamentoAberto` ou
  `tr.andamentoConcluido` com quatro `td` (unidade e usuário dentro de
  `a.ancoraSigla`, descrição seguida de `<span><br /></span>`).

## Visões R e P

O GET abre a visão **resumida**: `hdnTipoHistorico` vale `R` e o link é
`<a id="ancTipoHistorico" href="#" onclick="verHistorico('P');">Ver histórico completo</a>`.
A função servida, nas duas instalações:

```text
function verHistorico(valor){
  document.getElementById('hdnTipoHistorico').value = valor;
  document.getElementById('frmProcedimentoHistorico').submit();
}
```

O POST do formulário com os ocultos servidos e `hdnTipoHistorico=P`
(envio por script, nenhum botão) devolveu a visão **completa**:
`hdnTipoHistorico` vale `P` e o link passa a `verHistorico('R')` ("Ver
histórico resumido"). Em GO, 20 linhas viraram 49; noutro processo, 17
continuaram 17. Nenhum paginador apareceu até 49 linhas, e a legenda
declarou exatamente as linhas servidas.

Não medido: se o servidor lembra a visão na sessão (um GET seguinte
abriria `P`), se o GET do histórico lança andamento, e a forma da
legenda com paginação.

Veja também [árvore do processo](arvore-do-processo.md),
[formulários e JavaScript](formularios.md) e
[tabelas e paginação](tabelas-e-paginacao.md).
