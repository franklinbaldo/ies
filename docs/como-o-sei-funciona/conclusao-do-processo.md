---
type: SEIObservation
title: Concluir o processo na unidade pela barra da árvore
description: Duas formas observadas de publicar "Concluir Processo" (link para a tela Conclusão de Processo com POST de Salvar no SEI 4.1 e 5.0; função concluirProcesso() com GET assinado no 4.0), a resposta, a árvore relida e o reabrir
---

# Conclusão do processo

Observado nas cassettes públicas do Todos (sete instalações de
treinamento, capturas até 19/10/2026), lidas pela estrutura: 232 GET e 161
POST de `procedimento_concluir` e 11 GET de `procedimento_reabrir`, em 66
cenários. A gravação não guarda o corpo dos POST; os campos enviados vêm do
formulário servido e do teste do Todos que mediu o envio do navegador na
UFRJ (`tests/test_ciclo_conclusao_por_formulario_010.py`).

## Onde a ação é publicada

A ação está na barra do nó `PROCESSO` da árvore (`procedimento_visualizar`,
linha `Nos[0].acoes = '…';`, ver [árvore do processo](arvore-do-processo.md)),
ícone `svg/processo_concluir.svg`, título "Concluir Processo". Duas formas:

| Forma | Instalações (versão nas cassettes) | Barra | Escrita |
| --- | --- | --- | --- |
| Tela | UFRJ 4.1.5, PEN2 4.1.5, GO 5.0.3, PEN3 5.0.4, PEN4 5.0.4 | `<a href="controlador.php?acao=procedimento_concluir&acao_origem=arvore_visualizar&acao_retorno=arvore_visualizar&id_procedimento=ID&arvore=1&…&infra_hash=…">` | GET da tela, POST do formulário |
| Função | UnB 4.0.9, PEN1 4.0.12 | `<a href="#" onclick="concluirProcesso();">` | **o próprio GET** da URL do corpo da função |

A PEN1 passou a 5.0.4 em 02/11/2026 ([instalações e versões](../interfaces/instalacoes-e-versoes.md));
a forma dela hoje não foi observada (inferência: a da tela, como a PEN3 e a
PEN4).

**Forma da função.** A URL não está na barra: está no corpo de
`concluirProcesso()` na tela do nó (`arvore_visualizar`, o `href` do
`Nos[0]` `PROCESSO`, carregada no `ifrVisualizacao`), em três linhas:

```
function concluirProcesso(){
  //if (confirm('Confirma conclusão do processo?')){
   location.href = 'controlador.php?acao=procedimento_concluir&acao_origem=arvore_visualizar&id_procedimento=ID&atualizar_arvore=1&…&infra_hash=…';
```

O `confirm()` está comentado. A mesma página define todas as funções
(`excluirProcesso`, `reabrirProcesso`, …), oferecidas ou não: o que diz
que concluir é oferecido é o `onclick` da barra. O GET devolve 200, uma
página no molde de `arvore_visualizar` (título "SEI - "), e o processo já
está concluído na leitura seguinte da árvore (34 na UnB, 32 na PEN1).
Nenhum GET dessa forma foi gravado sem efeito.

**Forma da tela.** O GET do link da barra devolve 200 com a tela
"Conclusão de Processo" (janela sem `lnkInfraUnidade`), idêntica nas cinco
instalações:

- `form#frmDesentranharDocumento method="post"`, `action` assinado
  `procedimento_concluir&acao_origem=procedimento_concluir&id_procedimento=ID&arvore=1`;
- ocultos `hdnInfraTipoPagina=2`, `hdnIdProtocolos=ID`,
  `hdnIdProcedimentoDestino` vazio; `button#sbmSalvar` (submit, "Salvar")
  e `btnCancelar` (`type="button"`);
- `select#selProcedimentos multiple` com uma opção (o processo), nenhuma
  selecionada;
- rádios `rdoConcluir`: `S` "Somente concluir" (`checked`) e `V`
  "Concluir e agendar reabertura na unidade"; rádios
  `rdoPrazoReaberturaProgramada` (1, 2) sem marca, textos
  `txtPrazoReaberturaProgramada` e `txtDiasReaberturaProgramada` vazios,
  caixa `chkSinDiasUteisReaberturaProgramada` desmarcada.

O `onsubmit` só valida a opção (`validarConcluir()`); nenhum oculto é
reescrito. O navegador envia, na ordem do documento:
`hdnInfraTipoPagina`, `sbmSalvar`, `rdoConcluir=S`,
`txtPrazoReaberturaProgramada=`, `txtDiasReaberturaProgramada=`,
`hdnIdProtocolos`, `hdnIdProcedimentoDestino=`. A resposta é **302**,
nunca seguida, para `arvore_visualizar` (`acao_origem=procedimento_concluir`,
mesmo `id_procedimento`), 152 de 153; uma vez na UFRJ, para `principal`
com a mesma origem e id (o Todos mediu esse caso depois de agendar e
cancelar uma publicação). Nos dois casos a árvore seguinte mostra a
conclusão. Nenhum GET da tela foi gravado sem o POST em seguida: o efeito
de só abrir a tela não foi medido.

## O que muda na árvore

A situação é o `.html` da raiz ([situação na árvore](situacao-na-arvore.md)).
Nas 219 conclusões gravadas (153 pela tela, 66 pela função), a árvore
seguinte mostrou: "aberto somente na unidade U" → "Processo não possui
andamentos abertos." (200); e, com o processo aberto em mais de uma
unidade, **só a unidade da sessão sai** ("aberto nas unidades: A, U" →
"aberto somente na unidade A", 19, nas sete instalações).
A barra muda junto: concluído, ela não publica mais `procedimento_concluir`
nem `concluirProcesso()`, publica `onclick="reabrirProcesso();"` e troca
`procedimento_alterar` por `procedimento_consultar` (UFRJ).

## Já concluído, sem permissão

Concluído na unidade (ou aberto só em outra), a barra não oferece concluir;
a situação não lista a unidade da sessão. Uma tela de recusa do SEI para
concluir não foi gravada (o Todos recusa antes, pela barra).

## Reabrir

Nas sete instalações, inclusive nas da forma tela, a barra do processo
concluído publica `onclick="reabrirProcesso();"`. A URL
`procedimento_reabrir` está na tela do nó, mas **não** na mesma forma em
todas: no SEI 4.0 no corpo da função, em três linhas; no 4.1 e 5.0 na
linha `var linkReabrirProcesso = '…';` (a função vem de um `.js` à parte).
O GET devolveu 200 e a árvore seguinte mostrou o processo de novo aberto
na unidade da sessão, **atribuído** ao usuário (11 gravações). Detalhes em
[reabertura do processo](reabertura-do-processo.md).

## Limites

Sem corpo de POST gravado, a ordem e os valores enviados são os do
formulário servido e do teste do Todos, não de uma captura. A tela de
recusa e o efeito de abrir a tela sem salvar não foram observados.
