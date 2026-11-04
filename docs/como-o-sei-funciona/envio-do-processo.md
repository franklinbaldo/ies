---
type: SEIObservation
title: Enviar o processo pela barra da árvore
description: O link "Enviar Processo" da barra, a tela de envio (formulário frmAtividadeListar e o autocomplete de unidades declarado no script), a consulta XML, o POST com hdnUnidades e o 302, e a árvore relida com o destino em aberto_em
---

# Envio do processo

Observado nas cassettes públicas do Todos (sete instalações de
treinamento, capturas até 19/10/2026), lidas pela estrutura: 37 GET e 27
POST de `procedimento_enviar` e 66 POST de
`unidade_auto_completar_envio_processo`, em 33 cenários (26 com
`procedimento_enviar`). A gravação não guarda o corpo dos POST; os campos
enviados vêm do formulário servido, do script da tela e do código do Todos
que fez os envios (`src/todos/processos.py`, `_enviar_na_arvore`).

## Onde a ação é publicada

Um link na barra do nó `PROCESSO` da árvore (`procedimento_visualizar`,
linha `Nos[0].acoes = '…';`), nas sete instalações:

```
<a href="controlador.php?acao=procedimento_enviar&acao_origem=arvore_visualizar&acao_retorno=arvore_visualizar&id_procedimento=ID&arvore=1&…&infra_hash=…">
```

Nas 381 árvores lidas pelo replay, o link aparece sempre que a unidade da
sessão está em `aberto_em` e nunca quando não está.

## A tela "Enviar Processo"

O GET do link devolve HTML 200 (`divInfraBarraLocalizacao` "Enviar
Processo", sem `lnkInfraUnidade`) com o formulário `frmAtividadeListar`
(`method="post"`, `action` assinado `acao=procedimento_enviar&acao_origem=procedimento_enviar`):

| Controle | Servido |
| --- | --- |
| `hdnInfraTipoPagina` | `2` |
| `sbmEnviar` | botão submit "Enviar" |
| `selProcedimentos` (`multiple`) | uma opção, o processo, não marcada |
| `selOrgao` | `null` e "Todos" (`""`) com `selected`, depois os órgãos; o navegador fica com a última marcada, `""` |
| `txtUnidade`, `hdnIdUnidade` | vazios (o autocomplete) |
| `selUnidades` (`multiple`) | vazio (a lista de destinos) |
| `chkSinManterAberto`, `chkSinRemoverAnotacoes`, `chkSinEnviarEmailNotificacao` | desmarcadas |
| prazos | 4.0 (`UnB` 4.0.9, PEN1 4.0.12): `rdoPrazo`, `txtPrazo`, `txtDias`, `chkSinDiasUteis`; 4.1 e 5.0 (UFRJ, PEN2, GO, PEN3, PEN4): os mesmos com sufixo `RetornoProgramado` e a "Reabertura programada" (`…ReaberturaProgramada`) |
| `hdnIdProtocolos`, `hdnIdAtividades`, `hdnUnidades` | o id, a atividade, vazio |

A tela não lista unidades: o destino entra pelo autocomplete (ou pelas
lupas `unidade_selecionar_envio_processo`, `grupo_unidade_selecionar`,
`unidade_tramitacao_selecionar`, janelas à parte). O script declara, numa
linha:

```
objAutoCompletarUnidade = new infraAjaxAutoCompletar('hdnIdUnidade','txtUnidade','controlador_ajax.php?acao_ajax=unidade_auto_completar_envio_processo&…&infra_hash=…');
```

e o corpo da consulta no `prepararExecucao`, em duas formas:

```
return 'palavras_pesquisa='+…txtUnidade…+'&id_orgao='+…selOrgao….value;                      (4.0)
return 'palavras_pesquisa='+…txtUnidade…+'&id_orgao='+…selOrgao….value+'&unidade_atual=0'; (4.1, 5.0)
```

O `processarResultado` põe o item escolhido em `selUnidades`/`hdnUnidades`
pelo `infraLupaSelect` (`objLupaUnidades.adicionar(id, descricao, …)`). O
`OnSubmitForm` só confere que `selOrgao` está escolhido e `hdnUnidades`
não está vazio; o `manterAberto()` do 4.1/5.0 esconde a reabertura
programada quando a caixa é marcada.

**Efeito da tela e da consulta.** Nenhum observado: no envio da `UnB`
(`processo-enviar-retorno-unb`), duas telas e duas consultas antes do POST,
e o histórico seguinte só tem "Processo público gerado" e "Processo
remetido pela unidade". O Todos pede as duas na prévia.

## A consulta

POST a `controlador_ajax.php` (a URL do script), resposta 200
`application/xml; charset=iso-8859-1`:

```
<?xml version="1.0" encoding="iso-8859-1"?>
<itens><item id="110000847" descricao="Unidade2 - Unidade2"></item></itens>
```

`descricao` é `SIGLA - Nome`. Um item na `UnB` e nos PEN; vários na UFRJ
(`ADCENTRAL`, `ADCENTRAL/GR`, … — a busca casa prefixos) e no GO (a busca
"TESTE" devolve `SEAD/SUPTESTE-15887 - …` e `TESTE - Unidade de Teste`).
Sem resultado, `<itens></itens>` (medido pelo Todos na `UnB`).

### No GO 5.0.3: órgãos, a `/` e o corte

O `selOrgao` do GO tem 64 opções: `null`, "Todos" (`""`) e 62 órgãos,
cada rótulo uma vez (`ABC` = `28`, `SEAD`, `CASA CIVIL`, `GOIAS TURISMO`…);
as siglas das unidades começam pelo órgão sem espaço
(`CASACIVIL/GECONT-18148`). Gravado (`grupo-envio-ciclo-go`, tela de
grupo, mesma `acao_ajax`): uma resposta `<itens></itens>` seguida de 39
itens `…/GECONT-…` de 35 órgãos, sem nenhum `ABC/…`. Medido pelo Todos (o
corpo não foi gravado): a sigla inteira `ABC/GECONT-18141` não devolve
nada, porque a busca se parte na `/`; `GECONT` devolve as GECONT de
outros órgãos; com "Todos" a lista corta em 50 itens. Segundo o
changelog do Todos, `ABC/GECONT-18141` não existe. Ao vivo (04/11/2026,
unidade `SEAD/CDE-09516`): ela veio vazia no órgão `28` e em "Todos", e
`GECONT` em "Todos" devolveu os mesmos 39 itens gravados;
`CBM/GECONT-18149` no órgão `11` e `DETRAN/GECONT-18166` no `25` vieram
com um item, o pedido ([relatório](../validacao-live/2026-11-04-processo-enviar-go.md)).

## O POST e a resposta

O navegador envia o formulário com `hdnUnidades=ID±DESCRIÇÃO` (vários
destinos separados por `¥`; medido pelo Todos no PEN1 4.0.12), `selOrgao`
e, marcada, `chkSinManterAberto=on`. Os 27 POST gravados responderam 302
com `Location: controlador.php?acao=arvore_visualizar&acao_origem=procedimento_enviar&atualizar_arvore=1&arvore=1&id_procedimento=ID&…`.
Todos relativos, os 3 do GO inclusive; o Todos mediu no GO `Location` de
escrita em `http://` no mesmo host (issue #766), forma que as cassettes
do GO só gravam no `infra_trocar_unidade` e no `procedimento_controlar`.

## A árvore relida

O `.html` da raiz passa a listar o destino:

| Antes | Depois |
| --- | --- |
| `Processo aberto somente na unidade CAXIAS.` | `Processo aberto nas unidades: ADCENTRAL CAXIAS` |
| `Processo aberto somente na unidade TESTE.` | `Processo aberto nas unidades: TESTE TESTE_1_1` |

A sigla em `aberto_em` é a mesma da `descricao` do item. Os 25 envios a
uma unidade gravados foram feitos com "manter aberto" (o roteiro do Todos
envia mantendo aberto para concluir e reabrir em seguida): a unidade da
sessão continua em `aberto_em`. **O envio sem manter aberto não está
gravado**; pelo rótulo da caixa, o processo deixaria a unidade da sessão.
Um PEN1 enviou a duas unidades de um grupo de uma vez e, depois, de novo às
mesmas: 302 e `aberto_em` igual (o reenvio não muda a situação).

## Desfazer

Não há "desfazer envio" na barra. Com manter aberto, a origem segue com o
processo e o destino fica com ele aberto; sem manter aberto, o processo
sai da unidade da sessão. Voltar exige uma sessão na unidade de destino
(enviar de volta ou concluir lá).
