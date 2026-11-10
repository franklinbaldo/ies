---
type: SEIObservation
title: Onde o processo está aberto e natureza dos documentos na árvore
description: O .html do nó PROCESSO publica as unidades onde o processo está aberto (com atribuição) ou sobrestado e os argumentos a6 a a9 do nó DOCUMENTO publicam o ícone e a dica, que marca o cancelamento
---

# Situação do processo e natureza dos documentos na árvore

Observado em capturas privadas `0600` (`iso-8859-1`) da árvore
(`procedimento_visualizar`) na UnB 4.0.9 (03/11/2026, seis árvores), em
GO 5.0.3 (03/11/2026, uma) e na PEN1 5.0.4 (04/11/2026, uma), lidas só
pela estrutura; o sobrestamento, no
[acervo público](../validacao-replay/2026-11-04-processo-obter-acervo.md). Os nomes abaixo são conteúdo das telas de treinamento;
nenhuma URL, hash ou e-mail. Declaração dos nós em
[árvore do processo](arvore-do-processo.md).

## O `.html` do nó `PROCESSO`

É um literal JS de linha inteira, `Nos[0].html = '<texto>';` (na UnB,
seguido da declaração da primeira `PASTA` na mesma linha). Formas vistas:

| Instalação | Texto (entre aspas simples, sem quebras) |
| --- | --- |
| UnB, PEN1 | `Processo aberto somente na unidade <a alt="Unidade1" title="Unidade1" class="ancoraSigla">Unidade1</a>.<br />` (na PEN1, `alt`/`title` `Unidade de Teste 1` e texto `TESTE`) |
| UnB | `Processo aberto somente na unidade <a … class="ancoraSigla">Unidade1</a> (atribuído para <a alt="aluno2" title="aluno2" class="ancoraSigla">aluno2</a>).<br />` |
| UnB (2 árvores) | `Processo aberto nas unidades:<br /><a alt="Arquivo Central" title="Arquivo Central" class="ancoraSigla">ACE</a><br />…<a … >Unidade4</a><br /><br />` (quatro unidades) |
| GO | a mesma forma plural com quatro unidades (siglas com `/`, `-` e dígitos, nomes com acento) e, numa delas, ` (atribuído para <a alt="<nome da pessoa>" title="<nome da pessoa>" class="ancoraSigla">aluno05</a>)` antes do `<br />` |
| UnB (ao vivo, 04/11/2026) | a forma plural com duas unidades, a primeira atribuída (` (atribuído para <a …>aluno2</a>)`) |
| UnB | `Processo não possui andamentos abertos.<br />` |
| UnB, GO, UFRJ, PEN1 a PEN4 (acervo) | `Processo sobrestado na unidade <a alt="Unidade1" title="Unidade1" class="ancoraSigla">Unidade1</a>.<br />` (na PEN, `Unidade de Teste 1`/`TESTE`; na UFRJ, `Campus Duque de Caxias`/`CAXIAS`; em GO, `SEAD/CDE-09516`) |
| UnB (anexado) | `<div style="font-size:.875rem;display:inline;">Processo anexado ao processo <a href="…" …>NUP do pai</a>.</div>`: o [vínculo de anexação](arvore-do-processo.md), sem unidades |

Em todas, `alt` e `title` são iguais: o nome da unidade (ou da pessoa); o
texto do link é a sigla (ou o login). A atribuição é por unidade, dentro
da linha dela. A frase muda de singular para plural com o número de
unidades. Formas de outras versões (unidade sem `ancoraSigla`,
processo concluído em todas fora da frase "não possui andamentos
abertos"): **não observadas**.

**Sobrestado.** No acervo público (`processo-sobrestar-retomar-*`, sete
instalações, 4.0.9 a 5.0.4), as 21 árvores servidas entre
`procedimento_sobrestar` e `procedimento_remover_sobrestamento` trazem
**só** a frase acima, sempre com uma única unidade (a da sessão) e sem
atribuição; nenhuma forma plural, com atribuição ou com outro texto.
Antes de sobrestar, a mesma árvore dizia `Processo aberto somente na
unidade` com a mesma âncora; depois de retomar, de novo; depois de
concluir, `Processo não possui andamentos abertos.`. A frase não diz se o
processo continua aberto na unidade nem em outras: o IES não deduz.

## Natureza do `DOCUMENTO`

Nos 16 argumentos (`"DOCUMENTO", id, pai, url, alvo, rótulo, dica,
ícone, ícone, ícone, true, bool, null, null, "noVisitado"|null, número`),
a7, a8 e a9 vieram **sempre** iguais, `svg/<radical>.svg?<n>` (`?11` na
UnB, `?25` em GO e na PEN1). Radicais vistos (contagem de nós):

| Radical | UnB | GO | PEN1 |
| --- | --- | --- | --- |
| `documento_interno` | 188 | 6 | 1 |
| `documento_pdf` | 7 | — | — |
| `documento_html` | — | — | 1 |
| `documento_email` | — | 1 | — |
| `documento_email_cco` | — | 1 | — |
| `documento_circular` | — | 1 | — |
| `documento_cancelado` | — | 1 | — |
| `documento_txt` | — | [replay](../validacao-replay/2026-11-04-processo-obter-acervo.md) | replay (também PEN3, PEN4) |

A dica (a6) repetiu o rótulo (a5) em todos os nós, menos três, em GO:

- **Cancelado**: rótulo `Consulta Jurídica 8441007`, dica
  `Documento Cancelado: erro` (o motivo digitado), ícone
  `documento_cancelado`, a3 `about:blank` e a14 `null`. a3 `about:blank`
  também aparece num documento interno não cancelado: não classifica.
- **E-mail**: dica `Data de Envio: dd/mm/aaaa hh:mm:ss\nDe: …\nPara: …\nAssunto: …\n`
  com `Para (com cópia oculta):` no `documento_email_cco`; rótulo
  diferente da dica. Os cabeçalhos trazem endereços de e-mail.

**Inferência, não observação:** `documento_pdf` parece ser documento
externo (anexado como arquivo) e `documento_interno` documento gerado no
editor do SEI; o IES publica só o radical, e a dica como servida quando
difere do rótulo. Leitura do IES:
[contrato](../../contracts/cli/processo-obter-situacao.md).
