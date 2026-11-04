---
type: SEIObservation
title: Reabrir o processo na unidade pela barra da árvore
description: A ação reabrirProcesso() da barra do processo concluído (sete instalações), as duas formas de a tela do nó publicar a URL procedimento_reabrir (corpo da função no SEI 4.0, variável linkReabrirProcesso no 4.1 e 5.0), o GET que é a escrita e a árvore relida
---

# Reabertura do processo

Observado nas cassettes públicas do Todos (sete instalações de
treinamento, capturas até 19/10/2026), lidas pela estrutura: as 11 GET de
`procedimento_reabrir` (uma ou duas por instalação, cenários
`processo-escritas-*` e `processo-escrita-ciclo-*`), as 431 telas do nó
(`arvore_visualizar`) gravadas e as árvores das 190 cassettes que gravaram
concluir ou reabrir. Contagens no
[replay do acervo](../validacao-replay/2026-11-04-processo-reabrir-acervo.md).

## Onde a ação é publicada

Na barra do nó `PROCESSO` da árvore (`procedimento_visualizar`, linha
`Nos[0].acoes = '…';`, ver [árvore do processo](arvore-do-processo.md)),
**igual nas sete instalações**: `<a href="#" onclick="reabrirProcesso();">`
com o ícone "Reabrir Processo". Nas árvores gravadas (situação lida e
unidade da sessão conhecida), reabrir aparece **sempre** que a unidade da
sessão está fora de `aberto_em` e **nunca** com ela dentro; com ela
dentro, a barra publica concluir ([conclusão do processo](conclusao-do-processo.md)).
Processo que nunca passou pela unidade não aparece nas gravações
(inferência: o SEI não oferece reabrir onde não houve andamento).

## A URL, na tela do nó

A URL não está na barra: está na tela do nó (`arvore_visualizar`, o `href`
assinado do `Nos[0]` `PROCESSO`, carregada no `ifrVisualizacao`), em uma
de duas formas, **toda** tela do nó gravada com exatamente uma (inclusive
a de processo aberto):

| Forma | Instalações (versão nas cassettes) | Na tela do nó |
| --- | --- | --- |
| Função | UnB 4.0.9, PEN1 4.0.12 | o corpo de `reabrirProcesso()` em três linhas, como `concluirProcesso()` |
| Variável | UFRJ 4.1.5, PEN2 4.1.5, GO 5.0.3, PEN3 5.0.4, PEN4 5.0.4 | a linha `var linkReabrirProcesso = '…';` entre as outras `var link…` |

Forma da função (o `confirm()` está comentado, e com o erro de digitação
servido):

```
function reabrirProcesso(){
  //if (confirm('Confirma reabetura do processo?')){
   location.href = 'controlador.php?acao=procedimento_reabrir&acao_origem=arvore_visualizar&id_procedimento=ID&atualizar_arvore=1&…&infra_hash=…';
```

Forma da variável:

```
var linkReabrirProcesso = 'controlador.php?acao=procedimento_reabrir&acao_origem=arvore_visualizar&id_procedimento=ID&atualizar_arvore=1&…&infra_hash=…';
```

Na forma da variável, `reabrirProcesso()` não está na página: vem do
`js/arvore_visualizar.js` servido à parte, não gravado. Que ela só navega
para `linkReabrirProcesso` é inferência do que o Todos fez e do que
aconteceu: o GET da URL da variável reabriu o processo nas cinco
instalações.

## A escrita e o que muda

O GET da URL é a escrita: devolve **200** com uma página no molde de
`arvore_visualizar` (título "SEI - Reabrir Processo" na GO, PEN3 e PEN4;
"SEI - " nas demais), sem 302 nem mensagem da Infra. A árvore seguinte
mostrou, nas 11 gravações, o processo aberto na unidade da sessão e
**atribuído** ao usuário:

```
Processo aberto somente na unidade <a … class="ancoraSigla">CAXIAS</a> (atribuído para <a … class="ancoraSigla">servidor1</a>).<br />
```

e a barra voltou a publicar concluir em vez de reabrir. Nas 11 gravações o
processo estava concluído em todas as unidades antes ("Processo não possui
andamentos abertos."); a reabertura com o processo aberto noutra unidade
não foi gravada.

Na UnB e na PEN1, o Todos usou a URL lida na tela do nó **antes** de
concluir (a página do GET de concluir também define `reabrirProcesso()`,
com `acao_origem=procedimento_concluir`). O IES pede a tela do nó de novo,
depois da abertura: a URL vem sempre da página do estado atual.

## Limites

O `.js` da forma variável não foi gravado. Reabertura com o processo aberto
noutra unidade, reabertura recusada pelo SEI e o efeito de pedir só a tela
do nó (sem o GET de reabrir) não foram observados; a tela do nó é a mesma
que a árvore carrega no `ifrVisualizacao` a cada clique na raiz.
