---
type: SEIObservation
title: Tela de trabalho e árvore do processo
description: Estrutura observada de procedimento_trabalhar, ifrArvore e das declarações infraArvoreNo
---

# Tela de trabalho e árvore do processo

Observado na UnB 4.0.9 e em Goiás (treinamento, 03/11/2026; documentos
movidos na PEN1 4.0.12, 04/11/2026), em
capturas privadas `0600` lidas só pela estrutura. Nenhum valor real
aparece aqui ou nas fixtures.

## Como se chega à árvore

1. `controlador.php?acao=procedimento_trabalhar&…&id_procedimento=N&…&infra_hash=<64 hex>`:
   o `Location` do 302 da [pesquisa](pesquisa-de-protocolo.md) com NUP
   exato, ou o `href` do link do processo na linha `tr#P{id}` do
   Controle (visões R e D). Na **pesquisa rápida**
   (`form#frmProtocoloPesquisaRapida`, um campo `txtPesquisaRapida`), o
   302 das cassettes de GO tem `acao_origem=protocolo_pesquisa_rapida` e o
   id em `id_protocolo`; o `src` da árvore volta a usar `id_procedimento`.
2. A resposta 200 (`<title>SEI - Processo`) traz exatamente um
   `<iframe id="ifrArvore" name="ifrArvore" … src="controlador.php?acao=procedimento_visualizar&acao_origem=procedimento_trabalhar&acao_retorno=…&id_procedimento=N&id_documento=&infra_sistema=…&infra_unidade_atual=…&infra_hash=…">`,
   com `&` cru nas duas instalações, e um `iframe#ifrVisualizacao` com
   `src="about:blank"`. O `acao_retorno` variou (`principal` na UnB,
   `procedimento_controlar` em GO).
3. O GET do `src` de `ifrArvore` devolve a árvore: um `script` inline com
   as declarações dos nós e, no corpo, `div#divArvore` vazio (a árvore
   é montada no navegador), `form#frmArvore` com `hdnArvore`,
   `hdnPastaAtual` e `hdnProtocolos` vazios e `iframe#ifrPasta` oculto.

## Efeito medido

Em GO, num processo recebido ainda não visualizado
(`processoNaoVisualizado`), o GET de `procedimento_trabalhar` sozinho
**não** mudou a caixa; o GET seguinte de `procedimento_visualizar` (a
árvore) o marcou como visualizado em `ies unidade recebidos`. Nenhuma
outra linha mudou. Se um andamento de recebimento também é lançado:
**NOT_RUN**.

## Declaração de um nó

Cada nó é uma linha inteira, sem indentação:

`Nos[i] = new infraArvoreNo(a0,…,a15);`

com 16 argumentos literais (aspas duplas, `null`, `true`, `false`),
índices `i` consecutivos a partir de 0. Seguem-se linhas
`Nos[i].acoes = '…'`, `Nos[i].src = '…'` e `Nos[i].html = '…'`; o
`Nos[0].processar = function(){…}` vem depois, dentro de `if (Nos.length)`.
As ações da barra (`NosAcoes[k] = new infraArvoreAcao(…)`) são outro
construtor.

| Arg. | PROCESSO | DOCUMENTO |
| --- | --- | --- |
| a0 | `"PROCESSO"` | `"DOCUMENTO"` |
| a1 | id do procedimento | id do documento |
| a2 | `null` | id do pai (o procedimento) |
| a3 | `arvore_visualizar` assinado | idem, ou outra forma (não lido) |
| a4 | `"ifrVisualizacao"` | `ifrVisualizacao` (UnB) ou `ifrConteudoVisualizacao` (GO) |
| a5 | NUP | rótulo: tipo do documento e número SEI |
| a6 | tipo do processo (dica) | dica; igual a a5 ou texto livre (em e-mail, cabeçalhos com `\n`) |
| a7–a9 | ícone, três vezes igual | idem (`documento_interno`, `documento_email`, `documento_cancelado`…) |
| a10, a11 | `true`, `true` | `true`, `true` |
| a12–a14 | `"noVisitado"`, `null`, `"noVisitado"` | `null`, `null`, `"noVisitado"` ou `null` |
| a15 | NUP outra vez | número SEI do documento |

O significado de a10–a14 não foi demonstrado (o Todos também não o
publica). Nos documentos, a15 aparece dentro de a5. A dica de e-mail traz
remetente e destinatários: dado pessoal que o IES **não** publica.

## A palavra PASTA

Na UnB, `PASTA` aparece 11 vezes e `AGUARDE` 8 vezes sem nenhuma pasta:
todas dentro de funções inline (`objArvore.getNoAnterior`, navegação por
teclado, `processarPasta`) que comparam `this.nodes[i].tipo == 'PASTA'`
ou trocam `PASTA` por `AGUARDE`, e nos ids `ifrPasta`/`hdnPastaAtual`.
Em GO, `PASTA` aparece uma vez e `AGUARDE` nenhuma. Pasta só existe como
nó se houver `new infraArvoreNo("PASTA",…)`; o texto solto não é nó.

## Pastas

Observado na UnB 4.0.9 nas cassettes públicas `processo-em-pastas-unb`
(interações 8 e 9) e `processo-pasta-mista-unb` (411 a 487), lidas só pela
estrutura. O processo de 20 documentos não tem pasta; com 21 aparecem
duas. Cada pasta tem até 20 documentos e só a mais recente vem carregada.

- **Declaração.** Antes dos nós, por pasta:
  `  Pastas[k] = [];`, `  Pastas[k]['link'] = 'controlador.php?acao=procedimento_paginar&…&no_pai=PASTAk&pagina_hash=…&infra_hash=…';`
  e `  Pastas[k]['protocolos'] = '<p1>,<p2>,…';` (decimais).
- **Nó `PASTA`**, 15 argumentos:
  `("PASTA","PASTAk","<id>","javascript:abrirFecharPasta('PASTAk');",null,"<rótulo romano>","Pasta k",null,"svg/processo_aberto.svg…","svg/processo_fechado.svg…",<aberta>,true,null,null,null)`,
  seguido de `Nos[i].carregado = true|false;`. A primeira pasta vem na
  mesma linha do `.html` do processo:
  `Nos[0].html = '…';Nos[1] = new infraArvoreNo("PASTA",…);`.
- **Nó `AGUARDE`**, 15 argumentos, filho da pasta fechada:
  `("AGUARDE","AGUARDEk","PASTAk",null,null,"Aguarde...","Aguarde...","/infra_css/svg/aguarde.svg",null,null,false,true,null,null,null)`.
- **Documentos** da pasta têm a2 `"PASTAk"`. A barra de cada um
  (`Nos[i].acoes = '…';`, uma linha inteira) traz o
  `id_rel_protocolo_protocolo` no link `comentario_listar`; é esse valor,
  não o id do documento, que `Pastas[k]['protocolos']` lista (17 de 17 e
  157 de 157 nas duas leituras da primeira cassette). Ao vivo na UnB
  (03/11/2026), em dois processos abertos só em outra unidade (o `.html`
  do nó do processo diz "Processo aberto somente na unidade …"), 150 barras
  vieram vazias (`Nos[i].acoes = '';`) e algumas com links sem
  `id_rel_protocolo_protocolo` (`documento_consultar`,
  `documento_imprimir_web`, `procedimento_gerar_pdf`…), misturadas na
  mesma pasta com barras que o trazem; a contagem de cada pasta igualou a
  declarada. A identidade é, portanto, **por documento**. Todo documento
  tinha uma linha `.acoes`.
- **Ações.** Na barra do processo,
  `NosAcoes[1] = new infraArvoreAcao("ABRIR_PASTAS","AP<id>","<id>","controlador.php?acao=procedimento_visualizar&acao_origem=procedimento_visualizar&id_procedimento=<id>&abrir_pastas=1&infra_sistema=…&infra_unidade_atual=…&infra_hash=…",null,"Abrir todas as Pastas","/infra_css/svg/mais.svg",true);`
  e a irmã `FECHAR_PASTAS` (`fechar_pastas=1`).

O GET de `ABRIR_PASTAS` devolve a árvore inteira: todas as `PASTA`
primeiro (abertas, `carregado = true`), depois os documentos de cada uma,
sem `AGUARDE` (8 pastas e 157 documentos, cerca de 1 MB). Na cassette
mista (uma pasta fechada de 20, outra aberta de 1 ou 2), a árvore simples
lida **depois** do GET de abrir voltou fechada. Nenhuma das duas cassettes
tem POST de `procedimento_paginar` nem GET de `FECHAR_PASTAS`. O navegador
abre uma pasta só preenchendo `hdnPastaAtual` e `hdnProtocolos` e
enviando `frmArvore` por POST para `Pastas[k]['link']` (alvo `ifrPasta`);
o Todos mediu que várias pastas num POST dão "Conjunto de protocolos
inválido". Leitura do IES: [contrato das pastas](../../contracts/cli/processo-obter-pastas.md).

## Processos anexados

Observado ao vivo na UnB 4.0.9 (03/11/2026), numa captura privada lida só
pela estrutura: um processo com um anexado e 7 documentos. O anexado é um
nó próprio, logo depois do processo:

| Arg. | PROCESSO_ANEXADO |
| --- | --- |
| a0 | `"PROCESSO_ANEXADO"` |
| a1 | id do procedimento anexado |
| a2 | id do processo (o pai) |
| a3 | `controlador.php?acao=arvore_visualizar&acao_origem=procedimento_visualizar&id_procedimento=<pai>&id_procedimento_anexado=<a1>&infra_sistema=…&infra_unidade_atual=…&infra_hash=…` (`&` cru) |
| a4 | `"ifrVisualizacao"` |
| a5 | NUP do anexado |
| a6 | tipo do anexado |
| a7–a9 | `svg/processo_anexado.svg?…`, três vezes |
| a10, a11 | `true`, `true` |
| a12–a14 | `null`, `null`, `null` (no processo, `"noVisitado"`, `null`, `"noVisitado"`) |
| a15 | NUP do anexado outra vez |

Seguem-se `.acoes` (links `procedimento_alterar` e `comentario_listar`),
`.src` e `.html` (com um link `procedimento_trabalhar`). **Todos** os
documentos da árvore tinham a2 igual ao id do processo; nenhum nó tinha o
anexado como pai: os documentos do anexado não vêm nesta árvore. O Todos
mede a anexação (`processo anexar`, "Lista de Processos Anexados") e a
regra do SEI de que um processo anexado não pode estar anexado a outro,
mas não lê esse nó.

Aberto pelo próprio NUP (pesquisa rápida), o anexado tem árvore própria:
raiz `PROCESSO` (a7–a9 `svg/processo_anexado.svg`, só corroboração) e
`Nos[0].html = '<div style="…;display:inline;">Processo anexado ao processo <a href="controlador.php?acao=procedimento_trabalhar&acao_origem=arvore_visualizar&id_procedimento=<pai>&id_procedimento_anexado=<próprio>&infra_sistema=…&infra_unidade_atual=…&infra_hash=<64 hex>" target="_blank" class="ancoraVisualizacaoArvore"><NUP do pai></a>.</div>';`,
com exatamente um `<a>`; o texto do link é o NUP do pai. Leitura do IES:
[contrato](../../contracts/cli/processo-obter-arvore.md).

## Documentos movidos

Observado ao vivo na PEN1 4.0.12 (04/11/2026), numa captura privada
`iso-8859-1` lida só pela estrutura: árvore com 1 `PROCESSO`, 2
`DOCUMENTO` e 1 `DOCUMENTO_MOVIDO`, um documento movido para outro
processo:

| Arg. | DOCUMENTO_MOVIDO |
| --- | --- |
| a0 | `"DOCUMENTO_MOVIDO"` |
| a1 | `"<n>-<m>"`: `n` é o `id_documento` do link do destino, `m` o `id_rel_protocolo_protocolo` da barra |
| a2 | id do processo |
| a3 | o mesmo `arvore_visualizar` assinado do nó `PROCESSO` (sem `id_documento`) |
| a4 | `"ifrConteudoVisualizacao"` |
| a5 | rótulo: tipo do documento e, entre parênteses, o número SEI |
| a6 | dica: `Documento movido para o processo <NUP do destino>: <motivo digitado>` |
| a7–a9 | `svg/documento_movido.svg?…`, três vezes |
| a10–a14 | `true`, `true`, `null`, `null`, `"noVisitado"` |
| a15 | número SEI (o de a5) |

Seguem-se `.acoes` com um só link (`comentario_listar`, `id_procedimento`
do processo e `id_rel_protocolo_protocolo` = `m`), `.src = ''` e
`Nos[i].html = 'Documento movido para o processo <a href="controlador.php?acao=procedimento_trabalhar&acao_origem=arvore_visualizar&id_procedimento=<destino>&id_documento=<n>&infra_sistema=…&infra_unidade_atual=…&infra_hash=<64 hex>" target="_blank" class="ancoraVisualizacaoArvore"><NUP do destino></a>.';`,
com exatamente um `<a>`; o texto é o mesmo NUP da dica. `n` não é id de
nenhum `DOCUMENTO` da árvore, a15 não repete o de nenhum deles e o id do
destino não aparece em outro nó: o movido não está mais no processo. Não
havia pastas; movido dentro de pasta não foi observado. O Todos não
menciona esse nó. Leitura do IES:
[contrato](../../contracts/cli/processo-obter-arvore.md).

Veja também [rotas e árvore](rotas-e-arvore.md) e
[efeitos e verificação](efeitos-e-verificacao.md).
