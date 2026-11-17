---
type: Guide
title: Decisão de processo obter — anexados, propriedades e movidos
description: Processos anexados, propriedades, documentos movidos, situação e natureza na decisão de processo obter
---

# Decisão de processo obter — anexados, propriedades e movidos

Parte do [decisão de processo obter](../processo-obter.md), separada por responsabilidade.

## Processos anexados

**Alternativas.** (a) **web-anexados-do-no-da-arvore** (escolhida): o nó
`PROCESSO_ANEXADO` da árvore já lida dá número, id e tipo de cada
anexado; o índice aprende o par NUP ↔ id; os documentos do anexado não
são lidos e a saída diz isso (`documentos_dos_anexados` em `nao_lidos`).
(b) **web-abrir-cada-anexado**: seguir o a3 de cada anexado para listar
os documentos dele; rejeitada: uma requisição por anexado, efeito do GET
não medido e forma da resposta nunca observada. (c) Continuar recusando
a árvore inteira: o comando falhava num processo real.

**Evidência.** Captura privada da UnB 4.0.9 (03/11/2026): um
`PROCESSO_ANEXADO` de 16 argumentos com pai igual ao processo, a15 igual
a a5 e nenhum nó filho dele ([evidência](../../../contracts/evidencias/processo-obter.md)).

**Orçamento.** 0 requisições a mais em todos os caminhos.

**Trade-offs.** Quem quer os documentos do anexado roda
`ies processo <NUP do anexado>`, que já sai pelo índice. Filho de
anexado, nunca observado, é recusa (`no_com_pai_no_processo_anexado`):
falha fechada até haver captura.

**Pai do processo aberto.** Aberto um anexado, o `.html` da raiz traz o
link assinado do pai (UnB 4.0.9, 03/11/2026). O IES publica `anexado` e
`anexado_a` e aprende o par e o link `procedimento_trabalhar` do pai, sem
requisição: depois, `ies processo <NUP do pai>` custa 2 GET (passo 1b).
Link candidato fora da forma ou repetido: (a) **web-vinculo-recusa-fechada**
(escolhida), recusa tipada; (b) **web-vinculo-nulo-com-motivo**, publicar
a árvore com `anexado_a: null` e um motivo. Escolhida (a): com um link
candidato malformado, `anexado: false` seria afirmação falsa e (b)
exigiria um terceiro estado; o leitor da árvore inteira já falha fechado,
e a recusa diz o fato.

## Propriedades

**Alternativas.** (a) **web-propriedades-da-barra** (escolhida): um GET
do `procedimento_alterar` assinado que a barra do nó `PROCESSO` publica
(ou de `procedimento_consultar`, quando alterar não é oferecido), lendo os
controles servidos de `form#frmProcedimentoCadastro`, sem envio. (b)
**web-propriedades-todas-as-telas**: as quatro telas do Todos (metadados,
anotação, atribuição, relacionamentos); rejeitada nesta fatia, 4 GETs e
três telas nunca capturadas pelo IES. (c)
**web-propriedades-listas-ocultas**: ler `hdnAssuntos` e
`hdnInteressadosProcedimento`; rejeitada: vêm vazias no HTML servido (o
`infraLupaSelect` as monta no navegador), então o `select` é a única
fonte; lista servida preenchida é recusa até ser observada.

**Evidência.** Captura privada `0600` da UnB 4.0.9 (04/11/2026), só
estrutura ([observação](../../como-o-sei-funciona/propriedades-do-processo.md)):
a tela passou pelo leitor (tipo marcado, 1 assunto com código, 0
interessados, observação vazia, `publico`, sem hipótese, data); as seis
árvores capturadas publicaram o link (quatro alterar, duas consultar,
processos abertos só noutra unidade); teste local temporário, apagado.
Oráculo: `todos/propriedades.py` lê os mesmos controles e troca alterar
por consultar com o processo fechado (UFRJ 4.1).

**Forma publicada.** `assuntos` saem como `{codigo, descricao}` porque o
texto servido é `<código> - <descrição>` com código de dígitos e pontos;
fora disso `codigo` é `null` e o texto inteiro é a descrição, sem
palpite. `tipo` e `hipotese_legal` são a `option` marcada (nenhuma:
`null`); `hdnIdTipoProcedimento` confere o tipo. Ids de assunto e de
contato não são publicados. `fonte` diz qual tela foi lida.

**Orçamento.** +1 GET; `--sem-propriedades` +0; barra sem o link +0.
Repetição quente no caminho 1: árvore, histórico e propriedades
guardados, **3** (**2** com `--sem-andamentos`, **1** com os dois
`--sem-*`). Guardado só da `acao` que a barra recém-lida oferece:
diferente dos andamentos, a barra decide qual tela vale (alterar some
quando o processo fecha na unidade).

**Efeito.** GET de formulário de edição, declarado leitura (o Todos o
usa como leitura); nunca POST, nunca `btnSalvar`. Nada a restaurar.

**Trade-offs.** Ler o formulário de edição em vez de uma tela de
consulta custa o mesmo GET e publica o que o SEI gravou; recusar formas
não observadas (listas ocultas, nível sem marca) pode derrubar o comando
numa instalação nova, e por isso `--sem-propriedades` existe.

## Documentos movidos

`DOCUMENTO_MOVIDO` (PEN1 5.0.4) fica fora dos documentos; o destino sai
do link assinado do `.html` do nó (**web-movidos-pelo-link-do-no**), nunca
da frase da dica (**web-movidos-pela-dica**, rejeitada). 0 requisições a
mais. Alternativas e trade-offs no
[apêndice](documentos-movidos.md); documento de novo no
processo: [apêndice](movido-presente.md).

## Situação e natureza

`aberto_em` e ícone/cancelamento dos documentos saem da árvore já lida,
0 requisições, só nas formas observadas
([apêndice](situacao.md)).
Conformidade offline com o acervo público:
[apêndice](replay-do-acervo.md).
